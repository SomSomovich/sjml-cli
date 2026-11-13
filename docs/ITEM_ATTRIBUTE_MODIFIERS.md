# Registered item attribute modifiers

An item may declare up to 16 vanilla-style attribute modifiers during its
module's `#[mod_register]` callback. The host applies the declarations to the
item's `DataComponents.ATTRIBUTE_MODIFIERS` component while registering the
item. The same API works for standalone items, block items, and generated
fluid buckets declared by that module.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item(
        "ruby_sword",
        ItemDefinition {
            max_stack_size: 1,
            fire_resistant: false,
        },
    )?;
    let attack = ItemAttributeModifier::new(
        "minecraft:generic.attack_damage",
        "attack_bonus",
        3.5,
        AttributeModifierOperation::AddValue,
        ItemAttributeSlot::Mainhand,
    )?;
    registry::set_item_attribute_modifier("ruby_sword", &attack)?;
    Ok(())
}
```

`attribute_id` must be a canonical, fully qualified registry ID, such as
`minecraft:generic.attack_damage`. The host resolves it against the active
Minecraft attribute registry when it registers items, so vanilla and
registered mod attributes are supported. Unknown IDs reject item registration.
`modifier_path` is a lowercase module-local content path. SJML scopes the final
modifier ID to the owning package and module; duplicate modifier IDs on one item
are rejected.

Amounts must be finite and within `-1024..=1024`. Operations use the vanilla
`ADD_VALUE`, `ADD_MULTIPLIED_BASE`, or `ADD_MULTIPLIED_TOTAL` behavior. Slot
groups are `Any`, `Mainhand`, `Offhand`, `Hand`, `Feet`, `Legs`, `Chest`, `Head`,
`Armor`, and `Body`. Slots that are not currently worn or supported by a given
entity simply do not contribute an attribute modifier there.

The declaration requires the module's `content.register` grant and is accepted
only during `#[mod_register]`. It is immutable after item registration. This
does not mutate modifiers on existing stacks and does not expose arbitrary data
component or NBT editing. The GameTest guest declares a main-hand attack bonus;
the host GameTest checks the registered component and how Minecraft applies and
removes the modifier on a live server zombie as the item moves between slots.
