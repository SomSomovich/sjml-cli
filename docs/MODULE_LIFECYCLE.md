# Module Lifecycle

The state machine in this document describes the portable core's
`ModuleInstance`. It is not the active NeoForge lifecycle implementation. A
`.sjml` archive is package input to SJML; it is not an FML mod container.
Package discovery and validation do not make guest code runnable until the
host has resolved dependencies and completed each bootstrap stage.

## Bootstrap and shutdown states

| State | Meaning |
| --- | --- |
| `DISCOVERED` | Descriptor is known to the host and the instance has its own execution scope, permissions, handles, and task group. |
| `VALIDATED` | Package and module metadata passed validation. |
| `RESOLVED` | Required module and mod dependencies have a valid load order. |
| `COMPILED` | Compiled-only WASM runtime is ready; interpreter fallback is disabled. |
| `INSTANTIATED` | This instance owns its runtime instance and may create scoped host handles. |
| `REGISTERED` | Static registrations for this instance were accepted by the host. |
| `INITIALIZED` | Module initialization completed. |
| `RUNNING` | The host may admit module-owned asynchronous tasks. |
| `RELOADING` | Resource reload is in progress; it returns to `RUNNING` without registering static content again. |
| `DISABLING` | Shutdown has started and new operations are rejected. |
| `DISABLED` | Shutdown completed after an explicit host disable. |
| `FAILED` | The instance failed and was isolated from other module instances. |
| `CLOSED` | The instance has been closed by its owner. |

In the portable core, normal startup follows `DISCOVERED -> VALIDATED -> RESOLVED -> COMPILED ->
INSTANTIATED -> REGISTERED -> INITIALIZED -> RUNNING`. Any out-of-order
transition is rejected. Resource reload uses `RUNNING -> RELOADING -> RUNNING`.
It does not repeat registration. Disable, failure, or close prevents new task
admission and closes resources idempotently. A failure in one instance does not
change the lifecycle state of other modules.

## Current NeoForge adapter

NeoForge wraps each retained `CompiledModule` in an `ActiveModule`, recording
its package, descriptor, execution context, and whether `mod_init` completed.
On close, an initialized module may run `mod_shutdown` before its compiled
runtime is closed. Static `mod_register` callbacks run earlier in separate
temporary compiled instances. The adapter does not currently use the core
`ModuleInstance` state machine, its handle table, task group, reload state, or
guest async scheduler.

## Physical and logical sides

`CLIENT` is hosted in the physical client process with logical client state.
`INTEGRATED_SERVER` also runs in the physical client process, but owns separate
logical server state. `DEDICATED_SERVER` runs in the physical server process.
The portable core creates a separate `ExecutionScope` for each
physical/logical context; the NeoForge adapter instead retains independent
`ActiveModule` instances for its selected contexts.

A module declared `side = both` is represented by independent instances in the
client and server contexts when both are present. Each instance has its own
runtime, granted capabilities, owner scope token, handles, tasks, lifecycle,
and logs tagged with its execution context. A handle created by one instance
cannot be resolved by the other, including client and integrated-server
instances in the same process. A side-specific module cannot be instantiated
in a context that does not include its declared side.

## Resource invalidation

In the portable core, disable or failure first closes the guest task scheduler,
then attached async resources, the task group, the handle table, and finally
the runtime. Closing the task group rejects new work and requests cancellation;
handles become invalid before runtime teardown. Runtime close errors are logged
and do not prevent the instance from remaining disabled or failed. This order
describes `ModuleInstance`; the current NeoForge `ActiveModule` invokes an
eligible `mod_shutdown` callback, closes its compiled runtime, and then closes
its per-module event-subscription registry.

Task cancellation is cooperative. Closing the runtime can wait for an active
guest invocation to leave Endive's synchronized invocation path. A platform
adapter should run disable/close on a lifecycle control executor instead of a
latency-sensitive game tick. See [the async model](ASYNC_MODEL.md) and
[security boundaries](SECURITY.md) for limits and host obligations.
