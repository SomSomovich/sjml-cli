# Entity custom names

SJML modules can read, set, and clear a plain custom name on a live, loaded,
non-player entity:

```rust
let current: Option<String> = sjml::entity::custom_name(entity)?;
sjml::entity::set_custom_name(entity, "Armored Boar")?;
sjml::entity::clear_custom_name(entity)?;
```

Reading requires `entity.read`. Setting or clearing requires both `entity.read`
and the separate `entity.custom_name.write` capability. Calls run on the
owning logical server thread and revalidate the module-scoped handle before
accessing the entity.

Names are limited to 1,024 UTF-8 bytes and 256 Unicode scalar values. Set
values must be nonempty and cannot contain control characters; use
`clear_custom_name` to remove a name. Reads return `None` when an entity has no
custom name. Existing styled names are flattened to their display text.

Guest text is always assigned with Minecraft's `Component.literal`. It is
never parsed as a rich component, command, selector, or formatting expression.
For example, `§cName` remains literal text. Minecraft's normal entity data
synchronization handles updates to connected clients. The API does not expose
player names or arbitrary entity metadata/NBT.

The ABI uses the `sjml_entity_v1` imports `custom_name`, `set_custom_name`, and
`clear_custom_name`. The optional read response is schema v1: a `u32` schema,
one option byte, then a stable UTF-8 string when present. Host and SDK enforce
the same byte and character bounds.
