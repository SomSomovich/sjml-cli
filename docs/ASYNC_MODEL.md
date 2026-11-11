# Asynchronous Execution Model

The NeoForge adapter connects the portable async runtime to logical-server and
physical-client main-thread lanes. Modules run `mod_init` with async admission
rejected, enter `RUNNING`, and then attach their `GuestTaskScheduler` to the
context's tick queue. Static `mod_register` remains fail-closed. Client guest
tasks can run on the physical client main thread; client service calls,
`task.compute`, render-lane tasks, and client service providers remain
fail-closed.

Each module owns one portable-core `ModuleInstance` and bounded `TaskGroup`.
Tasks inherit the instance's `ExecutionContext` and select a host `TaskLane`:

| Lane | Intended executor |
| --- | --- |
| `WORKER` | Bounded background executor for work that does not touch thread-affine game state. |
| `CLIENT_MAIN` | Physical client main/game thread. |
| `SERVER_MAIN` | Logical server thread, including the integrated server thread. |
| `RENDER` | Client render thread. |
| `NETWORK` | Host network executor appropriate to the operation. |

NeoForge binds `SERVER_MAIN` to a queue drained by `ServerTickEvent.Post` on
the owning server thread and `CLIENT_MAIN` to a separate queue drained by
`ClientTickEvent.Post` on the physical client main thread. Each queue admits at
most 2048 callbacks and drains at most 256 callbacks from its initial queue
snapshot per tick. Waking a guest never calls it inline; a wake during a poll
yields until a later tick. Both dedicated and integrated servers use their
actual owning thread. A task scheduled in another context or on the `RENDER`
or `NETWORK` lane is rejected. Client modules cannot use the server lane, and
server modules cannot use the client lane. Guest polls and async event callbacks
use the owning main-thread lane; service callbacks remain on `SERVER_MAIN`.
A module granted `task.compute` in a logical-server context may run a named synchronous
`#[compute]` callback in a fresh worker WASM instance. The host copies the
stable-encoded request and response, gives the worker no shared memory or host
imports, and rejects game-thread capabilities on the `WORKER` lane. The
callback is bounded to 256 KiB per request and response, with at most eight
operations per module. The server worker pool has two threads and a bounded
queue of 256 callbacks. Worker timeout or cancellation requests Java thread
interruption as a best-effort stop; the worker slot remains occupied until the
compiled export actually returns. Other guest WASM remains on `SERVER_MAIN`.
Existing Minecraft imports retain their actual server-thread checks.

The adapter uses 128 outstanding tasks per module, a 30-second maximum timeout,
32 queued calls per service endpoint, and the 1 MiB request/response limit.
Closing a module stops guest polling, closes its service operations, unregisters
its provider endpoints, and cancels its task group before invoking `mod_shutdown`
and closing the compiled runtime once. Server or client unbinding discards
queued callbacks and shuts down owned worker/timer executors; a later bind can
create fresh executors.

`ExecutorTaskScheduler.boundedWorkerPool` supplies only the bounded `WORKER`
lane and timeout service. NeoForge registers the actual main-thread queues for
`SERVER_MAIN` and `CLIENT_MAIN`; it does not register `RENDER` or `NETWORK` for
guest task polling. If a lane is not registered, admission fails instead of
silently running the task on a wrong thread. Worker computations must not
directly access world, rendering, or other thread-affine host APIs.

## Admission, completion, and timeout

The host chooses a per-module maximum outstanding task count and maximum
timeout. Queued and running tasks both consume one admission slot. Submission
is accepted only while the module is `RUNNING`; exceeding the limit or a host
scheduler rejecting work returns a rejection. Completion is exposed as a
read-only `CompletionStage`.

Each task receives a cancellation token. Caller cancellation, module shutdown,
or timeout completes the task's result exceptionally and signals that token.
Queued work is removed when the scheduler can guarantee it has not started.
Timeout also requests cancellation for running work. It does not forcibly stop
ordinary guest polling, and it does not release the slot while an uncooperative
running operation is still executing. Isolated worker computations also request
thread interruption, but interruption remains best-effort. The slot is released
when that operation returns, so repeated timeouts cannot make the configured
concurrency bound fictitious.

