# SJML Binary ABI v1

This document defines the core wire values implemented by the sjml-core ABI v1.
The scalar and handle layouts are independent of Rust and NeoForge; later
sections describe the current Rust SDK and NeoForge adapter bindings.

## Versioning

The current core ABI version is **1.33.0**. Versions use SemVer. A host supports
a requested ABI when the major versions match and the host version is greater
than or equal to the requested version. A different major version is
incompatible. Build metadata does not affect version ordering.

## Guest instruction budget

The core instruments guest WASM functions with a basic-block instruction
counter. Each exported callback receives a fresh budget; the runtime default is
1,000,000 guest instructions per export and trusted host code may configure a
limit up to 100,000,000. An exhausted budget traps that callback and follows
the normal module callback failure path. Modules exceeding the instrumentation
limits (2,000,000 decoded instructions or 500,000 basic blocks) are rejected.
This limits guest instruction work, not elapsed time: execution inside a host
import and time spent in Minecraft or the JVM are outside the guest budget.

## Scalar and span layouts

All multibyte values are little-endian. Guest pointers and lengths are wasm32
unsigned values even though a WASM function passes them as i32.

| Value | Wire layout |
| --- | --- |
| i32 | 4-byte two's-complement integer |
| i64 | 8-byte two's-complement integer |
| bool | 1 byte; only 0 and 1 are valid |
| ByteSpan | 8 bytes: pointer: u32 at offset 0, length: u32 at offset 4 |
| UTF-8 string | A ByteSpan to strictly valid UTF-8 bytes; no NUL terminator |

Zero-length spans are valid. Every span is checked against the current guest
memory capacity before access. Range calculations reject negative host values,
out-of-bounds ranges, and arithmetic overflow. A span can encode any u32
address, but this Java core can only access addresses backed by its
ByteBuffer (at most Integer.MAX_VALUE bytes).

The default maximum byte payload is 1 MiB; the default maximum UTF-8 string is
64 KiB. AbiMemory can be configured with lower or higher limits by a trusted
host. UTF-8 decoding and encoding report malformed input rather than replacing
it.

Arrays and records are carried as bounded byte payloads. The Rust SDK adds an
opt-in, version-prefixed little-endian serializer for named structs; its exact
field order and limits are documented in `SDK_GUIDE.md`. Host adapters must
only decode types for which that schema is known.

## Batched host imports

An adapter may mark selected read-only host functions as batchable. The runtime
then exposes `sjml_core_v1.batch` to that module:

```text
batch(request_ptr: i32, request_len: i32,
      response_ptr: i32, response_capacity: i32) -> i64
```

The request and response spans are unsigned wasm32 pointers, each bounded by
the ABI's 1 MiB payload limit. The return value packs the signed status in its
low 32 bits and the initialized response length in its high 32 bits. Status
zero means success; failures use the negative of the stable ABI error code and
report a zero response length.

Both messages start with little-endian `u32` schema version `1`, then a
little-endian `u32` call count. Requests contain 1–1,024 calls. Each call
contains a `u16` byte length and strict UTF-8 import-module name, a `u16` byte
length and strict UTF-8 import name, then a `u16` argument count and that many
typed numeric values. Names are 1–128 bytes and argument lists contain at most
16 values. Each value is a one-byte type tag followed by eight little-endian
bytes: `0=i32`, `1=i64`, `2=f32`, and `3=f64`. The upper four bytes of `i32`
and `f32` encodings must be zero; floating-point payloads preserve their raw
IEEE-754 bits.

Responses contain one result record for each request in the same order. A
record contains a `u16` value count, followed by values with the same tagged
numeric encoding. The host checks every import name, batchable allowlist entry,
signature, argument type, result-size requirement, and the full output memory
span before invoking any operation. Calls then execute in request order in the
same module instance and guest memory. Each host adapter keeps its ordinary
capability, owner, context, and thread checks. The dispatcher cannot call
itself, and imports not explicitly marked batchable cannot be invoked through
it. Adapters currently expose selected read-only world, player, entity,
inventory, and block-entity imports this way; mutations are deliberately
excluded from the batch dispatcher.

## Rust guest callback convention

The SDK exports `sjml_v1_mod_register`, `sjml_v1_mod_init`, and
`sjml_v1_mod_shutdown` as no-argument functions returning an `i32` status
(`0` means success). `mod_register` runs before registry events and stages
static content; `mod_init` runs after registration. Named event,
command, and service callbacks are exported as
`sjml_v1_{kind}_{identifier}`. They take `(input_ptr: i32, input_len: i32)`
and return an `i64` whose low u32 is the output pointer and high u32 is the
output length. The span is at most 1 MiB including its header. The pointed-to
bytes are a status u32 followed by the encoded success value, or by a
length-prefixed UTF-8 diagnostic on failure. The host
must copy the span and call `sjml_v1_dealloc(pointer, length)` once with that
exact pair.

For non-empty host-to-guest input, the SDK exports
`sjml_v1_alloc(length: i32) -> pointer: i32` and
`sjml_v1_dealloc_input(pointer: i32, length: i32) -> ()`. The host requests an
allocation, validates it against current linear memory, writes the input, and
releases it with the exact length after callback completion. Empty input uses
`(pointer = 0, length = 0)` and does not allocate. The output span is copied
only after its unsigned pointer/length are checked against current memory and
the 1 MiB frame bound; a valid copied span is passed once to
`sjml_v1_dealloc`, including when it contains a guest error. Invalid spans are
rejected without passing guest-controlled bounds to a deallocator.

The guest SDK declares `sjml.abi_version: () -> i32` and
`sjml.log: (i32, i32, i32) -> i32`. In the portable core runtime, imports are
rejected unless an adapter explicitly allowlists and binds them; SDK
declarations do not enable them by themselves. The NeoForge adapter currently
binds both imports unconditionally, without an administrator capability grant.
The core `CompiledModule.invokeByteCallback` implements bounded
input/output transport, frame validation, error decoding, and paired memory
release. An `#[event(name = "...")]` attribute exports a callback but does not
subscribe it. NeoForge invokes only callbacks with an active runtime
subscription. It dispatches the non-cancellable unit-valued
`sjml_v1_event_server.started` and `sjml_v1_event_server.stopping` exports,
plus the non-cancellable `sjml_v1_event_player.joined`,
`sjml_v1_event_player.left`, `sjml_v1_event_entity.death`, and
`sjml_v1_event_entity.spawned`, `sjml_v1_event_entity.effect.tick`,
`sjml_v1_event_block.place`, `sjml_v1_event_player.interacted`,
`sjml_v1_event_entity.damaged`, `sjml_v1_event_world.loaded`,
`sjml_v1_event_world.unloaded`, `sjml_v1_event_server.tick`,
`sjml_v1_event_inventory.changed`, and `sjml_v1_event_resources.reloaded`
exports. The
player events receive their SDK schema-v1 identity record (`PlayerJoined` or
`PlayerLeft`): little-endian
version `1`, then a stable UTF-8 string containing the canonical UUID, then a
stable UTF-8 string containing the profile name. All fifteen notifications run
only after explicit subscription, synchronously on the owning logical-server
thread. The module must request `events.subscribe` in its manifest and receive
an administrator grant. Subscriptions run by priority, with equal priorities
preserving module activation order and per-module registration order. Callback
failures disable only the affected module.

