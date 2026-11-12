# Item tag membership

Logical-server modules can check whether an item registry ID belongs to a
vanilla, mod, or data-pack item tag:

```rust
use sjml::{world, SjmlResult, WorldHandle};

fn is_plank(world: WorldHandle, item_id: &str) -> SjmlResult<bool> {
    world::item_has_tag(world, item_id, "minecraft:planks")
}
```

Both IDs use the canonical `namespace:path` form without a `#` prefix and are
limited to 256 UTF-8 bytes. The membership result uses the current server item
registry tags, so later calls observe data-pack/resource reloads. An unknown
item or absent tag returns `false`.

The operation requires a live module-owned world handle, the owning logical
server thread, and `world.read`. It is unavailable to physical-client modules
and during static registration. The query reads registry membership only; it
does not inspect a player's or container's inventory and does not load chunks.
To enumerate bounded item-tag members, see
[`WORLD_TAG_MEMBERS.md`](WORLD_TAG_MEMBERS.md). Codec, import-boundary,
permission, ownership, and dedicated-server GameTests cover the contract.
