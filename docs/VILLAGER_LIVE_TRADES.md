# Live Villager Trade API

The `sjml::entity` API reads the current offers from a loaded Minecraft `AbstractVillager`, which
includes villagers and wandering traders. It can also update stock and price state on an existing
offer that the same package module registered with `registry::register_villager_trade`, or add and
remove bounded runtime offers owned by that module.

```rust
let offers = sjml::entity::villager_offers(merchant)?;
for offer in offers {
    println!("{}: {} for {}", offer.id, offer.input_a.count, offer.result.item_id);
}

sjml::entity::set_villager_offer_uses(merchant, "example:content/emerald_trade", 2)?;
sjml::entity::set_villager_offer_special_price(
    merchant,
    "example:content/emerald_trade",
    -1,
)?;
sjml::entity::update_villager_offer_demand(merchant, "example:content/emerald_trade")?;

let offer = sjml::entity::VillagerTradeOfferDefinition::new(
    sjml::entity::VillagerOfferItem { item_id: "minecraft:emerald".into(), count: 2 },
    sjml::entity::VillagerOfferItem { item_id: "minecraft:book".into(), count: 1 },
    12,
)?;
sjml::entity::add_villager_offer(merchant, "daily/book", &offer)?;
sjml::entity::remove_villager_offer(merchant, "daily/book")?;
```

## Permissions and handles

Reading requires `entity.read`. Each write also requires `villager.trade.write`; declare both in the
module's `permissions` list and request the write capability from the host. Writes are available
only from the logical server. Calls run synchronously on its server thread and require a live
merchant handle owned by the calling module. They do not load chunks. A missing, stale, foreign,
dead, or non-merchant handle is rejected.

The host accepts registered-offer mutation IDs only when they identify a currently present offer
registered by the same package and module. Runtime additions take a module-relative path; the host
qualifies it as `<package>:<module>/<path>`, and the same module can remove it with that local path.
A module cannot remove another module's, vanilla, or other mods' offers. Vanilla and other mods'
offers can be read, but have synthetic IDs such as `live/0` and cannot be changed through this API.
The host creates runtime offers from live item registry entries and does not accept arbitrary
Minecraft objects or item components.

The owner marker is attached to the offer instance when SJML's registered trade listing creates
it. Minecraft does not persist that marker in villager save data. If an offer is reconstructed from
a saved villager after a reload, the host reports it with a synthetic read-only ID until SJML
creates a newly registered offer instance; it does not guess ownership from matching item stacks.
Runtime-created offers use the same in-memory marker. Minecraft can save their contents, but after a
merchant is reloaded the marker is gone and the offer becomes a synthetic read-only offer; the host
does not infer module ownership from matching item stacks.

## Snapshot fields and limits

`villager_offers` returns a detached snapshot of all current offers, up to 256. Each
`VillagerOffer` contains:

- `id` and `registered`: a stable registration ID with `registered = true`, or a read-only
  `live/<index>` ID;
- `input_a`: the first input's base item ID and count, before price modifiers;
- `current_input_a_count`: the first input count after Minecraft applies demand and special price;
- `input_b`: an optional second input item ID and count;
- `result`: the output item ID and count;
- `uses`, `max_uses`, `demand`, and `special_price` from the current `MerchantOffer`.

Each returned item count is bounded to 1..=4,096, uses and max uses to 1..=1,000, demand to
±1,000,000, and `special_price` to ±10,000. Item data components are not included in the snapshot.
The host rejects a merchant with more than 256 offers rather than returning a partial list.

## Write behavior

`set_villager_offer_uses(merchant, id, uses)` accepts 0..=1,000, and `uses` must not exceed that
offer's `max_uses`. `max_uses` is fixed when Minecraft constructs the `MerchantOffer` and cannot be
changed here. Setting uses to `max_uses` marks the offer out of stock; setting it to zero restocks
it. If the offer is open in a player's trading menu, the host refreshes the menu's offer list.

`set_villager_offer_special_price(merchant, id, value)` sets Minecraft's `specialPriceDiff` to a
value in ±10,000. Minecraft continues to compute the displayed first-input cost from the base
cost, demand, price multiplier, and special price. The API does not write an input stack or its
components.

Minecraft 1.21.1 exposes `MerchantOffer.updateDemand()` but no public setter for demand. Therefore
`update_villager_offer_demand(merchant, id)` performs that exact vanilla update:

```text
demand = demand + uses - (max_uses - uses)
```

The host rejects an update if the current or resulting demand would exceed ±1,000,000. It does not
offer an arbitrary demand setter. An open trading menu is refreshed after a successful mutation.

`add_villager_offer(merchant, local_id, definition)` creates a fresh vanilla offer from item IDs
resolved in the live item registry. Each item count must fit in both 1..=64 and the item's actual
stack size. Maximum uses are 1..=1,000, XP reward 0..=1,000, price multiplier 0..=1, and initial
demand ±1,000. The optional second cost uses `with_second_cost`. One module can keep at most 32
runtime offers on one merchant and 256 tracked runtime offers overall; a merchant is limited to 256
offers. Open trading menus are refreshed after insertion and removal.

`remove_villager_offer(merchant, local_id)` removes exactly one currently present offer created
through this API by the same package module. Runtime ownership is tracked in memory and is not
persisted with villager data, so a saved and reloaded runtime offer remains in Minecraft but is
reported as a synthetic read-only offer.

## API summary

| Function | Capability | Effect |
| --- | --- | --- |
| `entity::villager_offers(merchant)` | `entity.read` | Reads the bounded current offer snapshot |
| `entity::set_villager_offer_uses(merchant, id, uses)` | `entity.read`, `villager.trade.write` | Sets stock within immutable max uses |
| `entity::set_villager_offer_special_price(merchant, id, value)` | `entity.read`, `villager.trade.write` | Sets `specialPriceDiff` |
| `entity::update_villager_offer_demand(merchant, id)` | `entity.read`, `villager.trade.write` | Applies vanilla `updateDemand()` |
| `entity::add_villager_offer(merchant, local_id, definition)` | `entity.read`, `villager.trade.write` | Adds a bounded owner-scoped offer from live item IDs |
| `entity::remove_villager_offer(merchant, local_id)` | `entity.read`, `villager.trade.write` | Removes one live offer owned by this module |
