# Player custom statistics

The player API can read and increment the vanilla 1.21.1 `Stats.CUSTOM` counters for a live server player. Calls are bound to the player's active login session and run on the owning logical-server thread.

```rust
use sjml::{player, PlayerCustomStat};

let deaths = player::custom_stat(player_handle, PlayerCustomStat::Deaths)?;
player::increment_custom_stat(player_handle, PlayerCustomStat::Deaths, 1)?;
```

Reads require `entity.read`. Increments require both `entity.read` and `player.statistics.write`, declared by the module and granted by the server administrator. These imports are server-only. Client modules, stale handles, static registration, and calls off the owning server thread are denied.

`player::set_custom_stat` can assign an exact value to one supported custom counter. For
registry-backed item, block, and entity counters, `player_statistics::set_value` provides the same
operation for a typed `PlayerStatistic`. Both setters require `entity.read` and
`player.statistics.write`, accept only `0..=i32::MAX`, and reject unregistered statistic targets.
They return the value that the server actually stored after NeoForge `StatAwardEvent` listeners
run. The host routes the change through the server's `ServerStatsCounter`, updates matching
scoreboard statistic objectives, and sends the resulting dirty stat to the client; the server's
normal player save path persists it. A listener may cancel or alter the requested value, so compare
the returned value with the requested value when that distinction matters.

```rust
use sjml::{player, player_statistics, PlayerCustomStat, PlayerStatistic, StatisticFamily};

let custom_value = player::set_custom_stat(player_handle, PlayerCustomStat::Deaths, 25)?;
let diamond_uses = PlayerStatistic::new(StatisticFamily::ItemUsed, "minecraft:diamond")?;
let item_value = player_statistics::set_value(player_handle, &diamond_uses, 10)?;
```

Each increment must be between `1` and `player::MAX_PLAYER_STATISTIC_INCREMENT` (`1,000,000`). Minecraft stores these counters as signed 32-bit integers; the host rejects an increment that would raise the current value above `i32::MAX`. Reads return the stored nonnegative integer as `u32`.

## Supported counters

`PlayerCustomStat` names the vanilla custom counters registered by Minecraft 1.21.1. Its `id()` method returns the canonical `minecraft:` ID.

