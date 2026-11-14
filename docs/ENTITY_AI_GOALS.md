# Registered entity AI goals

A module can attach a bounded set of vanilla AI goals to an entity type it declared during `#[mod_register]`. Register the entity first, then add its goals before the NeoForge entity registry phase closes. The host installs the real vanilla `Goal` implementations on each spawned `SjmlRegisteredEntity`.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_entity("river_mite", EntityDefinition::default())?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::WaterAvoidingRandomStroll { priority: 5, speed: 0.8 },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::Panic { priority: 1, speed: 1.4 },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::RandomSwimming { priority: 4, speed: 0.7 },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::AvoidPlayer {
            priority: 2,
            range: 12.0,
            walk_speed: 0.8,
            sprint_speed: 1.4,
        },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::AvoidEntityType {
            priority: 3,
            entity_type_id: "minecraft:cat".to_owned(),
            range: 10.0,
            walk_speed: 0.8,
            sprint_speed: 1.4,
        },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::TargetEntityType {
            priority: 1,
            entity_type_id: "minecraft:spider".to_owned(),
            must_see: true,
        },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::LookAtEntityType {
            priority: 4,
            entity_type_id: "minecraft:cow".to_owned(),
            range: 24.0,
        },
    )?;
    registry::register_entity_ai_goal(
        "river_mite",
        EntityAiGoal::Retaliate { priority: 0 },
    )?;
    Ok(())
}
```

Every goal requires `content.register`. Priorities are `0..=31`; movement speeds are finite and within `0.01..=8.0`. A type accepts at most 16 goals, with additional module and total-host quotas.

| Rust goal | Vanilla behavior | Parameters |
| --- | --- | --- |
| `Swim` | Float while in fluid | Priority |
| `RandomStroll` | Walk to random nearby positions | Priority, speed |
| `WaterAvoidingRandomStroll` | Choose random paths that avoid water | Priority, speed |
| `Panic` | Flee after damage or ignition | Priority, speed |
| `RandomSwimming` | Swim to random water positions | Priority, speed; vanilla target selection interval is 120 ticks |
| `LookAtPlayer` | Turn toward a nearby player | Priority, range in blocks (`1..=64`); target class is fixed to `Player` |
| `LookAtEntityType` | Turn toward the nearest living entity whose registry ID exactly matches the declared type | Priority, fully qualified entity type ID, range (`1..=64`) |
| `AvoidPlayer` | Flee from nearby players using vanilla avoidance pathfinding | Priority, range (`1..=32`), walk and sprint speeds (`0.01..=8.0`), sprint speed must be at least walk speed; target class is fixed to `Player` |
| `AvoidEntityType` | Flee from living entities whose registry ID exactly matches the declared type | Priority, fully qualified entity type ID, range (`1..=32`), walk and sprint speeds (`0.01..=8.0`) |
| `MeleeAttack` | Attack a visible target | Priority, speed, long-memory flag |
| `RandomLookAround` | Look around while idle | Priority |
| `TargetPlayer` | Select a nearby player target | Priority, must-see flag |
| `TargetEntityType` | Select a nearby living entity whose registry ID exactly matches the declared type | Priority, fully qualified entity type ID, must-see flag |
| `Retaliate` | Target an entity that recently hurt this mob | Priority |

Goals are immutable after registration. They apply only to SJML-declared entity types and are never installed on vanilla or another mod's entities. Guest callbacks, arbitrary goal classes, arbitrary goal priority controls, and custom entity subclasses are not exposed. Loaded vanilla Mobs have separate bounded AI enable/disable and navigation APIs; see [entity AI control](ENTITY_AI_CONTROL.md) and [entity navigation](ENTITY_NAVIGATION.md).

`AvoidPlayer` uses the additive `register_entity_ai_goal_v2` host import to carry its range and two speed modifiers. The type-specific goals use `register_entity_ai_goal_v3`, which also carries a bounded canonical entity type ID; vanilla goal selectors inspect living entities and then filter to that exact type. `LookAtEntityType` uses vanilla look-at scheduling and a static exact-type selector. The existing v1 and v2 imports remain unchanged. A target type absent from the registry is skipped with a host warning when the mob is created; a non-living registered type has no living candidates and therefore never matches. Use a registered living entity type ID. All static goal imports require `content.register` and are accepted only during `mod_register`. No guest AI callback runs from Minecraft's AI tick.

`EntityAiGameTests.guestEntityInstallsItsDeclaredVanillaAiGoals` spawns the GameTest guest's entity, verifies it targets a farther cow instead of a closer pig, and checks the exact-type avoidance predicate.
