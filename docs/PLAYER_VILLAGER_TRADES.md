# Live villager trade notifications

Server modules can observe completed villager and wandering-trader offers with
the non-cancellable `player.villager_traded` event. The callback runs after
NeoForge's `TradeWithVillagerEvent`, which `AbstractVillager.notifyTrade`
posts after `MerchantOffer.take` accepts the offer inputs and increments its
use count.

Subscribe from a retained server module and decode the detached event:

```rust
use sjml::{event, events::{self, EventPriority, PlayerVillagerTraded}, SjmlResult};

fn initialize() -> SjmlResult<()> {
    events::subscribe("player.villager_traded", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.villager_traded")]
fn on_trade(event: PlayerVillagerTraded) -> SjmlResult<()> {
    if event.profession_id == "minecraft:librarian" {
        // Record the completed trade using its detached item IDs and counts.
    }
    Ok(())
}
```

The schema-v1 snapshot includes the player and merchant UUIDs, merchant type,
dimension and block position, villager profession and level when present,
effective first and optional second item costs, result item and count, and the
offer's use count after the trade. It does not include item components or live
Minecraft objects. IDs are limited to 256 UTF-8 bytes and the complete ABI
payload is limited to 8 KiB.

The event only reports a completed trade. NeoForge marks it non-cancellable;
the callback cannot change the offer, cost, result, experience, or inventory.
It requires `events.subscribe`, is synchronous on the owning logical-server
thread, and may also use `#[event_async]` when the callback does not need to
finish before the current server operation continues.