NeoForge also invokes the synchronous cancellable
`sjml_v1_event_block.break` callback on that same server thread only after
explicit subscription. Its `BlockBreak`
schema-v1 input is little-endian version `1`, three stable UTF-8 strings in
order (`player_id` canonical UUID, `dimension_id`, `block_id`), then signed
little-endian `i32` coordinates `x`, `y`, and `z`. A module must request and
receive both `events.subscribe` and the separate `events.cancel` grant. The
successful callback payload after its `u32` status prefix must be exactly one
stable boolean byte (`0` or `1`); any authorized `true` vetoes the break, while
`false` does not. An invalid result or callback failure disables only that
module and does not veto the break. Existing cancellation by another handler
remains in effect. The `entity.death` callback receives a schema-v1
`EntityDeath` snapshot: little-endian version `1`, stable UTF-8 strings in
order (`entity_id` canonical UUID, `entity_type_id`, and `dimension_id`), then
little-endian IEEE-754 `f64` coordinates `x`, `y`, and `z`. Coordinates must be
finite. It notifies on NeoForge's `LivingDeathEvent` and does not expose a
cancellation result to guest code. The `entity.spawned` callback receives a
schema-v1 `EntitySpawned` snapshot: little-endian version `1`, stable UTF-8
strings in order (`entity_id` canonical UUID, `entity_type_id`, and
`dimension_id`), finite little-endian IEEE-754 `f64` coordinates `x`, `y`, and
`z`, then a stable boolean byte indicating whether the entity was restored
from disk. It observes non-cancelled entity joins on the logical server and
cannot cancel the join. The `block.place` callback receives a schema-v1
`BlockPlace` snapshot: little-endian version `1`, an optional stable UTF-8
placer UUID, stable strings for `dimension_id` and `block_id`, then signed
little-endian `i32` coordinates `x`, `y`, and `z`. It observes successful
entity placement after other handlers have had a chance to cancel it and
cannot change the placement result. The `entity.effect.tick` callback receives a
schema-v1 `EffectTickEvent`: little-endian version `1`, stable UTF-8 strings in
order (`effect_id`, canonical `entity_id`, `entity_type_id`, and `dimension_id`),
then finite IEEE-754 `f64` coordinates `x`, `y`, and `z`, signed `i32` amplifier
and remaining duration, and an unsigned `u32` configured tick interval. A
duration of `-1` means infinite. The callback is delivered after living-entity
ticks on the logical server thread only. The module must first declare the
effect tick interval during static content registration and separately
subscribe to `entity.effect.tick`; intervals are limited to 1–1,200 ticks.
The `player.interacted` callback receives schema-v1 `PlayerInteracted` data:
version `1`, stable strings for the canonical `player_id`, `dimension_id`,
`action`, and `hand`, optional stable strings for `target_id` and `face`, then an
optional nested schema-v1 `BlockPos` record. Its `action` identifies the
interaction attempt (for example `right_click_block`,
`left_click_block_start`, `entity_interact`, or `right_click_item`); `hand` is
`main_hand` or `off_hand`. The target is a registry ID when available, and the
position and face are present for block interactions. This observation does
not change or cancel the original action.
The `entity.damaged` callback receives schema-v1 `EntityDamaged` data: version
`1`, stable strings for `entity_id`, `entity_type_id`, `dimension_id`, and
`damage_type_id`, an optional stable source entity UUID, finite little-endian
`f64` coordinates, then finite non-negative little-endian `f32` values for
`original_damage` and `applied_damage`. NeoForge emits it after damage is
applied to a living entity; the callback cannot alter the damage.
The `world.loaded` and `world.unloaded` callbacks receive the same schema-v1
`WorldLifecycle` record: version `1`, stable `dimension_id`, and signed
little-endian `i64` `game_time`. Initial server levels are reported after SJML
runtime activation, later level loads are reported as they occur, and unloads
are reported before world-scoped handles are revoked. These callbacks do not
expose a world object to guest code.
The `server.tick` callback receives schema-v1 `ServerTick`: version `1`, then a
non-negative little-endian `i64` server tick count. NeoForge dispatches after
the server tick on its owning thread. Plain `events::subscribe("server.tick",
priority)` uses a 20-tick cadence; `events::subscribe_with_interval` allows a
cadence from 1 through 1,200 ticks. This callback cannot cancel or change the
server tick.
The `resources.reloaded` callback receives schema-v1 `ResourcesReloaded`:
version `1`, then the non-negative little-endian `i64` server tick when the
server observes that successfully reloaded resources have become active. The
host emits it after the resource-manager replacement commits and after server
tick processing resumes; failed reloads do not emit this notification. It is
non-cancellable and runs synchronously on the owning server thread.
The `inventory.changed` callback receives schema-v1 `InventoryChanged`: version
`1`, stable strings for the canonical player UUID, item registry ID, and change
type, an optional signed little-endian `i32` slot, then signed little-endian
`i32` count. Change types are `picked_up`, `crafted`, and `dropped`. Item IDs
are limited to 256 UTF-8 bytes, known slot indices to `0..=255`, and counts to
`1..=4096`. Pickup callbacks use NeoForge's successful post-insertion event;
craft callbacks run after the result is taken. Toss events are cancellable and
are only reported later if that same item entity was actually added to the
server level. Slots are `None` when the successful hook does not expose an
exact player inventory slot. No live inventory or item stack is exposed, and
the callback cannot alter the completed change.
Guest commands are described below.
Subscriptions are accepted only in
integrated-server and dedicated-server execution contexts. An `ActiveModule`
in a client context cannot subscribe. Non-cancellable callbacks may use
`#[event_async]`; cancellable `block.break` callbacks remain synchronous.

## Event subscription imports (NeoForge adapter)

The SDK's `events::subscribe` and `events::unsubscribe` functions use these
wasm32 imports when `events.subscribe` is both requested by the module and
granted by the administrator:

| Host import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_events_v1.subscribe` | `(event_ptr: i32, event_len: i32, priority: i32, handle_output_ptr: i32) -> i32` | Adds one callback subscription and writes its 24-byte handle to guest memory. |
| `sjml_events_v1.subscribe_interval` | `(event_ptr: i32, event_len: i32, priority: i32, interval_ticks: i32, handle_output_ptr: i32) -> i32` | Adds a bounded-cadence `server.tick` subscription and writes its handle. |
| `sjml_events_v1.unsubscribe` | `(handle_ptr: i32, handle_len: i32) -> i32` | Removes exactly one live subscription represented by a 24-byte handle. |

All three imports return `0` on success or a nonzero stable ABI error code. The
subscribe output buffer must be writable for 24 bytes; unsubscribe requires
`handle_len == 24`. The handle is the ABI's three little-endian `u64` words:
owner scope token, execution-context token, and packed slot/generation. It is
non-transferable and scoped to the module instance and its execution context.
The `priority` values are `0=Highest`, `1=High`, `2=Normal`, `3=Low`, and
`4=Lowest`; lower values run first and equal priorities retain module
activation and registration order. An active module instance may hold at most
256 live event subscriptions.
`subscribe_interval` is valid only for `server.tick` and requires an interval
from 1 through 1,200 ticks. Plain `subscribe` uses a 20-tick cadence for that
event.

The event name is strict UTF-8, 1–128 bytes, lowercase ASCII letters, digits,
`.`, `_`, or `-`, and starts with a letter or digit. The accepted event names
are `server.started`, `server.stopping`, `player.joined`, `player.left`,
`entity.death`, `entity.spawned`, `entity.effect.tick`, `block.break`,
`block.place`, `player.interacted`, `entity.damaged`, `world.loaded`,
`world.unloaded`, `server.tick`, `inventory.changed`, and `resources.reloaded`.
Subscribing to `block.break` additionally
requires the module to request and receive the separate `events.cancel` capability. Static
registration instances cannot subscribe. A subscription callback runs
synchronously on the owning logical-server thread. Non-cancellable handlers may
use the SDK's `#[event_async(name = "...")]` macro: its generated synchronous
ABI export admits a guest task and returns without waiting for the async body.
Task-admission and async handler errors are logged; they do not affect the
already-delivered notification. These tasks use the existing bounded
`sjml_async_v1.wake`/poll runtime and inherit its task quota, timeout,
cancellation, and module-shutdown behavior. The async macro rejects
`block.break`; its cancellation result remains strictly synchronous.

## Async task and service imports

The async guest runtime uses these wasm32 exports and host imports:

| Direction | Name | Signature | Meaning |
| --- | --- | --- | --- |
| Guest export | `sjml_v1_task_poll` | `(task_id: i32) -> i32` | Poll one local future; `0` means pending and `1` means ready. |
| Guest export | `sjml_v1_task_cancel` | `(task_id: i32) -> i32` | Drop the local future; `1` means it existed, `0` means it did not. |
| Host import | `sjml_async_v1.wake` | `(task_id: i32) -> i32` | Queue a later poll; `0` accepts/coalesces the wake, a negative value is an ABI error. |
| Host import | `sjml_async_v1.cancel` | `(task_id: i32) -> i32` | Request cancellation; `1` means the host accepted it, `0` means the id was already terminal or unknown, and a negative value is an ABI error. |
| Host import | `sjml_async_v1.timer_start` | `(delay_millis: i64) -> i32` | Start a bounded timer for the currently polled task; returns a positive timer id or a negative ABI error. |
| Host import | `sjml_async_v1.timer_poll` | `(timer_id: i32) -> i32` | Return `0` while pending, `1` when elapsed (and retire the timer), or a negative ABI error. |
| Host import | `sjml_async_v1.timer_cancel` | `(timer_id: i32) -> i32` | Cancel a timer; `1` means it was present and `0` means it was already gone. |
| Host import | `sjml_async_v1.service_start` | `(request_ptr: i32, request_len: i32, response_ptr: i32, response_cap: i32) -> i32` | Copy and validate a service request, then return a positive operation id or a negative ABI error. |
| Host import | `sjml_async_v1.service_poll` | `(operation_id: i32) -> i64` | Return operation status and response length. |
| Host import | `sjml_async_v1.service_cancel` | `(operation_id: i32) -> i32` | Cancel a pending operation; `1` means it was found, `0` means it was already gone. |
| Host import | `sjml_async_v1.compute_start` | `(callback_ptr: i32, callback_len: i32, input_ptr: i32, input_len: i32) -> i32` | Copy a callback name and stable-encoded request, then return a positive worker-operation id or a negative ABI error. |
| Host import | `sjml_async_v1.compute_poll` | `(operation_id: i32, output_ptr: i32, output_cap: i32) -> i64` | Return pending, ready plus output length, or a negative ABI error; successful output is copied into the supplied span. |
| Host import | `sjml_async_v1.compute_cancel` | `(operation_id: i32) -> i32` | Cancel an operation owned by the currently polled guest task; `1` means found, `0` means already gone. |

Task ids are nonzero, monotonic u32 values local to one WASM instance. The host
casts the signed wasm32 i32 argument back to u32. `service_poll` packs a signed
status in its low 32 bits and a response length in its high 32 bits: `0` means
pending, `1` means ready, and a negative status is the negated stable ABI error
code. When ready, the host has copied the response into the output span supplied
to `service_start`; the operation id is then retired. Responses larger than the
guest-provided capacity fail with `PAYLOAD_TOO_LARGE`.

Timer ids are positive monotonic i32 values local to one WASM instance and bound
to the task that started them. `timer_start` accepts a positive millisecond delay
no longer than the module's configured task timeout. The host limits each
module to 1,024 active timers, wakes the owning task through the normal queued
poll path, and removes pending timers on task completion, cancellation, timeout,
or module shutdown.

Worker computation additionally requires the requested and granted
`task.compute` capability in a logical-server context. `compute_start` accepts
only lowercase ASCII callback names of at most 128 bytes and stable-encoded
input no larger than 256 KiB. The worker uses a fresh WASM instance with
separate memory and fail-closed host imports. Requests and responses are each
limited to 256 KiB, each module may hold eight operations, and the server pool
has two worker threads with a bounded 256-entry queue. `compute_poll` packs a
signed status in the low 32 bits and the output length in the high 32 bits:
`0` means pending, `1` means ready, and a negative status is the negated ABI
error code. Dropping the Rust future or cancelling its task requests
interruption; the operation keeps its quota slot until the callback exits.

