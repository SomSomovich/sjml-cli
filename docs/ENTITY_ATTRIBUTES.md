# Live entity attributes

SJML exposes 31 bounded vanilla attributes to logical-server modules. Calls
target an already-loaded non-player living entity or an online server player.
The Rust `LivingAttribute` enum names the stable registry IDs and the host
applies a per-attribute write range:

| Rust value | Registry ID | Writable base range |
| --- | --- | ---: |
| `LivingAttribute::Armor` | `minecraft:generic.armor` | 0–30 |
| `LivingAttribute::ArmorToughness` | `minecraft:generic.armor_toughness` | 0–20 |
| `LivingAttribute::AttackDamage` | `minecraft:generic.attack_damage` | 0–1024 |
| `LivingAttribute::AttackKnockback` | `minecraft:generic.attack_knockback` | 0–5 |
| `LivingAttribute::AttackSpeed` | `minecraft:generic.attack_speed` | 0–1024 |
| `LivingAttribute::BlockBreakSpeed` | `minecraft:player.block_break_speed` | 0–1024 |
| `LivingAttribute::BlockInteractionRange` | `minecraft:player.block_interaction_range` | 0–64 |
| `LivingAttribute::BurningTime` | `minecraft:generic.burning_time` | 0–1024 |
| `LivingAttribute::EntityInteractionRange` | `minecraft:player.entity_interaction_range` | 0–64 |
| `LivingAttribute::ExplosionKnockbackResistance` | `minecraft:generic.explosion_knockback_resistance` | 0–1 |
| `LivingAttribute::FallDamageMultiplier` | `minecraft:generic.fall_damage_multiplier` | 0–100 |
| `LivingAttribute::FlyingSpeed` | `minecraft:generic.flying_speed` | 0–1024 |
| `LivingAttribute::FollowRange` | `minecraft:generic.follow_range` | 0–1024 |
| `LivingAttribute::Gravity` | `minecraft:generic.gravity` | -1–1 |
| `LivingAttribute::JumpStrength` | `minecraft:generic.jump_strength` | 0–32 |
| `LivingAttribute::KnockbackResistance` | `minecraft:generic.knockback_resistance` | 0–1 |
| `LivingAttribute::Luck` | `minecraft:generic.luck` | -1024–1024 |
| `LivingAttribute::MaxAbsorption` | `minecraft:generic.max_absorption` | 0–2048 |
| `LivingAttribute::MaxHealth` | `minecraft:generic.max_health` | 1–1024 |
| `LivingAttribute::MiningEfficiency` | `minecraft:player.mining_efficiency` | 0–1024 |
| `LivingAttribute::MovementEfficiency` | `minecraft:generic.movement_efficiency` | 0–1 |
| `LivingAttribute::MovementSpeed` | `minecraft:generic.movement_speed` | 0–1 |
| `LivingAttribute::OxygenBonus` | `minecraft:generic.oxygen_bonus` | 0–1024 |
| `LivingAttribute::SafeFallDistance` | `minecraft:generic.safe_fall_distance` | -1024–1024 |
| `LivingAttribute::Scale` | `minecraft:generic.scale` | 0.0625–16 |
| `LivingAttribute::SneakingSpeed` | `minecraft:player.sneaking_speed` | 0–1 |
| `LivingAttribute::SpawnReinforcementsChance` | `minecraft:zombie.spawn_reinforcements` | 0–1 |
| `LivingAttribute::StepHeight` | `minecraft:generic.step_height` | 0–10 |
| `LivingAttribute::SubmergedMiningSpeed` | `minecraft:player.submerged_mining_speed` | 0–20 |
| `LivingAttribute::SweepingDamageRatio` | `minecraft:player.sweeping_damage_ratio` | 0–1 |
| `LivingAttribute::WaterMovementEfficiency` | `minecraft:generic.water_movement_efficiency` | 0–1 |

Read a detached base/effective-value snapshot with `sjml::entity_attributes::read`:

```rust
use sjml::entity_attributes::LivingAttribute;

let values = sjml::entity_attributes::read(entity, LivingAttribute::MaxHealth)?;
sjml::host::log(
    sjml::host::LogLevel::Info,
    &format!("max health base={} current={}", values.base_value, values.value),
)?;
```

Change only the base value with `set_base_value`:

```rust
sjml::entity_attributes::set_base_value(entity, LivingAttribute::MaxHealth, 40.0)?;
```

The same allowlist is available for an online server player with a
`PlayerHandle`. Use `read_player` and `set_player_base_value`; the non-player
functions continue to accept only `EntityHandle` values. Player reach is a
player-only attribute, for example:

```rust
let players = sjml::player::online()?;
if let Some(player) = players.first() {
    let reach = sjml::entity_attributes::read_player(
        *player,
        LivingAttribute::BlockInteractionRange,
    )?;
    sjml::entity_attributes::set_player_base_value(
        *player,
        LivingAttribute::BlockInteractionRange,
        reach.base_value.min(64.0),
    )?;
}
```

Reading requires `entity.read`. Setting a value requires both `entity.read` and
`entity.attributes.write`. The host checks the module-owned live handle,
logical-server context, owner thread, living-entity type, exact vanilla
allowlist, numeric range, and presence of the requested attribute on each call.
Player-only attributes are absent on most mobs and return an error when queried
there. The API never creates attribute instances or adds modifiers. The
effective value includes active modifiers; writing the base leaves those
modifiers intact. Entity handles expire when the entity leaves its level, its
world unloads, or the module closes.

Player calls are available only to logical-server modules on the server main
thread and while the `ServerPlayer` login session is live. Client and
static-registration calls are denied. These calls do not change the separate
client-side player view.

The ABI uses `sjml_entity_v1.attribute` and `set_attribute_base` for non-player
entities and the matching imports from `sjml_player_v1` for players. Both carry a
24-byte handle and a canonical allowlisted ID. A schema-v1 base-value request
is exactly 12 little-endian bytes (`u32 schema = 1`, then `f64 value`). A read
response is exactly 20 little-endian bytes (`u32 schema = 1`, `f64 base_value`,
`f64 effective_value`).
