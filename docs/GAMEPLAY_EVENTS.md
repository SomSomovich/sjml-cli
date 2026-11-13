# Advancement and explosion events

The server event API exposes an observation callback for earned advancements and
a synchronous veto before a vanilla explosion is applied. Subscribe from the
module initializer:

```rust
use sjml::{event, events::{self, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("player.advancement_earned", EventPriority::Normal)?;
    events::subscribe("world.explosion_attempt", EventPriority::Normal)?;
    Ok(())
}
```

`player.advancement_earned` is a non-cancellable notification fired after a
server player completes an advancement. Its `PlayerAdvancementEarned` payload
contains the player's UUID and name, the current dimension ID, and the
advancement ID. It requires `events.subscribe`.

`player.advancement_progress` reports each individual criterion grant or
revoke after Minecraft changes the progress. Its detached
`PlayerAdvancementProgressEvent` record contains the player UUID, canonical
advancement ID, criterion name, `AdvancementProgressOperation::Grant` or
`::Revoke`, and `is_done`, which reflects completion after the change. It
requires `events.subscribe` and the separate `player.advancement.observe`
capability. The notification cannot cancel or alter progress and can use
`#[event_async]`. See [criterion progress events](PLAYER_ADVANCEMENT_PROGRESS_EVENT.md)
for the wire schema and grant/revoke example.

`world.explosion_attempt` runs synchronously from NeoForge's cancelable
`ExplosionEvent.Start`, before explosion damage, block changes, or fire are
applied. Return `Ok(true)` to cancel the whole explosion. Its detached
`ExplosionAttempt` payload contains the dimension, optional direct source
entity UUID and type, block interaction mode, center coordinates, and radius.
The source fields are both present or both absent. Radius is bounded to 128.
The callback requires both `events.subscribe` and `events.cancel`; a module
error does not cancel vanilla behavior.

Both callbacks run on the logical server thread and receive bounded schema-v1
payloads. They expose no Minecraft objects and do not cause chunks to load.

The physical-client `client.item.tooltip` callback is synchronous and can
append bounded literal lines to a vanilla tooltip; it is documented separately
in [Client item-tooltip events](ITEM_TOOLTIP_EVENTS.md).

The physical-client `client.chat.send_attempt` callback synchronously receives
the original outgoing text and may veto it. It never rewrites the text, so
allowed messages retain Minecraft's normal signed-chat path. It requires
`client.chat.send.observe`, `events.subscribe`, and `events.cancel`; see
[outgoing client chat veto](CLIENT_CHAT_SEND.md).

Logical-server modules can veto a right-click item use through
`player.item_use_attempt`. The callback runs before Minecraft calls the item's
`use` method and after the general `player.interaction` veto has passed. Return
`Ok(true)` to cancel that use. Its `PlayerItemUseAttempt` snapshot contains the
player UUID, dimension, hand, item registry ID, and stack count; an empty hand
is `minecraft:air` with count zero. It exposes no live stack, cannot change the
stack, and requires `events.subscribe` plus `events.cancel`. See
[item-use attempts](PLAYER_ITEM_USE_ATTEMPT.md).

The separate `item.use` callback lets a module handle right-click use of its
own registered items. It returns `Pass`, `Success`, or `ConsumeOne`; the last
option consumes one item only when the held stack still matches the captured
item ID and count. Other modules never receive or control this item callback.
See [module-owned item behavior](CUSTOM_ITEM_BEHAVIOR.md).

`item.use_on_block` handles a module's registered item when a player uses it
on a block. The clicked block's owner-scoped `block.use` callback runs first;
the item callback runs only if that handler passes, before vanilla `Item.useOn`.
It returns the same bounded `ItemUseAction` as `item.use`, requires
`events.subscribe` and `events.cancel`, and routes only to the held item's
declaring module. See [registered item use on blocks](ITEM_USE_ON_BLOCK.md).

`item.use_on_entity` handles a registered item during vanilla's general entity
interaction stage. NeoForge first gives `Entity.interactAt` a chance to handle
the click; if it passes, SJML applies the general `player.interaction` veto and
then routes to only the held item's declaring module before vanilla entity/item
interaction behavior. The synchronous callback receives detached player, item,
entity, position, and hand values and returns `Pass`, `Success`, or
`ConsumeOne`. It requires `events.subscribe` and `events.cancel`; see
[registered item use on entities](ITEM_USE_ON_ENTITY.md).

