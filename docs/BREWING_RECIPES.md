# Brewing recipes

The content registration API can add bounded mixes to Minecraft's vanilla brewing stand. Declare
each mix from `#[mod_register]` and request `content.register`:

```rust
registry::register_potion("ruby_tonic", "minecraft:jump_boost", 1_200, 1)?;
registry::register_brewing_recipe(
    "water_to_ruby_tonic",
    "minecraft:water",
    "minecraft:emerald",
    "your_package:your_module/ruby_tonic",
)?;
```

The module must use `side = "server"` or `side = "both"`. A module can declare an output potion with
`register_potion`; it is created from one existing mob effect and its resulting ID is
`package:module/path`. The brewing declaration names an existing input potion, one existing item
ingredient, and an existing output potion. Potion and item IDs must be fully qualified registry
IDs. Brewing declaration paths are owned by the package and module and share the recipe namespace
with other SJML recipe families.

The host stages declarations during `mod_register` and adds them through NeoForge's
`RegisterBrewingRecipesEvent`. Unknown potion or item IDs are skipped with a bounded startup
diagnostic rather than crashing a server. Brewing remains in Minecraft's normal brewing-stand
flow, including its fuel, ingredient consumption, and bottle handling.

The potion declaration adds one status effect with a duration of 1–72,000 ticks and amplifier of
0–255. Each brewing mix accepts one item ingredient. Multiple potion effects, item-tag ingredients,
arbitrary brewing callbacks, and direct inventory mutation are not exposed.
