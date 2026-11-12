# Live entity attributes

SJML exposes a narrow set of live vanilla attributes for loaded, non-player
living entities:

| Rust value | Registry ID | Writable base range |
| --- | --- | ---: |
| `LivingAttribute::MaxHealth` | `minecraft:generic.max_health` | 1–1024 |
| `LivingAttribute::MovementSpeed` | `minecraft:generic.movement_speed` | 0–1 |
| `LivingAttribute::AttackDamage` | `minecraft:generic.attack_damage` | 0–1024 |
| `LivingAttribute::Armor` | `minecraft:generic.armor` | 0–30 |
| `LivingAttribute::FollowRange` | `minecraft:generic.follow_range` | 0–1024 |
| `LivingAttribute::KnockbackResistance` | `minecraft:generic.knockback_resistance` | 0–1 |

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

The same allowlist is available for a live server player when the module has a
`PlayerHandle` from `sjml::player::online()`. Use `read_player` and
`set_player_base_value`; the existing `read` and `set_base_value` functions
continue to accept only non-player `EntityHandle` values:

```rust
let players = sjml::player::online()?;
if let Some(player) = players.first() {
    let values = sjml::entity_attributes::read_player(*player, LivingAttribute::MaxHealth)?;
    sjml::entity_attributes::set_player_base_value(
        *player,
        LivingAttribute::MaxHealth,
        values.base_value.max(40.0),
    )?;
}
```

Reading requires `entity.read`. Setting a value requires both `entity.read` and
`entity.attributes.write`. The host validates the module-owned live handle,
logical-server context, owner thread, living-entity type, allowlisted attribute,
and numeric range on every call. Attributes missing from a particular living
entity return an error. The API never creates attribute instances or adds
attribute modifiers. The returned effective value includes active modifiers
such as effects; writing the base does not remove them. Entity handles expire
when the entity leaves its level, its world unloads, or the module closes.

Player calls require `entity.read`; player writes also require
`entity.attributes.write`. They are available only to logical-server modules,
on the server main thread, and only while the `ServerPlayer` login session is
live. Client and static-registration calls are denied. This extends player
targets without changing the existing non-player `EntityHandle` behavior.

The ABI uses `sjml_entity_v1.attribute` and `set_attribute_base` for non-player
entity handles, and `sjml_player_v1.attribute` and `set_attribute_base` for
player handles. Both carry a 24-byte handle and a canonical allowlisted ID. A
schema-v1 base-value request is exactly 12 little-endian bytes: `u32 schema =
1`, then `f64 value`. The read response is exactly 20 little-endian bytes:
`u32 schema = 1`, `f64 base_value`, and `f64 effective_value`.
