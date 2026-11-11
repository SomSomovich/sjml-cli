# Smithing transform recipes

With the `content.register` capability, a module can stage a vanilla smithing
transform recipe from its `#[mod_register]` callback. SJML supports the single
vanilla recipe type `minecraft:smithing_transform`: one template, one base, one
addition, and one output item. Each input may be an item ID or an item-tag ID.

```rust
use sjml::registry::{self, RecipeIngredient, SmithingTransformRecipeDefinition};

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
```

The output count is limited to 1–64. Paths and resource IDs are bounded and
validated before they reach the host. Unqualified IDs resolve in the owning
package and module. Recipe IDs are unique across direct guest recipe families;
duplicate IDs and invalid input are rejected. If static registration fails,
SJML rolls the package's staged recipe back with its other content.

The host writes the declaration to its generated server data pack and Minecraft
loads it using the normal recipe reload mechanism:

```json
{
  "type": "minecraft:smithing_transform",
  "template": { "item": "minecraft:netherite_upgrade_smithing_template" },
  "base": { "item": "minecraft:diamond_sword" },
  "addition": { "item": "minecraft:netherite_ingot" },
  "result": { "id": "minecraft:netherite_sword", "count": 1 }
}
```

The ABI import is
`sjml_registry_v1.register_smithing_transform_recipe` with eleven `i32`
parameters: pointer/length pairs for the recipe path, template, base, addition,
and output item, followed by the output count. It returns the standard SJML
status code. The guest API is available starting with ABI 1.36.0.
