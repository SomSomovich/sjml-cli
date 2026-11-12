# World gamerules

The world API exposes a fixed allowlist of vanilla boolean gamerules for a world that is already
loaded on the logical server. The Rust SDK provides:

```rust
let enabled = sjml::world::game_rule(world, sjml::world::WorldGameRule::KeepInventory)?;
sjml::world::set_game_rule(
    world,
    sjml::world::WorldGameRule::KeepInventory,
    true,
)?;
```

Reads require `world.read`. Writes require both `world.read` and `world.write`, because the write
also validates a module-owned world handle. Calls run synchronously on the owning server thread and
are available only to server execution contexts. Client execution and static `mod_register`
registration deny access. Each handle addresses one loaded dimension; the value is read or changed
through that dimension's vanilla `GameRules` instance and Minecraft persists changes through its
normal gamerule path. These calls never load a dimension or chunk.

The allowlist is deliberately limited to boolean rules. Integer-valued rules such as random tick
speed are not exposed, so a guest cannot pass out-of-range values or change gamerules that have not
been reviewed for this ABI.

| Stable ID | Rust variant | Vanilla rule |
| ---: | --- | --- |
| 0 | `DaylightCycle` | `doDaylightCycle` |
| 1 | `WeatherCycle` | `doWeatherCycle` |
| 2 | `MobSpawning` | `doMobSpawning` |
| 3 | `MobGriefing` | `mobGriefing` |
| 4 | `KeepInventory` | `keepInventory` |
| 5 | `FireTick` | `doFireTick` |
| 6 | `MobLoot` | `doMobLoot` |
| 7 | `BlockDrops` | `doTileDrops` |
| 8 | `EntityDrops` | `doEntityDrops` |
| 9 | `NaturalRegeneration` | `naturalRegeneration` |
| 10 | `DisableRaids` | `disableRaids` |
| 11 | `AnnounceAdvancements` | `announceAdvancements` |
| 12 | `ShowDeathMessages` | `showDeathMessages` |
| 13 | `Insomnia` | `doInsomnia` |
| 14 | `ImmediateRespawn` | `doImmediateRespawn` |
| 15 | `PatrolSpawning` | `doPatrolSpawning` |
| 16 | `TraderSpawning` | `doTraderSpawning` |
| 17 | `WardenSpawning` | `doWardenSpawning` |

## ABI v1 wire format

All integers are little-endian. The `sjml_world_v1.game_rule` import takes a pointer and length for
an exact 32-byte query, followed by a pointer and capacity for an exact 5-byte response. The query
contains a 32-bit schema version (`1`), the 24-byte world handle, and a 32-bit allowlist ID. The
response contains schema version `1` and one strict boolean byte (`0` or `1`).

The `sjml_world_v1.set_game_rule` import takes a pointer and length for an exact 33-byte mutation:
the same query fields followed by one strict boolean byte. Unknown IDs, different schema versions,
trailing or missing bytes, and boolean bytes other than zero or one are rejected before world
access. The ID values are fixed ABI values; they are not Minecraft field ordinals.
