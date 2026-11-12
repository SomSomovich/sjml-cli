# Cancellable fishing catches

`player.fished_attempt` lets a logical-server module veto a fishing catch before
NeoForge applies its drops. Subscribe in `mod_init` and return `Ok(true)` to
suppress the catch:

```rust
use sjml::{event, events::{self, EventPriority, PlayerFishedAttempt}, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.fished_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.fished_attempt")]
fn prevent_cod(event: PlayerFishedAttempt) -> SjmlResult<bool> {
    Ok(event.drops.iter().any(|drop| drop.item_id == "minecraft:cod"))
}
```

The module must request and receive both `events.subscribe` and `events.cancel`.
The callback is synchronous because its boolean controls the live event, so it
cannot use `#[event_async]`. A veto suppresses the event's drops; NeoForge still
applies the rod damage requested by the event. The callback cannot change the
drop list or rod damage. A callback trap or malformed return disables only that
module and does not cancel the catch on its behalf.

The host listens to NeoForge `ItemFishedEvent` at `LOWEST`, ignores events that
another listener already canceled, and routes the callback only to subscribed
modules in the owning logical-server context. It captures the event on the
server thread. Physical-client event subscriptions are not supported for this
server-only callback.

`PlayerFishedAttempt` contains the canonical player UUID, dimension ID, fishing
hook coordinates, requested rod damage, up to 16 detached item ID/count
snapshots, and `drops_truncated`. The truncation flag is true when NeoForge's
source drop list has entries beyond the first 16 inspected. `FishedDropSnapshot`
contains only a canonical item registry ID and count; item components and NBT
are not copied. Resource strings are limited to 256 UTF-8 bytes, hook
coordinates to finite values in ±30,000,000, requested rod damage to
`0..=1,000,000`, and each nonempty stack count to `1..=1,000,000`. The complete
schema-v1 payload is limited to 8 KiB.

Schema v1 is little-endian: `u32` version `1`, stable UTF-8 strings
`player_id` and `dimension_id`, `f64` hook X/Y/Z, `u32` rod damage, `u32` drop
count, nested `FishedDropSnapshot` values, then a strict boolean byte for
`drops_truncated`. Each nested snapshot is schema version `1`, a stable UTF-8
`item_id`, and `u32` count. Each string uses a `u32` byte length followed by
its UTF-8 bytes.

`PlayerFishedAttemptGameTests` posts a real `ItemFishedEvent` through NeoForge's
event bus with the SJML guest test module armed. It verifies the guest receives
the detached snapshot and vetoes the catch, while the event's rod damage and
drop list remain unchanged.
