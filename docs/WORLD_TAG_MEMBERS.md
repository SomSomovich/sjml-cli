# Registry tag members

Logical-server modules with `world.read` can enumerate a bounded tag from the
current block, item, fluid, or entity-type registry:

```rust
use sjml::{world, SjmlResult, TagRegistry, WorldHandle};

fn item_members(world: WorldHandle) -> SjmlResult<Option<Vec<String>>> {
    world::tag_members(world, TagRegistry::Item, "minecraft:planks")
}
```

The `world` module also has `block_tag_members`, `item_tag_members`,
`fluid_tag_members`, and `entity_type_tag_members` convenience functions. Pass
a canonical `namespace:path` tag ID without `#`. Members are returned as
canonical IDs in lexical order. Calls use the active logical server's current
registries, so resource reloads are reflected on subsequent queries; no chunks
are read or loaded.

The result is `None` when a tag is absent and `Some(Vec::new())` when it exists
but has no entries. A tag may return at most 1,024 members, each at most 256
UTF-8 bytes. Larger results fail with `PayloadTooLarge`; the host never returns
a partial list. Enumeration requires a live module-owned world handle and runs
on the owning server thread. Static registration and physical-client modules
cannot access this API.

The registry selectors and wire schema are ABI-v1 values. This API does not
enumerate arbitrary registries or expose Minecraft objects. At the raw ABI
level, the output has a little-endian `u32` payload length followed by the
versioned response bytes. A four-byte output buffer probes the encoded
response size; the caller then repeats with an exact-sized buffer. The SDK
handles this bounded framing and same-thread query. See
[`ABI.md`](ABI.md#world-registry-tag-members) and the source-based
[`minecraft-api-coverage.md`](minecraft-api-coverage.md).
