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
| `LookAtPlayer` | Turn toward a nearby player | Priority, range in blocks (`1..=64`) |
| `MeleeAttack` | Attack a visible target | Priority, speed, long-memory flag |
| `RandomLookAround` | Look around while idle | Priority |
| `TargetPlayer` | Select a nearby player target | Priority, must-see flag |
| `Retaliate` | Target an entity that recently hurt this mob | Priority |

Goals are immutable after registration. They apply only to SJML-declared entity types and are never installed on vanilla or another mod's entities. Guest callbacks, arbitrary goal classes, arbitrary goal priority controls, and custom entity subclasses are not exposed. Loaded vanilla Mobs have separate bounded AI enable/disable and navigation APIs; see [entity AI control](ENTITY_AI_CONTROL.md) and [entity navigation](ENTITY_NAVIGATION.md).

`EntityAiGameTests.guestEntityInstallsItsDeclaredVanillaAiGoals` spawns the GameTest guest's entity and checks that its goal selector contains the three newly exposed vanilla goal classes.
