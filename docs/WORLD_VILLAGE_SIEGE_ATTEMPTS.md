# Village siege attempts

Server modules can veto a village siege before the zombie raid starts:

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    world_village_siege_attempt::WorldVillageSiegeAttempt,
    SjmlResult,
};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("world.village_siege_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "world.village_siege_attempt")]
fn stop_overworld_sieges(event: WorldVillageSiegeAttempt) -> SjmlResult<bool> {
    Ok(event.dimension_id == "minecraft:overworld")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `VillageSiegeEvent`, which Minecraft posts just before a
zombie siege finds a successful location. Returning `Ok(true)` calls
`setCanceled(true)`, so the siege attempt ends without placing any zombies.
Returning `Ok(false)` keeps vanilla behavior. The event only fires for
integrated or dedicated server contexts, and a capture attempted off the owning
thread is rejected. Already canceled events are ignored.

The detached schema-v1 payload contains the dimension ID, the targeted player's
canonical UUID, and the attempted spawn position `x`, `y`, `z`. Each coordinate
must be finite and within `±30,000,000`. The dimension ID must be a canonical
lowercase resource location; strings are bounded to 256 UTF-8 bytes, the payload
to 1 KiB, and up to 64 subscribed handlers run per attempt. All subscriptions
require both `events.subscribe` and `events.cancel`. `#[event_async]` is rejected
because the veto must apply before the siege starts.

GameTests post a real `VillageSiegeEvent` with a mock server player and verify
that the guest cancels the attempt. Unit tests cover the stable codec, finite and
bound checks, the cancellable NeoForge contract, and detached snapshot values.
