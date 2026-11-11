# Loaded block-state properties

The Rust SDK can inspect the named properties on a block at an already-loaded
position:

```rust
let properties = sjml::world::block_properties(world, position)?;
if let Some(properties) = properties {
    for property in properties {
        sjml::log::info(&format!("{} = {}", property.name, property.value));
    }
}
```

This call requires `world.read` and a logical-server module. It returns
`None` when the position is outside build height or its chunk is not already
loaded; it never requests a chunk load. A present result contains at most 64
detached `{ name, value }` strings, sorted by property name. Empty lists are
valid for block states without properties. Property names are canonical
lowercase identifiers, and property values are bounded printable strings.

Use `world::block_states` to read the block ID and canonical state string in a
batch. `block_properties` is the structured counterpart when code needs to
inspect individual state properties. Neither API exposes a Minecraft object or
block-entity data.
