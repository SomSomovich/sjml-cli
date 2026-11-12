# Client item-tooltip events

Physical-client modules can append bounded literal text lines to a vanilla item
tooltip through the synchronous `client.item.tooltip` callback. The hook is
client-local; it does not edit item data, send text to a server, or require the
server to install the module. A `client` module runs only when that package is
installed on the client. Set `[mod].distribution = "client_only"` when the
package should never require a server-side copy; `[[modules]].side = "client"`
controls where that module runs, while distribution controls peer installation.
A `both` module can subscribe in its client runtime; its logical-server runtime
cannot subscribe to this event.

The callback receives a detached snapshot containing the item registry ID,
stack count, current damage, and maximum damage. It never receives an
`ItemStack`, components, NBT, or other game objects. The host appends the
returned lines after the existing vanilla tooltip as literal `Component`
values, so they cannot add click actions, hover content, formatting, or
translated components.

## Subscribe and return lines

```rust
use sjml::events::{self, ClientItemTooltip, EventPriority, ItemTooltipLines};
use sjml::{event, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("client.item.tooltip", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "client.item.tooltip")]
fn add_durability_line(event: ClientItemTooltip) -> SjmlResult<ItemTooltipLines> {
    let mut lines = ItemTooltipLines::new();
    if event.max_damage > 0 {
        lines.push(format!("Durability: {}/{}", event.damage, event.max_damage))?;
    }
    Ok(lines)
}
```

The module must request both capabilities in its manifest, and the local
administrator must grant both in `config/sjml-permissions.toml`:

```toml
[[modules]]
id = "main"
side = "client"
permissions = ["events.subscribe", "client.item.tooltip.modify"]
```

```toml
[[grant]]
module = "example/main"
capabilities = ["events.subscribe", "client.item.tooltip.modify"]
```

The client event uses the same subscription priority order as other SJML
events. Its callback must be a regular synchronous `#[event]` handler because
Minecraft needs the lines before it completes tooltip construction; the
`#[event_async]` macro rejects this event. Callbacks run on the client main
thread and may be called repeatedly while the same tooltip is rebuilt, so keep
them short and avoid doing unrelated work there.

Each callback can return at most 16 lines, 256 UTF-8 bytes per line, and 2 KiB
combined. Across all matching subscriptions, the host invokes at most 64
callbacks and appends at most 64 lines and 8 KiB. Empty lines and control
characters are rejected. A callback error or malformed result disables that
module and leaves the vanilla tooltip intact.

The headless test suite verifies the schema, bounds, client-only subscription,
and adapter behavior. Visible tooltip rendering still requires a real client
with a display.
