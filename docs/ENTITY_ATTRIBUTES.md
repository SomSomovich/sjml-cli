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

Reading requires `entity.read`. Setting a value requires both `entity.read` and
`entity.attributes.write`. The host validates the module-owned live handle,
logical-server context, owner thread, living-entity type, allowlisted attribute,
and numeric range on every call. Attributes missing from a particular living
entity return an error. The API never creates attribute instances or adds
attribute modifiers. The returned effective value includes active modifiers
such as effects; writing the base does not remove them. Entity handles expire
when the entity leaves its level, its world unloads, or the module closes.

The ABI uses `sjml_entity_v1.attribute` and `set_attribute_base`. Both carry the
24-byte entity handle and a canonical allowlisted ID. A schema-v1 base-value
request is exactly 12 little-endian bytes: `u32 schema = 1`, then `f64 value`.
The read response is exactly 20 little-endian bytes: `u32 schema = 1`, `f64
base_value`, and `f64 effective_value`.
