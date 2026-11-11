# Rust guest SDK

This repository is a Cargo workspace: `sjml-cli` is the root package,
`sjml-sdk/` contains the guest API, and `sjml-macros/` contains the Rust
procedural macros used by the SDK. This workspace version provides the ABI v1
guest-side building blocks: typed resource handles, stable little-endian
serialization, a separate registration callback, item/block/block-entity/entity/fluid/recipe/particle/sound declarations,
lifecycle exports, typed event/command/service wrappers, numeric host imports,
a local async executor, versioned cross-module service calls, and a buildable
WebAssembly guest example. It also provides read-only block-entity access and
typed network channel APIs.

The SDK provides world lookup, block-state snapshots and capability-gated
block updates, player/entity snapshots, block
inventory and block-entity snapshots, bounded NBT values, and typed network
subscriptions/sends. It also provides typed persistent records and schema
migrations through the storage module, plus runtime HUD and screen closure
subscriptions and a typed OpenGL bridge for client modules. The NeoForge runtime binds network
imports during activation, registers the bounded payload types, sends and
checks connection handshakes, and routes messages on the owning logical side.
A live multiplayer session has not yet been verified end to end. Rendering
supports HUD and active-screen GUI callbacks, but a live Minecraft client render
pass has not been verified. Server-side chest menus support bounded action
callbacks while keeping item movement disabled. Guest-declared entities and block entities can
attach bounded colored or textured cuboid models with bounded sine rotations;
live client visuals remain
unverified.
Save-scoped storage uses the current server save. Physical-client modules also
have a separate `ClientProfile` scope backed by per-mod local configuration
files; they cannot access server save scopes.
Most callback macros require synchronous functions; use `#[event_async]` for
non-cancellable notifications or `task::spawn` to start owned async work from a
synchronous callback. See [Client rendering](RENDERING.md).

SDK 0.41.0 adds `client_chat::{subscribe, layout, set_layout, unsubscribe}`
with typed `ChatLayout`, `ChatLayoutHandle`, and `ChatScreenEvent` values. It
requires the separately requested and granted `client.chat.modify` capability
and runs callbacks on the client main thread. See [Native client chat layout](CHAT.md)
for the ABI contract, clamping, lifecycle, and client-profile persistence.

Client modules can subscribe to bounded active-screen mouse-click, key-press,
and typed-character events through `screen_input::subscribe`. This requires a
separately requested and granted `client.input` capability; callbacks run on
the client main thread and return `true` when Minecraft should consume the
event. The SDK provides bounded `Button`, `TextField`, `Tooltip`, and
`ProgressBar` controls for screen callbacks. See [Client rendering](RENDERING.md)
for their bounds and usage examples.

For input while no screen is open, use `client_input::subscribe` to receive
global key press/release/repeat and mouse-button press/release events. The
subscription also requires `client.input` and runs on the client main thread.
NeoForge exposes global key input as observational only; only a mouse-button
press can be consumed by returning `true`. Releases can be observed but are
never consumed. These detached input values include GLFW codes and modifier
bits, not Minecraft objects.

```rust
#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _input = client_input::subscribe(|event| {
        match event {
            ClientInputEvent::MouseButton {
                button: 1,
                action: ClientInputAction::Press,
                ..
            } => Ok(true), // prevent this press from reaching vanilla
            _ => Ok(false),
        }
    })?;
    Ok(())
}
```

## Target and build

Install Rust's `wasm32-unknown-unknown` standard library target:

```sh
rustup target add wasm32-unknown-unknown
```

Build a real SDK guest from the repository root:

```sh
cargo build -p sjml-sdk --example guest --target wasm32-unknown-unknown --locked
```

Run the SDK and macro tests:

```sh
cargo test --workspace --locked
```

`wasm32-unknown-unknown` is the only guest target. WASI targets are rejected
because the SJML guest ABI does not grant ambient filesystem, process, or raw
socket access. Named, capability-gated SJML networking is a separate API.
Native builds remain available for Rust-side tests; they do not emit callback
exports or make host imports callable.

## Guest crate

The SDK package is named `sjml-sdk` and its Rust crate is `sjml`:

```toml
[dependencies]
sjml-sdk = "0.41.0"
```

For development against this checkout, use
`sjml-cli init mymod --sdk-path ./sjml-sdk` from the repository root; otherwise
Cargo resolves the published SDK version. The
`sjml-cli` executable creates guest projects and builds, checks, tests,
packages, and inspects `.sjml` archives. It does not install or hot-reload a
package in Minecraft; see the [project configuration guide](PROJECT_CONFIGURATION.md).

## Public API map