A service request is little-endian and versioned. It consists of a u32 encoding
version (`1`), then four fields, each encoded as `u32 byte_length` followed by
its bytes: provider package id (1–64 ASCII bytes using the manifest package-id
grammar), service id (1–512 ASCII bytes using the dotted service-id grammar),
version constraint (1–512 strict UTF-8 bytes), and opaque payload. The total
request and response are bounded to 1 MiB. The host parses the version
constraint and applies the manifest visibility, capability, logical-context,
lane, timeout, and queue checks before calling provider code.

All guest imports must still be allowlisted by a trusted runtime adapter. Wake
and cancellation only enqueue guest work on the configured host lane; they do
not re-enter a guest poll that is already running. NeoForge binds guest tasks
for logical-server contexts to `SERVER_MAIN` and for physical-client instances
to `CLIENT_MAIN`, each through a bounded tick queue. Client service operations
and `RENDER`-lane guest polling remain unavailable. Module shutdown drops the
instance and its pending operations.

## Static registry declarations

The Rust SDK's registration wrappers use these imports during the
`sjml_v1_mod_register` phase:

| Import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_registry_v1.register_item` | `(path_ptr: i32, path_len: i32, max_stack: i32, fire_resistant: i32) -> i32` | Stage an item under `<package>/<module>/<path>`. |
| `sjml_registry_v1.register_block` | `(path_ptr: i32, path_len: i32, strength_bits: i32, resistance_bits: i32, requires_tool: i32, has_item: i32) -> i32` | Stage a basic block and optionally its `BlockItem`. |
| `sjml_registry_v1.register_block_entity` | `(path_ptr: i32, path_len: i32, block_path_ptr: i32, block_path_len: i32) -> i32` | Stage a generic block entity type for a block declared by the same module. |
| `sjml_registry_v1.register_entity_model` | `(path_ptr: i32, path_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded colored model for an entity declared by the same module, with optional model-wide and per-cuboid sine rotations. |
| `sjml_registry_v1.register_block_entity_model` | `(path_ptr: i32, path_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded colored model for a block entity declared by the same module, with optional model-wide and per-cuboid sine rotations. |
| `sjml_registry_v1.register_entity_textured_model` | `(path_ptr: i32, path_len: i32, texture_ptr: i32, texture_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded entity cuboid model using a PNG resource in the owning package namespace; the model may include host-evaluated animation. |
| `sjml_registry_v1.register_block_entity_textured_model` | `(path_ptr: i32, path_len: i32, texture_ptr: i32, texture_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded block-entity cuboid model using a PNG resource in the owning package namespace; the model may include host-evaluated animation. |
| `sjml_registry_v1.register_entity` | `(path_ptr: i32, path_len: i32, width_bits: i32, height_bits: i32, tracking_range: i32, update_interval: i32, fire_immune: i32) -> i32` | Stage a generic living `MobCategory.MISC` entity type. |
| `sjml_registry_v1.register_entity_ai_goal` | `(path_ptr: i32, path_len: i32, goal: i32, priority: i32, value_bits: i32, flags: i32) -> i32` | Add one bounded vanilla goal to an entity declared by the same module. |
| `sjml_registry_v1.register_fluid` | `(path_ptr: i32, path_len: i32, tick_rate: i32, level_decrease_per_block: i32, slope_find_distance: i32, explosion_resistance_bits: i32) -> i32` | Stage a flowing fluid pair, liquid block, and bucket with vanilla water visuals. |
| `sjml_registry_v1.register_fluid_with_visuals` | `(path_ptr: i32, path_len: i32, tick_rate: i32, level_decrease_per_block: i32, slope_find_distance: i32, explosion_resistance_bits: i32, still_texture_ptr: i32, still_texture_len: i32, flowing_texture_ptr: i32, flowing_texture_len: i32) -> i32` | Stage a fluid with package-owned `block/` still and flowing texture IDs. |
| `sjml_registry_v1.register_shapeless_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, ingredient_lines_ptr: i32, ingredient_lines_len: i32) -> i32` | Stage a bounded shapeless recipe with one to nine item or item-tag ingredients. |
| `sjml_registry_v1.register_shaped_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, pattern_ptr: i32, pattern_len: i32, key_pairs_ptr: i32, key_pairs_len: i32) -> i32` | Stage a bounded 1–3 by 1–3 shaped recipe with uppercase-symbol item or item-tag bindings. |
| `sjml_registry_v1.register_cooking_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, ingredient_token_ptr: i32, ingredient_token_len: i32, kind: i32, experience_bits: i32, cooking_time: i32) -> i32` | Stage a vanilla cooking recipe with an item or item-tag input. |
| `sjml_registry_v1.register_recipe_serializer` | `(path_ptr: i32, path_len: i32) -> i32` | Register a guest callback serializer for package-owned crafting recipe JSON. |
| `sjml_registry_v1.register_particle` | `(path_ptr: i32, path_len: i32) -> i32` | Stage a simple particle type using the vanilla `minecraft:generic_0` sprite fallback. |
| `sjml_registry_v1.register_particle_visual` | `(path_ptr: i32, path_len: i32, texture_ptr: i32, texture_len: i32, scale_bits: i32) -> i32` | Stage a simple particle with a package-owned `particle/` atlas resource and a scale from 0.05 through 4.0. |
| `sjml_registry_v1.set_entity_attribute` | `(path_ptr: i32, path_len: i32, attribute: i32, value_bits: i32) -> i32` | Set one bounded vanilla base attribute on a previously declared same-module entity. |
| `sjml_registry_v1.register_sound` | `(path_ptr: i32, path_len: i32) -> i32` | Stage a variable-range sound event under the module's resource namespace. |
| `sjml_registry_v1.register_attribute_effect` | `(path_ptr: i32, path_len: i32, category: i32, color: i32, attribute: i32, amount_bits: i32, operation: i32) -> i32` | Stage a vanilla status effect with one bounded vanilla attribute modifier. |
| `sjml_registry_v1.register_effect_tick` | `(path_ptr: i32, path_len: i32, interval_ticks: i32) -> i32` | Enable a server-thread `entity.effect.tick` callback for a previously declared effect at a 1–1,200 tick interval. |
| `sjml_registry_v1.register_custom_registry` | `(path_ptr: i32, path_len: i32, sync_to_clients: i32) -> i32` | Declare a module-owned NeoForge registry before `NewRegistryEvent`; the flag syncs numeric registry IDs. |
| `sjml_registry_v1.register_custom_value` | `(registry_path_ptr: i32, registry_path_len: i32, value_path_ptr: i32, value_path_len: i32, value_ptr: i32, value_len: i32) -> i32` | Stage a UTF-8 string value of at most 65,536 bytes in a previously declared same-module registry. |
| `sjml_registry_v1.get_custom_registry_value` | `(registry_path_ptr: i32, registry_path_len: i32, value_path_ptr: i32, value_path_len: i32, output_ptr: i32, output_capacity: i32) -> i32` | Write a schema-v1 `Option<String>` response after the custom registry's `RegisterEvent`. |
| `sjml_registry_v1.register_creative_tab` | `(path_ptr: i32, path_len: i32, title_ptr: i32, title_len: i32, icon_path_ptr: i32, icon_path_len: i32) -> i32` | Stage a module-owned tab with a previously declared item icon. |
| `sjml_registry_v1.add_to_creative_tab` | `(tab_path_ptr: i32, tab_path_len: i32, item_path_ptr: i32, item_path_len: i32) -> i32` | Add a previously declared module-owned item or block item to a tab. |
| `sjml_registry_v1.register_string_component` | `(path_ptr: i32, path_len: i32, max_characters: i32) -> i32` | Register a persistent/network-synchronized bounded UTF-8 string component type. |
| `sjml_registry_v1.set_item_string_component` | `(item_path_ptr: i32, item_path_len: i32, component_path_ptr: i32, component_path_len: i32, value_ptr: i32, value_len: i32) -> i32` | Set a static string-component default on a previously declared item or block item. |

Paths are 1–256 lowercase ASCII letters, digits, `.`, `_`, or `-`. Boolean
fields accept exactly zero or one. Item stack sizes are 1–99; block strengths
and resistances must be finite and non-negative. Declarations have bounded
per-module and total counts, duplicate IDs are rejected, and the block
registry phase must finish before generated block items are materialized.
`register_entity_ai_goal` is accepted only during `mod_register`, after the
same module has declared that entity. At most sixteen goals may be attached
to one type, 512 per module, and 8,192 total; priority is 0–31. Goal tags are
`0` swim, `1` random stroll,
`2` look at player, `3` melee attack, `4` random look around, `5` target
player, and `6` retaliate. `value_bits` is finite speed 0.01–8 for tags 1 and
3, finite range 1–64 for tag 2, and exactly positive zero for the other tags.
`flags` is zero except tags 3 and 5, where 0/1 select melee long-memory or
player-target must-see behavior. Invalid tags, parameters, entity ownership,
and attempts after the entity registry phase closes return
`INVALID_ARGUMENT` or `PERMISSION_DENIED` without allocating a Minecraft goal.
Model imports run only during `mod_register` and require `content.register`.
The static payload is schema `u32=1`, cuboid count `u32`, then that many
28-byte records: six little-endian finite `f32` bounds and an ARGB `u32`.
Animated payloads with only one model-wide rotation use schema `u32=2`, the same
header and cuboids, followed by an animation marker `u32`. Marker `0` means no
animation and has no trailing bytes; marker `1` is followed by axis `u32` (`0`
X, `1` Y, `2` Z), three pivot `f32` coordinates, amplitude in degrees `f32`,
period `u32` ticks and phase `u32` ticks. The marker-1 section is 32 bytes
including the marker.

Schema `u32=3` adds independent child rotations. After the same header and
cuboids, it contains exactly `count + 1` 32-byte animation slots: the first is
the optional model-wide parent rotation, followed by one optional rotation per
cuboid in insertion order. In each slot, marker `0` is followed by 28 zero
padding bytes; marker `1` is followed by the axis, pivot, amplitude, period and
phase fields described above. The v3 payload is at most 1,960 bytes for 32
cuboids. Schema-v1 and schema-v2 payloads retain their original layouts. Axes
must be in range, pivots must be within the model's local bounds, amplitude
finite and within -180..180 degrees, period 1..12,000 ticks and phase below
the period. The host applies model-wide transforms before each cuboid's local
transform and computes deterministic sine rotations from game time and partial
tick without calling guest code during rendering. Each model has 1–32 cuboids,
at most 256 cuboids are staged per module and 8,192 total. Entity-local
coordinates are bounded to +/-4; block-entity geometry and animation pivots
must remain inside `[0,1]`. Models can only target an entity or block entity
declared by the same module, and one model can be registered per type. The
client draws them through Minecraft's buffered vertex pipeline; the model API
does not grant raw GL access.
Custom registry IDs use `<package-id>:custom/<module-id>/<path>` and entry IDs
use the module's normal `<package-id>:<module-id>/<path>` namespace. Registry
declarations and entries are staged only during `mod_register`; NeoForge
creates the registry during `NewRegistryEvent` and adds entries during its
`RegisterEvent`. Each module may declare only its own registries and values.
The registry can contain at most 256 entries, each UTF-8 value is bounded to
65,536 bytes, and the platform caps custom registries at 256 globally. The
`sync_to_clients` flag follows NeoForge's numeric-ID synchronization behavior;
clients must load the same package and declarations, and the flag does not
serialize arbitrary value payloads. Runtime lookup is available after
registration and returns the same module's string value or no value when the
entry is absent. The response frame is little-endian `u32 schema=1`, `u8`
option tag (`0=None`, `1=Some`), then for `Some`, `u32 UTF-8 byte length` and
the bytes.
Block entity declarations require a block previously staged by the same module;
the block entity registry event runs after blocks and attaches a generic host
block-entity class. Generic entity dimensions are finite and 0.01–16 blocks,
tracking range is 1–128 chunks, update interval is 1–1,200 ticks, and its
boolean accepts only zero or one. Entity attribute IDs are `0=max health`,
`1=movement speed`, `2=attack damage`, `3=armor`, `4=follow range`, and
`5=knockback resistance`; values must be finite and fit the per-attribute ranges
documented in `SDK_GUIDE.md`. Each may be assigned once to an entity declared by
the same module. The host applies these values in NeoForge's
`EntityAttributeCreationEvent`. `register_entity_ai_goal` adds the bounded
vanilla goals listed above; arbitrary guest AI callbacks and custom synchronized
fields are not exposed. Guest-declared entities and block entities can render
colored or package-textured cuboid models with bounded host-evaluated sine
rotation. Modules can also attach opaque persistent bytes through the separate
`entity.data.read` and `entity.data.write` permissions described below. These
attachments are distinct from per-save entity-scope storage. Static registry
declarations are staged during `mod_register` and materialized only in their
matching NeoForge registry events.
Fluid declarations stage a source and flowing fluid, a `LiquidBlock`, and a
single-stack bucket. The generated IDs append `_source`, `_flowing`, `_block`,
and `_bucket` to the declared path; fluid paths are limited to 248 bytes so
each generated path stays within the 256-byte resource path bound. Tick rate is 1–1,200, level decrease is
1–8, slope search is 1–16 blocks, and explosion resistance is finite from 0 to
1,000,000. `register_fluid` uses NeoForge's water fluid type and textures.
`register_fluid_with_visuals` registers a package-scoped NeoForge fluid type
with the provided still and flowing textures. Its texture IDs use Minecraft's
`namespace:block/path` convention and resolve to package assets at
`assets/<namespace>/textures/block/path.png`. The host generates blockstate,
block model, bucket model, and English language metadata for both possible
liquid-block IDs. Package-owned client resources under `assets/<namespace>/`
are mounted automatically.
The block and fluid registry events can arrive in either order; the host keeps
their supplier links valid and emits the bucket only after both phases. If
another mod owns the generated `_block` ID, the host uses `_liquid` when that
fallback ID is available.
Shapeless, shaped, and cooking recipes are emitted into an always-enabled generated
server data pack and decoded by Minecraft's vanilla
`minecraft:crafting_shapeless`, `minecraft:crafting_shaped`, `minecraft:smelting`,
`minecraft:blasting`, `minecraft:smoking`, and `minecraft:campfire_cooking` serializers.
Ingredient tokens are ASCII resource IDs: an item is written as `namespace:path`
or `path`, and an item tag is written as `#namespace:path` or `#path`. An
unqualified path resolves inside the declaring package and module; a namespaced
ID can refer to vanilla or another mod. Shapeless recipes accept one to nine
ingredient tokens. Shaped patterns use one to three equal-width rows of one to
three uppercase-letter or space cells, with one item or tag binding for every
used letter and no unused keys. Both families accept an output count of one to
64. Cooking kind codes are `0=smelting`, `1=blasting`, `2=smoking`, and
`3=campfire cooking`. Cooking recipes use one item or tag input, experience
from 0 to 1,000, and cooking time from 1 to 36,000 ticks. All recipe families
share the output-count range of one to 64. Brewing and arbitrary Java recipe
serializers are not provided. Guest-defined WASM serializers are registered
with `register_recipe_serializer` and a `#[recipe(name = "...")]` export; their
callback name must match the registered path and their module must declare
`side = "both"`. Package files under
`data/<mod.id>/recipe/` use the serializer's type ID
`<mod.id>:<module>/<path>`, an `ItemStack` `result` preview, and an optional
UTF-8 `payload` string up to 65,536 bytes. The host invokes the callback
synchronously on the owning client/server thread with the schema-v1 crafting
grid snapshot and opaque payload. The callback returns `matches` and an output
count from one to 64; a non-match must return count zero. The output count may
not exceed the result item's stack limit. Callback failures disable that
module and make the recipe fail closed.
The callback export name is `sjml_v1_recipe_<name>`. Its stable input record is
`u32 schema=1`, a length-prefixed payload string, `u32 width`, `u32 height`, and
a length-prefixed row-major vector of optional slots. Each occupied slot is an
option tag followed by `u32 schema=1`, a length-prefixed item ID string, and a
`u32` item count. The stable result is `u32 schema=1`, a one-byte match flag,
and a `u32` output count.
For `register_shaped_recipe`, `pattern` is UTF-8 text with rows separated by
line feed bytes. `key_pairs` is a sequence of uppercase-symbol, line-feed,
ingredient-token groups, with groups separated by a line feed and no trailing
line feed; for example `R\n#forge:ingots/ruby\nS\nminecraft:stick`. Ingredient
tokens are strictly decoded and bounded before staging.
Attribute effects use category IDs `0=beneficial`, `1=harmful`, and `2=neutral`;
their color is a 24-bit RGB integer. They can add one of the supported vanilla
attributes using operation IDs `0=add value`, `1=add multiplied base`, and
`2=add multiplied total`. The modifier amount is finite and limited to `-16`
through `16`. The resulting `MobEffect` is available to vanilla `/effect` and
datapack commands. Guest modules can apply registered effects to live living
entities and login-session player handles through separately permissioned APIs.
Sound declarations create a variable-range `SoundEvent` in the NeoForge sound
registry. The host mounts package-owned client assets under
`assets/<namespace>/` from its generated resource pack. Include a matching
`sounds.json` and referenced audio file there; vanilla `/playsound` can address
the registered event.
Creative tab titles are 1–256 UTF-8 bytes without control characters. Declare
the icon and entries in the same module: stage their items first, declare the
tab with `CreativeTabDefinition`, then add item and block-item paths. Tabs are
materialized after the item registry event; missing/colliding items are omitted
or cause a tab without its required icon to be skipped. A tab may contain at
most 128 entries, and content and tab entries share existing per-module/global
registration quotas.
String components are persistent and network-synchronized. Their values are
limited to 1,024 Unicode scalar values and 4 KiB of UTF-8 per value; each type
may lower the character limit. A module may set a static default on its own
items during `mod_register`, read its own registered string component from a
loaded inventory slot with `inventory.read`, and update it with the separate
`inventory.write` grant on loaded container-backed block entities.
These imports are available only when the module requested `content.register`
and the host explicitly granted it. NeoForge registry events perform the
actual registrations.

