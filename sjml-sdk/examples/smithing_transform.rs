#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[mod_register]
fn register_smithing_recipe() -> SjmlResult<()> {
    registry::register_smithing_transform_recipe(
        "netherite_sword",
        SmithingTransformRecipeDefinition {
            template: RecipeIngredient::item("minecraft:netherite_upgrade_smithing_template"),
            base: RecipeIngredient::item("minecraft:diamond_sword"),
            addition: RecipeIngredient::item("minecraft:netherite_ingot"),
            output_item: "minecraft:netherite_sword".into(),
            output_count: 1,
        },
    )?;
    Ok(())
}
