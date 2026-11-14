# Block drop attempts

`block.drops_attempt` is a synchronous, cancellable logical-server event backed by NeoForge's
`BlockDropsEvent`. NeoForge posts it after computing a broken block's drops and before spawning the
drop entities. Returning `true` removes the event's already-computed item drops and sets its
experience amount to zero. It does not cancel the NeoForge event, undo the block break, or suppress
the block's `spawnAfterBreak` hook. An event already canceled by another listener is left untouched
and is not sent to guest callbacks.

```rust
use sjml::{
    event,
    events::{self, BlockDropsAttempt, EventPriority},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.drops_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.drops_attempt")]
fn suppress_sample_ore_drops(attempt: BlockDropsAttempt) -> SjmlResult<bool> {
    Ok(attempt.block_id == "example:sample_ore")
}
```

Declare the event capabilities on a server module and have the server operator grant them:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["events.subscribe", "events.cancel"]
```

```toml
[[grant]]
module = "example_mod/main"
capabilities = ["events.subscribe", "events.cancel"]
```

The snapshot contains the dimension, block ID and position, optional breaker UUID/type, optional
tool snapshot, up to 32 computed item snapshots, the computed experience amount, and a truncation
flag. Item snapshots include only registry ID and count, plus optional durability for damageable
items; they omit item components and NBT. The payload is capped at 16 KiB, each resource ID at 256
UTF-8 bytes, and each represented item count and durability value at one million. Guest callbacks
run synchronously on the owning server
thread and are capped at 64 per drop event. Async handlers are rejected. Both capabilities are
required, and the event cannot be subscribed to from a client module.

The standalone SDK example is
[`block_drops_attempt.rs`](../sjml-cli/sjml-sdk/examples/block_drops_attempt.rs).