Server modules can observe a successful ordinary crafting result with
`player.item_crafted`. The non-cancellable `PlayerItemCrafted` payload contains
the player UUID, matched recipe ID, output item ID, and output count. The host
emits it after the crafting result is taken, alongside the existing
`inventory.changed` notification; it does not expose the ingredient grid,
change the output, or cancel crafting. It requires `events.subscribe` and runs
on the logical-server thread.

Server modules can observe a successful villager or wandering-trader offer with
`player.villager_traded`. Its `PlayerVillagerTraded` snapshot includes both
entity UUIDs, merchant type and position, dimension, villager profession and
level when available, the effective item costs, result, and offer use count.
The NeoForge event is posted only after the offer accepts the trade inputs and
cannot be cancelled or used to mutate the trade. The callback requires
`events.subscribe` and runs synchronously on the logical-server thread. See
[live villager trades](PLAYER_VILLAGER_TRADES.md).

Server modules can also observe `player.tick` after each live player's logical
server tick. Its `PlayerTick` snapshot contains the player UUID, dimension ID,
and non-negative tick count. The default cadence is 20 ticks;
`subscribe_with_interval` accepts 1 through 1,200 ticks. See
[player tick](PLAYER_TICK_EVENT.md).

Physical-client modules can observe `client.tick` after the local client tick.
Its `ClientTick` snapshot contains the non-negative tick count and uses the
same bounded subscription cadence. This callback cannot change gameplay. See
[client tick](CLIENT_TICK_EVENT.md).

Server modules can observe `entity.tick` after each loaded non-player entity
tick. The detached `EntityTick` snapshot contains the entity UUID, registry
type ID, dimension ID, position, and entity-local tick count. Its default
cadence is every 20 entity ticks; `subscribe_with_interval` accepts 1–1,200.
The host caps this event at 4,096 guest callbacks per logical-server tick.
Only subscribed server modules receive it; it is observational and does not
load chunks. See [entity tick](ENTITY_TICK_EVENT.md).

Server modules can veto a proposed animal offspring through
`entity.breed_attempt`. `EntityBreedAttempt` contains detached UUIDs and
registry type IDs for both parents and the child, plus its dimension. Return
`Ok(true)` to cancel that child spawn. This requires both `events.subscribe`
and `events.cancel`; see [cancellable breeding attempts](ENTITY_BREED_ATTEMPTS.md).

Server modules can veto a crop or other growing-age block's `CropGrowEvent.Pre`
through `block.grow_attempt`. Its `BlockGrowAttempt` snapshot contains the
dimension ID, position, and block ID. Return `Ok(true)` to set `DO_NOT_GROW`;
false leaves the current result intact and never forces growth. The event
requires both `events.subscribe` and `events.cancel`; see
[cancellable block growth](BLOCK_GROW_ATTEMPTS.md).

Server modules can inspect and veto a fishing catch through
`player.fished_attempt`. The bounded `PlayerFishedAttempt` snapshot includes
player UUID, dimension, hook position, requested rod damage, and up to 16 item
ID/count pairs without item components or NBT. Returning `Ok(true)` suppresses
the catch drops while retaining the requested rod damage. It requires both
`events.subscribe` and `events.cancel`; see
[cancellable fishing catches](PLAYER_FISHED_ATTEMPTS.md).

Server modules can veto a player's attack through `player.attack_attempt`.
`PlayerAttackAttempt` exposes detached player/target UUIDs, the target type and
dimension IDs, and target coordinates. Returning `Ok(true)` cancels the real
attack before the weapon hook or damage handling. Both `events.subscribe` and
`events.cancel` are required; see
[cancellable player attacks](PLAYER_ATTACK_ATTEMPTS.md).


Server modules can veto an impending living-entity knockback through
`entity.knockback_attempt`. The synchronous callback receives the target UUID,
type, dimension, position, original/current strength, and original/current X/Z
ratios as detached values. Return `Ok(true)` to cancel the knockback. These
parameters are read-only to the guest; the API does not change Minecraft's
strength or direction. The event can arise from damage, shield blocking, mob
attacks, or player attacks and requires `events.subscribe` plus `events.cancel`.
See [entity knockback attempts](ENTITY_KNOCKBACK_ATTEMPTS.md).
