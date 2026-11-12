# Integer world gamerules

The SDK exposes a separate typed API for four vanilla integer gamerules:

```rust
use sjml::prelude::*;

fn disable_random_ticks(world: WorldHandle) -> SjmlResult<()> {
    let speed = world::integer_game_rule(world, WorldIntegerGameRule::RandomTickSpeed)?;
    world::set_integer_game_rule(world, WorldIntegerGameRule::RandomTickSpeed, 0)?;
    Ok(())
}
```

Each identifier maps to a vanilla `GameRules.Key<GameRules.IntegerValue>` in
Minecraft 1.21.1: `RULE_MAX_ENTITY_CRAMMING`,
`RULE_PLAYERS_SLEEPING_PERCENTAGE`, `RULE_RANDOMTICKING`, and
`RULE_SPAWN_RADIUS`. These IDs are a stable SJML allowlist, separate from the
boolean gamerule IDs and Minecraft field ordinals.

| Rust variant | Vanilla rule | Accepted values for writes |
| --- | --- | ---: |
| `MaxEntityCramming` | `maxEntityCramming` | 0–128 |
| `PlayersSleepingPercentage` | `playersSleepingPercentage` | 0–100 |
| `RandomTickSpeed` | `randomTickSpeed` | 0–4,096 |
| `SpawnRadius` | `spawnRadius` | 0–64 |

Reads require `world.read`. They return the exact integer currently stored by
Minecraft, even if a console, another mod, or a save file set it outside the
SDK's write range. Writes require both `world.read` and `world.write`; the
selected rule's range is checked before reaching Minecraft. The value is
updated through `GameRules.IntegerValue.set(value, server)`, preserving
Minecraft's normal gamerule persistence and callbacks. Both operations require
a loaded logical-server dimension and the owning server thread. Client and
static registration contexts are denied.

## ABI v1

The ABI uses separate imports, `sjml_world_v1.integer_game_rule` and
`sjml_world_v1.set_integer_game_rule`; existing boolean-rule imports are
unchanged. Both requests are little-endian stable schema-v1 records. A query is
32 bytes: schema `u32`, world handle (24 bytes), and stable rule ID `u32`. The
read import writes an 8-byte response: schema `u32` followed by the exact
Minecraft value `i32`. A mutation is 36 bytes: the query followed by value
`i32`. Unknown IDs, incorrect lengths, unsupported schema versions, and
out-of-range writes are rejected before world mutation.