## Read-only menu import and action callback

`sjml_menu_v1.open` opens a server-side, read-only vanilla chest screen for a
live player handle:

| Import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_menu_v1.open` | `(player_ptr: i32, player_len: i32, menu_ptr: i32, menu_len: i32) -> i32` | Opens a bounded snapshot menu for that login session. |

The 24-byte player handle uses the ABI handle layout. The menu payload is at
most 16 KiB and starts with little-endian `u32` version `2`, `u32` row count
(1–6), a length-prefixed UTF-8 title, and a `u32` slot count (0–54). Each slot
contains a `u32` slot index, a length-prefixed canonical item ID, a `u32` count
(1–99), a length-prefixed display label, and an optional action marker byte.
Marker `0` has no action; marker `1` is followed by a length-prefixed action ID
of 1–64 lowercase ASCII characters from `[a-z0-9_.-]`. Slot indices must fit
the selected rows and be unique. Text is limited to 256 UTF-8 bytes and rejects
control characters. Unknown item IDs and counts above the registered item
stack limit are rejected.
The host continues to accept version `1` menus from older ABI-v1 guests; those
menus have no action markers and remain read-only.

The menu uses Minecraft's registered generic chest menu types and vanilla
screen, so common/server classes do not link client screen classes. The menu is
a snapshot: item movement, quick-move, and all clicks except a left-click on a
declared action slot are ignored. Action slots dispatch the schema-v1
`menu.action` callback with fields `player_id: String`, `slot_index: u32`, and
`action_id: String`. This requires a live `events.subscribe` subscription for
`menu.action` plus `entity.read` and `menu.open`; callbacks are bounded to eight
per module per server tick. Calls and callbacks run on the owning server thread.
This operation is unavailable from static `mod_register` and client-only
module contexts.

## Guest command callbacks (NeoForge adapter)

The manifest declares each command under its module with
`[[modules.commands]]`; ordered `[[modules.commands.arguments]]` entries define
the input record layout. `name` is the callback identifier used by
`#[command(name = "...")]`, `schema_version` must be a positive `u32` matching
`#[sjml(version = N)]` on the Rust input record, and `permission_level` is the
minimum Brigadier source permission level (0 through 4). A module must request
`content.register` and receive the administrator grant before its commands
are registered. The command is available at
`/sjml guest <package-id> <module-id> <command-name> ...`.

