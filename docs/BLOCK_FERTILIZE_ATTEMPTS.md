# Bone-meal attempt events

Server modules can observe and veto vanilla bone-meal attempts with the
synchronous `block.fertilize_attempt` event:

```toml
permissions = ["events.subscribe", "events.cancel"]
```

```rust
use sjml::{BlockFertilizeAttempt, SjmlResult};

#[sjml::event(name = "block.fertilize_attempt")]
fn avoid_invalid_targets(attempt: BlockFertilizeAttempt) -> SjmlResult<bool> {
    // A dispenser or another non-player source has no player UUID.
    Ok(!attempt.valid_target)
}
```

The event uses NeoForge's cancellable `BonemealEvent` on the logical server.
It includes the optional player UUID, dimension, block position and ID, the
fertilizer item ID and stack count, and Minecraft's `isValidBonemealTarget`
result. It fires for player and non-player attempts, including attempts whose
target is not eligible for fertilization.

Return `Ok(true)` to call `setSuccessful(false)` on the NeoForge event, cancel
vanilla application, and keep the fertilizer stack untouched. `Ok(false)`
leaves the existing event result alone. An event another mod already canceled
is not sent to guest code. The callback runs synchronously on the owning server
thread and receives detached values only; it cannot edit the item stack, block
state, or world. Callback failures do not veto vanilla behavior.

The SDK bounds strings to 256 UTF-8 bytes, fertilizer count to 0–99, and the
full payload to 2 KiB. The Rust callback cannot be asynchronous because
Minecraft needs the decision before applying bone meal.
