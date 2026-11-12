# Editable server menus

Editable menus provide a bounded, server-authoritative chest screen with
allowlisted input slots and take-only output slots. The existing
`MenuDefinition`/`menu.open` API remains read-only and is unchanged.

```rust
use sjml_sdk::menu::{
    EditableMenuDefinition, EditableMenuEndpoint, EditableMenuSlot,
};
use sjml_sdk::{PlayerHandle, SjmlResult};

fn open_storage(player: PlayerHandle) -> SjmlResult<()> {
    let mut menu = EditableMenuDefinition::new("Stone exchange", 1)?;
    menu.slot(EditableMenuSlot::input(0, ["minecraft:stone"])?)?;
    menu.slot(EditableMenuSlot::output(1)?)?;
    menu.open(player)?;

    if let Some(stack) = menu.read_slot(player, 0)? {
        if stack.item_id == "minecraft:stone" {
            menu.transfer(
                player,
                EditableMenuEndpoint::Menu(0),
                EditableMenuEndpoint::Menu(1),
                stack.count.min(64) as u8,
            )?;
        }
    }
    Ok(())
}
```

The host accepts one to six rows and at most 54 unique slot declarations. An
input slot requires an explicit allowlist of 1–32 canonical registry IDs; a
player can insert or take only those items using normal vanilla container
clicks. An output slot starts empty, rejects player insertion, and allows
withdrawal. Only the module transfer API can put an existing stack into an
output slot.

`read_slot` returns only the selected slot's canonical item ID and count. A
transfer moves 1–64 existing items between a declared menu slot and a player
main-inventory/hotbar slot, or between two declared menu slots. It does not
create or transform items. The host preflights the source count, item and data
component identity, slot role and allowlist, and destination capacity before
writing either side. A module can issue at most 32 transfers per server tick.

Every operation requires a live player handle, `entity.read`, `menu.open`, and
the owning logical-server thread. Transfers also require the separate
`menu.transfer` permission. The active menu must belong to the calling runtime
module; a different module or a vanilla screen cannot be read or changed.
On close, the host returns remaining stacks to the player's main inventory and
hotbar, then drops only any exact overflow remainder into the world. This menu
is temporary storage, not a persistent block inventory. The host never
provides item crafting, arbitrary item creation, or item-data transformation
through this API.