All manifest arguments are required and appear in declaration order. The
supported argument types and stable Rust field types are:

| Manifest `type` | Parsed value | Stable record field |
| --- | --- | --- |
| `word` | One unquoted Brigadier word | `String` |
| `string` | One word or one quoted string | `String` |
| `greedy_string` | Remaining command input, including spaces | `String` |
| `integer` | Signed 32-bit integer with optional inclusive `min`/`max` | `i32` |
| `boolean` | `true` or `false` | `bool` |

`min` and `max` are accepted only for `integer` and must fit `i32`; when
omitted, the respective bound is the full `i32` limit. A `greedy_string` must
be the final argument. String values are limited to 65,536 UTF-8 bytes each,
and the complete input record is limited to 1 MiB. A module may declare up to
64 commands, and a command may declare up to 16 arguments; the host stages at
most 8,192 commands across all packages. Client-only module command declarations
are rejected. The manifest format and a complete example are documented in
`FORMAT.md`.

For a command with schema version `N`, the host encodes its callback input as
the little-endian `u32` value `N`, followed by each declared argument in
order. Strings use the stable encoding `u32 byte_length` followed by strict
UTF-8 bytes; integers use one little-endian two's-complement `i32`; booleans
use one byte (`0` or `1`). There is no argument count, name, optional value,
or default in the binary record: the manifest and Rust field order define the
same schema. The callback's decoded return value must be a stable `String`,
limited to 65,536 UTF-8 bytes. The normal callback frame still adds its
four-byte status prefix and remains subject to the 1 MiB frame bound. An
invalid callback ABI prevents the package's declarations from staging. At
invocation, a guest error, trap, or malformed reply disables only that module;
the server command reports failure without exposing guest details.

The host checks `permission_level` during command parsing and again before
invocation. On invocation it resolves the currently active module for the
command's logical server, so a Brigadier tree rebuilt during resource reload
does not retain a closed WASM instance. Command callbacks run synchronously
on the owning server thread.

## Error values

Host adapters should translate AbiException into the matching numeric ABI
error. Values are stable and must not be reassigned.

| Code | Name |
| ---: | --- |
| 1 | INCOMPATIBLE_ABI_VERSION |
| 2 | MEMORY_OUT_OF_BOUNDS |
| 3 | PAYLOAD_TOO_LARGE |
| 4 | INVALID_UTF8 |
| 5 | INVALID_BOOLEAN |
| 6 | ARITHMETIC_OVERFLOW |
| 7 | INVALID_ARGUMENT |
| 16 | INVALID_HANDLE |
| 17 | STALE_HANDLE |
| 18 | HANDLE_OWNER_MISMATCH |
| 19 | HANDLE_CONTEXT_MISMATCH |
| 20 | HANDLE_KIND_MISMATCH |
| 21 | HANDLE_NOT_TRANSFERABLE |
| 22 | HANDLE_TABLE_CLOSED |
| 23 | HANDLE_TABLE_FULL |
| 24 | PERMISSION_DENIED |
| 25 | INVALID_TRANSFER |
| 26 | TASK_LIMIT_REACHED |
| 27 | TASK_REJECTED |
| 28 | SERVICE_NOT_FOUND |
| 29 | SERVICE_VERSION_INCOMPATIBLE |
| 30 | SERVICE_ACCESS_DENIED |
| 31 | SERVICE_BUSY |
| 32 | SERVICE_CALL_FAILED |
| 33 | SERVICE_TIMEOUT |
| 34 | REGISTRATION_LIMIT_REACHED |
| 255 | UNKNOWN_ERROR_CODE |

## Opaque resource handles

A handle occupies **24 bytes**, encoded as three little-endian u64 words:

| Byte offset | Width | Field |
| ---: | ---: | --- |
| 0 | 8 | owner scope token |
| 8 | 8 | execution-context token |
| 16 | 8 | slot token |

The slot token packs the one-based slot index into its low 32 bits and the
generation into its high 32 bits. Index zero and generation zero are invalid.
Generation starts at one and advances on revoke. When its 32-bit space is
exhausted, that slot is retired and never reused.

The resource kind and required capability set stay in the host-side table;
they are not trusted from guest memory. The table also retains the Java object.
No Java reference is serialized into guest memory. Resolution checks owner,
execution context, generation, expected kind, and the caller's granted
capabilities before returning a host-side resource to trusted adapter code.

Each module owner and execution context must get separate non-zero scope
tokens. HandleTable.newScopeToken() generates opaque tokens. A handle from a
different owner or context fails before slot lookup. A revoked or reused slot
fails its generation check.

Transfer is explicit and host-controlled. A handle can move to another table
only when it was created as transferable. Transfer creates a handle in the
target scope and revokes the source handle atomically; ordinary handles cannot
be shared between modules or contexts.

Stable HandleKind codes:

| Code | Kind |
| ---: | --- |
| 1 | ENTITY |
| 2 | PLAYER |
| 3 | WORLD |
| 4 | BLOCK |
| 5 | BLOCK_ENTITY |
| 6 | ITEM_STACK |
| 7 | EVENT_SUBSCRIPTION |
| 8 | TASK |
| 9 | NETWORK_CHANNEL |
| 10 | STORAGE_SCOPE |
| 11 | RENDER_RESOURCE |
| 12 | GL_RESOURCE |

These codes are reserved for their current meanings and must not be reused.
The portable core and NeoForge server adapter implement async task polling and
service routing. Logical-server modules admit tasks after `mod_init`; the host
polls them on its bounded server tick queue. Physical-client contexts and
static registration remain fail-closed, and task admission is unavailable
during `mod_init`. NeoForge also implements the synchronous event, world,
player, entity, inventory, and block-entity imports described below. The Rust
networking SDK and handshake codec are wired into NeoForge module activation
and payload handling. Persistent storage imports use per-save `SavedData` on
logical-server contexts. A live client/server network session has not been
verified end to end. NeoForge also wires client-only render callback imports
(`client.render`) and typed OpenGL imports (`client.render.opengl`). The callback
layers are `0=HUD`, `1=active screen`, and `2=world`; the world layer carries the
camera position and accepts bounded line/outlined-box batches. A live Minecraft
client/OpenGL context has not been verified. See [Client rendering](RENDERING.md)
for the complete `sjml_render_v1` and `sjml_gl_v1` import signatures, wire
formats, limits, handle kinds, and the runtime verification boundary. Those
render and OpenGL import tables are maintained in that companion ABI document.

## Active-screen input imports

Client-screen click and key callbacks use `client.input`, requested separately
from `client.render`. The `sjml_screen_input_v1` import functions are
client-main-thread-only, module-scoped, and recheck the current grant. Static
registration and dedicated-server module instances fail closed.

| Import | WASM signature | Purpose |
| --- | --- | --- |
| `subscribe` | `(i64 callback_id, i32 handle_out_ptr) -> i32` | Register a typed click/key callback and write its 24-byte event-subscription handle |
| `unsubscribe` | `(i32 handle_ptr, i32 handle_len) -> i32` | Revoke one subscription owned by this module runtime |

The host calls `sjml_v1_screen_input_dispatch(pointer, length) -> i64`. Its
little-endian payload begins with schema `u32` (currently `1`), callback ID
`u64`, and event tag `u8`. Tag `0` is a mouse click followed by X and Y as
bounded `f32`, button `u8`, and modifiers `u8` (23 bytes total). Tag `1` is a
key press followed by key code `i32`, scan code `u16`, modifiers `u8`, and
repeat `u8` (21 bytes total). The callback result is stable bool `u8`: `1`
consumes the event, while `0` leaves it available. Coordinates are finite and
bounded to ±1,048,576; buttons are 0–31; key codes are -1–65,535; scan codes
are 0–65,535; key modifier bits are limited to the six standard GLFW flags.
Mouse events report Shift, Control, and Alt. A
module may hold 256 active input subscriptions.

## Native client chat layout imports

ABI 1.33.0 adds client-only `sjml_client_chat_v1` imports. Each operation
requires requested and currently granted `client.chat.modify`, the client main
thread, and runtime phase. Static registration and dedicated-server instances
fail closed. Only the active exclusive controller can query or update geometry.

| Import | WASM signature |
| --- | --- |
| `subscribe` | `(i64 callback_id, i32 output_handle_ptr) -> i32` |
| `unsubscribe` | `(i32 handle_ptr, i32 handle_len) -> i32` |
| `layout` | `(i32 handle_ptr, i32 handle_len, i32 output_layout_ptr) -> i32` |
| `set_layout` | `(i32 handle_ptr, i32 handle_len, i32 layout_ptr, i32 layout_len, i32 output_layout_ptr) -> i32` |

Zero status means success; nonzero is a stable ABI error. Subscription output
is the existing 24-byte ResourceHandle encoding with handle kind `13`.
Handle inputs must be exactly 24 bytes. Layout inputs and outputs are exactly
16 bytes: four little-endian IEEE-754 `f32` values in order `x,y,width,height`.
Coordinates must be finite and dimensions finite positive. The host returns a
rectangle clamped to the viewport and a 120×56 minimum reduced to fit smaller
viewports. A competing controller returns `REGISTRATION_LIMIT_REACHED` (34).
Runtime closure or unsubscription invalidates the controller and callback route.

