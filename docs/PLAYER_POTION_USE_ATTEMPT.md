# Player potion-use attempt event

`player.potion_use_attempt` is a synchronous, cancellable logical-server event emitted by NeoForge `LivingEntityUseItemEvent.Start`. It runs before Minecraft records the item as being used and before a potion can finish drinking or apply its effects. Returning `PlayerPotionUseAttemptAction::Cancel` prevents the use from starting.

The host includes only `PotionItem` instances, including subclasses registered by other mods. It does not include milk, food, bows, or generic right-clicks. The snapshot contains a player UUID, dimension ID, registry item ID, stack count, use duration in ticks, and `main_hand` or `off_hand`. It contains no live Minecraft objects or item components.

The callback runs on the server thread and is synchronous. Async handlers cannot cancel this event. A module needs both `events.subscribe` and `events.cancel`.

```rust
use sjml::{
    event,
    events::{self, EventPriority, PlayerPotionUseAttempt, PlayerPotionUseAttemptAction},
    mod_init, SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.potion_use_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.potion_use_attempt")]
fn deny_regular_potion(attempt: PlayerPotionUseAttempt) -> SjmlResult<PlayerPotionUseAttemptAction> {
    if attempt.item_id == "minecraft:potion" {
        Ok(PlayerPotionUseAttemptAction::Cancel)
    } else {
        Ok(PlayerPotionUseAttemptAction::Pass)
    }
}
```

`count` is bounded to `1..=64`; `duration_ticks` is bounded to `1..=72_000`; item IDs are canonical resource locations up to 256 UTF-8 bytes. A single event invokes at most 64 authorized module callbacks. The response is one stable byte: `Pass = 0`, `Cancel = 1`.

This callback runs only when use starts. It does not cancel an already active use or undo effects from a potion that has finished.
