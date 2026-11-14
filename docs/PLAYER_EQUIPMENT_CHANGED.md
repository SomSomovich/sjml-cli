# `player.equipment_changed`

`player.equipment_changed` reports a real equipment difference detected by
NeoForge for a `ServerPlayer`. SJML listens to NeoForge 21.1.265's
`LivingEquipmentChangeEvent`, which Minecraft posts from `LivingEntity.tick()`
on the logical server. This can include initial equipment changes detected as a
player joins or is cloned. The callback cannot change or cancel the equipment
operation.

The notification contains the player's UUID, a stable equipment-slot value,
and bounded snapshots of the previous and current stacks. Empty stacks are
`None`. Each stack reports its canonical item ID, count, current damage, and
maximum damage. Item names, lore, arbitrary item components, and NBT are not
included.

## Permissions and execution

Declare both permissions in the module manifest:

```toml
permissions = ["events.subscribe", "player.equipment.observe"]
```

The server operator must grant both to the module as well:

```toml
[[grant]]
module = "example_mod/main"
capabilities = ["events.subscribe", "player.equipment.observe"]
```

The event only runs in an integrated or dedicated server context. It is
delivered synchronously on the owning server thread; a client-only module does
not receive it. A `#[event_async]` handler may queue follow-up work, but it
receives the same immutable snapshot.

Each module may hold up to 256 event subscriptions. For one equipment
transition, the host invokes at most 64 subscribed callbacks. The complete
payload is limited to 2 KiB; strings and item IDs are limited to 256 UTF-8
bytes, and stack counts are limited to 1,000,000.

## Rust handler

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.equipment_changed", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.equipment_changed")]
fn equipment_changed(event: PlayerEquipmentChanged) -> SjmlResult<()> {
    if event.slot == PlayerEquipmentSlot::MainHand {
        if let Some(item) = event.current {
            host::log(
                LogLevel::Info,
                &format!("{} equipped {} x{}", event.player_id, item.item_id, item.count),
            )?;
        }
    }
    Ok(())
}
```

The compilable SDK example is
[`player_equipment_changed.rs`](../sjml-cli/sjml-sdk/examples/player_equipment_changed.rs).

## Schema v1

The callback export is
`sjml_v1_event_player.equipment_changed`. Its payload fields use the stable
little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. `u32` slot code: `0` main hand, `1` off hand, `2` feet, `3` legs, `4` chest,
   `5` head, `6` body.
4. Previous stack: `u8(0)` if empty, otherwise `u8(1)`, item ID string, and
   `u32` count, damage, and maximum damage.
5. Current stack with the same optional-stack format.

All strings use a little-endian `u32` byte length followed by UTF-8 bytes.
