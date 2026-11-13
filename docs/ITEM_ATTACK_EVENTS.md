# Registered item attack callbacks

`item.attack_attempt` lets a module observe and veto a server-player attack made
while one of that module's registered items is in the main hand. It runs after
the general `player.attack_attempt` handlers have allowed the event and before
vanilla attack processing.

```rust
use sjml::{event, ModuleItemAttackAttempt, SjmlResult};

#[event(name = "item.attack_attempt")]
fn protect_friendly_target(event: ModuleItemAttackAttempt) -> SjmlResult<bool> {
    Ok(event.target_type_id == "minecraft:villager")
}
```

The host calls only the module whose reserved `package:module/path` namespace
owns `event.item_id`; other modules cannot inspect or veto the attack through
this callback. The detached schema contains player and target UUIDs, dimension
and entity-type IDs, the registered item ID and count, and target coordinates.
It does not expose a Minecraft object, attack damage, arbitrary item
components, or mutable stack access. Counts are bounded to `1..=99`.

Declare `events.subscribe` and `events.cancel` in the module manifest and grant
both on the server. The callback must be synchronous and return
`SjmlResult<bool>`: `Ok(true)` cancels this attack, while `Ok(false)` allows the
normal attack flow to continue. Async handlers are rejected because the
NeoForge attack hook must resolve cancellation before Minecraft processes the
attack. A failure disables the module using the normal event-callback failure
policy.

Only server-player attacks are delivered. The event is skipped when an earlier
listener has already canceled the attack, when either entity is outside the
same server level, or when the main-hand stack has no canonical item registry
ID. The general `player.attack_attempt` remains independent and can veto any
player attack before this owner-scoped callback is considered.

The ABI snapshot and permission boundary have Java and Rust contract coverage.
`ModuleItemAttackAttemptGameTests` runs a registered test item through the real
server attack hook and verifies that the owning guest veto prevents vanilla damage.
