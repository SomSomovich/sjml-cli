# Player inventory reads

The SDK module `sjml::player_inventory` exposes `read(player)` for a live player handle. It returns an immutable snapshot of the player's ordinary inventory slots. Each slot is either empty or contains the canonical item registry ID, current count, item stack maximum, and effective slot limit. The snapshot carries no Minecraft objects, components, NBT, or mutation authority.

```rust
let inventory = sjml::player_inventory::read(player)?;
if let Some(Some(stack)) = inventory.slots.get(0) {
    sjml::host::log(
        sjml::host::LogLevel::Info,
        &format!("hotbar slot 0 contains {} x{}", stack.item_id, stack.count),
    )?;
}
```

Declare both `entity.read` and `inventory.read` in the module's `permissions` list and grant them in the package's permission policy. Access is restricted to logical-server modules and the owning server thread. The host resolves the handle against the current login session, including after respawn; stale or logging-out sessions fail closed.

Minecraft 1.21.1 reports 41 slots: indices `0..=8` are the hotbar, `9..=35` are main storage, `36..=39` are armor from feet through head, and `40` is offhand. Empty slots decode as `None`. The shared ABI enforces an upper bound of 128 slots and a 1 MiB encoded response. Reading the inventory does not change it, synchronize it, or expose stack components. Use the existing block-inventory API for loaded block handlers; there is no player-inventory write or transfer operation in this contract.