Cancellation listeners run synchronously on the thread that signals
cancellation. They must return promptly and must not block a game, render, or
shutdown thread. A task should check its token at safe points and use
cancellable host operations. Java thread interruption is not a guaranteed
WebAssembly cancellation mechanism; CPU interruption remains cooperative and
best-effort.

Callback-driven pending operations reserve the same per-module slot while
waiting but do not occupy a lane between callbacks. A wake schedules one later
poll on the configured lane; repeated wakes coalesce, including a wake received
while the guest is polling. Host lane executors must enqueue work and must not
run it inline, because inline execution would re-enter guest code through an
import. The guest scheduler accepts only host callback lanes, never `WORKER`.

Guest code can call `task::cancel(task_id)` to request cancellation of another
task in the same module. The host import reports whether it accepted the request;
the future's drop is queued on its configured lane, so the call never re-enters
the guest cancel export. Cancellation and timeout reject future polls and queue
that export on the same lane. The admission slot remains occupied until queued
polls and cancellation cleanup have finished. Closing the owning module skips
guest cleanup because the WASM instance is being destroyed. Async service
operations use the same rule for their host operation handles; dropping the
Rust service future cancels the handle and signals the provider's cancellation
token.

`task::sleep(Duration)` uses a module-bounded host timer rather than occupying a
worker or blocking a game thread. When it expires, the timer queues a normal
wake for its owning guest task. Dropping the sleep future cancels the timer;
task completion, timeout, and module shutdown also retire any remaining timer
operations. Durations are rounded up to milliseconds and cannot exceed the
module's task timeout.

Guest CPU work should be divided into bounded chunks and await
`task::yield_now()` between chunks. This asks the host to enqueue a later poll;
it does not move the current guest instance to the worker lane. Use a typed
`#[compute]` callback for CPU-only work that can receive and return stable
values. World-dependent work remains on the server lane and should re-resolve
its canonical dimension ID at each boundary so it can complete early after the
level unloads and its old handle is revoked.

## Thread-affine capabilities

Platform adapters must check thread-affine capabilities with
`ModuleInstance.requireCapabilityOnLane`, passing only the lane on which the
underlying API is valid. This rejects calls from worker callbacks before the
adapter resolves or touches a world, render, or other thread-affine object.
The lane itself is supplied by the platform integration; the core does not
invent a game or render executor. Async service providers are dispatched on
their registered lane and sync service calls are accepted only from that lane.

## Cross-module services

For server providers, every declared export must expose the complete byte
callback ABI as `sjml_v1_service_<service-id>`, including the allocator and
input/output deallocators. Registration occurs after successful initialization;
failed activation and module closure remove the endpoints. Async callers invoke
these byte callbacks on the provider's server lane, then wake and re-poll the
caller on its own server lane when the copied response is ready.

Service calls are scoped to modules in the same logical execution context. A
caller needs `service.call`; the provider must declare the service in both
`exports` and `provides`. Public exports are visible to other packages, while
private exports are package-local. Version constraints select the highest
matching declared version. Requests and responses are copied and bounded to
the configured payload limit.

`callSync` must be entered from the provider's declared lane. It starts only
when the endpoint is idle and has no queued async calls; if the endpoint is
occupied, it returns `SERVICE_BUSY` immediately. A sync call does not enter the
FIFO queue or reserve task-group slots.

`callAsync` reserves one pending-task slot in both the caller and provider task
groups. If the endpoint is occupied, it enters the bounded FIFO queue; a full
queue returns `SERVICE_BUSY`. Async handlers return a `CompletionStage` and do
not block the declared lane while the result is pending. Cancellation and
timeout signal the provider and may remove the call from the router's active
slot before uncooperative provider code has actually stopped. Handlers must
check their cancellation token; the router cannot guarantee that timed-out or
cancelled provider code never overlaps a later call if it ignores cancellation.

The scheduler owns its worker and timer executors when created with
`boundedWorkerPool`; closing it requests executor shutdown. A scheduler built
around host-owned executors does not own those executors. Portable-core task
groups belong to individual module instances and are closed during disable,
failure, and shutdown, invalidating queued work and signaling running work for
cancellation.