The guest exports `sjml_v1_client_chat_dispatch(pointer:u32, length:u32)->u64`.
It uses the standard packed ByteSpan response framing: success contains one
stable bool byte; failure contains the standard stable error response. The
input is schema `u32=1`, callback ID `u64`, event tag `u8`, then:

| Tag | Variant | Body in exact field order |
| --- | --- | --- |
| 0 | Opened | `f32 viewport_width, viewport_height, x, y, width, height` |
| 1 | Closed | `f32 viewport_width, viewport_height, x, y, width, height` |
| 2 | ViewportResized | `f32 viewport_width, viewport_height, x, y, width, height` |
| 3 | MousePressed | `f32 mouse_x, mouse_y; u8 button` |
| 4 | MouseDragged | `f32 mouse_x, mouse_y; u8 button; f32 delta_x, delta_y` |
| 5 | MouseReleased | `f32 mouse_x, mouse_y; u8 button` |

All fields use stable little-endian encoding. No trailing bytes are accepted.
Viewport dimensions are finite positive and at most 1,048,576. Pointer
coordinates and deltas are finite within ±1,048,576; buttons are 0–31. Only
left-button (`0`) grip events may return consumed; other events are observational.
Callbacks are synchronous and reject reentrance with `SERVICE_BUSY` (31).
See [Native client chat layout](CHAT.md) for typed APIs, persistence, and the
live-client verification boundary. Existing ABI 1.32 guests remain supported.

## Global client input imports

Global key and mouse-button subscriptions are a separate API, available only
to client modules with the `client.input` capability. The
`sjml_client_input_v1` `subscribe` and `unsubscribe` signatures and
module-scoped 24-byte handles match the active-screen imports above. Calls and
callbacks run on the client main thread. Global callbacks are delivered only
when no screen is open, keeping screen-only and gameplay input paths distinct.

The host calls `sjml_v1_client_input_dispatch(pointer, length) -> i64`. The
little-endian payload is exactly 21 bytes: schema `u32` (currently `1`),
callback ID `u64`, kind `u8` (`0=key`, `1=mouse button`), action `u8`
(`0=press`, `1=release`, `2=repeat`), code `i32`, scan code `u16`, and
modifier bits `u8`. Key code -1 means an unknown key; key and scan codes are
bounded to -1..65,535 and 0..65,535. Mouse button codes are 0..31 and do not
repeat. Modifier bits are limited to the six standard GLFW flags. The callback
returns stable bool `u8`. NeoForge does not make global key notifications
cancellable, so their return value is ignored; only mouse-button presses can
be consumed. Releases remain observable but cannot be consumed. A module may
hold 256 active global input subscriptions. Static registration and dedicated
server instances fail closed.

## Logical-server worlds

ABI v1 provides the following synchronous `sjml_world_v1` imports. All parameters
and statuses are `i32`; zero is success and nonzero values use the error table.
World reads require the requested and granted `world.read` capability. Block
mutation and particle spawning additionally require requested and granted `world.write`. Static
registration binds the signatures but every call during `mod_register` returns
`PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `get` | `id_ptr, id_len, out_ptr, out_capacity` | Stable `Option<WorldHandle>`: byte `0`, or byte `1` followed by the 24-byte handle |
| `loaded` | `out_ptr, out_capacity` | Stable `Vec<WorldHandle>`: little-endian u32 count, then 24 bytes per handle |
| `dimension_id` | `handle_ptr, handle_len, out_ptr, out_capacity` | Stable `String`: little-endian u32 UTF-8 length, then bytes |
| `game_time` | `handle_ptr, handle_len, out_ptr` | Little-endian signed i64 |
| `block_states` | `query_ptr, query_len, out_ptr, out_capacity` | Stable ordered `Vec<Option<BlockStateSnapshot>>` |
| `raycast` | `query_ptr, query_len, out_ptr, out_capacity` | Stable `Option<RaycastHit>` for one bounded segment |
| `collides` | `query_ptr, query_len, out_ptr, out_capacity` | Stable `Option<bool>` for one bounded axis-aligned box |
| `set_block` | `handle_ptr, handle_len, x, y, z, block_id_ptr, block_id_len` | Replace one loaded position with the registered block's default state; status only |
| `spawn_entity` | `query_ptr, query_len` | Spawn one registered, non-player entity from a bounded schema-v1 world/type/position record; status only |
| `spawn_particle` | `handle_ptr, handle_len, x, y, z, particle_id_ptr, particle_id_len` | Spawn one registered simple particle at the center of a loaded block; status only |
| `play_sound` | `handle_ptr, handle_len, x, y, z, sound_id_ptr, sound_id_len, volume_bits, pitch_bits` | Play one registered sound at the center of a loaded block; status only |

`handle_len` must be 24. `get` takes raw UTF-8, limited to 65,536 bytes,
and requires a canonical `namespace:path` (no implicit `minecraft:` namespace).
Namespace bytes are lowercase ASCII letters, digits, `_`, `.`, or `-`; paths
add `/`. Empty components and malformed IDs return `INVALID_ARGUMENT`;
invalid UTF-8 returns `INVALID_UTF8`. Unknown or unloaded IDs return `None`.

Output buffers belong to the guest and stay live for the synchronous call.
`get` requires capacity at least 25; `loaded` requires at least `4 + 24 * count`;
`dimension_id` requires `4 + utf8_length`; `game_time` needs 8 writable bytes.
The complete declared capacity is bounds checked. Only the encoded value is
written; the remaining capacity is padding, not part of its stable encoding.
Insufficient capacity returns `PAYLOAD_TOO_LARGE` before handle allocation.
The SDK allocates 25, 24,580, 65,540 and 8 bytes respectively. The string limit
remains 65,536 UTF-8 bytes.

Each active module owns one world handle registry with a 1,024-live-handle
quota. `get` and `loaded` share handles. `loaded` sorts all currently loaded
dimensions, including dimensions from other mods, by canonical ID and returns
the complete list or `HANDLE_TABLE_FULL`, with no partial allocations.
Neither operation loads a dimension. Every read validates `world.read`, kind
`WORLD` (3), module owner, context, slot and generation. World handles cannot
be transferred. `LevelEvent.Unload` revokes that level's handles before close
and prevents reacquisition while unload listeners run; module/runtime close
revokes all remaining handles. Reloaded worlds receive new handles.

All enumeration, reads, and mutations require the owning logical server thread. Before
server binding, `get` returns `None` and `loaded` returns an empty vector so
early dedicated `mod_init` remains safe. Integrated activation binds its server
before `mod_init`; server-start callbacks run after binding. Client module
instances expose no server-world data. The current permission model is per
module: `world.read` exposes all loaded dimensions on that server, with no
per-dimension ACL. `world.write` likewise covers every loaded dimension and
has no per-position ACL.

`block_states` reads 1–1,024 coordinates in one host call; an empty position
list is also valid. Its request is little-endian: query schema `u32(1)`, the
24-byte world handle, a `u32` count, then for each position schema `u32(1)` and
signed `i32` x/y/z. Requests are limited to 16,416 bytes and reject unknown
schemas, truncation, excess counts, and trailing bytes before resolving the
world. The response is schema `u32(1)`, a `u32` count equal to the request,
then one `u8` option tag per position (`0` for unavailable; `1` followed by a
versioned `BlockStateSnapshot` for available). Each snapshot contains the
length-prefixed UTF-8 block ID and state string. The complete response is
bounded to 1 MiB and is written only after encoding succeeds and the guest's
entire output span is valid. Results preserve input order and duplicate
coordinates. Positions outside build height and positions in unloaded chunks
return `None`; the host checks chunk availability before reading and never
loads one. Every query retains the world handle's module, context, permission,
generation, and server-thread checks.

`raycast` is server-only and follows the world handle's `world.read`, module-owner,
generation, and owning-server-thread checks. Its request is exactly 78 bytes:
schema `u32(1)`, the 24-byte world handle, start and end coordinates as six
little-endian `f64` values, then block-shape and fluid-shape tags as `u8`. All
coordinates must be finite and within ±30,000,000; the segment length is at most
128 blocks. Block tags `0/1/2` select collider, outline, or visual shapes. Fluid
tags `0/1/2` select none, source-only, or any fluid. Unknown tags, trailing
bytes, and invalid bounds are rejected before world access. The response is a
stable optional: `u8(0)` for a miss, or `u8(1)` followed by versioned
`RaycastHit` data (position, face, hit location, and detached block snapshot).
The complete encoded hit is bounded to 65,857 bytes. Before clipping, the host
checks each horizontal chunk touched by the segment through the server chunk
cache's loaded-only `hasChunk` query. If any is unavailable, the result is a
miss. Clipping uses a read-only block view backed by `getChunkNow`; shape-side
reads that reach an unloaded chunk also produce a miss. The host never calls a
chunk-loading accessor for the ray or shape checks.

`collides` is server-only and requires the same world handle, `world.read`,
module-owner, and server-thread checks. Its request is exactly 76 bytes:
schema `u32(1)`, the 24-byte world handle, then six little-endian `f64`
coordinates in `min_x, min_y, min_z, max_x, max_y, max_z` order. Bounds must
be finite, ordered, and within ±30,000,000; no axis may exceed 16 blocks and
the covered voxel volume is limited to 4,096 positions. The response is a
stable `Option<bool>` (`u8(0)`, or `u8(1)` followed by bool `u8(0/1)`). The host
uses empty-entity block collision shapes only. If the query leaves build height,
touches an unloaded chunk, or a block shape reads an unloaded neighbor, the
answer is `None`; otherwise it reports whether any solid shape intersects the
box. Chunk preflight uses loaded-only cache checks and shape reads use the same
no-load block view as raycasting.

`set_block` additionally requires `world.write`, a canonical lowercase block
registry ID, a position inside build height, and an already loaded chunk. It
places the block's default state on the owning server thread and never loads a
chunk. Invalid IDs, out-of-height positions, and unloaded chunks return
`INVALID_ARGUMENT`; static-registration calls are denied.

`spawn_entity` requires `world.read` and the separate administrator-granted
`entity.spawn` capability. Its stable request is schema `u32(1)`, the 24-byte
world handle, a length-prefixed canonical entity type ID, and three finite
little-endian `f64` coordinates within ±30,000,000. It accepts only registered
non-player entity types and is server-only. The position and entity bounds
must fit build height, the world border, and already loaded chunks; no chunk is
loaded. The module can spawn at most eight entities per server tick. Invalid
types or positions return `INVALID_ARGUMENT`, rate limits return `SERVICE_BUSY`,
and static-registration calls are denied.

`spawn_particle` has the same handle, thread, build-height, loaded-chunk, and
`world.write` checks. It accepts only a registered `SimpleParticleType`, emits
one particle at the block center, and never loads a chunk. Guest-declared
particle types use SJML's shared translucent renderer and the vanilla
`minecraft:generic_0` sprite through an always-enabled generated client resource
pack. Static-registration calls are denied.

`play_sound` requires the same world capabilities, handle, thread, build-height,
and loaded-chunk checks. It accepts a registered sound-event ID, finite volume
from 0 to 16, and finite pitch from 0.5 to 2.0; the event plays at the block
center through Minecraft's `MASTER` sound source. Client assets under the
package's own `assets/<namespace>/` directory are mounted from a generated
resource pack. Static-registration calls are denied.

## Logical-server players

ABI v1 provides synchronous `sjml_player_v1` imports. All parameters and
statuses are `i32`; zero is success and nonzero values use the error table.
The module must request and receive `entity.read`. Static registration binds
signatures but calls return `PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `online` | `out_ptr, out_capacity` | Stable `Vec<PlayerHandle>`: little-endian u32 count, then 24 bytes per handle |
| `snapshot` | `handle_ptr, handle_len, out_ptr, out_capacity` | `PlayerSnapshot`: u32 schema version `1`, three length-prefixed UTF-8 strings (`uuid`, `profile_name`, `dimension_id`), then three little-endian `f64` values (`x`, `y`, `z`) |
| `apply_effect` | `handle_ptr, handle_len, effect_id_ptr, effect_id_len, duration_ticks, amplifier` | Apply one registered effect to the current player for that login session; status only |

