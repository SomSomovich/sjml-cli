# Food status effects

`registry::ItemFood` can add up to eight vanilla status effects that Minecraft applies when a registered food is eaten. Declare the item and its food properties during `#[mod_register]`, with the existing `content.register` permission:

```rust
use sjml::{mod_register, registry::{self, ItemDefinition, ItemFood, ItemFoodEffect, ItemFoodEffects}, SjmlResult};

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("trail_ration", ItemDefinition::default())?;
    let effects = ItemFoodEffects::new()
        .with_effect(ItemFoodEffect::new("minecraft:poison", 1200, 1, 1.0)?)?;
    registry::set_item_food_with_effects(
        "trail_ration",
        ItemFood::new(6, 0.6)?.fast().always_eat(),
        effects,
    )?;
    Ok(())
}
```

`ItemFoodEffect::new(effect_id, duration_ticks, amplifier, probability)` accepts a canonical fully qualified registry ID, such as `minecraft:poison` or `othermod:custom_effect`. IDs are case-sensitive lowercase Minecraft resource IDs. The effect must exist in the effect registry when the host materializes items.

Duration is `1..=72_000` ticks, amplifier is `0..=255`, and probability is finite `0.0..=1.0`. A zero amplifier is effect level I. Minecraft uses its vanilla random probability check; `1.0` always applies and `0.0` never applies. Each food supports at most eight effects.

`ItemFood` keeps its existing `Copy` and `const` builder API. `ItemFoodEffects` collects up to eight effects and rejects a ninth before host calls. The SDK validates values before calling the host, and the host validates them again while staging. `set_item_food_with_effects` first calls the unchanged `set_item_food` import with its existing parameters, then stages each effect through a separate bounded import under the same `content.register` grant. During item materialization, the host resolves every ID to a registered `MobEffect` holder and uses vanilla `FoodProperties.Builder.effect` suppliers, so each consumption receives a fresh effect instance. Unknown effect IDs fail item materialization.
