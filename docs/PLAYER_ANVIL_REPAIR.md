# `player.anvil_repair`

`player.anvil_repair` reports a real anvil result observed through NeoForge's
`AnvilRepairEvent`, which fires when a `ServerPlayer` removes a repaired or
renamed item from the anvil's output slot. The callback cannot change the
result, the inputs, the break chance, or the anvil damage; it only observes the
operation after the fact.

The notification contains the player's canonical UUID, the dimension ID, and
detached snapshots of the output, left, and right slots. Each slot reports its
canonical item ID and count. Empty slots use `minecraft:air` with count zero,
and `minecraft:air` is used exactly when the count is zero. Item names, lore,
arbitrary item components, and NBT are not included. The payload also reports
the finite anvil break chance in the `0.0..=1.0` range, which is `0.12` unless
another hook changed it.

## Permissions and execution

Declare `events.subscribe` in the module manifest:

```toml
permissions = ["events.subscribe"]
```

The server operator must grant the same capability to the module as well:

```toml
[[grant]]
module = "example_mod/main"
capabilities = ["events.subscribe"]
```

The event only runs in an integrated or dedicated server context. It is
delivered on the owning server thread; a client-only module does not receive
it. The snapshot is detached, so `#[event_async]` handlers are allowed and may
queue follow-up work, although the notification itself is not cancellable.

Each module may hold up to 256 event subscriptions. For one anvil operation,
the host invokes at most 64 subscribed callbacks. The complete payload is
limited to 2 KiB; strings and item IDs are limited to 256 UTF-8 bytes, and
stack counts are limited to 1,000,000.

## Rust handler

```rust
use sjml::{
    event,
    events::{self, EventPriority},
    player_anvil_repair::PlayerAnvilRepair,
    SjmlResult,
};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("player.anvil_repair", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.anvil_repair")]
fn anvil_repair(event: PlayerAnvilRepair) -> SjmlResult<()> {
    if event.output.item_id == "minecraft:diamond_sword"
        && event.left.item_id == "minecraft:diamond_sword"
        && event.break_chance > 0.0
    {
        // Inspect event.player_id, event.dimension_id, event.output,
        // event.left, event.right, or event.break_chance here.
    }
    Ok(())
}
```

The compilable SDK example is
[`player_anvil_repair.rs`](../sjml-cli/sjml-sdk/examples/gametest/player_anvil_repair.rs).

## Schema v1

The callback export is `sjml_v1_event_player.anvil_repair`. Its payload fields
use the stable little-endian encoding:

1. `u32(1)` schema version.
2. Length-prefixed UTF-8 canonical player UUID.
3. Length-prefixed UTF-8 dimension resource location.
4. Output item snapshot.
5. Left item snapshot.
6. Right item snapshot.
7. `f32` break chance as IEEE-754 bits.

Each item snapshot carries its own nested `u32(1)` schema version, followed by
the length-prefixed UTF-8 canonical item ID and a `u32` count in the
`0..=1_000_000` range. All strings use a little-endian `u32` byte length
followed by UTF-8 bytes.

GameTests post a real `AnvilRepairEvent` for a mock server player and assert
the detached player, dimension, three slots, and `0.12` break chance. Unit
tests cover the exact wire bytes including an empty slot, the air/count
identity rule, break-chance bounds, oversized IDs, the nested schema version,
and the observational NeoForge contract.