`handle_len` must be 24. UUIDs are canonical lowercase UUID text, and
dimension IDs are canonical `namespace:path`. Each stable string is limited to
65,536 UTF-8 bytes. The SDK reserves `4 + 24 * 1024` bytes for enumeration and
`4 + 3 * (4 + 65,536) + 24` bytes for a snapshot. The complete declared output
capacity is bounds checked; insufficient capacity returns
`PAYLOAD_TOO_LARGE` before new enumeration handles are allocated.

Each module owns a 1,024-live-handle quota. Enumeration sorts by UUID and
returns the complete list or an error. Every snapshot validates capability,
kind `PLAYER` (2), module owner, context, slot, and generation. Handles are
non-transferable and scoped to one login session; they survive respawn and
dimension changes, then are revoked as logout begins before callbacks run.
Calls are synchronous on the owning logical-server thread. Client and
pre-server-binding enumeration return an empty vector. `entity.read` does not
require `world.read`. Applying an effect additionally requires
`entity.effect.apply`, a canonical registered effect ID, duration of 1–72,000
ticks, and amplifier of 0–255. The host resolves the current `ServerPlayer`
from the login-session handle on the server thread, so a handle remains valid
through respawn but becomes invalid when logout begins. Static registration is
denied; this operation does not load a world or expose player inventory.

## Loaded non-player entities

ABI v1 provides synchronous `sjml_entity_v1` imports. All parameters and
statuses are `i32`; zero is success and nonzero values use the error table.
Entity snapshots require `entity.read`. The `loaded` import is also available
to modules that request and receive either entity data capability so they can
obtain opaque attachment handles without snapshot access. This global API does
not take a world handle and does not require `world.read`. Static registration
binds signatures but calls return `PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `loaded` | `out_ptr, out_capacity` | Stable `Vec<EntityHandle>`: little-endian u32 count, then 24 bytes per handle |
| `snapshot` | `handle_ptr, handle_len, out_ptr, out_capacity` | `EntitySnapshot`: u32 schema version `1`, three length-prefixed UTF-8 strings (`uuid`, `entity_type_id`, `dimension_id`), then three little-endian `f64` values (`x`, `y`, `z`) |
| `apply_effect` | `handle_ptr, handle_len, effect_id_ptr, effect_id_len, duration_ticks, amplifier` | Apply one registered effect to a loaded living non-player entity; status only |

`handle_len` must be 24. `uuid` is canonical lowercase UUID text;
`entity_type_id` and `dimension_id` are canonical `namespace:path` IDs. Each
string is limited to 65,536 UTF-8 bytes. The SDK reserves `4 + 24 * 1024` bytes
for enumeration and `4 + 3 * (4 + 65,536) + 24` bytes for a snapshot. The complete declared output
capacity is bounds checked; insufficient capacity returns
`PAYLOAD_TOO_LARGE` before allocating handles. Effect application additionally
requires both `entity.read` and `entity.effect.apply`; static-registration calls
are denied. The ID is strict UTF-8 in canonical `namespace:path` form and at
most 256 bytes. Duration is 1–72,000 ticks and amplifier is 0–255. The host
revalidates the handle and resolves the effect from the active registry on the
owning server thread. Non-living targets and unknown IDs return
`INVALID_ARGUMENT`; this operation cannot target players.

Each module owns a 1,024-live-handle quota. Enumeration returns loaded
non-player entities from every loaded server dimension, including modded
dimensions, sorted by canonical lowercase UUID. It shares handles for the
same live entity instance and returns the complete vector or an error without
partial allocations. It never loads chunks. Every snapshot validates
capability, kind `ENTITY` (1), module owner, context, slot, and generation.
Handles are tied to one entity instance and are revoked when it leaves its
level (including chunk unload or dimension transfer), when its world unloads,
or when the module/runtime closes. Players use `sjml_player_v1`. Calls are
synchronous on the owning server thread; clients and pre-server-binding
contexts enumerate an empty vector. Snapshot values expose no Minecraft object,
NBT, components, or mutable references; `apply_effect` is the separate bounded
mutation operation and requires its own capability.

## Non-player entity attachments

ABI v1 provides byte-only `sjml_entity_v1` imports for values saved in the
current entity's NeoForge persistent NBT. Their permissions are independent:
`data_read` requires `entity.data.read`; `data_write` and `data_remove` require
`entity.data.write`. The guest still needs an opaque handle from `loaded`, but
either data permission is sufficient for that enumeration. It does not grant
`snapshot`, which remains gated by `entity.read`. Static registration links
declared imports but all calls return `PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `data_read` | `request_ptr, request_len, out_ptr, out_capacity` | Schema-v1 response: u32 schema `1`, signed i32 value length (`-1` means absent; `0` is a present empty value), then value bytes |
| `data_write` | `request_ptr, request_len` | Replace one module-owned byte value; status only |
| `data_remove` | `request_ptr, request_len, out_ptr, out_capacity` | Schema-v1 response: u32 schema `1`, then u8 boolean indicating whether a value existed |

Each request is at most 16,549 bytes and contains little-endian
`u32(1)` schema, a u8 operation (`1` get, `2` set, `3` remove), the 24-byte
`ENTITY` handle, a u32-length-prefixed strict UTF-8 key, and for set a
u32-length-prefixed value. Keys are 1–128 bytes and use lowercase ASCII
letters, digits, `.`, `_`, `-`, or `/`; empty, `.` and `..` path segments are
rejected. Values are limited to 16 KiB. Read output is bounded to 16 KiB plus
its eight-byte header; removal output is five bytes. Insufficient output
capacity fails before any response bytes are written.

The host stores entries under its fixed `sjml:entity_attachments` NBT key,
then separates them by package ID, module ID, and local key. It never uses a
guest or third-party mod namespace for the host's NBT key, and it exposes only
copied bytes. Each module is limited to 128 entries and 64 KiB of accounted
data per entity; the aggregate SJML data on an entity is limited to 1,024
entries and 256 KiB, including key and owner-name overhead. Replacements are
checked atomically against these limits. Missing reads return length `-1`,
empty values return length `0`, and removing a missing entry returns `false`.

Calls require a current loaded non-player handle, run on the owning
logical-server thread, and never load chunks. NeoForge serializes this key
with the entity, so values follow it across save/load, unload, and dimension
transfer. This API is distinct from `sjml_storage_v1` entity scope, which is a
per-mod saved database keyed by UUID and is managed independently of the
entity's own NBT.

## Read-only block inventory snapshots

