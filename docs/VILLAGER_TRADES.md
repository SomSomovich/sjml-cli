# Villager trade offers

Modules can stage static villager offers from `#[mod_register]` with
`registry::register_villager_trade`. The module must request and receive
`content.register`, and its side must be `server` or `both`:

```rust
use sjml::prelude::*;

#[mod_register]
fn register_trades() -> SjmlResult<()> {
    let offer = VillagerTradeOffer::new(
        "minecraft:farmer",
        1,
        VillagerTradeItem::new("minecraft:emerald", 18)?,
        VillagerTradeItem::new("minecraft:bread", 4)?,
    )?
    .with_economy(16, 2, 0.05)?;

    registry::register_villager_trade("emerald_for_bread", offer)
}
```

`VillagerTradeOffer::new` defaults to 12 uses, one villager XP, and a price
multiplier of `0.05`. Add a second input with
`.with_second_cost(VillagerTradeItem::new("minecraft:book", 1)?)`. The result
and both costs use item registry IDs; the API does not accept arbitrary NBT or
data components.

The host stages the declaration during `mod_register`, then appends a
`BasicItemListing` to the requested profession and level when NeoForge fires
`VillagerTradesEvent` during server data loading. NeoForge 21.1.265 fires this
main-bus event once per registered profession after a server data/tag reload.
Levels are 1–5, matching the mutable trade map supplied by that event. Reloads
start from NeoForge's base trade lists, so the same declaration is not appended
repeatedly. Existing villagers keep already-generated offers; the hook changes
the table used when offers are generated.

The offer accepts a fully qualified canonical profession ID and item IDs of up
to 256 UTF-8 bytes. Each item count is 1–64 and must also fit that item's
Minecraft stack limit. The profession level is 1–5, max uses is 1–1,000,
villager XP is 0–1,000, and the finite price multiplier is 0–100. A module may
stage up to 128 offers and the host accepts up to 4,096 total. Unknown item IDs
or counts above the item's stack limit are skipped with a host warning when
the offer is applied.

Client-only modules cannot declare villager trades. Server-only packages using
`content.register` are subject to SJML's existing registry synchronization
handshake requirements for clients.
