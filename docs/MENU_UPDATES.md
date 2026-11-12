# Dynamic read-only menus

`MenuDefinition::open` creates the module's bounded vanilla chest menu. A module
can replace its title, row count, displayed stacks, and action IDs while that
menu is still the player's active screen:

```rust
let mut refreshed = MenuDefinition::new("Rewards claimed", 2)?;
refreshed.slot(13, MenuItem::new("minecraft:emerald")?.count(3)?)?;
refreshed.update(player_handle)?;
```

The host checks that the current container is an SJML menu opened by the same
runtime module. The update fails if the player closed that menu or opened a
vanilla screen or a menu belonging to another module. A successful update
replaces the exact module-owned menu through Minecraft's normal menu-opening
and synchronization path, so row count and title can change safely. It does
not modify vanilla containers or another module's screen.

Menus remain read-only: displayed stacks cannot be moved, taken, shift-clicked,
or transferred. Only a left-click on a declared action slot calls the
subscribed `menu.action` handler. Opening or updating requires `entity.read`
and `menu.open`; action slots additionally require `events.subscribe` and a
registered `menu.action` callback. Updates are limited to eight replacements
per module per server tick.
