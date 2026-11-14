# Player enchanting notification

`player.enchanted` is a non-cancellable, logical-server notification for an item
enchanted through Minecraft's enchanting menu. It is delivered by NeoForge's
`PlayerEnchantItemEvent` after `IItemExtension.applyEnchantments` updates the
stack. The event reports the resulting item ID/count/durability and the
enchantment IDs and levels applied by that operation.

The callback receives only detached values. Item and enchantment IDs use
canonical registry syntax; the list is limited to 32 entries and levels to
`1..=255`. Unsupported or oversized event data is skipped as a whole. Up to 64
authorized module callbacks run for one operation.

This notification specifically covers the enchanting-menu path. It does not
observe arbitrary item component or NBT writes, anvil operations, loot
generation, or other code that edits an ItemStack through a different path.
The event is observational and callback failures do not undo the enchantment.

Request both event subscription and the separate observation capability:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["events.subscribe", "player.enchantments.observe"]
```

```rust
use sjml::events::{PlayerEnchanted, PLAYER_ENCHANTED_EVENT_NAME};
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe(PLAYER_ENCHANTED_EVENT_NAME, EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.enchanted")]
fn on_enchanted(event: PlayerEnchanted) -> SjmlResult<()> {
    for enchantment in event.enchantments {
        host::log(LogLevel::Info, &format!(
            "{} applied {} level {} to {}",
            event.player_id, enchantment.id, enchantment.level, event.item_id,
        ))?;
    }
    Ok(())
}
```