| Rust variant | ID |
| --- | --- |
| `LeaveGame` | `minecraft:leave_game` |
| `PlayTime` | `minecraft:play_time` |
| `TotalWorldTime` | `minecraft:total_world_time` |
| `TimeSinceDeath` | `minecraft:time_since_death` |
| `TimeSinceRest` | `minecraft:time_since_rest` |
| `SneakTime` | `minecraft:sneak_time` |
| `WalkOneCm` | `minecraft:walk_one_cm` |
| `CrouchOneCm` | `minecraft:crouch_one_cm` |
| `SprintOneCm` | `minecraft:sprint_one_cm` |
| `WalkOnWaterOneCm` | `minecraft:walk_on_water_one_cm` |
| `FallOneCm` | `minecraft:fall_one_cm` |
| `ClimbOneCm` | `minecraft:climb_one_cm` |
| `FlyOneCm` | `minecraft:fly_one_cm` |
| `WalkUnderWaterOneCm` | `minecraft:walk_under_water_one_cm` |
| `MinecartOneCm` | `minecraft:minecart_one_cm` |
| `BoatOneCm` | `minecraft:boat_one_cm` |
| `PigOneCm` | `minecraft:pig_one_cm` |
| `HorseOneCm` | `minecraft:horse_one_cm` |
| `AviateOneCm` | `minecraft:aviate_one_cm` |
| `SwimOneCm` | `minecraft:swim_one_cm` |
| `StriderOneCm` | `minecraft:strider_one_cm` |
| `Jump` | `minecraft:jump` |
| `Drop` | `minecraft:drop` |
| `DamageDealt` | `minecraft:damage_dealt` |
| `DamageDealtAbsorbed` | `minecraft:damage_dealt_absorbed` |
| `DamageDealtResisted` | `minecraft:damage_dealt_resisted` |
| `DamageTaken` | `minecraft:damage_taken` |
| `DamageBlockedByShield` | `minecraft:damage_blocked_by_shield` |
| `DamageAbsorbed` | `minecraft:damage_absorbed` |
| `DamageResisted` | `minecraft:damage_resisted` |
| `Deaths` | `minecraft:deaths` |
| `MobKills` | `minecraft:mob_kills` |
| `AnimalsBred` | `minecraft:animals_bred` |
| `PlayerKills` | `minecraft:player_kills` |
| `FishCaught` | `minecraft:fish_caught` |
| `TalkedToVillager` | `minecraft:talked_to_villager` |
| `TradedWithVillager` | `minecraft:traded_with_villager` |
| `EatCakeSlice` | `minecraft:eat_cake_slice` |
| `FillCauldron` | `minecraft:fill_cauldron` |
| `UseCauldron` | `minecraft:use_cauldron` |
| `CleanArmor` | `minecraft:clean_armor` |
| `CleanBanner` | `minecraft:clean_banner` |
| `CleanShulkerBox` | `minecraft:clean_shulker_box` |
| `InteractWithBrewingStand` | `minecraft:interact_with_brewingstand` |
| `InteractWithBeacon` | `minecraft:interact_with_beacon` |
| `InspectDropper` | `minecraft:inspect_dropper` |
| `InspectHopper` | `minecraft:inspect_hopper` |
| `InspectDispenser` | `minecraft:inspect_dispenser` |
| `PlayNoteBlock` | `minecraft:play_noteblock` |
| `TuneNoteBlock` | `minecraft:tune_noteblock` |
| `PotFlower` | `minecraft:pot_flower` |
| `TriggerTrappedChest` | `minecraft:trigger_trapped_chest` |
| `OpenEnderChest` | `minecraft:open_enderchest` |
| `EnchantItem` | `minecraft:enchant_item` |
| `PlayRecord` | `minecraft:play_record` |
| `InteractWithFurnace` | `minecraft:interact_with_furnace` |
| `InteractWithCraftingTable` | `minecraft:interact_with_crafting_table` |
| `OpenChest` | `minecraft:open_chest` |
| `SleepInBed` | `minecraft:sleep_in_bed` |
| `OpenShulkerBox` | `minecraft:open_shulker_box` |
| `OpenBarrel` | `minecraft:open_barrel` |
| `InteractWithBlastFurnace` | `minecraft:interact_with_blast_furnace` |
| `InteractWithSmoker` | `minecraft:interact_with_smoker` |
| `InteractWithLectern` | `minecraft:interact_with_lectern` |
| `InteractWithCampfire` | `minecraft:interact_with_campfire` |
| `InteractWithCartographyTable` | `minecraft:interact_with_cartography_table` |
| `InteractWithLoom` | `minecraft:interact_with_loom` |
| `InteractWithStonecutter` | `minecraft:interact_with_stonecutter` |
| `BellRing` | `minecraft:bell_ring` |
| `RaidTrigger` | `minecraft:raid_trigger` |
| `RaidWin` | `minecraft:raid_win` |
| `InteractWithAnvil` | `minecraft:interact_with_anvil` |
| `InteractWithGrindstone` | `minecraft:interact_with_grindstone` |
| `TargetHit` | `minecraft:target_hit` |
| `InteractWithSmithingTable` | `minecraft:interact_with_smithing_table` |

The host checks every ID against the vanilla custom-stat allowlist and the live `BuiltInRegistries.CUSTOM_STAT` registry. It does not accept arbitrary strings or mod-defined counters.

## Registry-backed statistic families

For block and entity counters, use the separate typed `player_statistics` API. A family selects the
registry type, and its `registry_id` is checked against that registry before the operation:

```rust
use sjml::{player_statistics, PlayerStatistic, StatisticFamily};

let diamond_uses = PlayerStatistic::new(StatisticFamily::ItemUsed, "minecraft:diamond")?;
let current = player_statistics::read(player_handle, &diamond_uses)?;
player_statistics::increment(player_handle, &diamond_uses, 1)?;
```

The supported families are `BlockMined`, `ItemUsed`, `ItemBroken`, `ItemCrafted`, `EntityKilled`,
and `EntityKilledBy`. `ITEM_PICKED_UP` and `ITEM_DROPPED` remain unavailable. Registry-backed
reads require `entity.read`; increments also require `player.statistics.write`. IDs are canonical
`namespace:path` values and are verified against the selected live vanilla registry, so an item ID
cannot be used for a block statistic.

Each increment is limited to `0..=1,000,000`; zero is a no-op. Minecraft stores these values as
signed 32-bit integers, so the host rejects overflow. The same server-thread and current
login-session checks as custom counters apply.