ABI v1 provides the synchronous `sjml_inventory_v1.read` import. All parameters
and the status are `i32`; zero is success and nonzero values use the error table.
The module must request and receive both `world.read` and `inventory.read`.
Static registration binds the authorized signature but every call returns
`PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `read` | `world_ptr, world_len, position_ptr, position_len, side_ptr, side_len, out_ptr, out_capacity` | Stable `Option<InventorySnapshot>`: byte `0`, or byte `1`, snapshot schema version `1`, u32 slot count, then one `Option<ItemStackDescriptor>` per slot |

The world handle length must be 24 bytes. `BlockPos` is schema-v1 followed by
three little-endian signed `i32` coordinates. `side` is stable
`Option<Direction>`: one zero byte for unsided lookup, or byte `1` followed by
a little-endian u32 direction (`0` down, `1` up, `2` north, `3` south, `4`
west, `5` east). Incorrect input byte lengths and invalid option tags or
directions return `INVALID_ARGUMENT`; unsupported schema versions return
`INCOMPATIBLE_ABI_VERSION`. A decoded world handle is then checked by the
existing registry, which returns the corresponding invalid, stale, owner,
context, kind, closed-table, or permission error when validation fails.

Each non-empty slot contains byte `1`, descriptor schema version `1`, a
length-prefixed UTF-8 canonical item registry ID, and little-endian u32 values
for stack count, item maximum stack size, and handler slot limit. An empty slot
contains byte `0`. These are detached values; no `ItemStack`, NBT, data
components, or handler object is exposed. Counts and the two limits are kept
separate and are not assumed to be less than one another.

The import queries `Capabilities.ItemHandler.BLOCK` for any provider at the
position, including providers that are not block entities. `None` side means
unsided lookup; a direction performs the normal sided query. An unloaded
position or absent capability returns `None`; a present zero-slot handler
returns `Some` with an empty vector. Reads are synchronous on the owning server
thread and never load a chunk. The complete declared output span is bounds
checked, and a result is fully encoded before any bytes are written. A read is
limited to 4,096 slots and 1 MiB; an oversized result returns
`PAYLOAD_TOO_LARGE` without partial output. Each active module's world handle
must pass the existing owner, context, kind, generation, `world.read`, and
server-thread checks before slot access.

## Module-owned string components on inventory items

The `sjml_inventory_v1` component imports operate on one slot and only on
string components registered by the calling module. A module-local component
path is resolved under that module's package and module namespace; another
module's component type cannot be selected by passing a foreign registry ID.

| Import | WebAssembly signature | Capability and behavior |
| --- | --- | --- |
| `read_string_component` | `(world_ptr, world_len, position_ptr, position_len, side_ptr, side_len, slot, component_path_ptr, component_path_len, out_ptr, out_capacity) -> i32` | Requires `world.read` and `inventory.read`. Writes stable `Option<String>`: byte `0`, or byte `1`, little-endian u32 UTF-8 byte length, then strict UTF-8 bytes. |
| `write_string_component` | `(world_ptr, world_len, position_ptr, position_len, slot, component_path_ptr, component_path_len, value_ptr, value_len) -> i32` | Requires `world.read` and the separate `inventory.write` grant. Zero status means the copied stack was updated. |

Both imports run synchronously on the server thread and query only an already
loaded chunk. Reads return `None` for an unloaded position, missing capability,
empty slot, or unset component. Writes reject invalid slots and empty stacks;
the host copies the existing `ItemStack` before replacing its component, preserving its item,
count, and unrelated components. Writes are unsided and use the block entity's
standard `Container.setItem` API; they reject positions without a `Container`
and do not use sided item-handler capabilities. Values are limited by both the component's
registered character bound and 4 KiB of UTF-8. Paths are at most 256 UTF-8
bytes. Static `mod_register` calls are denied, and the host binds each import
only when its complete capability set is requested and granted. The write
grant is not scoped to particular blocks or items, so it permits a module to
change its own registered component on any reachable container slot.

## Read-only block entities

ABI v1 provides synchronous imports in `sjml_block_entity_v1`. Every import
requires the module to request and receive `world.read`; calls from static
registration return `PERMISSION_DENIED`. Block entity handles are module- and
runtime-scoped, use kind `BLOCK_ENTITY` (5), are non-transferable, and have a
1,024-handle per-module limit.

| Import | Parameters | Success output |
| --- | --- | --- |
| `get` | `world_ptr, world_len, position_ptr, position_len, out_ptr, out_capacity` | Stable `Option<BlockEntityHandle>`: byte `0`, or byte `1` followed by a 24-byte handle |
| `snapshot` | `handle_ptr, handle_len, out_ptr, out_capacity` | `BlockEntitySnapshot`: schema-v1 version, type ID, dimension ID, schema-v1 `BlockPos`, and block-state string |
| `inventory` | `handle_ptr, handle_len, side_ptr, side_len, out_ptr, out_capacity` | Same stable `Option<InventorySnapshot>` as `sjml_inventory_v1.read` |
| `nbt` | `handle_ptr, handle_len, out_ptr, out_capacity` | Versioned NBT tag tree described below |

`get` takes a 24-byte `WorldHandle` and a 16-byte stable `BlockPos` (schema
version 1, then signed little-endian `i32` x/y/z). The output capacity must be
at least 25 bytes and the complete declared span is bounds checked. It only
examines an already-loaded chunk; absence of a block entity returns `None`.
`snapshot` requires a 24-byte handle and encodes three stable UTF-8 strings
(`type_id`, `dimension_id`, `block_state`) around the versioned position
record. The SDK exposes this as `world_handle.block_entity(position)?`, with
`snapshot()`, `type_id()`, and `block_state()` accessors.

`inventory` accepts the same stable `Option<Direction>` encoding as the block
inventory import and requires both `world.read` and `inventory.read`. It
returns detached slot descriptors, not Minecraft objects. `nbt` additionally
requires the requested and granted `block_entity.nbt.read` capability. Its
output uses a little-endian u32 schema version `1`, followed by a recursively
tagged NBT tree: numeric values are little-endian; strings and arrays use a
u32 length; lists and compounds contain tagged children; compound keys precede
their values. Compound keys are encoded in sorted order.

NBT output is limited to 1 MiB, depth 64 (root depth 1), 65,536 nodes,
65,536 items per array or container, and 65,536 UTF-8 bytes per string.
Unknown tags, duplicate compound keys, mixed-type lists, malformed strings,
serialization errors, or values over a limit fail without a partial output.
The explicit NBT grant can expose mod-specific persistent data; it is not a
stable substitute for typed capabilities. Lookups do not load chunks. Handles
become stale on block-entity replacement/removal, chunk or world unload, or
module/runtime close. Every operation revalidates the live entity and required
permissions. Native SDK builds report unsupported host imports.

## Named network channels

`sjml.toml` declares each channel under its owning module with a name, positive
u32 `schema_version`, and one or both directions (`client_to_server`,
`server_to_client`). The package, module, and channel name form the identity.
The portable handshake contract compares ABI/protocol versions, required peer
mods and capabilities, and shared channels. Shared-channel schema or direction
set mismatches reject the connection; a missing channel by itself is allowed.

The Rust SDK imports `sjml_network_v1` through the `network` module. Operations
require `network.mod`; references to another module additionally require
`network.external`. `send_to` also requires `entity.read` for its player target.
Direction checks, session checks, negotiated-channel compatibility, and the
1 MiB delivery limit apply to sends and receives.

| Import | Parameters | Behavior |
| --- | --- | --- |
| `subscribe` | `channel_ptr, channel_len, callback_id: i64, out_handle_ptr` | Register a typed callback and write a 24-byte handle |
| `unsubscribe` | `handle_ptr, handle_len` | Remove that runtime subscription |
| `send_to_server` | `channel_ptr, channel_len, payload_ptr, payload_len` | Queue client-to-server message |
| `send_to` | `channel_ptr, channel_len, player_ptr, player_len, payload_ptr, payload_len` | Queue server-to-one-player message |
| `broadcast` | `channel_ptr, channel_len, payload_ptr, payload_len` | Queue server-to-compatible-clients message |
| `reply` | `channel_ptr, channel_len, payload_ptr, payload_len` | Reply to the active callback peer on the reverse direction |

Channel references use schema version 1, a scope byte (`0` local, `1` explicit
owner), optional package/module stable strings, and the channel stable string.
The host dispatches `sjml_v1_network_dispatch(pointer, length) -> i64`; input
contains u32 schema `1`, u64 callback ID, a versioned `NetworkContext`, then
the remaining stable payload bytes. Context contains the channel reference,
direction (`1` client-to-server, `2` server-to-client), receiving logical side
(`1` client, `2` server), and an optional UUID encoded as a tag byte plus 16
network-order bytes. The host must derive a client-to-server sender UUID from
authenticated connection metadata, never from payload data. `reply` authority
exists only during that callback and no connection identifier is exposed.

The SDK exposes closure subscriptions and explicit `send_to_server`,
`send_to`, `broadcast`, and callback-scoped `reply` methods. The NeoForge
activation path binds these imports, registers the static payload envelope,
sends a handshake when a player joins, and tracks authenticated per-connection
sessions. A channel version or direction mismatch disconnects the connection;
missing optional channels remain compatible. Unit-level adapter tests cover
the code paths, but live client/server interoperability has not been verified.
See [NETWORKING.md](NETWORKING.md) for the full guest API and envelope bounds.

## Persistent storage

The Rust SDK's `storage` module uses `sjml_storage_v1` imports to read, write,
remove, and enumerate typed records. On logical-server contexts, `save`,
`dimension`, `player`, `entity`, and `block_entity` scopes remain inside the
current Minecraft save under a `SavedData` namespace for the owning mod. On
physical-client contexts, the explicit `client_profile` scope is local to the
game installation and is stored under `config/sjml/client-storage/<mod-id>.dat`;
it is not a view of a world or remote server save. The host checks
`storage.read` or `storage.write` on each operation. Static registration cannot
access storage, client-profile data is denied in server contexts, and save
scopes are denied in client contexts.

The Rust SDK provides automatic and explicit typed migrations. For exact
storage signatures, scope/envelope layouts, bounds, schema rules, and migration
semantics, see [STORAGE.md](STORAGE.md).

`sjml_storage_v1.write_batch(scope_ptr: i32, scope_len: i32, entries_ptr: i32,
entries_len: i32) -> i32` commits a scope migration's staged records. It
requires `storage.write`, which the host checks on every call. `entries` is a
little-endian `u32` record count followed by that many pairs: each key is a
little-endian `u32` UTF-8 byte length and strict UTF-8 bytes, followed by a
little-endian `u32` complete record-envelope length and the envelope bytes.
The complete entries buffer is limited to 1 MiB, the record count to 65,536,
and each key to 1–256 bytes; duplicate keys are rejected. The host validates
the full batch, record envelopes, and storage quotas before changing the
scope. It applies all entries together or none of them, so an invalid record,
quota failure, or other rejected batch leaves every original record intact.
See [STORAGE.md](STORAGE.md) for the complete wire and migration contract.
