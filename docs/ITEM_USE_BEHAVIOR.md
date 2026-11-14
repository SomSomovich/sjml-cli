# Static item use behavior

During `mod_register`, a module can choose a vanilla use animation and hold
duration for an item it registers:

```rust
registry::register_item("canteen", ItemDefinition::default())?;
registry::set_item_use_behavior(
    "canteen",
    registry::ItemUseBehavior::new(registry::ItemUseAnimation::Drink, 32)?,
)?;
```

The supported animations are `None`, `Eat`, `Drink`, `Block`, `Bow`, `Spear`,
`Crossbow`, `Spyglass`, `TootHorn`, and `Brush`. Duration is inclusive from 1
to 72,000 ticks. Registration requires `content.register`, the normal
`mod_register` phase, and ownership of the registered item.

The host applies these values to the actual Minecraft `Item` implementation
and preserves vanilla behavior when no override is declared. The override is
supported for standalone registered items, registered spawn eggs, registered
block items, and SJML fluid buckets. It changes only `getUseAnimation` and
`getUseDuration`; it does not add an item-use effect, consume the item, or
change the behavior of vanilla items. Existing `item.use` and
`player.item_use_attempt` callbacks remain separate interaction APIs.

The SDK validates the duration before calling the host. The host also checks
the item owner, registration phase, animation code, and duration before staging
the override. A live GameTest checks the animation and duration reported by a
registered Minecraft item. See [`CUSTOM_ITEM_BEHAVIOR.md`](CUSTOM_ITEM_BEHAVIOR.md)
for the other bounded item properties.
