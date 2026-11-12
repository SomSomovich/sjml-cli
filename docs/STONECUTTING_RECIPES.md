# Stonecutting recipes

Modules with the `content.register` capability can register a vanilla
stonecutting recipe from their `#[mod_register]` callback. SJML writes the
declaration to its generated server data pack, which Minecraft loads through
the normal recipe reload path.

```rust
use sjml::registry::{self, RecipeIngredient, StonecuttingRecipeDefinition};

registry::register_stonecutting_recipe(
    "polished_granite_slab",
    StonecuttingRecipeDefinition {
        ingredient: RecipeIngredient::item("minecraft:polished_granite"),
        output_item: "minecraft:polished_granite_slab".into(),
        output_count: 2,
    },
)?;
```

An ingredient may be one item or an item tag. Unqualified IDs resolve in the
owning package and module. Recipe IDs share one namespace across shaped,
shapeless, cooking, and stonecutting declarations, and the host rejects a
duplicate ID. The output count is limited to 1–64. Accepted declarations are
staged until registration completes; declarations from a rejected package are
removed together with its other staged content.

The host emits the standard Minecraft 1.21 recipe form:

```json
{
  "type": "minecraft:stonecutting",
  "ingredient": { "item": "minecraft:polished_granite" },
  "result": { "id": "minecraft:polished_granite_slab", "count": 2 }
}
```

This is a server data recipe declaration. Minecraft synchronizes recipe data
to connected clients through its normal recipe synchronization; the guest does
not edit a live recipe manager or install a custom serializer.
