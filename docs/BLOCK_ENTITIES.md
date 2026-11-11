# Block entity reads

A module must request `world.read` and receive an administrator grant. On a server, `world.block_entity(pos)?` looks only in loaded chunks and returns `None` if there is no block entity. The returned reference is scoped to the module and runtime. Replacement, removal, chunk unload, world unload and runtime close invalidate it. Calls are synchronous on the owning server thread.

```rust
use sjml::prelude::*;
# fn inspect(world: WorldHandle, pos: BlockPos) -> SjmlResult<()> {
if let Some(block) = world.block_entity(pos)? {
    let type_id = block.type_id()?;
    let state = block.block_state()?;
    let inventory = block.inventory(None)?;
    let nbt = block.nbt()?;
}
# Ok(()) }
```

Inventory requires the module to request and receive `inventory.read` as well as `world.read`; side `None` requests the unsided item handler. It returns detached values using the same semantics as `inventory::read`.

NBT requires the additional requested and explicitly granted `block_entity.nbt.read`. This grant permits reading the bounded persistent tag snapshot produced by the block entity, including mod-specific data. Mod-specific tags are unstable and may expose data that an administrator would prefer not to grant. Use typed inventory and capability APIs for their respective data; the NBT snapshot is not a replacement for those APIs.

The schema-v1 snapshot uses little-endian version `u32`, type and dimension UTF-8 strings, a schema-v1 position and a UTF-8 block-state string. NBT schema 1 is a `u32` version followed by a recursively tagged tree: each value has its usual NBT tag ID as `u8`; numeric values are little-endian, strings and arrays have a `u32` length, and lists and compounds contain tagged children. Compound keys precede their values as strings. Compounds returned by the host are sorted by key. The wire value is limited to 1 MiB, depth 64 (root depth 1), 65,536 nodes, 65,536 items per array or container, and 65,536 UTF-8 bytes per string. Unknown tags, duplicate compound keys, mixed-type lists, malformed strings and values exceeding a limit are rejected.
