# Player totem-use attempt

`player.totem_use_attempt` is a synchronous logical-server callback for a
player about to use a totem to prevent death. It is backed by NeoForge
21.1.265's cancellable `LivingUseTotemEvent`, which runs before the totem effect
is applied. The event is emitted only on the logical server. Non-player
entities are not delivered to this player event.

Subscribe with `events::subscribe` and return
`PlayerTotemUseAttemptAction::Pass` or `Cancel` from a synchronous `#[event]`
handler. A cancellation prevents the totem from saving the player, so the
pending damage and death continue. The event cannot change the damage, the
totem, the hand, or the player's inventory. Multiple authorized callbacks are
run in subscription-priority order; any `Cancel` vetoes the use. At most 64
callbacks are run for one attempt.

The detached schema-v1 snapshot carries the player UUID, dimension ID, damage
type ID, totem item ID and count, and the hand. When damage has a causing entity,
the snapshot also includes its UUID and entity-type ID; environmental damage
uses `None` for both fields. No Minecraft object or mutable stack crosses the
ABI. Each string is bounded to 256 UTF-8 bytes and the full payload is limited
to 2 KiB. The count range is 1–64.

The handler requires both `events.subscribe` and `events.cancel`. It cannot run
as `#[event_async]` because NeoForge requires the decision before the totem
effect proceeds.

```rust
use sjml::{
    event,
    events::{self, EventPriority, PlayerTotemUseAttempt, PlayerTotemUseAttemptAction},
    SjmlResult,
};

pub fn initialize() -> SjmlResult<()> {
    events::subscribe("player.totem_use_attempt", EventPriority::Normal)
}

#[event(name = "player.totem_use_attempt")]
fn reject_void_damage(event: PlayerTotemUseAttempt) -> SjmlResult<PlayerTotemUseAttemptAction> {
    Ok(if event.damage_type_id == "minecraft:out_of_world" {
        PlayerTotemUseAttemptAction::Cancel
    } else {
        PlayerTotemUseAttemptAction::Pass
    })
}
```

The SDK example is [`player_totem_use_attempt.rs`](../sjml-cli/sjml-sdk/examples/player_totem_use_attempt.rs).
