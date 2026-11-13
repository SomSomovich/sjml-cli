# Registered block scheduled ticks

Registered SJML blocks deliver `block.tick` after Minecraft runs that block's
ordinary scheduled tick. Subscribe once from `#[mod_init]` and handle the
detached snapshot with `#[event]`:

```rust
use sjml::{event, events::{self, BlockTick, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.tick", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.tick")]
fn on_block_tick(event: BlockTick) -> SjmlResult<()> {
    if event.block_id == "example_pack:main/timer" {
        let powered = event.property("powered").unwrap_or("false");
        // Use separately granted, loaded-world APIs for any mutation.
        let _ = (event.position, event.dimension_id, powered, event.tick_count);
    }
    Ok(())
}
```

The event contains the canonical registered block ID, dimension, block
position, logical server tick count, and up to seven state properties as
detached strings. `BlockTick::property` looks up a captured property by name.
Only the module that declared the block receives its callback. Other modules
cannot observe the block's tick or state through this event.

The callback runs synchronously on the logical-server thread and requires the
module's `events.subscribe` grant. It is emitted only for scheduled ticks of
SJML-registered blocks; it does not observe vanilla block ticks, random ticks,
or blocks registered by another module. There is no cancellation result.
`world::schedule_block_tick` still requires `world.read` and `world.write` and
uses Minecraft's normal scheduler. Per module, that scheduling API accepts at
most 16 requests per server tick; Minecraft de-duplicates matching pending
ticks. The host caps all `block.tick` callback invocations at 512 per logical
server tick and does not call guest code from collision or rendering queries.

If a module needs to change world state in response to a scheduled tick, use a
synchronous `#[event]` handler and the relevant world write grant. Keep that
work small; the host's callback and scheduler limits do not replace a mod's
own gameplay-level throttling. Blocks without a `block.tick` subscription keep
their normal no-op scheduled-tick behavior.

## Opt-in vanilla random ticks

During `#[mod_register]`, a module can enable Minecraft's normal random-tick
selection for one of its registered blocks:

```rust
use sjml::{mod_register, registry, BlockDefinition, SjmlResult};

#[mod_register]
fn register() -> SjmlResult<()> {
    registry::register_block("crop", BlockDefinition::default())?;
    registry::set_block_random_ticking("crop", true)?;
    Ok(())
}
```

Subscribe to the distinct `block.random_tick` event from `#[mod_init]`. Its
guest export is `sjml_v1_event_block.random_tick` and its callback receives a
`BlockRandomTick` snapshot:

```rust
use sjml::{event, events::{self, BlockRandomTick, EventPriority}, mod_init, SjmlResult};

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("block.random_tick", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "block.random_tick")]
fn on_random_tick(event: BlockRandomTick) -> SjmlResult<()> {
    if event.block_id == "example_pack:main/crop" {
        let age = event.property("age").unwrap_or("0");
        // Use separately granted, loaded-world APIs for any mutation.
        let _ = (event.position, event.dimension_id, age, event.tick_count);
    }
    Ok(())
}
```

The snapshot has the same bounded schema-v1 field layout as `BlockTick`, with
up to seven detached state properties. It uses a separate event name and ABI
export so scheduled and random tick handlers cannot be confused. Callbacks run
synchronously on the logical-server thread, only for blocks owned by the
subscribing module, and never on the client. The shared host budget limits the
combined `block.tick` and `block.random_tick` callbacks to 512 per logical
server tick. Random ticking is disabled for registered blocks unless the owner
opts in during registration.
