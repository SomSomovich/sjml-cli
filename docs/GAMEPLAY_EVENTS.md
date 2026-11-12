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
