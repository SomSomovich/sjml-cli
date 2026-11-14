# Player statistic changes

`player.statistic_changed` reports committed changes to a live server player's registered Minecraft
statistic. Subscribe with `events.subscribe` and handle the detached `PlayerStatisticChanged` type:

```rust
use sjml::events::PlayerStatisticChanged;
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.statistic_changed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.statistic_changed")]
fn statistic_changed(event: PlayerStatisticChanged) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!(
            "{}: {} / {} changed {} -> {}",
            event.player_id,
            event.statistic_type_id,
            event.statistic_id,
            event.previous_value,
            event.current_value,
        ),
    )?;
    Ok(())
}
```

Receiving the event requires `events.subscribe` and `player.statistics.observe`. The payload contains
the player UUID, registered statistic type and target IDs, and signed 32-bit values before and after
the change. The maximum schema-v1 payload is 572 bytes. For example, vanilla play time is
represented by type `minecraft:custom` and target `minecraft:play_time`; mined blocks use type
`minecraft:mined` and the block's registry ID.

## Hook timing and limits

NeoForge 1.21.1's cancellable `StatAwardEvent` is posted from `StatsCounter#setValue` before the
value is stored. Vanilla `ServerPlayer#awardStat` reaches `ServerStatsCounter#setValue` through the
inherited `StatsCounter#increment`, and `ServerPlayer#resetStat` calls `ServerStatsCounter#setValue`
directly. The host captures
the previous value at the hook, then confirms the resulting value at `ServerTickEvent.Post` before
dispatching the guest notification. It does not call guest code while another mod can still cancel
or rewrite the mutable event.

Canceled writes and writes redirected to a different statistic are omitted. Notifications are
delivered at the end-of-tick drain; writes made after that drain wait for the next one. A bounded
queue holds at most 4,096 attempts per tick. If it overflows, that tick's statistic notifications
are dropped as a batch so the guest does not receive a partial stream. Direct mutations that bypass
Minecraft's `StatsCounter#setValue` path and changes loaded from the player stats file are not events.

This is a read-only notification. Existing statistic reads and writes remain in
[`PLAYER_STATISTICS.md`](PLAYER_STATISTICS.md).

`PlayerStatisticChangedGameTests` checks the actual dedicated-server hook for both
`ServerPlayer#awardStat` and `ServerPlayer#resetStat`.
