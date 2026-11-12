use sjml::{
    ErrorCode, ItemFood, ItemFoodEffect, ItemFoodEffects, MAX_ITEM_FOOD_EFFECTS,
    MAX_ITEM_FOOD_EFFECT_DURATION_TICKS, MAX_ITEM_FOOD_EFFECT_ID_BYTES,
};

#[test]
fn public_food_builder_accepts_bounded_vanilla_effects() {
    let effect = ItemFoodEffect::new("minecraft:poison", 1200, 1, 1.0).unwrap();
    assert_eq!(effect.effect_id(), "minecraft:poison");
    assert_eq!(effect.duration_ticks(), 1200);
    assert_eq!(effect.amplifier(), 1);
    assert_eq!(effect.probability(), 1.0);

    let effects = ItemFoodEffects::new().with_effect(effect).unwrap();
    let food = ItemFood::new(6, 0.6).unwrap().fast().always_eat();
    assert_eq!(food.nutrition(), 6);
    assert_eq!(food.saturation_modifier(), 0.6);
    assert!(food.is_fast());
    assert!(food.can_always_eat());
    assert_eq!(effects.effects().len(), 1);
    assert_eq!(
        (
            MAX_ITEM_FOOD_EFFECTS,
            MAX_ITEM_FOOD_EFFECT_ID_BYTES,
            MAX_ITEM_FOOD_EFFECT_DURATION_TICKS
        ),
        (8, 256, 72_000)
    );
}

#[test]
fn public_food_effect_constructor_rejects_out_of_range_values() {
    for invalid in [
        ItemFoodEffect::new("poison", 1, 0, 1.0),
        ItemFoodEffect::new("Minecraft:poison", 1, 0, 1.0),
        ItemFoodEffect::new("minecraft:poison", 0, 0, 1.0),
        ItemFoodEffect::new("minecraft:poison", 72_001, 0, 1.0),
        ItemFoodEffect::new("minecraft:poison", 1, 256, 1.0),
        ItemFoodEffect::new("minecraft:poison", 1, 0, f32::NAN),
        ItemFoodEffect::new("minecraft:poison", 1, 0, 1.01),
    ] {
        assert_eq!(invalid.unwrap_err().code(), ErrorCode::InvalidArgument);
    }
}
