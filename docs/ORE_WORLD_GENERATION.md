# Bounded ore world generation

Content registration can add vanilla ore generation for a block declared earlier by the same module. SJML writes a configured feature, placed feature, and NeoForge biome modifier into its generated server data pack. Minecraft applies them through normal world generation; the API does not place blocks directly, load chunks, or execute from ticks.

```rust
use sjml::prelude::*;

fn register_content() -> SjmlResult<()> {
    registry::register_block("ruby_ore", BlockDefinition::default())?;
    registry::register_ore_generation(
        "ruby_veins",
        OreGenerationDefinition::new("ruby_ore", "minecraft:is_overworld")?
            .replaceable_blocks(OreReplaceableBlocks::Deepslate)
            .vein_size(12)?
            .count_per_chunk(8)?
            .y_range(-48, 32)?,
    )
}
```

The ore block path must resolve to a block staged by the same module. The biome selector is a fully qualified biome tag ID without a leading `#`; the generated modifier adds the `#`. The replacement target is one of vanilla's stone, deepslate, or netherrack ore-host tags.

Vein size and placement attempts are each limited to `1..=64`. The inclusive absolute Y endpoints must be ordered within `-64..=319`. The API uses vanilla's `minecraft:ore` feature and the `underground_ores` generation step, so the normal world-generation pipeline controls placement. Existing chunks are not retroactively populated. A new world or newly generated chunks are needed to see the feature.

Request `content.register` in the module manifest and have the server administrator grant it:

```toml
[[modules]]
id = "content"
path = "modules/content.wasm"
side = "server"
permissions = ["content.register"]
```

The generated resources are owned by the package namespace and module path. A declaration cannot target another module's block, and path traversal or foreign resource namespaces are rejected before data-pack output.
