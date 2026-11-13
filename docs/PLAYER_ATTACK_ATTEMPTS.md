# Player attack attempts

`player.attack_attempt` is a synchronous cancellable event raised from
NeoForge's `AttackEntityEvent` when a player calls `Player.attack` on the
logical server. A handler returning `Ok(true)` cancels that attack before
Minecraft runs the weapon hook or applies damage. The event is useful for
server rules such as protected targets or per-player attack restrictions.

Subscribe from a retained server module and grant both event capabilities:

```rust
use sjml::{event, events::{self, EventPriority, PlayerAttackAttempt}, SjmlResult};

pub fn initialize() -> SjmlResult<()> {
    events::subscribe("player.attack_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.attack_attempt")]
fn protect_target(event: PlayerAttackAttempt) -> SjmlResult<bool> {
    Ok(event.target_type_id == "minecraft:villager")
}
```

`PlayerAttackAttempt` contains the attacker's UUID, target UUID and entity type
ID, the shared dimension ID, and the target's X/Y/Z position at the hook.
These are detached, bounded values. The event does not expose a live entity,
damage amount, weapon stack, or mutable NeoForge fields. The callback runs
synchronously on the owning logical-server thread. It requires
`events.subscribe` and `events.cancel`; events already canceled by another
listener are not sent to guest modules. Callback failures fail open and disable
the failing module. See the [ABI callback contract](ABI.md).
