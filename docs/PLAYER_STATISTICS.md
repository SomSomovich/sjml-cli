# Player custom statistics

The player API can read and increment the vanilla 1.21.1 `Stats.CUSTOM` counters for a live server player. Calls are bound to the player's active login session and run on the owning logical-server thread.

```rust
use sjml::{player, PlayerCustomStat};

let deaths = player::custom_stat(player_handle, PlayerCustomStat::Deaths)?;
player::increment_custom_stat(player_handle, PlayerCustomStat::Deaths, 1)?;
```

Reads require `entity.read`. Increments require both `entity.read` and `player.statistics.write`, declared by the module and granted by the server administrator. These imports are server-only. Client modules, stale handles, static registration, and calls off the owning server thread are denied.

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

## Unsupported statistic types

The API does not expose the other `Stats` groups: `BLOCK_MINED`, `ITEM_CRAFTED`, `ITEM_USED`, `ITEM_BROKEN`, `ITEM_PICKED_UP`, `ITEM_DROPPED`, `ENTITY_KILLED`, and `ENTITY_KILLED_BY`. Those are registry-backed `StatType<T>` values with block, item, or entity keys; they are not entries in `Stats.CUSTOM` and are deliberately omitted from `PlayerCustomStat`.

Vanilla can update counters such as play time, movement, and time-since-death during normal gameplay. An increment writes through `ServerPlayer.awardStat`, keeping the normal vanilla stat, scoreboard, and save path.
