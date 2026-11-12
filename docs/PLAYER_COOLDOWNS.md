# Player item cooldowns

Server modules can read and change vanilla cooldowns for registered items on a live player session.
Cooldowns use Minecraft's public `ItemCooldowns` API, so the server remains authoritative and vanilla
handles synchronization to the client.

```rust
use sjml::{player, PlayerHandle, SjmlResult};

fn limit_teleport_item(player: PlayerHandle) -> SjmlResult<()> {
    if player::has_item_cooldown(player, "minecraft:ender_pearl")? {
        return Ok(());
    }
    // One minute at the default 20 server ticks per second.
    player::set_item_cooldown(player, "minecraft:ender_pearl", 1_200)
}
```

`has_item_cooldown` reports only whether a cooldown is active. Minecraft does not expose the exact
remaining tick count through its public API. Durations accepted by `set_item_cooldown` are bounded to
1–72,000 ticks. `clear_item_cooldown` removes the vanilla cooldown for the same item.

Every item ID must be a canonical registered, non-air `namespace:path`. Every operation resolves the
module-owned player handle against the current login session and runs on its owning logical-server
thread. Reads require the declared and granted `entity.read` capability. Changes also require
`player.cooldown.write`; the host does not bind these imports to client-only modules. Static
registration can link server imports, but calls before a live server session are denied.

## ABI v1 imports

The functions are in `sjml_player_v1`:

| Import | Parameters | Result |
| --- | --- | --- |
| `has_item_cooldown` | handle pointer/length, UTF-8 item ID pointer/length, output pointer/capacity | `i32` status; one output byte (`0` or `1`) on success |
| `set_item_cooldown` | handle pointer/length, UTF-8 item ID pointer/length, unsigned tick count | `i32` status |
| `clear_item_cooldown` | handle pointer/length, UTF-8 item ID pointer/length | `i32` status |

Player handles use the stable 24-byte ABI v1 encoding. Item IDs are limited to 256 UTF-8 bytes.
Set operations accept an unsigned tick count from 1 through 72,000.
