# Isolated worker computations

`#[compute(name = "...")]` marks a synchronous, typed Rust callback that can
run as bounded CPU work away from the logical server thread. The macro emits a
byte-callback export and a typed `<function>_compute(input)` helper. Await the
returned future from an ordinary guest task:

```rust
use sjml::prelude::*;

#[derive(StableEncode, StableDecode)]
#[sjml(version = 1, id = "example.digest_request")]
struct DigestRequest {
    bytes: Vec<u8>,
}

#[derive(StableEncode, StableDecode)]
#[sjml(version = 1, id = "example.digest_response")]
struct DigestResponse {
    digest: u64,
}

#[compute(name = "example.digest")]
fn digest(input: DigestRequest) -> SjmlResult<DigestResponse> {
    let digest = input.bytes.iter().fold(0_u64, |state, byte| {
        state.wrapping_mul(31).wrapping_add(u64::from(*byte))
    });
    Ok(DigestResponse { digest })
}

async fn caller(input: DigestRequest) -> SjmlResult<u64> {
    Ok(digest_compute(input)?.await?.digest)
}
```

The module manifest must request the `task.compute` capability and the user
must grant it. Worker computation is available in dedicated and integrated
server contexts. It is not available in the physical client context.

The host serializes the input using SJML's stable value format and copies it to
a fresh WASM instance for each operation. It creates that instance from the
same module bytes, binds every imported host function to a fail-closed handler,
and accepts only the named `sjml_v1_compute_<name>` byte-callback export. The
worker has separate linear memory and module globals; it shares no closure,
reference, guest allocation, or game object with the server instance. A
callback should therefore be a pure computation over its input value.

Requests and responses are each limited to 256 KiB. A module may hold at most
eight worker operations at once, including queued work and results not yet read
by the guest. The server's shared worker pool has two threads and a bounded
queue of 256 callbacks. Computation has the owning guest task's timeout, capped
at 30 seconds. Dropping the future, cancelling or timing out its task, or
closing the module cancels queued work and requests interruption of a running
callback. Interruption is best-effort; an operation continues to occupy its
module quota until its callback actually exits.

With the pinned Endive 1.1.0 compiler, the core test suite forks a compiled
WASM module containing an infinite loop and confirms that interrupting its
worker produces `WasmInterruptedException` and stops the loop. This verifies
the compiled loop back-edge path in the current backend; it does not establish
a hard real-time bound for every generated instruction sequence, JVM/GC pause,
or host operation, so the timeout remains best-effort.

The worker runs under the `WORKER` execution lane, where thread-affine host
capabilities are rejected. It must not read or mutate Minecraft state. Gather
all required values on the server lane, send only stable-encoded data to the
callback, then reacquire current handles after the future resumes.
