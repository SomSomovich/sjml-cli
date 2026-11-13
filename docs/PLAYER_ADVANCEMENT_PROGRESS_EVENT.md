# Advancement criterion progress events

`player.advancement_progress` is a logical-server notification for an individual
advancement criterion being granted or revoked. NeoForge 21.1.265 exposes it as
`AdvancementEvent.AdvancementProgressEvent` on the main event bus. The event is
not cancellable and does not expose a way to change the progress.

Subscribe from a server module initializer and handle the detached snapshot:

```rust
use sjml::{event, events::{self, AdvancementProgressOperation, EventPriority,
    PlayerAdvancementProgressEvent}, mod_init, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.advancement_progress", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.advancement_progress")]
fn on_progress(event: PlayerAdvancementProgressEvent) -> SjmlResult<()> {
    match event.operation {
        AdvancementProgressOperation::Grant => {
            // event.is_done describes the advancement after this grant.
        }
        AdvancementProgressOperation::Revoke => {
            // event.is_done describes the advancement after this revoke.
        }
    }
    Ok(())
}
```

`PlayerAdvancementProgressEvent` contains the canonical player UUID, canonical
advancement registry ID, criterion name, operation, and `is_done`. NeoForge
reports the `AdvancementProgress` supplied by its event after the grant or
revoke operation has updated criterion state. A multi-criterion advancement
can remain complete after one criterion is revoked if its requirements are
still met.

The callback requires both `events.subscribe` and the separate
`player.advancement.observe` capability. It runs on the owning logical-server
thread and can be declared with `#[event_async]` because it is observation-only.
All strings are bounded to 256 UTF-8 bytes, IDs are validated, and the schema-v1
payload is limited to 2 KiB. The wire layout is a version `u32(1)`, three
length-prefixed strings (`player_id`, `advancement_id`, `criterion_id`), an
operation `u32` (`0=Grant`, `1=Revoke`), and a boolean byte (`is_done`).

The live `GameplayEventGameTests` path grants and revokes `crafting_table` on
`minecraft:story/root` through `ServerPlayer`'s advancement API, then checks the
guest callback observed the grant with `is_done=true` and the revoke with
`is_done=false`.