The SDK's Rust crate name is `sjml`; each public module below is linked to its
versioned [docs.rs API](https://docs.rs/sjml-sdk/0.41.0/sjml/). The exact
function signatures and public types there are the reference for Rust callers.

| Module | Main API |
| --- | --- |
| [`abi`](https://docs.rs/sjml-sdk/0.41.0/sjml/abi/) | Typed, generation-checked handles, wire spans, host input allocation, and guest ABI glue. |
| [`batch`](https://docs.rs/sjml-sdk/0.41.0/sjml/batch/) | Batches only imports that the host marks as batchable. |
| [`block_entity`](https://docs.rs/sjml-sdk/0.41.0/sjml/block_entity/) | Loaded block-entity lookup and detached snapshots, including bounded NBT. |
| [`client_chat`](https://docs.rs/sjml-sdk/0.41.0/sjml/client_chat/) | Native chat layout lifecycle, geometry, and pointer callbacks. |
| [`client_input`](https://docs.rs/sjml-sdk/0.41.0/sjml/client_input/) | Global key and mouse-button events when no screen is open. |
| [`entity`](https://docs.rs/sjml-sdk/0.41.0/sjml/entity/) | Loaded non-player entity snapshots, enumeration, and effect application. |
| [`entity_data`](https://docs.rs/sjml-sdk/0.41.0/sjml/entity_data/) | Bounded module-owned opaque bytes attached to non-player entities. |
| [`events`](https://docs.rs/sjml-sdk/0.41.0/sjml/events/) | Typed lifecycle, world, player, entity, inventory, and resource events. |
| [`gl`](https://docs.rs/sjml-sdk/0.41.0/sjml/gl/) | Typed OpenGL resources, bounded command submission, and GUI textures. |
| [`host`](https://docs.rs/sjml-sdk/0.41.0/sjml/host/) | ABI version query and host logging. |
| [`inventory`](https://docs.rs/sjml-sdk/0.41.0/sjml/inventory/) | Detached snapshots of loaded block inventories and bounded item updates. |
| [`menu`](https://docs.rs/sjml-sdk/0.41.0/sjml/menu/) | Bounded server menus with read-only slots and explicit action callbacks. |
| [`network`](https://docs.rs/sjml-sdk/0.41.0/sjml/network/) | Declared network channels, subscriptions, sends, authenticated context, and replies. |
| [`player`](https://docs.rs/sjml-sdk/0.41.0/sjml/player/) | Online player handles, detached snapshots, and supported effect operations. |
| [`recipe`](https://docs.rs/sjml-sdk/0.41.0/sjml/recipe/) | Typed recipe callback inputs and evaluation results. |
| [`registry`](https://docs.rs/sjml-sdk/0.41.0/sjml/registry/) | Static item, block, entity, fluid, recipe, sound, particle, effect, and related declarations. |
| [`render`](https://docs.rs/sjml-sdk/0.41.0/sjml/render/) | HUD, screen, and camera-relative world callbacks, widgets, and model drawing. |
| [`screen_input`](https://docs.rs/sjml-sdk/0.41.0/sjml/screen_input/) | Bounded mouse, key, and text events delivered to the active screen. |
| [`storage`](https://docs.rs/sjml-sdk/0.41.0/sjml/storage/) | Stable typed records, save/client-profile scopes, and schema migrations. |
| [`task`](https://docs.rs/sjml-sdk/0.41.0/sjml/task/) | Owned async work, timers, cancellation, worker compute, and asynchronous service calls. |
| [`widgets`](https://docs.rs/sjml-sdk/0.41.0/sjml/widgets/) | Bounded button, text-field, tooltip, and progress-bar state helpers. |
| [`world`](https://docs.rs/sjml-sdk/0.41.0/sjml/world/) | Loaded-world handles, block-state snapshots, and capability-gated block updates. |
| [`prelude`](https://docs.rs/sjml-sdk/0.41.0/sjml/prelude/) | Common callbacks, derives, handles, event types, and result/error types. |

Stable codecs (`StableEncode`, `StableDecode`, `Encoder`, and `Decoder`) are
re-exported at the crate root; the implementation module is intentionally
private. The crate root also re-exports common typed handles, records, error
types, render types, and widget types; docs.rs lists the complete set. The SDK
re-exports the procedural macros `mod_register`,
`mod_init`, `mod_shutdown`, `event`, `event_async`, `command`, `service`,
`recipe`, and `compute`. This guide documents their SJML-specific callback
contracts; `docs.rs` provides the full type reference. The low-level WebAssembly
calling convention and every import signature are in the [ABI v1 reference](ABI.md).

Use `sjml::prelude::*` for the callback attributes, result/error types,
serialization derives, common handles, and logging API. Guest functions stay
ordinary Rust functions; the macro adds the ABI entrypoint on wasm32.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("ruby", ItemDefinition::default())?;
    registry::register_block("ruby_block", BlockDefinition::default())?;
    Ok(())
}

#[mod_init]
fn initialize() -> SjmlResult<()> {
    host::log(LogLevel::Info, "loaded")?;
    // Keep a returned handle in guest state if you will later unsubscribe.
    let _started = events::subscribe("server.started", EventPriority::Normal)?;
    let _stopping = events::subscribe("server.stopping", EventPriority::Normal)?;
    let _joined = events::subscribe("player.joined", EventPriority::Normal)?;
    let _left = events::subscribe("player.left", EventPriority::Normal)?;
    let _death = events::subscribe("entity.death", EventPriority::Normal)?;
    let _spawned = events::subscribe("entity.spawned", EventPriority::Normal)?;
    let _effect_tick = events::subscribe("entity.effect.tick", EventPriority::Normal)?;
    let _break = events::subscribe("block.break", EventPriority::Normal)?;
    let _place = events::subscribe("block.place", EventPriority::Normal)?;
    let _interacted = events::subscribe("player.interacted", EventPriority::Normal)?;
    let _damaged = events::subscribe("entity.damaged", EventPriority::Normal)?;
    let _world_loaded = events::subscribe("world.loaded", EventPriority::Normal)?;
    let _world_unloaded = events::subscribe("world.unloaded", EventPriority::Normal)?;
    let _server_tick = events::subscribe_with_interval("server.tick", EventPriority::Normal, 20)?;
    let _inventory_changed = events::subscribe("inventory.changed", EventPriority::Normal)?;
    let _resources_reloaded = events::subscribe("resources.reloaded", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "server.started")]
fn on_server_started(_: ()) -> SjmlResult<()> {
    Ok(())
}

#[event(name = "server.stopping")]
fn on_server_stopping(_: ()) -> SjmlResult<()> {
    Ok(())
}

#[event(name = "player.joined")]
fn on_player_joined(player: PlayerJoined) -> SjmlResult<()> {
    host::log(LogLevel::Info, &player.display_name)?;
    Ok(())
}

#[event(name = "player.left")]
fn on_player_left(player: PlayerLeft) -> SjmlResult<()> {
    host::log(LogLevel::Info, &player.display_name)?;
    Ok(())
}

#[event(name = "entity.death")]
fn on_entity_death(event: EntityDeath) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.entity_type_id)?;
    Ok(())
}

#[event(name = "entity.spawned")]
fn on_entity_spawned(event: EntitySpawned) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.entity_type_id)?;
    Ok(())
}

#[event(name = "entity.effect.tick")]
fn on_effect_tick(event: EffectTickEvent) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.effect_id)?;
    Ok(())
}

#[event(name = "block.break")]
fn on_block_break(event: BlockBreak) -> SjmlResult<bool> {
    Ok(false)
}

#[event(name = "block.place")]
fn on_block_place(event: BlockPlace) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.block_id)?;
    Ok(())
}

#[event(name = "player.interacted")]
fn on_player_interacted(event: PlayerInteracted) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.action)?;
    Ok(())
}

#[event(name = "entity.damaged")]
fn on_entity_damaged(event: EntityDamaged) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.entity_type_id)?;
    Ok(())
}

#[event(name = "world.loaded")]
fn on_world_loaded(event: WorldLifecycle) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.dimension_id)?;
    Ok(())
}

#[event(name = "world.unloaded")]
fn on_world_unloaded(event: WorldLifecycle) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.dimension_id)?;
    Ok(())
}

#[event(name = "server.tick")]
fn on_server_tick(event: ServerTick) -> SjmlResult<()> {
    host::log(LogLevel::Debug, &event.tick_count.to_string())?;
    Ok(())
}

#[event(name = "inventory.changed")]
fn on_inventory_changed(event: InventoryChanged) -> SjmlResult<()> {
    host::log(LogLevel::Info, &event.item_id)?;
    Ok(())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EchoArgs {
    message: String,
    repeat: i32,
    loud: bool,
}

#[command(name = "chat.echo")]
fn echo(args: EchoArgs) -> SjmlResult<String> {
    if !(1..=5).contains(&args.repeat) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "repeat must be between 1 and 5",
        ));
    }
    let reply = format!("{} x {}", args.message, args.repeat);
    let reply = if args.loud {
        reply.to_uppercase()
    } else {
        reply
    };
    if reply.len() > MAX_STRING_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "command reply exceeds the stable string limit",
        ));
    }
    Ok(reply)
}

#[service(name = "chat.player_count")]
fn player_count(_: ()) -> SjmlResult<u32> {
    Ok(0)
}
```

The subscription calls in this example assume a `side = "server"` module whose
manifest and administrator grant include both event capabilities. A
`side = "both"` module receives a separate client instance, where subscription
calls fail.

`#[mod_register]`, `#[mod_init]`, and `#[mod_shutdown]` accept a no-argument
function returning `SjmlResult<()>`. `mod_register` is invoked before the
NeoForge registry events; use it only to stage static declarations. `mod_init`
runs after registry registration. The event, command, and service attributes each require one
`name = "..."` identifier and one typed payload argument, returning
`SjmlResult<T>`. Callback identifiers are at most 128 bytes and contain only
lowercase ASCII letters, digits, `.`, `_`, and `-`; their first character is
an ASCII letter or digit.

### Manifest-declared commands

The `#[command]` attribute exports the callback; it does not choose its
arguments. Declare the Brigadier argument schema in the same module's
`sjml.toml`. The command name must match the callback identifier, and the
ordered manifest arguments must match the fields in the Rust input record by
type and position. For the `EchoArgs` callback above, add this declaration to
the module:

```toml
[[modules]]
id = "content"
path = "modules/content.wasm"
side = "server"
permissions = ["content.register", "events.subscribe", "events.cancel"]

[[modules.commands]]
name = "chat.echo"
schema_version = 1
permission_level = 2

[[modules.commands.arguments]]
name = "message"
type = "string"

[[modules.commands.arguments]]
name = "repeat"
type = "integer"
min = 1
max = 5

[[modules.commands.arguments]]
name = "loud"
type = "boolean"
```

`schema_version` must match `#[sjml(version = N)]` on the callback input
record. The names in the manifest are used for Brigadier parsing and
tab-completion; the record's Rust field names are not encoded. All arguments
are required, with no defaults, and the manifest declaration order is the
record field order. Supported types are `word`, `string`, `greedy_string`,
`integer`, and `boolean`. The first three decode as `String`, `integer` as
`i32`, and `boolean` as `bool`. Only `integer` accepts inclusive `min` and
`max` bounds; absent bounds use the corresponding full `i32` limit.
`greedy_string` must be the final argument. Each string is limited to 64 KiB
of UTF-8 and the complete callback input to 1 MiB. A module can declare at
most 64 commands with at most 16 arguments each; the host stages at most
8,192 commands across all packages.

The module must request `content.register`, and the administrator must grant
it in `config/sjml-permissions.toml`. A command runs at
`/sjml guest <package-id> <module-id> <command-name> ...`; the example is
`/sjml guest ruby_pack content chat.echo "hello there" 3 true`. Its
`permission_level` is checked by the server when parsing and executing the
command; it does not grant extra host capabilities. The complete manifest
rules and input byte layout are in [FORMAT.md](FORMAT.md) and [ABI.md](ABI.md).

```toml
[[grant]]
module = "ruby_pack/content"
capabilities = ["content.register", "events.subscribe", "events.cancel"]
```

This example module also subscribes to events, so its manifest and administrator
grant include both event capabilities in addition to `content.register`.
Guest callback errors, traps, or malformed replies disable only the module
whose command callback failed; the command reports a generic failure to its
caller. Brigadier rejects malformed command input before invoking the guest.

The `#[event]` attributes above export callbacks; they do not register them for
delivery. From `mod_init` in a retained logical-server instance, explicitly
subscribe with `events::subscribe(event_name, priority)`. The NeoForge adapter
recognizes `server.started`, `server.stopping`, `player.joined`, `player.left`,
`entity.death`, `entity.spawned`, `entity.effect.tick`, `block.break`,
`block.place`, `player.interacted`, `entity.damaged`, `world.loaded`, and
`world.unloaded`, `server.tick`, `inventory.changed`, and `resources.reloaded`;
callbacks run on the owning logical-server thread. The fifteen notifications require
`events.subscribe`. The cancellable
`block.break` callback requires both `events.subscribe` and the distinct
`events.cancel` capability. Both capabilities appear in the combined module
and administrator-grant declarations above.

`events::subscribe` returns an `EventSubscriptionHandle`. Keep that handle if
you need to call `events::unsubscribe(handle)` later; closing the module also
removes its subscriptions. Each active module instance may hold at most 256
subscriptions. Event
names must be 1–128 bytes of lowercase ASCII letters, digits, `.`, `_`, or `-`,
starting with a letter or digit, and must be one of the sixteen names listed
above. Priorities are `Highest`, `High`, `Normal`, `Low`, and `Lowest` in that
order; equal priorities retain module activation order and subscription order.
The subscription imports are available only when `events.subscribe` is both
requested and granted. A `block.break` subscription also requires the request
and grant for `events.cancel`. Subscriptions are accepted only in integrated-server
and dedicated-server contexts. A `side = "both"` module has a separate client
instance, where subscription calls fail; there is no SDK context-query API to
guard such a call from `mod_init`. The static `mod_register` instance cannot
subscribe.

Use `#[event_async(name = "...")]` for non-cancellable event work that can
continue after the event callback returns. The generated ABI callback queues
the async function on the normal bounded guest task runtime and returns
immediately, so it cannot change the original event decision or block the
server tick while awaiting:

```rust
use std::time::Duration;
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _joined = events::subscribe("player.joined", EventPriority::Normal)?;
    Ok(())
}

#[event_async(name = "player.joined")]
async fn welcome_after_a_delay(player: PlayerJoined) -> SjmlResult<()> {
    task::sleep(Duration::from_secs(2)).await?;
    host::log(LogLevel::Info, &format!("Welcome, {}", player.display_name))?;
    Ok(())
}
```

An async event function must take one owned stable-decoded payload and return
`SjmlResult<()>`. Its failures and task-admission failures are logged; they do
not cancel or delay the originating event. Async work is subject to the usual
per-module task count, timeout, cancellation, and shutdown rules. The
`block.break` event remains synchronous: `#[event_async(name = "block.break")]`
is rejected because only a synchronous result can veto the break.

## Stable values and handles

`#[derive(StableEncode, StableDecode)]` is restricted to structs with named
fields and requires `#[sjml(version = N)]` with a positive integer. The
version is written first as a little-endian `u32`, followed by fields in Rust
declaration order. A decoder rejects a different version instead of guessing
how to migrate it. Use an explicit new version and migration at the module or
application boundary when a schema changes.

The built-in scalar widths are fixed: integers use their declared width,
booleans use one byte (`0` or `1`), strings use a little-endian `u32` byte
length followed by strict UTF-8, vectors use a `u32` item count, and options
and results use a one-byte discriminant (`0` or `1`). Multibyte values are
little-endian. Encoded payloads are limited to 1 MiB, strings to 64 KiB, and
collections to 65,536 elements. Decoding also caps aggregate `Vec` backing
storage at 64 MiB per payload, even when several individual vectors are each
within their element-count limit.

Typed handles serialize as the ABI's three little-endian `u64` words: owner
scope, execution context, and packed slot/generation. Handle aliases such as
`EntityHandle` and `WorldHandle` prevent mixing kinds in Rust. A handle is
still only an opaque token; the host must validate its owner, context,
generation, kind, and capability on every operation.

## Generated callback ABI

On wasm32, the macros emit these WebAssembly exports:

| Rust attribute | Export |
| --- | --- |
| `#[mod_register]` | `sjml_v1_mod_register` |
| `#[mod_init]` | `sjml_v1_mod_init` |
| `#[mod_shutdown]` | `sjml_v1_mod_shutdown` |
| `#[event(name = "x")]` | `sjml_v1_event_x` |
| `#[command(name = "x")]` | `sjml_v1_command_x` |
| `#[service(name = "x")]` | `sjml_v1_service_x` |

Lifecycle functions return numeric status `0` on success and the stable ABI
error code on failure. Event, command, and service exports take
`(input_ptr: u32, input_len: u32)` and return an `i64` packed `ByteSpan`: the
pointer is in the low 32 bits and the output length in the high 32 bits. Input
must be the stable encoding of the declared argument type.

The callback result span is limited to 1 MiB including its header and starts
with a little-endian `u32` status. On success, the remaining bytes are the
stable encoding of `T` (so the result value can use at most 1 MiB minus four
bytes). On failure, the span is
the status followed by a `u32` UTF-8 message length and the message bytes. The
span is allocated in guest memory; after copying it, the host must call
`sjml_v1_dealloc(pointer, length)` exactly once with the exact returned span.
Every returned span has at least the four-byte status prefix. A handler panic
is not converted into an SJML error and must be treated by the runtime as a
guest trap.

For a non-empty callback input, the SDK exports `sjml_v1_alloc(length)` and
`sjml_v1_dealloc_input(pointer, length)`. The host asks the guest allocator for
space, copies the stable input bytes there, and releases that exact allocation
after the callback, including on callback failure. Empty inputs use a zero
pointer and length. The host copies and validates a returned frame before
calling `sjml_v1_dealloc` exactly once. The core Java `CompiledModule` provides
this bounded callback transport. NeoForge invokes only callbacks with an
active runtime subscription. It dispatches the unit-valued `server.started`
and `server.stopping` exports, the `player.joined`, `player.left`,
`entity.death`, `entity.spawned`, and `entity.effect.tick` exports, the
`block.place`, `player.interacted`, `entity.damaged`, `world.loaded`, and
`world.unloaded`, `server.tick`, `inventory.changed`, and `resources.reloaded`
exports, and the
cancellable `block.break` export. The player
events accept the SDK's schema-v1 `PlayerJoined` or `PlayerLeft` record with
`player_id` (canonical UUID string) then `display_name` (profile name).
`entity.death` accepts schema-v1 `EntityDeath` with `entity_id`,
`entity_type_id`, and `dimension_id` stable strings followed by finite
little-endian `f64` coordinates `x`, `y`, and `z`. The guest callback cannot
cancel the NeoForge death event. `entity.spawned` accepts schema-v1
`EntitySpawned` with the same identity strings and finite coordinates followed
by `loaded_from_disk: bool`; it reports non-cancelled entity join events.
`block.place` accepts schema-v1 `BlockPlace` with `placer_id: Option<String>`,
`dimension_id`, `block_id`, and signed `i32` coordinates. Both are
non-cancellable notifications. `entity.effect.tick` accepts schema-v1
`EffectTickEvent` with effect, entity, type, and dimension IDs; finite `f64`
position; signed amplifier and remaining duration; and the configured `u32`
interval. It runs after a living entity tick on the logical server, only for
effects registered with `registry::register_effect_tick`; interval values are
limited to 1–1,200 ticks.
`player.interacted` accepts `PlayerInteracted` with `player_id`, `dimension_id`,
`action`, and `hand` strings, optional `target_id` and `face` strings, and an
optional nested schema-v1 `BlockPos`. It observes block clicks, entity
interactions, and item clicks as attempts; it does not change the underlying
interaction result. `entity.damaged` accepts `EntityDamaged` with entity,
dimension, and damage type IDs, an optional source entity UUID, finite `f64`
coordinates, then finite non-negative `f32` original and applied damage. It is
sent after NeoForge applies damage to a living entity and cannot change it.
`world.loaded` and `world.unloaded` accept `WorldLifecycle` with a dimension ID
and signed `i64` game time. Initial server levels are reported after runtime
activation; subsequent level loads and unloads are reported as they occur, with
unload notification sent before world-scoped handles are revoked.
`server.tick` accepts schema-v1 `ServerTick` with a non-negative `i64`
`tick_count`; it runs after the server tick and cannot cancel it. A plain
`events::subscribe("server.tick", priority)` delivers every 20 ticks, while
`events::subscribe_with_interval("server.tick", priority, ticks)` selects a
cadence from 1 through 1,200 ticks. This bounds default guest work to one
callback per second; the guest can request a more frequent rate explicitly.
`resources.reloaded` accepts schema-v1 `ResourcesReloaded` with a non-negative
`i64` `tick_count`. It fires once after a successful server resource-manager
replacement is active, on the next server tick; failed reloads do not emit it.
This is a synchronous, observation-only notification and cannot affect reload.
`inventory.changed` accepts schema-v1 `InventoryChanged` with the canonical
player UUID, item registry ID, a change type string (`picked_up`, `crafted`, or
`dropped`), an optional inventory slot, and a positive item count. The item ID
is limited to 256 UTF-8 bytes, slots to `0..=255`, and counts to `1..=4096`.
Pickup uses NeoForge's successful post-insertion hook; crafting uses the
successful result-take hook. A cancellable toss is reported on the next server
tick only after its item entity is confirmed in the level. The slot is `None`
when the hook cannot identify the exact player inventory slot.
`block.break` accepts the schema-v1 `BlockBreak` record with `player_id`,
`dimension_id`, and `block_id` stable strings, then signed `i32` coordinates
`x`, `y`, and `z`. Its handler returns `SjmlResult<bool>`: only the exact stable
boolean encodings `0` and `1` are accepted, and any authorized `true` vetoes
the break. A malformed return or callback error disables only that module and
does not veto. All sixteen event adapters run on the owning logical-server
thread, in subscription priority order. The fifteen notifications require the
manifest request and administrator grant for `events.subscribe`, while
`block.break` requires both `events.subscribe` and `events.cancel`. Guest
commands use the manifest schema described above.

This defines the SDK guest convention. The NeoForge bootstrap invokes the
no-argument `mod_register` export for static item/block declarations, then
keeps a distinct compiled instance per execution context and invokes
`mod_init` after registry setup. A module with the `events.subscribe`
manifest permission and administrator grant must also call
`events::subscribe` for each callback it wants delivered. It can export either
`#[event(name = "server.started")] fn on_started(_: ()) -> SjmlResult<()>`
or `#[event(name = "server.stopping")] fn on_stopping(_: ()) -> SjmlResult<()>`
or `#[event(name = "player.joined")] fn on_joined(player: PlayerJoined) ->
SjmlResult<()>` or `#[event(name = "player.left")] fn on_left(player:
PlayerLeft) -> SjmlResult<()>` or `#[event(name = "entity.death")] fn
on_entity_death(event: EntityDeath) -> SjmlResult<()>`, `#[event(name =
"entity.spawned")] fn on_entity_spawned(event: EntitySpawned) -> SjmlResult<()>`,
or `#[event(name = "block.place")] fn on_block_place(event: BlockPlace) ->
SjmlResult<()>`. NeoForge invokes subscribed handlers on the
owning logical-server thread. Manifest-declared guest commands are wired into
the NeoForge Brigadier tree. Declared service exports are routed between active
server modules; client service providers remain unavailable. To
cancel block breaks, request and grant both capabilities and export
`#[event(name = "block.break")] fn on_break(event: BlockBreak) ->
SjmlResult<bool>`, then subscribe to `block.break`; returning `true` vetoes a
break.

## Host imports and capabilities

The current safe guest wrappers declare two baseline synchronous numeric
imports:

| Import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml.abi_version` | `() -> i32` | Returns the host-supported ABI major version. |
| `sjml.log` | `(level: i32, ptr: i32, len: i32) -> i32` | Reads strict UTF-8 from guest memory and returns `0` or a stable error code. |

Log levels are `1=error`, `2=warn`, `3=info`, `4=debug`, `5=trace`. Log text
is limited to 64 KiB. The wrappers keep pointer handling inside the SDK; user
callbacks pass `&str` and receive `SjmlResult`.

The portable core runtime starts with an empty import allowlist, so declaring a
wrapper in the SDK does not enable it. The NeoForge adapter binds
`sjml.abi_version` and `sjml.log` unconditionally; these baseline imports need
no administrator grant. Registry imports are bound only when both the
manifest request and administrator grant are present. Event subscription
imports are described below. WASI imports are neither enabled nor substituted.

### Event subscription imports

The SDK exposes `events::subscribe(&str, EventPriority) ->
SjmlResult<EventSubscriptionHandle>`, `events::subscribe_with_interval(&str,
EventPriority, u32) -> SjmlResult<EventSubscriptionHandle>`, and
`events::unsubscribe(EventSubscriptionHandle) -> SjmlResult<()>`. On wasm32,
these use the `sjml_events_v1` imports:

| Import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_events_v1.subscribe` | `(event_ptr: i32, event_len: i32, priority: i32, handle_output_ptr: i32) -> i32` | Registers the exported callback and writes a 24-byte handle to guest memory. |
| `sjml_events_v1.subscribe_interval` | `(event_ptr: i32, event_len: i32, priority: i32, interval_ticks: i32, handle_output_ptr: i32) -> i32` | Registers a bounded-cadence `server.tick` callback and writes its handle. |
| `sjml_events_v1.unsubscribe` | `(handle_ptr: i32, handle_len: i32) -> i32` | Removes the live subscription represented by an exact 24-byte handle. |

All three imports return `0` on success or a nonzero stable ABI error code. The
subscribe output pointer must cover 24 writable bytes; unsubscribe requires a
length of exactly 24. The handle uses the ABI's three little-endian `u64`
words: owner scope token, execution-context token, and packed slot/generation.
It is module- and context-scoped, non-transferable, and invalid after
unsubscription or module close. The `priority` argument is `0` for `Highest`,
`1` for `High`, `2` for `Normal`, `3` for `Low`, and `4` for `Lowest`; lower
values dispatch first, with ties preserving module activation and registration
order. The host accepts at most 256 subscriptions per active module instance.
`subscribe_with_interval` is valid only for `server.tick` and accepts intervals
from 1 through 1,200 ticks. The ordinary `subscribe` call uses 20 ticks for
`server.tick` and does not accept an interval for other event categories.

The host validates an event name as strict UTF-8 of 1–128 bytes using only
lowercase ASCII letters, digits, `.`, `_`, or `-`, with a letter or digit first;
it must also be one of the sixteen event names above. `events.subscribe` must be
requested in the module descriptor and granted by the administrator before
these imports are bound. A `block.break` subscription also requires the
separate `events.cancel` request and grant. Calls are accepted only from
integrated-server or dedicated-server execution contexts. Static registration
instances cannot subscribe. A callback failure disables only that module; an
invalid `block.break` result or callback failure does not veto the break.
Event callbacks are synchronous at the ABI boundary. A `#[event_async]`
callback queues its non-cancellable work on the module's bounded task runtime
and returns before that work runs to completion.

### Batched read imports

`batch::Batch` groups approved host reads into one transition across the
guest/host boundary. The host exposes only imports that the adapter explicitly
marked batchable, and it still checks each operation's capability, handle
owner, execution context, and thread requirements. A batch runs its operations
in order; results stay in the same order and are available by the indices
returned from `push`.

```rust
use sjml::{Batch, BatchValue};

let mut batch = Batch::new();
let game_time_index = batch.push("sjml_world_v1", "game_time", &[
    BatchValue::I32(handle_pointer as i32),
    BatchValue::I32(24),
    BatchValue::I32(time_output_pointer as i32),
])?;
let results = batch.execute()?;
let status = results[game_time_index].values[0]; // I32(0) on success
```

`BatchValue` represents raw `i32`, `i64`, `f32`, or `f64` values. The result
variants must match the selected import's declared numeric signature; adapter
imports commonly return one `i32` status, while memory-writing imports encode
their larger result into the caller-provided guest buffer. In this example,
`handle_pointer` points to the handle's 24-byte stable encoding and
`time_output_pointer` points to an eight-byte writable allocation; both guest
allocations must remain live until `execute` returns. Requests are limited to
1,024 calls, 16 arguments per call, 128 UTF-8 bytes per import name, and 1 MiB
total. The batch dispatcher itself cannot be nested in a batch. On native
targets `push` can be validated in tests, but `execute` requires a
`wasm32-unknown-unknown` guest. The SDK accepts short ASCII import identifiers;
the lower-level ABI wire format permits strict UTF-8 names. See [the binary contract](ABI.md#batched-host-imports)
for the exact wire format and error behavior. One host transition does not
guarantee a faster call: the generic dispatcher is not automatically faster
than direct calls for trivial operations. Measure the target workload before
using it for performance.

## Static content and creative-tab declarations

`registry::register_item(path, ItemDefinition)` and
`registry::register_block(path, BlockDefinition)` are synchronous imports for
`#[mod_register]`. They copy the path from guest memory, validate it, and stage
declarations for the NeoForge item and block registry events. Paths are
1–256 lowercase ASCII letters, digits, `.`, `_`, or `-`. Item stacks are
limited to 1–99. Block strength and resistance must be finite and
non-negative. A generated `BlockItem` is emitted during the item registry
event and points at the exact block instance registered earlier. The module
must request `content.register`, and the host must grant it in
`config/sjml-permissions.toml`; grants are empty when that file is absent.
Use one explicit TOML record for each package module:

```toml
[[grant]]
module = "ruby_pack/content"
capabilities = ["content.register"]
```

The same `#[mod_register]` callback may stage a generic block entity attached
to a block declared by that module, plus a `MobCategory.MISC` living entity
type with bounded vanilla base attributes and optional vanilla AI goals:

```rust
#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_block("ruby_block", BlockDefinition::default())?;
    registry::register_block_entity("ruby_machine", "ruby_block")?;
    registry::register_entity("ruby_mite", EntityDefinition::default())?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::Swim { priority: 0 })?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::TargetPlayer {
        priority: 1, must_see: true,
    })?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::MeleeAttack {
        priority: 2, speed: 1.0, long_memory: true,
    })?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::RandomStroll {
        priority: 5, speed: 0.8,
    })?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::LookAtPlayer {
        priority: 6, range: 8.0,
    })?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::RandomLookAround { priority: 7 })?;
    registry::register_entity_ai_goal("ruby_mite", EntityAiGoal::Retaliate { priority: 0 })?;
    registry::set_entity_attribute("ruby_mite", EntityAttribute::MaxHealth, 40.0)?;
    registry::set_entity_attribute("ruby_mite", EntityAttribute::MovementSpeed, 0.3)?;
    Ok(())
}
```

The host creates generic Java backing classes during NeoForge's block,
block-entity-type, and entity-type registry events. Registered entities have
no AI goals by default. `register_entity_ai_goal` adds up to sixteen bounded
vanilla goals after the entity declaration; goal priorities are 0–31, movement
speeds are 0.01–8, and look ranges are 1–64 blocks. The supported goals are
swimming, random strolling, looking at players, melee attacking, random looking,
targeting nearby players, and retaliating against a recent attacker. Goals are
attached to the declared entity type before any instance can spawn. Raw guest
callbacks, custom goal code, and guest-defined synchronized fields remain
unavailable. `EntityAttribute` values are applied during
`EntityAttributeCreationEvent`; supported base attributes are max health
(1–1,024), movement speed (0–1), attack damage (0–1,024), armor (0–30), follow
range (0–1,024), and knockback resistance (0–1). Each attribute may be declared
once. For persistent opaque entity bytes, modules use `entity_data` with the
separate `entity.data.read` grant for reads and `entity.data.write` for setting
and removing values; these grants do not expose entity snapshots or synced
fields. The client uses a shared magenta wireframe unless a guest cuboid model
is attached.
Store per-block data through the existing block-entity storage scope. Entity
dimensions are 0.01–16 blocks, tracking range is 1–128 chunks, and update
interval is 1–1,200 ticks. Use application events and explicit server
operations for behavior; static registration only declares registry objects.

`registry::register_entity_model(path, EntityModelDefinition)` and
`registry::register_block_entity_model(path, BlockEntityModelDefinition)` attach
one model to a type declared earlier by the same module. Add 1–32 colored
cuboids in entity-local coordinates bounded to +/-4 world units, or block-local
coordinates within `[0,1]`. Call `set_texture` with a package-owned PNG resource
such as `example:textures/entity/clockwork_mite.png` to render the full texture
on every cuboid face; the cuboid ARGB value tints the image. The image belongs
at `assets/example/textures/entity/clockwork_mite.png`. Omit `set_texture` to
render vertex colors. Use `set_animation(ModelAnimation::sine_rotation(...))`
for a deterministic host-side rotation around a local pivot; axes are X/Y/Z,
amplitude is limited to 180 degrees and periods to 12,000 ticks. The host
evaluates animation from game time, without calling guest code per frame. The
host caps model geometry at 256 cuboids per module and 8,192 total. Minecraft's
buffered vertex path renders these models without a raw OpenGL grant. See
[Client rendering](RENDERING.md) for an example.

A module can declare a simple water-like fluid with a source, flowing state,
liquid block, and filled bucket:

```rust
#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_fluid("ruby_fluid", FluidDefinition::default())?;
    registry::register_creative_tab(
        "ruby",
        CreativeTabDefinition::new("Ruby", "ruby_fluid_bucket")?,
    )?;
    registry::add_to_creative_tab("ruby", "ruby_fluid_bucket")?;
    Ok(())
}
```

The host registers IDs ending in `_source`, `_flowing`, `_block`, and `_bucket`
(or `_liquid` if the primary generated block ID is already taken);
fluid paths may be at most 248 ASCII bytes so those IDs remain within the host's
256-byte resource path limit.
`FluidDefinition` bounds tick rate to 1–1,200 ticks, level decrease to 1–8,
slope search to 1–16 blocks, and finite explosion resistance to 0–1,000,000.
The default uses a 5-tick rate, level decrease 1, slope search 4, and
resistance 100. `register_fluid` uses NeoForge's vanilla water type and
textures. To skin a fluid, use `register_fluid_with_visuals` with package-owned
still and flowing texture IDs:

```rust
let visuals = FluidVisualDefinition::new(
    "ruby_pack:block/ruby_still",
    "ruby_pack:block/ruby_flow",
)?;
registry::register_fluid_with_visuals(
    "ruby_fluid",
    FluidDefinition::default(),
    visuals,
)?;
```

Those IDs refer to `assets/ruby_pack/textures/block/ruby_still.png` and
`assets/ruby_pack/textures/block/ruby_flow.png`; optional `.png.mcmeta` files
are mounted too. The host registers a custom NeoForge fluid type for the
textures and generates blockstate, block model, bucket model, and English
language metadata. Both the normal and fallback liquid-block IDs receive
metadata. The bucket model currently uses Minecraft's filled-water-bucket
icon; the still and flowing textures skin the fluid in the world. Package
assets under `assets/<package-id>/` are mounted automatically. The
content-registration grant is required.

Shaped and shapeless recipes can be staged from the same `#[mod_register]` callback:

```rust
registry::register_shapeless_recipe(
    "ruby_block",
    ShapelessRecipeDefinition {
        output_item: "ruby_block".into(),
        output_count: 1,
        ingredients: vec!["ruby".into(); 9],
    },
)?;

registry::register_shaped_recipe(
    "ruby_pickaxe",
    ShapedRecipeDefinition {
        output_item: "ruby_pickaxe".into(),
        output_count: 1,
        pattern: vec!["RRR".into(), " S ".into(), " S ".into()],
        key: vec![
            ShapedRecipeKey { symbol: 'R', ingredient: RecipeIngredient::tag("c:gems/ruby") },
            ShapedRecipeKey { symbol: 'S', ingredient: "minecraft:stick".into() },
        ],
    },
)?;
```

Custom registry tags are included as package data files under
`data/<mod.id>/tags/<registry-path>/`; the
[package format](FORMAT.md#server-registry-tags) documents their JSON schema
and size bounds. The host also mounts package loot tables, world-generation
JSON, and NeoForge biome modifiers; see the [server package-data JSON
section](FORMAT.md#server-package-data) for supported paths and bounds.

An unqualified item or tag ID refers to the declaring module; use a namespaced
ID such as `minecraft:iron_ingot` or `c:ingots/iron` for vanilla or another
mod's content. The host writes
these declarations to an always-enabled generated server data pack, where
Minecraft's vanilla `minecraft:crafting_shapeless` and
`minecraft:crafting_shaped` serializers load them on resource reload. Shapeless
recipes have one to nine item or tag ingredients. Shaped recipes use one to
three rows of equal width with one to three cells; uppercase letters select
ingredients and spaces mark empty cells. Every used letter must have exactly
one item or tag binding. Both families yield one to 64 items. Use
`RecipeIngredient::item("minecraft:stick")` or
`RecipeIngredient::tag("c:ingots/ruby")` for an explicit ingredient kind; a
string converted with `.into()` is an item ID. Cooking recipes are staged
separately with a single item or tag input:

```rust
registry::register_cooking_recipe(
    "ruby_ingot",
    CookingRecipeDefinition {
        kind: CookingRecipeKind::Smelting,
        ingredient: RecipeIngredient::tag("c:ores/ruby"),
        output_item: "ruby_ingot".into(),
        output_count: 1,
        experience: 0.7,
        cooking_time: 200,
    },
)?;
```

The kind selects vanilla smelting, blasting, smoking, or campfire cooking.
Experience must be finite and between 0 and 1,000; cooking time is 1 to
36,000 ticks. Cooking recipes use item or tag IDs and share the output count
bound of 1 to 64. Brewing and arbitrary Java recipe serializers are not
provided. Guest-defined matching logic can use a Rust callback serializer:

```rust
#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_recipe_serializer("ruby_threshold")
}

// The callback name must match the serializer path above.
#[recipe(name = "ruby_threshold")]
fn ruby_threshold(input: RecipeMatchInput) -> SjmlResult<RecipeEvaluation> {
    if input.payload == "minimum_ruby=2"
        && input.slots.iter().flatten().any(|slot| {
            slot.item_id == "minecraft:emerald" && slot.count >= 2
        })
    {
        RecipeEvaluation::matched(2)
    } else {
        Ok(RecipeEvaluation::no_match())
    }
}
```

The module must request and receive `content.register` and declare `side =
"both"`, since crafting recipes are matched separately on client and server
threads. Put recipe JSON under `data/<mod.id>/recipe/` in the `.sjml` archive:

```json
{
  "type": "example:content/ruby_threshold",
  "result": {"id": "minecraft:emerald", "count": 1},
  "payload": "minimum_ruby=2"
}
```

The callback receives a versioned snapshot of grid width, height, row-major
slots (item ID and count or empty), and the optional payload. It returns a
match flag and output count; non-matches must return zero. The result item in
JSON is used for recipe-book display, and callback output is bounded by its
stack limit and 64 items. The host calls the callback synchronously on the
owning game thread. A callback error closes its module and makes the recipe
fail closed.

Simple particles can be registered from the same callback and emitted into a
loaded server world:

```rust
#[mod_register]
fn register_content() -> SjmlResult<()> {
    let particle = registry::ParticleDefinition::new("ruby_pack:particle/ruby_spark")?
        .with_scale(0.75)?;
    registry::register_particle_with_definition("ruby_spark", particle)
}

fn emit_spark(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    world::spawn_particle(world, position, "ruby_pack:content/ruby_spark")
}
```

The host attaches a shared translucent sprite renderer. `register_particle`
uses vanilla's `minecraft:generic_0` fallback; `ParticleDefinition` selects a
package-owned `particle/` atlas texture and a scale from `0.05` through `4.0`.
The texture is loaded from the mounted package assets; for the example ID,
include `assets/ruby_pack/textures/particle/ruby_spark.png` in the package.
Emission requires both `world.read` and `world.write`, a live world handle, an in-height
position in an already loaded chunk, and the owning server thread. It sends one
particle at the block center and never loads a chunk. Motion, lifetime, and
custom render behavior remain on safe vanilla defaults. See
[Guest particle visuals](PARTICLES.md) for texture path and bounds.

The same callback can register a vanilla status effect with one bounded
attribute modifier:

```rust
registry::register_attribute_effect(
    "ruby_mite_power",
    EffectCategory::Beneficial,
    0xD02090,
    EntityAttribute::AttackDamage,
    2.0,
    AttributeModifierOperation::AddValue,
)?;
```

Categories are beneficial, harmful, or neutral; the color is 24-bit RGB. The
modifier may use any supported vanilla attribute and one of the three vanilla
operations, with a finite amount from `-16` to `16`. Vanilla `/effect` and
datapack commands can apply the registered effect. Guest modules can also apply
registered effects to loaded living entities with `entity::apply_effect` when
granted `entity.effect.apply`. To execute bounded guest-defined behavior at an
effect interval, call `registry::register_effect_tick("ruby_mite_power", 20)`
during `mod_register`, subscribe to `entity.effect.tick` during `mod_init`, and
handle `EffectTickEvent` with `#[event(name = "entity.effect.tick")]`.

The same registration callback can define a module-owned creative tab after
staging its icon item, then add item or block-item paths:

```rust
registry::register_item("ruby", ItemDefinition::default())?;
registry::register_block("ruby_block", BlockDefinition::default())?;
registry::register_creative_tab(
    "ruby",
    CreativeTabDefinition::new("Ruby", "ruby")?,
)?;
registry::add_to_creative_tab("ruby", "ruby")?;
registry::add_to_creative_tab("ruby", "ruby_block")?;
```

Tabs and entries must be declared by the same module. The host materializes
tabs after item registration; absent or colliding entries are skipped, and a
tab is skipped if its icon did not register. Titles are plain UTF-8 text up to
256 bytes with no control characters, and each tab is limited to 128 entries.
These declarations share the content registration quotas and require the
`content.register` capability.

Modules can also declare a real NeoForge registry before registry freeze and
store bounded UTF-8 string values in it:

```rust
registry::register_custom_registry("ore_types", true)?;
registry::register_custom_value(
    "ore_types",
    "ruby",
    r#"{"hardness":3,"variant":"ruby"}"#,
)?;
```

The resulting registry ID is `ruby_pack:custom/content/ore_types` for package
`ruby_pack` and module `content`; values use the module's ordinary IDs. A
module may access only its own declarations. Registries and values must be
declared from `mod_register`; the host creates them during NeoForge's
`NewRegistryEvent` and registers entries through `RegisterEvent`. After
registry setup, `registry::custom_registry_value("ore_types", "ruby")?`
returns `Option<String>`. Each value is limited to 65,536 UTF-8 bytes, and a
registry may hold at most 256 values. `sync_to_clients` controls NeoForge's
numeric registry-ID synchronization. The server and client still need the
same SJML package declarations; custom value payloads are not sent by that
flag. Custom registry imports use the existing `content.register` grant.

`registry::register_sound(path)` stages a variable-range sound event named
`<package>/<module>/<path>`. Sound events are real NeoForge registry objects;
all regular client resources in the package's own `assets/<package-id>/`
namespace are mounted through an always-enabled generated resource pack. Put a
matching `sounds.json` and OGG file there, for example:

```text
assets/ruby_pack/sounds.json
assets/ruby_pack/sounds/content/engine.ogg
```

The `sounds.json` key for `ruby_pack:content/engine` is `content/engine`.
Registered sound IDs also work with vanilla `/playsound`.

Modules can also register a persistent, network-synchronized string data
component and set its default value on an item or block item:

```rust
registry::register_string_component(
    "ruby_origin",
    StringComponentDefinition { max_characters: 128 },
)?;
registry::set_item_string_component("ruby", "ruby_origin", "found in caves")?;
```

Declare the component before assigning it, and assign defaults during
`#[mod_register]`. Values are limited to 1,024 Unicode scalar values and 4 KiB
of UTF-8 per component; the declaration may lower the character limit. The
component type and item must belong to the same package module. Modules can
read their own registered string components from loaded inventory slots and
update them with `inventory.write` on container-backed blocks.

## Server menus with bounded action slots

`MenuDefinition` opens a bounded vanilla chest screen for an online player.
The menu uses Minecraft's built-in generic chest menu types, so dedicated
servers do not load any client screen classes. A menu contains one to six rows
and up to 54 registry items. Titles and labels are plain UTF-8 text; item IDs
must use canonical `namespace:path` syntax. The host rejects unknown items,
invalid counts, duplicate/out-of-range slots, oversized input, and all item
movement. A slot can use `.action("claim")`; a left-click on that slot calls
the registered `menu.action` handler with the player UUID, slot, and action ID.
Other click types are ignored, and each module dispatches at most eight menu
actions per server tick.

Opening a menu requires both `entity.read` and `menu.open`; the first capability
is required because the API accepts a module-scoped, login-scoped `PlayerHandle`.
It runs only on the owning logical-server thread and cannot be called from
`mod_register` or a client-only module. The user example registers an item
before NeoForge registry events and displays it after `player.joined`:

```rust
#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("menu_token", ItemDefinition::default())?;
    Ok(())
}

#[event(name = "player.joined")]
fn show_welcome_menu(event: PlayerJoined) -> SjmlResult<()> {
    let mut menu = MenuDefinition::new("Welcome to SJML", 1)?;
    menu.slot(4, MenuItem::new("menu_demo:content/menu_token")?
        .label("Registered content")?.action("open_rewards")?)?;
    for handle in player::online()? {
        if player::snapshot(handle)?.uuid == event.player_id {
            menu.open(handle)?;
            break;
        }
    }
    Ok(())
}

#[event(name = "menu.action")]
fn handle_menu_action(action: MenuAction) -> SjmlResult<()> {
    action.validate()?;
    if action.action_id == "open_rewards" {
        // Apply a bounded server-side response here.
    }
    Ok(())
}
```

The module must request `events.subscribe`, `content.register`, `entity.read`,
and `menu.open`; register `menu.action` before opening action slots. An
administrator must grant them. The example above shows the complete menu
callback shape.

## Async guest tasks and services

On wasm32, `task::spawn` accepts a local future with `Output = ()` and a
`'static` lifetime. Futures need not be `Send`: each future belongs to one
WASM instance, and ABI v1 does not enable WebAssembly threads. The SDK polls a
future only when the host dispatches its task id on the selected lane. Calling
the future's waker asks the host to enqueue a later poll; it never runs guest
code inline. `task::spawn` returns a `GuestTaskId` or a stable ABI error. On
native targets it returns `TaskRejected` because host imports are unavailable.
Call `task::cancel(task_id)` to request cancellation of a task owned by the same
module. It returns `Ok(true)` when the host accepted the request and `Ok(false)`
when the task id was already complete, cancelled, or unknown. Dropping the
future happens later on the host's configured callback lane; it is not run
inside the cancellation import. Native targets return `TaskRejected`.

Use `task::sleep` for a cancellable nonblocking delay:

```rust
use std::time::Duration;

fn start_timer() -> SjmlResult<GuestTaskId> {
    task::spawn(async {
        if let Err(error) = task::sleep(Duration::from_secs(2)).await {
            let _ = host::log(LogLevel::Error, &error.to_string());
        }
    })
}
```

The timer is rounded up to milliseconds and must fit within the module's
configured task timeout. A zero duration completes immediately. Nonzero guest
timers require `wasm32-unknown-unknown`; native polling returns `TaskRejected`.

Bound CPU work into short chunks and yield between them with
`task::yield_now().await`. The future is polled again by the host scheduler;
this does not move it to a worker thread. Reacquire world or entity handles at
chunk boundaries when the operation must stop after those resources unload.
Split long CPU operations into bounded chunks, and exit once the owning world
is no longer loaded.

For CPU-only work that can be expressed as stable values, define a typed worker
callback with `#[compute(name = "...")]` and await its generated helper:

```rust
#[compute(name = "example.fold")]
fn fold(input: FoldInput) -> SjmlResult<u64> {
    Ok(input.values.iter().fold(0, |sum, value| sum.wrapping_add(*value)))
}

async fn caller(input: FoldInput) -> SjmlResult<u64> {
    fold_compute(input)?.await
}
```

The module must request `task.compute`, and an administrator must grant it.
The host copies stable-encoded input and output between separate WASM instances;
the worker has no host imports, world handles, or shared guest memory. Requests
and responses are limited to 256 KiB, with at most eight operations per module.
Use this only for computations that do not need Minecraft or other
thread-affine APIs. Dropping the future or cancelling its task requests
best-effort interruption of the worker callback. See [WORKER_COMPUTE.md](WORKER_COMPUTE.md)
for the full contract and [ASYNC_MODEL.md](ASYNC_MODEL.md) for scheduling details.

Use `task::service_call` to await a versioned service response:

```rust
fn start_lookup() -> SjmlResult<GuestTaskId> {
    let request = vec![1, 2, 3];
    task::spawn(async move {
        let call = match task::service_call(
            "provider",
            "inventory.query",
            "^1.0",
            &request,
            4096,
        ) {
            Ok(call) => call,
            Err(error) => {
                let _ = host::log(LogLevel::Error, &error.to_string());
                return;
            }
        };
        if let Err(error) = call.await {
            let _ = host::log(LogLevel::Error, &error.to_string());
        }
    })
}
```

The request bytes are copied when the future is created. `maximum_response_bytes`
sets the guest output-buffer size and may not exceed 1 MiB. The call checks the
provider package, exported service, version constraint, visibility, and the
caller's `service.call` capability on the host. The service future's `Drop`
implementation cancels its host operation handle. Provider calls use a bounded
queue and are dispatched on the registered host lane; the caller must handle
`ServiceBusy`, timeout, cancellation, and provider failures.

The task imports (`wake`, `cancel`, and the timer operations) are bound for
logical-server and physical-client module instances. After `mod_init` succeeds,
the runtime admits guest tasks and polls them on the owning server or client
main thread through a bounded tick queue. Client `service.call`, client worker
computations, and `RENDER`-lane tasks remain fail-closed. Logical-server
`service.call` requests are routed to active providers and their responses
resume the caller's task. Static registration stays fail-closed, and a wake
attempted during `mod_init` is rejected because task admission starts only
after initialization. See `ABI.md` and `ASYNC_MODEL.md` for the exact
signatures, bounds, and lifecycle.

## Errors and current limits

`SjmlResult<T>` uses `SjmlError`, which carries one stable ABI error code and
a diagnostic string. The Rust and Java code tables match ABI v1. Unknown host
codes map to `UnknownErrorCode` (`255`). Serialization rejects invalid UTF-8,
noncanonical boolean values, out-of-bounds reads, trailing bytes, schema
mismatches, and payloads beyond configured SDK limits.

Non-cancellable event callbacks can use `#[event_async]` to queue a local guest
task; `block.break` remains synchronous because it returns a cancellation
decision. The network SDK is integrated with the NeoForge payload and
activation paths; an end-to-end live client/server session remains unverified. Typed storage is available to
logical-server instances, with per-save mod isolation and explicit
`storage.read` / `storage.write` capabilities. Physical-client modules can use
the separate `client_profile` scope backed by per-mod local configuration
storage. Server chest menus remain snapshots that reject item movement, but
declared action slots can dispatch subscribed `menu.action` callbacks. Rendering
supports the client HUD, active-screen GUI, and bounded camera-relative world
lines/outlined boxes, alongside the typed OpenGL bridge. Guest entities and
block entities can attach colored or textured cuboid models with a host-side
sine rotation. Live client rendering remains unverified. See
[Persistent storage](STORAGE.md), [Named networking](NETWORKING.md), and
[Client rendering](RENDERING.md) for their API contracts and limits.

## Server world handles

Request `world.read` in the module manifest and grant it to the same package/module
in `config/sjml-permissions.toml`. Block updates additionally require a request
and grant for `world.write`. The `world` module is available at the crate root
and through the prelude:

```rust
use sjml::prelude::*;

fn read_overworld() -> SjmlResult<Option<i64>> {
    let Some(handle) = world::get("minecraft:overworld")? else {
        return Ok(None);
    };
    let id: String = world::dimension_id(handle)?;
    let time: i64 = world::game_time(handle)?;
    let loaded: Vec<WorldHandle> = world::loaded()?;
    assert!(loaded.contains(&handle));
    assert_eq!(id, "minecraft:overworld");
    Ok(Some(time))
}
```

`get` accepts canonical `namespace:path` IDs and never loads worlds; an invalid
ID returns `InvalidArgument`, while an unknown/unloaded world returns `None`.
`loaded` returns all loaded server dimensions, including modded dimensions,
sorted by canonical ID, and shares its handles with `get`. There is no implicit
current world. `world.read` and `world.write` are module-wide grants without
per-dimension ACLs.

Calls are synchronous on the owning server thread. Before dedicated server
binding, lookup/enumeration return `None`/empty, including during early
`mod_init`. Integrated-server initialization occurs after binding. Client
instances expose no server worlds. Calls during static `mod_register` are denied;
native targets report unsupported imports. Each module is limited to 1,024 live
world handles. Enumeration is complete or returns `HandleTableFull` without a
partial allocation. Handles expire on world unload and module/runtime close;
a copied old handle then fails validation. The world metadata API reads
dimension ID and game time; block entities and inventories use the separate
APIs below.

For several coordinates, use one bounded host call instead of reading each
position separately:

```rust
use sjml::prelude::*;

fn read_blocks(world: WorldHandle) -> SjmlResult<Vec<Option<BlockStateSnapshot>>> {
    world::block_states(
        world,
        &[
            BlockPos { x: 0, y: 64, z: 0 },
            BlockPos { x: 1, y: 64, z: 0 },
        ],
    )
}
```

The result has one entry per requested coordinate in the same order. An
unloaded chunk or a y coordinate outside build height produces `None`; no
chunk is loaded. One call accepts at most 1,024 positions and returns detached
`BlockStateSnapshot` values containing the canonical block ID and state text.
The ABI bounds each encoded response to 1 MiB. Calls remain synchronous and
require `world.read` on the owning server thread.

For a line-of-sight query, choose the block collision shape and fluid policy
explicitly:

```rust
fn trace(world: WorldHandle) -> SjmlResult<Option<RaycastHit>> {
    world::raycast(
        world,
        RaycastPoint::new(0.5, 65.0, 0.5),
        RaycastPoint::new(0.5, 65.0, 8.5),
        RaycastBlockMode::Collider,
        RaycastFluidMode::None,
    )
}
```

Endpoints must be finite, lie within ±30,000,000 on each axis, and be no more
than 128 blocks apart. A hit includes its block position, face, exact location,
and detached `BlockStateSnapshot`. The call checks all chunks along the segment
without loading them; an unloaded chunk anywhere along the path returns
`Ok(None)`. Raycasts run synchronously on the owning logical-server thread and
require `world.read`.

To check whether a small box intersects a block shape, use `Aabb`. The query
returns `None` if any covered chunk is unloaded, the box leaves build height,
or a shape needs unloaded neighbor data. It checks at most 4,096 block
positions and never loads chunks:

```rust
fn blocked(world: WorldHandle, x: f64, y: f64, z: f64) -> SjmlResult<Option<bool>> {
    world::collides(
        world,
        world::Aabb::new(
            RaycastPoint::new(x, y, z),
            RaycastPoint::new(x + 0.8, y + 1.8, z + 0.8),
        ),
    )
}
```

Bounds must be finite and ordered, stay within ±30,000,000 on each axis, and
span no more than 16 blocks per axis. The result uses empty-entity block
collision shapes; fluids and entity collisions are separate concerns.

To replace one block, request `world.write` as well. The mutation uses a
registered block's default state and accepts only a position inside build
height whose chunk is already loaded:

```rust
fn place_marker(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    world::set_block(world, position, "ruby_pack:content/ruby_block")
}
```

Every call revalidates the world handle on the owning server thread. Unknown
block IDs, out-of-height positions, and unloaded chunks return an error; this
operation never loads chunks.

To emit a registered simple particle, use the same world handle and permissions:

```rust
fn spark(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    world::spawn_particle(world, position, "ruby_pack:content/ruby_spark")
}
```

The ID must name a registered simple particle type. The host sends one
particle at the block center after checking build height and loaded-chunk
status; the call never loads a chunk.

To spawn a registered non-player entity, request `entity.spawn` alongside
`world.read` and call the bounded server API:

```rust
fn summon(world: WorldHandle, position: world::RaycastPoint) -> SjmlResult<()> {
    world::spawn_entity(world, "ruby_pack:machines/worker", position)
}
```

The entity ID must already be registered. Spawning checks the server thread,
world border, build height, loaded chunks, and entity bounds; it does not load
chunks and is limited to eight entities per module per server tick. Player
entities are rejected, and administrators can grant spawn permission
independently of the broader `world.write` capability.

Play a registered sound event at a loaded position with the same world handle
and grants:

```rust
fn engine_sound(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    world::play_sound(world, position, "ruby_pack:content/engine", 1.0, 1.0)
}
```

Volume is limited to 0–16 and pitch to 0.5–2.0. The host uses Minecraft's
`MASTER` sound source and sends the sound from the block center to nearby
clients. Both `world.read` and `world.write` are required; the position must be
inside build height and in an already loaded chunk.

## Logical-server player handles

Request `entity.read` in the module manifest and grant it to the same
package/module in `config/sjml-permissions.toml`. Player snapshots do not
require `world.read`; applying a status effect additionally requires the
separate `entity.effect.apply` grant:

```rust
use sjml::prelude::*;

fn read_online_players() -> SjmlResult<Vec<PlayerSnapshot>> {
    player::online()?
        .iter()
        .copied()
        .map(player::snapshot)
        .collect()
}

fn grant_player_effect(handle: PlayerHandle) -> SjmlResult<()> {
    player::apply_effect(handle, "ruby_pack:content/power", 200, 0)
}
```

`player::online` returns all online players sorted by canonical lowercase UUID
and reuses handles across repeated enumeration. `PlayerSnapshot` is a
schema-v1 stable record containing `uuid`, `profile_name`, `dimension_id`, and
current `x`, `y`, and `z` coordinates as `f64`. Handles are scoped to one
active module and login session. They survive respawn and dimension changes,
then become stale as logout begins. Module/runtime close invalidates all
remaining handles.

Calls run synchronously on the owning logical-server thread. Before server
binding, including early `mod_init`, and in client instances, enumeration is
empty. Static `mod_register` calls are denied; native targets report unsupported
imports. Each module has a 1,024 live-player handle quota. Enumeration is
complete or returns `HandleTableFull`; no partial vector is returned.
`player::apply_effect` resolves the current server player for the login session,
so the handle remains valid through respawn. Duration is limited to 1–72,000
ticks and amplifier to 0–255; the API does not expose player inventory.

## Loaded entity snapshots and effect application

Request `entity.read` in the module manifest and grant it to the same
package/module in `config/sjml-permissions.toml`. This is a global read-only
API over loaded non-player entities in every loaded server dimension; it does
not take a `WorldHandle` and does not require `world.read`:

```rust
use sjml::prelude::*;

fn read_loaded_entities() -> SjmlResult<Vec<EntitySnapshot>> {
    entity::loaded()?
        .iter()
        .copied()
        .map(entity::snapshot)
        .collect()
}
```

`entity::loaded` returns handles sorted by canonical lowercase UUID and reuses
handles for the same loaded entity instance. `EntitySnapshot` is a schema-v1
stable record containing `uuid`, canonical registry `entity_type_id`, current
`dimension_id`, and `x`, `y`, and `z` coordinates as `f64`. Players remain
available through the separate player API. Entity handles are module/context
scoped and expire when NeoForge removes that entity from its level (including
chunk unload or dimension transfer), when the level unloads, or when the module
closes. Reloaded entities receive fresh handles.

Calls run synchronously on the owning logical-server thread and never load
chunks. Before server binding, including early dedicated `mod_init`, and in
client instances, enumeration is empty. Static `mod_register` calls are denied;
native targets report unsupported imports. Each module has a 1,024 live-entity
handle quota. Enumeration is complete or returns `HandleTableFull`, without a
partial vector.

The same module can apply a registered status effect to a loaded living
non-player entity when it also requests and receives `entity.effect.apply`.
Player targets use `player::apply_effect` with a login-session handle:

```rust
let target = entity::loaded()?.into_iter().next().ok_or_else(|| {
    SjmlError::new(ErrorCode::InvalidArgument, "no loaded entity")
})?;
entity::apply_effect(target, "ruby_pack:content/power", 200, 0)?;
```

Every call revalidates the entity handle on the server thread. Duration is
1–72,000 ticks and amplifier is 0–255. Players and non-living entities are
rejected; the API does not spawn entities or schedule delayed effects.

## Entity-attached opaque data

Modules can keep small opaque byte values attached to the entity itself. Request
`entity.data.read` to read values and `entity.data.write` to write or remove
them. These grants are separate from `entity.read`: a module with only an
`entity.data.*` grant can call `entity::loaded` to obtain handles, but it cannot
read snapshots unless it also has `entity.read`.

```rust
use sjml::prelude::*;

fn record_progress(handle: EntityHandle) -> SjmlResult<()> {
    entity_data::set(handle, "quest/progress", &[1, 3, 5])?;
    let mut saved = [0; 3];
    if entity_data::read(handle, "quest/progress", &mut saved)? != saved.len()
        || saved != [1, 3, 5]
    {
        return Err(SjmlError::new(ErrorCode::InvalidArgument, "entity data did not round-trip"));
    }
    entity_data::remove(handle, "quest/progress")?;
    Ok(())
}
```

`read` copies into the caller's bounded slice and returns the number of bytes. It
returns `entity_data::ENTITY_DATA_MISSING` for a missing key and `0` for a
present empty value; if the value is larger than the supplied slice, it returns
`PayloadTooLarge` without copying a partial value. The host validates exact wire
frames and the SDK checks the schema, value length, and caller capacity; the SDK
does not rescan zero padding in its pre-zeroed response buffer.

Each module owns its package/module/key bucket below NeoForge's persistent
entity NBT key `sjml:entity_attachments`. The SDK exposes only copied bytes; it
never passes Java objects or another mod's namespace. Keys are module-local,
lowercase paths of at most 128 UTF-8 bytes. Values are limited to 16 KiB, each
module to 128 keys and 64 KiB of accounted data, and all SJML attachments on
one entity to 1,024 keys and 256 KiB. Empty byte values differ from missing
keys. `read` returns `entity_data::ENTITY_DATA_MISSING` for a missing key, `remove` returns `false`
when the key was absent, and removing a module's last key prunes its NBT buckets.

Calls are synchronous on the logical server thread and require a current
non-player `EntityHandle`; they reject stale handles and never load chunks.
These values are serialized with the entity and follow it across saves,
unloads, and dimension transfers. They are distinct from
`storage::StorageScope::entity`: save storage is a per-mod database keyed by
entity UUID, while an attachment is a module-owned value physically stored on
that entity and disappears when the entity is permanently removed.

## Read-only block inventory snapshots

Request both `world.read` and `inventory.read` in the module manifest and grant
both to the same package/module in `config/sjml-permissions.toml`. `inventory`
is available at the crate root and through the prelude:

```rust
use sjml::prelude::*;

fn read_chest(world_handle: WorldHandle, position: BlockPos)
    -> SjmlResult<Option<InventorySnapshot>>
{
    inventory::read(world_handle, position, None)
}
```

`BlockPos` carries integer x/y/z coordinates. Pass `None` for an unsided
capability query or `Some(Direction::North)` (or another of the six directions)
for a sided query. The import queries NeoForge's block item-handler capability,
including providers that are not block entities, and returns detached values:
each slot is `None` when empty or an `ItemStackDescriptor` containing the
canonical item ID, current count, item maximum stack size, and that slot's
handler limit. No `ItemStack`, NBT, arbitrary components, or live handler is
returned.

An unloaded position or a block without the capability returns `Ok(None)`; a
present handler with zero slots returns `Some` with an empty slot vector. The
query never loads chunks, runs synchronously on the owning server thread, and
validates the supplied module-scoped world handle. The API is limited to 4,096
slots and 1 MiB of encoded output. Static `mod_register` calls are denied, and
native targets report unsupported imports. The two permissions are checked as
an intersection: neither alone authorizes an inventory read.

String data components registered by a module can also be read or changed on a
single inventory slot. Reading requires `world.read` and `inventory.read`;
writing requires `world.read` and the separate `inventory.write` grant. Paths
are module-local paths used during registration, and values stay within the
component's declared character bound and 4 KiB UTF-8 limit:

```rust
let current = inventory::read_string_component(
    world_handle, position, None, 0, "origin",
)?;
inventory::write_string_component(
    world_handle, position, 0, "origin", "found in a cave",
)?;
```

Reads return `None` for an unloaded position, absent handler, empty slot, or
unset component. Writes require a non-empty slot in a block entity implementing
Minecraft's `Container` API. The host copies the stack before replacing the
component through `Container.setItem`, so the item, count, and other data
components remain intact. Writes are unsided and address the whole container;
they do not use sided item-handler capabilities. Neither operation loads
chunks; calls run on the owning server thread. Static registration and native
targets cannot access the live inventory imports. Because `inventory.write`
has no per-block ACL, request it only for modules trusted to change slots in
containers reachable through their world handle.

## Read-only block entities

Request and receive `world.read` to inspect block entities. A `WorldHandle`
provides the convenience method `block_entity(position)`, which returns a
module-scoped `BlockEntity` value when the position contains a loaded block
entity:

```rust
use sjml::prelude::*;

fn inspect(world_handle: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    if let Some(block) = world_handle.block_entity(position)? {
        let type_id = block.type_id()?;
        let block_state = block.block_state()?;
        let inventory = block.inventory(None)?;
        let nbt = block.nbt()?;
        let _ = (type_id, block_state, inventory, nbt);
    }
    Ok(())
}
```

`block.snapshot()` returns the schema-v1 type ID, dimension ID, position, and
block-state string; `type_id()` and `block_state()` are shortcuts to those
fields. `inventory(side)` uses the same detached `InventorySnapshot` values as
`inventory::read` and requires both `world.read` and `inventory.read`. A missing
item-handler capability returns `None`.

`nbt()` returns an owned `NbtValue` tree and additionally requires the module to
request and receive `block_entity.nbt.read`. The tree supports all standard NBT
numeric, string, list, compound, and array tag types. The host bounds encoded
output to 1 MiB, nesting depth to 64, node count and per-container item count
to 65,536, and each UTF-8 string to 65,536 bytes. Compounds are encoded in key
order. Invalid tags, duplicate keys, mixed-type lists, serialization failures,
or values exceeding a limit return a typed error.

NBT can contain mod-specific persistent data. The explicit administrator grant
is the access policy: grant it only to modules trusted to read that block
entity's serialized data. NBT schemas are not stable APIs between mods; use the
typed inventory and capability APIs for their respective data. Handles become
stale when the block entity is replaced or removed, its chunk or world unloads,
or the module runtime closes. Lookups never load chunks. Static registration is
denied, and native builds report unsupported host imports. See
[Block entity reads](BLOCK_ENTITIES.md) for the wire format.

## Named networking

The `network` module exposes validated channel references, typed closure
subscriptions, an immutable callback context, and explicit send operations.
Channels must first be declared in the module's `sjml.toml`:

```toml
[[modules.channels]]
name = "chat"
schema_version = 1
directions = ["client_to_server", "server_to_client"]
```

Use `ChannelRef::local("chat")` for this module's channel or
`ChannelRef::new(package_id, module_id, "chat")` for a channel owned by another
module. All operations require `network.mod`; accessing another module's
channel also requires `network.external`. The channel must be declared and its
direction must allow the requested operation.

```rust,ignore
use sjml::network::{self, ChannelRef, NetworkContext, NetworkSubscriptionHandle};
use sjml::{SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ChatMessage {
    text: String,
}

fn subscribe_chat() -> SjmlResult<NetworkSubscriptionHandle> {
    network::subscribe(
        ChannelRef::local("chat")?,
        |context: NetworkContext, message: ChatMessage| {
            let _ = (context, message);
            Ok(())
        },
    )
}
```

Keep the returned handle to remove that closure with `network::unsubscribe`;
runtime close removes all subscriptions. Callbacks receive decoded
`StableDecode` values. `NetworkContext` exposes the channel, message direction,
receiving logical side, and an optional UUID from authenticated connection
metadata. On server-side client-to-server delivery, that UUID is the
authenticated sender and never comes from the payload. `context.player()`
resolves a `Player` through the permission-checked player API and requires
`entity.read`. `context.reply(&message)` sends only to the active callback's
peer and channel in the reverse direction; reply authority expires as soon as
the callback returns.

Use `network::send_to_server` for client-to-server delivery,
`network::send_to` for a server-to-client message to one `Player`, and
`network::broadcast` for server-to-client delivery to compatible connected
clients. `send_to` also requires `entity.read` to resolve and validate the
player session. Sends require a declared channel direction and live compatible
connection; a successful call means the host accepted the message for its
bounded queue, not that a peer received it. The SDK bounds each encoded message
to fit the 1 MiB delivery ABI.

The activation path composes the network imports with each module's runtime,
and NeoForge registers the payload adapter and negotiates handshakes for exact
connection sessions. Shared-channel schema or direction-set mismatches reject
the connection; a missing optional channel alone remains compatible. Code-level
adapter tests exist, but no live multiplayer session has been verified end to
end. See [Networking](NETWORKING.md) for the full contract and import encoding.

## Persistent storage

`sjml::storage` provides typed reads, writes, removals, automatic migrations,
and explicitly requested record or scope migrations. The host stores records
in `SavedData` under the current Minecraft save, namespaced by mod ID; modules
of one mod share that namespace. Every operation checks the declaring module's
`storage.read` or `storage.write` permission. Logical scopes cover save,
dimension, player, entity, and block entity data. Client instances have no
active server save, so those server scopes are denied. Physical-client modules
can use the separate `client_profile` scope backed by local per-mod storage.
See [Persistent storage](STORAGE.md) for bounds, lifecycle, and binary encoding.
