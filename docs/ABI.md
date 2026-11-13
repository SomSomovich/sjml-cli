# SJML Binary ABI v1

This document defines the core wire values implemented by the sjml-core ABI v1.
The scalar and handle layouts are independent of Rust and NeoForge; later
sections describe the current Rust SDK and NeoForge adapter bindings.

## Versioning

The current core ABI version is **1.74.0**. Versions use SemVer. A host supports
a requested ABI when the major versions match and the host version is greater
than or equal to the requested version. A different major version is
incompatible. Build metadata does not affect version ordering. ABI 1.74.0 adds per-state bounded shape overrides for
registered blocks with declared properties, and bounded dynamic suggestions for manifest-declared physical-client
commands. Client suggestions require `client.command.suggest` in addition to `client.command`, run synchronously on the
physical-client main thread, receive only bounded local text and previously parsed arguments, and return at most 64
values of 256 UTF-8 bytes each. ABI 1.73.0 adds bounded Boolean and integer properties to
module-registered block states. ABI 1.72.0 adds static bounded vanilla and modded attribute modifiers to registered items. ABI 1.71.0 adds
an owner-scoped synchronous `item.use_on_entity` callback for registered items.
ABI 1.70.0 adds
an owner-scoped synchronous `item.use_on_block` callback for registered items.
ABI 1.69.0 adds
the owner-scoped, synchronous `block.use` callback for registered blocks. ABI
1.68.0 adds bounded physical behavior settings for registered blocks. ABI 1.67.0 adds fixed
full-cube, empty, and single-cuboid shapes for registered blocks. ABI
1.66.0 adds bounded block, item, fluid, and entity-type tag-member enumeration.
ABI 1.65.0 adds loaded-only block and item tag membership queries. ABI 1.64.0 adds bounded physical-client
translation lookup, loaded-only structure queries, and synchronous cancellable
bonemeal/fertilization attempts. ABI 1.63.0 adds
bounded Brigadier command suggestions, owner-scoped registered-item use
callbacks, package sprite-region rendering, brewing mixes and single-effect
potions, player-inventory enchantment operations, tracked sound fading and
music selection, module-owned scoreboard teams, and synchronous cancellable
`entity.breed_attempt`, `block.grow_attempt`, and `player.fished_attempt`
events. ABI 1.62.0 adds
criterion-level advancement progress reads and single-criterion grant/revoke,
bounded reads and increments for six vanilla registry-backed statistic
families, and host-evaluated periodic keyframe rotation tracks for registered
entity/block-entity models. ABI 1.60.0 adds
owner-scoped handles for stopping exact one-shot client sounds, bounded held-key
and mouse-button state reads, server-side entity riding snapshots and guarded
non-forced mount/dismount calls, the observational `entity.tick` callback, and
loaded-only vanilla neighbor updates. Network inbound rate limits are now
isolated per authenticated player without changing the handshake or payload
format. ABI 1.59.0 adds schema-v4 client settings forms with bounded checkboxes,
physical-client FOV scaling, bounded `player.tick` / `client.tick` subscriptions,
detached world fluid snapshots, module-owned boolean item components, and the
synchronous cancellable `player.item_use_attempt` event. Schema-v1 through
schema-v3 screen requests remain compatible. ABI 1.58.0 adds schema-v3 client
list forms with one bounded selection list
alongside text fields.
ABI 1.57.0 adds bounded schema-v2 text-entry forms to interactive physical-client screens,
with owner-directed `client.screen.form_submit` callbacks. ABI 1.56.0 adds
schema-v1 screens under `client.ui.screen`, with owner-directed
`client.screen.action` callbacks. ABI 1.55.0 adds the
synchronous, cancellable, physical-client `client.chat.send_attempt` event.
It passes bounded original text and requires `client.chat.send.observe` plus
`events.subscribe` and `events.cancel`; it never rewrites signed chat. ABI
1.54.0 adds the synchronous, client-only `client.item.tooltip` event, whose
callback can append bounded literal tooltip lines when separately granted
`client.item.tooltip.modify`. ABI 1.53.0 adds
selected hotbar-slot reads and writes, bounded active-effect snapshots for
loaded living entities, reads and writes for entries in existing scoreboard
objectives, and the synchronous cancellable `player.sleep_attempt` event.
Hotbar writes require `entity.read` and `inventory.write`; effect reads require
`entity.read`; scoreboard reads/writes require `scoreboard.read` and
`scoreboard.write`; sleep vetoes require `events.subscribe` and `events.cancel`.
ABI 1.52.0 adds
loaded-Mob AI enable/disable, bounded player respawn-point read/set/clear,
literal custom names on existing player-inventory stacks, and vanilla
neighboring-redstone-signal reads that use only loaded chunks. AI control
requires `entity.read` and `entity.ai.control`; respawn-point writes require
`entity.read` and `player.respawn.write`; player-stack naming requires
`entity.read` and `inventory.write`; redstone reads require `world.read`.
ABI 1.51.0 adds
server-side advancement grants/revocations and custom-stat reads/increments;
world difficulty and the saved Overworld spawn point; cancellable entity heal,
status-effect application, and join attempts; bounded block-inventory custom
names and item transfers; registered food-consumption effects; per-axis model
animation tracks; and exact client GUI-layer suppression. Their permissions,
limits, and side requirements are documented with each API. ABI 1.50.0 adds
bounded static durability for module-owned standalone items, server-side player
item cooldown reads/writes, non-cancellable earned-advancement notifications,
and synchronous cancellable explosion-start events. Cooldown reads require
`entity.read`, and changes additionally require `player.cooldown.write`; item
durability declarations require `content.register`; advancement notifications
require `events.subscribe`; explosion vetoes require `events.subscribe` and
`events.cancel`. ABI 1.49.0 adds
synchronous cancellable `entity.projectile_impact_attempt`, literal custom-name
reads and writes for loaded non-player entities, bounded registered food
properties, and owner-scoped editable chest menus with allowlisted input/output
slots and conservation-preserving transfers of existing stacks. The projectile
event requires `events.subscribe` and `events.cancel`; entity name reads require
`entity.read` and writes also require `entity.custom_name.write`; item food
declarations require `content.register`; editable-menu reads require
`entity.read` and `menu.open`, while transfers also require `menu.transfer`.
ABI 1.48.0 adds
synchronous cancellable `player.drop_attempt` with inventory-safe stack
restoration, module-owned updates to the title and contents of its currently
open read-only menu, reads and writes for loaded entity glowing/silent flags,
and bounded item enchantment values during static registration. Drop vetoes
require `events.subscribe` and `events.cancel`; menu replacement requires
`entity.read` plus `menu.open` and ownership of the active SJML menu; entity
state reads require `entity.read` and writes also require `entity.state.write`;
enchantment declarations require `content.register`.

ABI 1.47.0 added bounded item-rarity declarations, module-owned
integer-component access on live player inventory stacks, loaded-entity
fire-tick access, synchronous cancellable `block.break_attempt`, observe-only
global key notifications, active-screen key vetoes, bounded module-owned
client-loop sound handles, and server title/subtitle packets. Item and rarity
registration requires `content.register`; player stack components require
`entity.read` plus `inventory.read` or `inventory.write` and a component
registered by the same module; fire reads require `entity.read` and writes also
require `entity.fire.write`; break vetoes require `events.subscribe` and
`events.cancel`; global key observation and screen vetoes use the separate
`client.input.observe` and `client.input.screen_cancel` grants; looping audio
requires `client.audio.loop`; titles require `entity.read` and `player.ui.title`.
ABI 1.46.0 adds registry-backed bounded biome ID enumeration, module-owned
bounded integer components on existing block-inventory stacks, and the
cancellable `entity.target_attempt` event. Biome enumeration requires
`world.read` and does not load chunks; component declarations require
`content.register`, and block-inventory reads/writes require `world.read` plus
`inventory.read` or `inventory.write`; target vetoes require `events.subscribe`
and `events.cancel`.
ABI 1.45.0 adds
a replacement result to local client chat-message callbacks, allowing bounded
literal spans with formatting, hover text, and safe local click actions under
`client.chat.rewrite`. ABI 1.44.0 adds
`sjml_world_v1.set_game_time` for the shared elapsed clock, guarded
`sjml_player_v1.teleport_to_world` to an already-loaded destination chunk,
bounded static villager trade declarations, and the synchronous cancellable
`entity.damage_attempt` callback before damage mitigation. Game-time writes
require `world.read` and `world.write`; cross-dimension teleport requires
`entity.read`, `player.teleport`, and `world.read`; trade declarations require
`content.register` on a server-capable module; damage vetoes require
`events.subscribe` and `events.cancel`. ABI 1.43.0 adds
bounded player-inventory durability writes, already-loaded chunk enumeration,
nearest loaded-entity raycasts, the synchronous cancellable
`entity.death_attempt` event, and manifest-declared physical-client commands.
Durability updates require `entity.read` and `inventory.write`; loaded-chunk
queries require `world.read`; entity raycasts require `entity.read` and
`world.read`; death vetoes require `events.subscribe` and `events.cancel`;
client commands require `client.command` and run on the physical-client main
thread. These additions never load chunks, expose live Minecraft objects, or
grant client commands authority over server state. ABI 1.42.0 adds bounded
player attribute reads and base-value writes, detached item-stack snapshots,
player inventory slot writes, allowlisted integer game-rule access,
property-based block-state updates, loaded-entity spatial queries, and the
synchronous cancellable `block.place_attempt` event. Player attributes require
`entity.read`, with `entity.attributes.write` for writes; item snapshots require
`world.read` and `inventory.read`; player slot writes require `entity.read` and
`inventory.write`; world reads and mutations require `world.read`, with
mutations also requiring `world.write`; spatial queries require `entity.read`
and `world.read`; placement vetoes require `events.subscribe` and
`events.cancel`. ABI 1.41.0 adds
`sjml_world_v1.schedule_block_tick` for guarded one-shot ticks on already-loaded
blocks, `sjml_entity_v1.remove_effect` for removing registered effects from
loaded living entities, a synchronous cancellable `player.interaction` event
for right-click block/entity/item actions, and
`sjml_client_input_v1.cursor_position` for bounded physical-client cursor
coordinates. These use `world.read` plus `world.write`, `entity.read` plus
`entity.effect.remove`, `events.subscribe` plus `events.cancel`, or
`client.input`. The new player interaction callback is synchronous because it
can veto a live Minecraft action. ABI 1.40.0 adds
allowlisted boolean world-gamerule reads and writes, bounded generic damage for
loaded living non-player entities, literal player action-bar messages,
declarative client keybindings, and opt-in active-screen scroll events. Screen
scroll delivery requires the new `client.input.screen_scroll` capability so
older ABI 1.39 guests never receive an unknown callback event tag. These
features use `world.read`/`world.write`, `entity.read`/`entity.damage`,
`entity.read`/`player.ui.actionbar`, or `client.input` as documented below.
ABI 1.39.0 adds
global client mouse-wheel notifications, read-only player inventory snapshots,
bounded reads and base-value writes for six vanilla living-entity attributes,
and loaded block-state property snapshots. These use `client.input`,
`entity.read` plus `inventory.read`, `entity.read` plus
`entity.attributes.write`, and `world.read`, respectively. ABI 1.38.0 adds
read and bounded flight-state changes for server players, ordinary client chat
sending through Minecraft's signed chat path, client-local sound playback, and
read/write access to loaded world-border settings. These additions use
`player.abilities.write`, `client.chat.send`, `client.audio.play`, or the
existing `entity.read`/`world.read`/`world.write` grants. ABI 1.37.0 adds
`sjml_player_v1.equipment` for detached main-hand/offhand/armor snapshots,
`sjml_world_v1.daylight_cycle_enabled` and `set_daylight_cycle_enabled` for the
vanilla `doDaylightCycle` gamerule, a client-local chat-message callback ABI
with independent `client.chat.observe` and `client.chat.filter` capabilities,
and bounded client toast notifications through `sjml_client_ui_v1.notify` under
`client.ui.notify`. These client APIs cannot alter server-authoritative state;
chat filtering only hides local display insertion after Minecraft's secure-chat
checks. ABI 1.36.0 adds
read-only player vitals, bounded server-player game-mode changes, loaded-world
biome and daylight-time queries, daylight-time writes, bounded player motion
reads and velocity writes, smithing-transform recipes, and the non-cancellable
`player.respawned` event. ABI 1.35.0 added bounded entity removal,
same-dimension player teleport, loaded-world weather read/write, and vanilla
stonecutting recipe declarations.

## Guest instruction budget

The core instruments guest WASM functions with a basic-block instruction
counter. Each exported callback receives a fresh budget; the runtime default is
1,000,000 guest instructions per export, while the one-time static
`sjml_v1_mod_register` callback receives a separate 2,000,000-instruction
ceiling. Trusted host code may configure a limit up to 100,000,000. An exhausted
budget traps that callback and follows
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
`sjml_v1_event_player.left`, `sjml_v1_event_player.moved`,
`sjml_v1_event_player.respawned`, `sjml_v1_event_player.tick`,
`sjml_v1_event_entity.death`, and
`sjml_v1_event_entity.spawned`, `sjml_v1_event_entity.effect.tick`,
`sjml_v1_event_entity.tick`,
`sjml_v1_event_entity.projectile_impact_attempt`,
`sjml_v1_event_block.place`, `sjml_v1_event_player.interacted`,
`sjml_v1_event_player.interaction`,
`sjml_v1_event_entity.damaged`, `sjml_v1_event_world.loaded`,
`sjml_v1_event_world.unloaded`, `sjml_v1_event_server.tick`,
`sjml_v1_event_client.tick`,
`sjml_v1_event_inventory.changed`, and `sjml_v1_event_resources.reloaded`
exports. The
player join/leave events receive their SDK schema-v1 identity record
(`PlayerJoined` or `PlayerLeft`): little-endian
version `1`, then a stable UTF-8 string containing the canonical UUID, then a
stable UTF-8 string containing the profile name. `player.moved` receives
schema-v1 `PlayerMoved`: the same identity strings, a stable UTF-8 dimension
ID, finite little-endian `f64` position coordinates, then finite `f32` yaw and
pitch values. All notification callbacks run
only after explicit subscription, synchronously on the owning logical-server
thread. `player.respawned` receives schema-v1 `PlayerRespawned`: little-endian
version `1`, stable strings in order (`player_id` canonical UUID,
`display_name`, and `dimension_id`), finite little-endian `f64` respawn
coordinates `x`, `y`, and `z`, then a stable boolean byte `end_conquered`.
NeoForge dispatches after the player has been placed in the respawned level;
the event cannot alter or cancel respawn. It requires only `events.subscribe`
and runs on the owning logical-server thread. The module must request
`events.subscribe` in its manifest and receive
an administrator grant. Subscriptions run by priority, with equal priorities
preserving module activation order and per-module registration order. Callback
failures disable only the affected module.

The synchronous cancellable `sjml_v1_event_entity.target_attempt` callback
receives schema-v1 `EntityTargetAttempt`: little-endian `u32` version `1`, a
length-prefixed canonical UUID string for the living entity, an optional
requested-target UUID (`u8` tag `0=None`, `1=Some` followed by the string), and
a length-prefixed target-change type string. It is captured from NeoForge's
cancellable `LivingChangeTargetEvent` on the owning server thread. The callback
must return exactly one stable boolean byte and requires both
`events.subscribe` and `events.cancel`; returning true preserves the entity's
existing target, including preserving `None` when a clear-target attempt is
vetoed. Already-cancelled events do not reach guest callbacks. This callback is
synchronous and cannot use `#[event_async]`.

The synchronous cancellable `sjml_v1_event_entity.breed_attempt` callback
receives schema-v1 `EntityBreedAttempt`: little-endian `u32(1)`, followed by
length-prefixed UTF-8 strings `parent_a_id`, `parent_a_type_id`, `parent_b_id`,
`parent_b_type_id`, `child_id`, `child_type_id`, and `dimension_id` in that
order. Entity type and dimension values are canonical lowercase resource
locations. Each string is limited to 256 bytes and the complete payload to
2 KiB. NeoForge's cancellable `BabyEntitySpawnEvent` supplies the proposed
child before it is added to the level. The host dispatches once on the owning
logical-server thread, skips events already canceled by another listener, and
routes only to subscribed owner modules. The callback must return exactly one
stable boolean byte and requires `events.subscribe` plus `events.cancel`;
returning true cancels the proposed child spawn. The callback cannot replace
the child, does not undo the parents' completed breeding interaction, and
cannot use `#[event_async]`.

The synchronous cancellable `sjml_v1_event_entity.projectile_impact_attempt`
callback receives schema-v1 `ProjectileImpactAttempt`: little-endian `u32(1)`;
length-prefixed strings `projectile_id`, `projectile_type_id`, `dimension_id`,
and `hit_kind`; optional strings `hit_entity_id`, `hit_entity_type_id`, and
`hit_block_id`; an optional schema-v1 `BlockPos` (`x`, `y`, `z` as little-endian
`i32`); optional `block_face`; then finite little-endian `f64` hit coordinates
`x`, `y`, and `z`. `hit_kind` is `entity` or `block`, with exactly the matching
target fields present. The host copies this bounded payload while NeoForge's
impact event is active. The callback must return one stable boolean byte and
requires `events.subscribe` and `events.cancel`; `true` cancels the impact.
Block details are omitted if the target chunk is no longer loaded. This
callback is synchronous and cannot use `#[event_async]`.

The non-cancellable `sjml_v1_event_player.advancement_earned` callback receives
schema-v1 `PlayerAdvancementEarned`: little-endian `u32(1)` followed by four
length-prefixed UTF-8 strings in order (`player_id` canonical UUID,
`display_name`, `dimension_id`, and `advancement_id`). The payload is limited to
2 KiB and each string to 256 bytes. It fires after a server player earns an
advancement and requires `events.subscribe`.

The synchronous cancellable `sjml_v1_event_world.explosion_attempt` callback
receives schema-v1 `ExplosionAttempt`: little-endian `u32(1)`, a
length-prefixed `dimension_id`, optional `source_entity_id` and
`source_entity_type_id` strings (each a stable `u8` 0/1 tag followed by a
length-prefixed UTF-8 string when present), a length-prefixed `block_interaction`
string, little-endian finite `f64` center coordinates `x`, `y`, `z`, and finite
`f32 radius`. Source identity fields must be present together; radius is
bounded to 128 and block interaction is one of `keep`, `destroy`,
`destroy_with_decay`, or `trigger_block`. The callback returns one stable
boolean byte; `true` cancels before explosion damage or block/fire changes.
It requires `events.subscribe` and `events.cancel`, runs synchronously on the
owning logical-server thread, and cannot use `#[event_async]`.

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
remains in effect. The synchronous cancellable `sjml_v1_event_block.grow_attempt`
callback receives schema-v1 `BlockGrowAttempt`: little-endian `u32(1)`, a
length-prefixed UTF-8 `dimension_id`, nested schema-v1 `BlockPos` (`i32` `x`,
`y`, and `z`), then a length-prefixed UTF-8 `block_id`. Both identifiers must
be canonical lowercase resource locations, each string is limited to 256
bytes, and the full payload is limited to 2 KiB. It is captured from NeoForge's
logical-server-only `CropGrowEvent.Pre` on the server owner thread. A true
callback result sets `DO_NOT_GROW`; false leaves the event's current result
unchanged, so `DEFAULT` retains vanilla behavior and guest code never forces
growth. It requires `events.subscribe` and `events.cancel` and must use
synchronous `#[event]`. The payload omits crop age because this hook also
covers growing-age blocks with different state properties, including cactus
and chorus plants. The synchronous cancellable
`sjml_v1_event_block.fertilize_attempt` callback receives schema-v1
`BlockFertilizeAttempt`: little-endian `u32(1)`, optional player UUID encoded
as a stable `u8` 0/1 tag followed by a length-prefixed UTF-8 string when
present, length-prefixed `dimension_id`, nested schema-v1 `BlockPos` (`i32`
`x`, `y`, and `z`), length-prefixed `block_id` and `fertilizer_item_id`,
little-endian `u32` fertilizer stack count, and a stable boolean byte for
`valid_target`. IDs must be canonical lowercase resource locations and the
optional actor must be a canonical UUID; strings are limited to 256 bytes, the
count to 0–99, and the full payload to 2 KiB. NeoForge's `BonemealEvent` is
captured on the logical-server owner thread. Returning true invokes
`setSuccessful(false)` and cancels vanilla application without consuming the
fertilizer stack; false leaves the current event result unchanged. Already
canceled events are not delivered. The callback requires `events.subscribe`
and `events.cancel` and must be synchronous. The `entity.death` callback receives a schema-v1
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
The `entity.tick` callback receives schema-v1 `EntityTick`: little-endian
version `1`, stable UTF-8 strings in order (canonical `entity_id`,
`entity_type_id`, `dimension_id`), finite little-endian IEEE-754 `f64`
coordinates `x`, `y`, and `z`, then a non-negative little-endian `i64`
entity-local `tick_count`. It runs after loaded non-player entity ticks on the
owning logical-server thread, is observation-only, and does not cause chunks to
load. Default cadence is 20 entity ticks; `subscribe_interval` accepts 1–1,200.
To bound guest work, at most 4,096 `entity.tick` callbacks are invoked per
logical-server tick; excess callbacks are skipped for that server tick.
The `player.interacted` callback receives schema-v1 `PlayerInteracted` data:
version `1`, stable strings for the canonical `player_id`, `dimension_id`,
`action`, and `hand`, optional stable strings for `target_id` and `face`, then an
optional nested schema-v1 `BlockPos` record. Its `action` identifies the
interaction attempt (for example `right_click_block`,
`left_click_block_start`, `entity_interact`, or `right_click_item`); `hand` is
`main_hand` or `off_hand`. The target is a registry ID when available, and the
position and face are present for block interactions. This observation does
not change or cancel the original action.
The separate synchronous `player.interaction` callback uses the same
schema-v1 field order as `PlayerInteracted` and is delivered before a supported
right-click block, entity, or item-use action is applied. Supported actions
are `right_click_block`, `entity_interact`, `entity_interact_specific`, and
`right_click_item`; item-use targets identify the held item. Its callback must
return exactly one stable boolean byte after the status prefix. `true` vetoes
the original action, while `false` leaves it available to vanilla and other
event handlers. It requires both `events.subscribe` and `events.cancel` and
cannot be asynchronous.

The synchronous `sjml_v1_event_item.use_on_entity` callback is an owner-scoped
registered-item behavior hook. Its schema-v1 payload contains stable strings
for the player UUID, dimension ID, held item ID, target entity UUID, target
entity-type ID, and hand; a little-endian `i32` stack count; and little-endian
`f64` target X/Y/Z coordinates. Strings are limited to 256 UTF-8 bytes,
coordinates must be finite and within ±30,000,000, the count is
`0..=1,000,000`, and the payload is limited to 4 KiB. The host routes the
callback only to the module that declared the held item under its
`package:module/path` namespace. It runs on the logical server after the
general `player.interaction` veto and after the more specific entity
`interactAt` stage has passed, before vanilla entity/item interaction behavior.
Return exactly one action byte: `0=Pass`, `1=Success`, or `2=ConsumeOne`;
invalid results disable the failing module callback. `Pass` continues vanilla
interaction. `Success` handles it. `ConsumeOne` handles it and consumes one
item only if its ID and count remain unchanged. A handled action sets
NeoForge's successful cancellation result. It requires `events.subscribe` and
`events.cancel` and cannot be asynchronous.
The synchronous cancellable `sjml_v1_event_player.drop_attempt` callback
receives schema-v1 `PlayerDropAttempt`: little-endian `u32(1)`, stable
length-prefixed strings for the canonical player UUID, dimension ID, and item
ID, little-endian `u32` stack count, then a strict boolean byte `can_cancel`.
The count is limited to 1–99 and IDs are bounded to 256 UTF-8 bytes. The
callback runs on the owning logical-server thread through NeoForge's
`ItemTossEvent`; it must return exactly one stable boolean byte and requires
`events.subscribe` plus `events.cancel`. A `true` result cancels only if the
host can restore the whole stack into the player's inventory. If capacity is
insufficient, the host preserves the vanilla toss to avoid losing items.
This event cannot be asynchronous.

The non-cancellable `sjml_v1_event_player.item_crafted` callback receives a
schema-v1 `PlayerItemCrafted`: little-endian `u32(1)`, stable length-prefixed
UTF-8 strings for canonical player UUID, matched recipe ID, and output item
ID, followed by little-endian `u32` output count. Each string is limited to
256 UTF-8 bytes, count to `1..=1,000,000`, and the complete payload to 2 KiB.
NeoForge emits it on the owning logical-server thread after a normal crafting
result is taken. It also sends the existing `inventory.changed` notification;
this new event does not expose ingredients, mutate the result, or cancel the
craft. It requires `events.subscribe` and cannot be asynchronous.

The synchronous cancellable `sjml_v1_event_player.item_use_attempt` callback
receives schema-v1 `PlayerItemUseAttempt`: little-endian `u32(1)`, stable
length-prefixed UTF-8 strings for canonical player UUID, dimension ID, hand,
and item registry ID, then little-endian `u32` stack count. Hand is `main_hand`
or `off_hand`; an empty hand is `minecraft:air` with count zero. Count is
limited to `0..=1,000,000` and each string to 256 UTF-8 bytes. NeoForge emits
it on the owning logical-server thread before `Item.use`, after the general
`player.interaction` veto. It has no live stack handle and cannot rewrite the
item or count. Return exactly one stable boolean byte after the status prefix;
`true` cancels the use. It requires `events.subscribe` and `events.cancel`, and
cannot be asynchronous.

The synchronous cancellable `sjml_v1_event_player.fished_attempt` callback
receives schema-v1 `PlayerFishedAttempt`: little-endian `u32(1)`, stable
length-prefixed UTF-8 strings for canonical player UUID and dimension ID,
little-endian `f64` hook X/Y/Z coordinates, `u32` requested rod damage, a
`u32` drop count, then that many nested schema-v1 `FishedDropSnapshot` values
(each has `u32(1)`, a stable item registry ID string, and `u32` count), and a
strict boolean byte indicating whether source drops were truncated. Strings
are limited to 256 UTF-8 bytes, coordinates to finite values within ±30,000,000,
rod damage and each count to `0..=1,000,000` and `1..=1,000,000` respectively,
and the drop list to 16 entries; the complete payload is limited to 8 KiB.
NeoForge emits this only for a logical-server `ItemFishedEvent` on the owning
server thread. Return exactly one stable boolean byte after the status prefix;
`true` cancels delivery of the catch drops, while requested rod damage remains
unchanged. Drop snapshots contain only item IDs and counts, with no components
or NBT. The callback requires `events.subscribe` and `events.cancel` and cannot
be asynchronous.

The synchronous `sjml_v1_event_item.use` callback is an owner-scoped item
behavior hook. Its payload has the same schema-v1 detached fields as
`PlayerItemUseAttempt`: version, player UUID, dimension, hand, canonical item
ID, and count. The host dispatches it only to a module whose declared item ID
uses that package and module path. The response is exactly one stable action
byte: `0=Pass`, `1=Success`, or `2=ConsumeOne`; other values disable the failing
module callback. `Pass` continues normal vanilla use. `Success` handles the
click without changing the stack. `ConsumeOne` handles the click and consumes
one item only if the same held item and count remain after the callback. A
handled result uses NeoForge's successful cancellation result. It runs after
`player.item_use_attempt`, requires `events.subscribe` and `events.cancel`, and
cannot be asynchronous.

The synchronous `sjml_v1_event_block.use` callback is an owner-scoped block
behavior hook. Its schema-v1 payload contains stable strings for the player
UUID, dimension ID, and clicked block ID; little-endian `i32` X/Y/Z block
coordinates; stable strings for the hit face, hand, and held item ID; and a
little-endian `i32` held stack count. Strings are limited to 256 UTF-8 bytes,
the count is `0..=1,000,000`, and the complete payload is limited to 4 KiB.
The host invokes it only for the module that declared the clicked block under
its `package:module/path` namespace. It runs on the logical server after the
generic `player.interaction` veto and before vanilla block use. Return exactly
one action byte: `0=Pass`, `1=Success`, or `2=ConsumeOne`; other values disable
the failing module callback. `Pass` leaves the click for vanilla behavior.
`Success` handles the click. `ConsumeOne` handles it and consumes one item only
if the held item ID and count still match the snapshot. A handled action sets
NeoForge's successful cancellation result. The callback requires
`events.subscribe` and `events.cancel` and cannot be asynchronous.

The synchronous `sjml_v1_event_item.use_on_block` callback is an owner-scoped
registered-item behavior hook. Its schema-v1 payload contains stable strings
for the player UUID, dimension ID, held item ID, clicked block ID, hit face,
and hand; a little-endian `i32` stack count; and little-endian `i32` X/Y/Z
block coordinates. Strings are limited to 256 UTF-8 bytes, the count is
`0..=1,000,000`, and the complete payload is limited to 4 KiB. The host routes
the callback only to the module that declared the held item under its
`package:module/path` namespace. On a right-click block, the host first gives
the registered block's `block.use` callback the chance to handle the click;
this item callback runs only if the block callback passes, before vanilla
`Item.useOn`. Return exactly one action byte: `0=Pass`, `1=Success`, or
`2=ConsumeOne`; invalid results disable the failing module callback. `Pass`
continues normal vanilla block and item behavior. `Success` handles the click.
`ConsumeOne` handles it and consumes one item only if the held item ID and
count still match the snapshot. A handled action sets NeoForge's successful
cancellation result. It requires `events.subscribe` and `events.cancel` and
cannot be asynchronous.

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
The `player.tick` callback receives schema-v1 `PlayerTick`: version `1`, a
stable UTF-8 canonical player UUID, a stable UTF-8 dimension ID, then a
non-negative little-endian `i64` player tick count. NeoForge dispatches after a
live server player's tick on the owning logical-server thread. Its default
cadence is 20 ticks; `subscribe_interval` accepts 1–1,200 ticks per
subscription. This callback cannot cancel or change the player tick.
The `client.tick` callback receives schema-v1 `ClientTick`: version `1`, then
a non-negative little-endian `i64` physical-client tick count. It runs after
the client tick on the client main thread. Its default cadence is 20 ticks;
`subscribe_interval` accepts 1–1,200 ticks. It is observational and cannot
change the client tick.
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
Server event subscriptions are accepted only in integrated-server and
dedicated-server execution contexts. `client.item.tooltip`,
`client.chat.send_attempt`, `client.action_attempt`, and `client.tick` are accepted only in the
physical-client context.
Non-cancellable notification callbacks may use
`#[event_async]`; cancellable callbacks and callbacks that return a value to
Minecraft synchronously, including `client.item.tooltip`, must use `#[event]`.
The owner-directed `client.screen.action` callback is not an event subscription
and does not require `events.subscribe`; it may use `#[event_async]` because it
returns no value to the screen synchronously.

## Event subscription imports (NeoForge adapter)

The SDK's `events::subscribe` and `events::unsubscribe` functions use these
wasm32 imports when `events.subscribe` is both requested by the module and
granted by the administrator:

| Host import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_events_v1.subscribe` | `(event_ptr: i32, event_len: i32, priority: i32, handle_output_ptr: i32) -> i32` | Adds one callback subscription and writes its 24-byte handle to guest memory. |
| `sjml_events_v1.subscribe_interval` | `(event_ptr: i32, event_len: i32, priority: i32, interval_ticks: i32, handle_output_ptr: i32) -> i32` | Adds a bounded-cadence tick subscription and writes its handle. |
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
`subscribe_interval` is valid only for `server.tick`, `player.tick`,
`entity.tick`, and `client.tick`, with an interval from 1 through 1,200 ticks. Plain `subscribe`
uses a 20-tick cadence for each event.

The event name is strict UTF-8, 1–128 bytes, lowercase ASCII letters, digits,
`.`, `_`, or `-`, and starts with a letter or digit. The accepted event names
are `server.started`, `server.stopping`, `player.joined`, `player.left`,
`player.moved`, `player.respawned`, `player.advancement_earned`,
`player.sleep_attempt`, `player.item_use_attempt`, `player.attack_attempt`, `entity.death`, `entity.death_attempt`,
`entity.damage_attempt`, `entity.knockback_attempt`, `entity.spawned`, `entity.join_attempt`,
`entity.effect.tick`, `entity.effect_application`, `entity.target_attempt`,
`entity.projectile_impact_attempt`, `entity.breed_attempt`, `entity.heal_attempt`, `block.break`,
`block.break_attempt`, `block.place`, `block.place_attempt`, `block.grow_attempt`, `block.fertilize_attempt`, `block.fluid_place_attempt`, `player.interacted`,
`player.interaction`, `player.drop_attempt`, `entity.damaged`, `world.loaded`,
`world.unloaded`, `world.explosion_attempt`, `server.tick`,
`inventory.changed`, `resources.reloaded`, `menu.action`, `player.tick`,
`entity.tick`, `client.tick`, `client.item.tooltip`, `player.item_crafted`,
`item.use`, `block.use`, `item.use_on_block`, `item.use_on_entity`, `player.fished_attempt`,
`client.chat.send_attempt`, and `client.action_attempt`.
Subscribing to `block.break`, `block.break_attempt`, `block.place_attempt`,
`player.interaction`, `player.drop_attempt`, `player.sleep_attempt`,
`player.item_use_attempt`, `item.use`, `block.use`, `item.use_on_block`, `item.use_on_entity`,
`client.chat.send_attempt`, `client.action_attempt`,
`entity.death_attempt`, `entity.damage_attempt`, `entity.knockback_attempt`, `entity.heal_attempt`,
`entity.effect_application`, `entity.join_attempt`, `entity.target_attempt`,
`entity.projectile_impact_attempt`, `entity.breed_attempt`, `block.grow_attempt`, `block.fertilize_attempt`,
`block.fluid_place_attempt`, `player.fished_attempt`, `player.attack_attempt`, or `world.explosion_attempt` additionally
requires the module to request and receive the separate `events.cancel`
capability. `client.action_attempt` additionally requires the separate
`client.input.action_cancel` capability. Static registration instances cannot subscribe. Server event
callbacks run synchronously on the owning logical-server thread. Non-cancellable
notifications may use the SDK's `#[event_async(name = "...")]` macro: its
generated synchronous ABI export admits a guest task and returns without
waiting for the async body. The client tooltip callback instead runs on the
client main thread and returns its value synchronously; it cannot use
`#[event_async]`.
Task-admission and async handler errors are logged; they do not affect the
already-delivered notification. These tasks use the existing bounded
`sjml_async_v1.wake`/poll runtime and inherit its task quota, timeout,
cancellation, and module-shutdown behavior. The async macro rejects all
cancellable events and result-bearing client events such as
`client.item.tooltip`.

### Client item tooltip callback

The guest export is `sjml_v1_event_client.item.tooltip`. It is available only
to physical-client module instances subscribed to `client.item.tooltip`.
Subscriptions still require `events.subscribe`; callbacks are invoked only
when the same module also requested and received `client.item.tooltip.modify`.
The callback executes synchronously on the client main thread.

Its schema-v1 input is little-endian: `u32 schema_version` (`1`), a stable
UTF-8 string `item_id`, then `u32 count`, `u32 damage`, and `u32 max_damage`.
The item ID is at most 256 UTF-8 bytes; count is 1 through 1,000,000; damage values
are at most 1,000,000. The host supplies only registry ID and stack durability
metadata, not an `ItemStack`, components, or NBT.

The callback returns the stable-encoded `ItemTooltipLines` record: `u32
schema_version` (`1`), `u32 line_count`, then that many stable UTF-8 strings.
At most 16 lines are allowed per callback, each at most 256 bytes, with at
most 2,048 combined bytes. Strings must be non-empty and contain no control
characters. The host caps one tooltip at 64 callbacks, 64 lines, and 8,192
combined output bytes. Returned strings become literal tooltip components
appended after vanilla content; they cannot carry style or interaction data.

### Client outgoing chat veto

The guest export is `sjml_v1_event_client.chat.send_attempt`; it runs only in a
physical-client module. The callback receives the original text from NeoForge's
cancellable `ClientChatEvent`, encoded as little-endian `u32 schema_version`
(`1`) followed by one stable UTF-8 string. The message is limited to 32,768
bytes and control characters are rejected. Guest code returns one stable
boolean byte: `1` vetoes sending, `0` allows it.

Subscribing requires `events.subscribe`, `events.cancel`, and the separate
`client.chat.send.observe` grant because the payload contains user-typed text.
The hook never replaces the message value, so Minecraft retains its ordinary
signed-chat route when the user sends it. A veto happens before it is sent.
Callbacks run on the client main thread and are synchronous; at most 64
callbacks are invoked for one send attempt. A nested chat send from a guest
callback is vetoed to prevent recursion and a callback bypass.

### Client vanilla action veto

The guest export is `sjml_v1_event_client.action_attempt`. It is available
only to physical-client modules and receives the fixed schema-v1 record:
little-endian `u32 schema_version` (`1`), `u32 action`, and `u32 hand`. Action
codes are `0=attack`, `1=use_item`, and `2=pick_block`; hand codes are
`0=main` and `1=off`. NeoForge fires use-item for each hand; attack and
pick-block normally report main hand. No key code, keymapping, device, player,
world, or item object is included.

The `#[event]` callback returns one stable boolean byte: `1` cancels that
vanilla action mapping, while `0` leaves it available to vanilla. It runs
synchronously on the physical-client main thread, before vanilla handles the
mapping. The host does not invoke callbacks if another mod already cancelled
the event and does not change NeoForge's hand-swing setting. Subscriptions
require `events.subscribe`, `events.cancel`, and the separately requested and
granted `client.input.action_cancel` capability. This event does not require
`client.input`. At most 64 subscribed handlers run per action attempt; a
malformed result or guest trap closes that module and does not itself cancel
the action. It cannot use `#[event_async]`.

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
| `sjml_registry_v1.set_item_enchantment_value` | `(item_path_ptr: i32, item_path_len: i32, value: i32) -> i32` | Set vanilla enchantability from 0 to 255 on a previously declared item owned by the same module. |
| `sjml_registry_v1.set_item_attribute_modifier` | `(item_path_ptr: i32, item_path_len: i32, attribute_id_ptr: i32, attribute_id_len: i32, modifier_path_ptr: i32, modifier_path_len: i32, amount_bits: i32, operation: i32, slot: i32) -> i32` | Add one static vanilla or modded item attribute modifier to an item declared by the same module. Attribute IDs are canonical registry IDs, modifier paths are package/module scoped, amounts are finite `-1024..=1024`, operation codes are `0 add_value`, `1 add_multiplied_base`, `2 add_multiplied_total`, and slot codes are `0 any`, `1 mainhand`, `2 offhand`, `3 hand`, `4 feet`, `5 legs`, `6 chest`, `7 head`, `8 armor`, `9 body`; each item accepts at most 16 entries. |
| `sjml_registry_v1.set_item_food` | `(item_path_ptr: i32, item_path_len: i32, nutrition: i32, saturation_modifier_bits: i32, flags: i32) -> i32` | Set nutrition 1–20, finite saturation modifier 0–10, and vanilla `fast` (bit 0) / `always_eat` (bit 1) flags on a previously declared item owned by the same module. |
| `sjml_registry_v1.set_item_max_damage` | `(item_path_ptr: i32, item_path_len: i32, max_damage: i32) -> i32` | Set vanilla maximum durability from 1 through 1,000,000 on a same-module standalone item declared with stack size 1. |
| `sjml_registry_v1.register_block` | `(path_ptr: i32, path_len: i32, strength_bits: i32, resistance_bits: i32, requires_tool: i32, has_item: i32) -> i32` | Stage a basic block and optionally its `BlockItem`. |
| `sjml_registry_v1.set_block_shape` | `(path_ptr: i32, path_len: i32, kind: i32, min_x_bits: i32, min_y_bits: i32, min_z_bits: i32, max_x_bits: i32, max_y_bits: i32, max_z_bits: i32) -> i32` | Set a fixed full-cube (`kind=0`), empty (`kind=1`), or normalized cuboid (`kind=2`) selection, collision, occlusion, and support shape on a block already staged by this module. Non-cuboid coordinates must be zero; cuboid floats must be finite and ordered within 0..=1. |
| `sjml_registry_v1.set_block_behavior` | `(path_ptr: i32, path_len: i32, friction_bits: i32, speed_factor_bits: i32, jump_factor_bits: i32, light_level: i32, flags: i32, push_reaction: i32, sound: i32) -> i32` | Set bounded friction (`0..=2`), speed/jump multipliers (`0..=4`), emitted light (`0..=15`), optional lava ignition (flag bit 0), piston reaction (`0 normal`, `1 destroy`, `2 block`, `3 ignore`, `4 push_only`), and an allowlisted sound group (`0 stone`, `1 wood`, `2 gravel`, `3 grass`, `4 metal`, `5 glass`, `6 sand`, `7 wool`, `8 snow`, `9 slime`, `10 honey`, `11 netherite`, `12 amethyst`) on a block already staged by this module. Float fields are IEEE-754 bits and must be finite. Reserved flag bits are rejected. |
| `sjml_registry_v1.register_block_entity` | `(path_ptr: i32, path_len: i32, block_path_ptr: i32, block_path_len: i32) -> i32` | Stage a generic block entity type for a block declared by the same module. |
| `sjml_registry_v1.register_entity_model` | `(path_ptr: i32, path_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded colored model for an entity declared by the same module, with optional model-wide and per-cuboid sine rotations. |
| `sjml_registry_v1.register_block_entity_model` | `(path_ptr: i32, path_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded colored model for a block entity declared by the same module, with optional model-wide and per-cuboid sine rotations. |
| `sjml_registry_v1.register_entity_textured_model` | `(path_ptr: i32, path_len: i32, texture_ptr: i32, texture_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded entity cuboid model using a PNG resource in the owning package namespace; the model may include host-evaluated animation. |
| `sjml_registry_v1.register_block_entity_textured_model` | `(path_ptr: i32, path_len: i32, texture_ptr: i32, texture_len: i32, model_ptr: i32, model_len: i32) -> i32` | Stage a bounded block-entity cuboid model using a PNG resource in the owning package namespace; the model may include host-evaluated animation. |
| `sjml_registry_v1.register_model_keyframe_track` | `(path_ptr: i32, path_len: i32, track_ptr: i32, track_len: i32) -> i32` | Stage one bounded host-evaluated periodic keyframe rotation on a previously registered same-module entity or block-entity model/cuboid. |
| `sjml_registry_v1.register_entity` | `(path_ptr: i32, path_len: i32, width_bits: i32, height_bits: i32, tracking_range: i32, update_interval: i32, fire_immune: i32) -> i32` | Stage a generic living `MobCategory.MISC` entity type. |
| `sjml_registry_v1.register_entity_ai_goal` | `(path_ptr: i32, path_len: i32, goal: i32, priority: i32, value_bits: i32, flags: i32) -> i32` | Add one bounded vanilla goal to an entity declared by the same module. |
| `sjml_registry_v1.register_entity_ai_goal_v2` | `(path_ptr: i32, path_len: i32, goal: i32, priority: i32, range_bits: i32, walk_speed_bits: i32, sprint_speed_bits: i32, flags: i32) -> i32` | Add the bounded AvoidPlayer goal to an entity declared by the same module. |
| `sjml_registry_v1.register_fluid` | `(path_ptr: i32, path_len: i32, tick_rate: i32, level_decrease_per_block: i32, slope_find_distance: i32, explosion_resistance_bits: i32) -> i32` | Stage a flowing fluid pair, liquid block, and bucket with vanilla water visuals. |
| `sjml_registry_v1.register_fluid_with_visuals` | `(path_ptr: i32, path_len: i32, tick_rate: i32, level_decrease_per_block: i32, slope_find_distance: i32, explosion_resistance_bits: i32, still_texture_ptr: i32, still_texture_len: i32, flowing_texture_ptr: i32, flowing_texture_len: i32) -> i32` | Stage a fluid with package-owned `block/` still and flowing texture IDs. |
| `sjml_registry_v1.register_shapeless_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, ingredient_lines_ptr: i32, ingredient_lines_len: i32) -> i32` | Stage a bounded shapeless recipe with one to nine item or item-tag ingredients. |
| `sjml_registry_v1.register_shaped_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, pattern_ptr: i32, pattern_len: i32, key_pairs_ptr: i32, key_pairs_len: i32) -> i32` | Stage a bounded 1–3 by 1–3 shaped recipe with uppercase-symbol item or item-tag bindings. |
| `sjml_registry_v1.register_cooking_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, ingredient_token_ptr: i32, ingredient_token_len: i32, kind: i32, experience_bits: i32, cooking_time: i32) -> i32` | Stage a vanilla cooking recipe with an item or item-tag input. |
| `sjml_registry_v1.register_stonecutting_recipe` | `(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32, ingredient_token_ptr: i32, ingredient_token_len: i32) -> i32` | Stage one vanilla stonecutting recipe with one item or item-tag input. |
| `sjml_registry_v1.register_smithing_transform_recipe` | `(path_ptr: i32, path_len: i32, template_ptr: i32, template_len: i32, base_ptr: i32, base_len: i32, addition_ptr: i32, addition_len: i32, output_item_ptr: i32, output_item_len: i32, output_count: i32) -> i32` | Stage a vanilla smithing transform with template, base, and addition item or item-tag inputs. |
| `sjml_registry_v1.register_brewing_recipe` | `(path_ptr: i32, path_len: i32, input_potion_ptr: i32, input_potion_len: i32, ingredient_item_ptr: i32, ingredient_item_len: i32, output_potion_ptr: i32, output_potion_len: i32) -> i32` | Stage a vanilla brewing mix from an existing input potion and item to an existing output potion. |
| `sjml_registry_v1.register_potion` | `(path_ptr: i32, path_len: i32, effect_id_ptr: i32, effect_id_len: i32, duration_ticks: i32, amplifier: i32) -> i32` | Stage a package-owned potion containing one bounded instance of an existing mob effect. |
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
| `sjml_registry_v1.register_integer_component` | `(path_ptr: i32, path_len: i32, minimum: i32, maximum: i32) -> i32` | Register a module-owned integer component with inclusive bounds. |
| `sjml_registry_v1.set_item_integer_component` | `(item_path_ptr: i32, item_path_len: i32, component_path_ptr: i32, component_path_len: i32, value: i32) -> i32` | Set an in-range static integer-component default on a previously declared item or block item. |
| `sjml_registry_v1.register_boolean_component` | `(path_ptr: i32, path_len: i32) -> i32` | Register a module-owned persistent/network-synchronized boolean component type. |
| `sjml_registry_v1.set_item_boolean_component` | `(item_path_ptr: i32, item_path_len: i32, component_path_ptr: i32, component_path_len: i32, value: i32) -> i32` | Set a boolean component default on a previously declared item or block item; value must be 0 or 1. |

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
player, `6` retaliate, `7` water-avoiding random stroll, `8` panic, `9`
random swimming, and `10` avoid player. The v1 import carries tags 0–9:
`value_bits` is finite speed 0.01–8 for tags 1, 3, 7, 8, and 9; finite
range 1–64 for tag 2; and exactly positive zero for tags 0, 4–6. `flags` is
zero except tags 3 and 5, where 0/1 select melee long-memory or player-target
must-see behavior. Random swimming uses a fixed vanilla target-selection
interval of 120 ticks. The v2 import carries only tag 10: range is finite and
1–32, walk and sprint speed are finite and 0.01–8, and sprint speed must be at
least the walk speed; `flags` must be zero. Both imports require
`content.register` during `mod_register`. LookAtPlayer always targets players,
and AvoidPlayer always avoids players; guest code cannot provide Java classes.
Invalid tags, parameters, entity ownership,
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
`register_model_keyframe_track` takes a schema-v1 byte record: little-endian
u32 schema (`1`), target kind (`0=entity`, `1=block entity`), cuboid index,
axis (`0=X`, `1=Y`, `2=Z`), three f32 pivot coordinates, u32 loop duration,
u32 frame count, then 2–32 pairs of `u32 tick` and `f32 angle_degrees`. Use
cuboid index `0xffffffff` for the whole model. Frame zero is required; later
ticks must increase and stay below the 2–12,000 tick loop duration. The host
linearly interpolates and wraps the last frame to the first. Each model target
permits one track per axis across sine and keyframe tracks; registration is
capped at 512 tracks/8,192 frames per module and 8,192 tracks/65,536 frames
total. The type and cuboid index must belong to the same module. Rendering
performs no guest call.
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
Shapeless, shaped, cooking, stonecutting, and smithing-transform recipes are emitted into an always-enabled generated
server data pack and decoded by Minecraft's vanilla
`minecraft:crafting_shapeless`, `minecraft:crafting_shaped`, `minecraft:smelting`,
`minecraft:blasting`, `minecraft:smoking`, `minecraft:campfire_cooking`,
`minecraft:stonecutting`, and `minecraft:smithing_transform` serializers. Stonecutting takes exactly one item or
item-tag ingredient; its path is 1–128 lowercase ASCII content-path characters,
item and encoded ingredient IDs are at most 256 UTF-8 bytes (including `#` for
a tag), and output count is 1–64. Vanilla recipe IDs must be unique across
shapeless, shaped, cooking, stonecutting, and smithing families. Smithing
transform recipes accept one template, one base, and one addition item or tag.
The host limits recipe
declarations to 256 aggregate content entries per module and 4,096 recipes
globally. See [stonecutting recipes](STONECUTTING.md),
[smithing transform recipes](SMITHING_TRANSFORM.md), and
[brewing recipes](BREWING_RECIPES.md) for their Rust helpers and input
conventions. Brewing mixes are applied through NeoForge's `RegisterBrewingRecipesEvent`; module-owned potions are registered with one
existing mob effect before the brewing registry phase.
The path, output-item ID, and ingredient token are passed as separate UTF-8
pointer/length spans; a tag token includes its leading `#`. The declaration
requires requested and granted `content.register` and is accepted only during
`sjml_v1_mod_register`; using it outside that phase returns
`PERMISSION_DENIED`.
Ingredient tokens are ASCII resource IDs: an item is written as `namespace:path`
or `path`, and an item tag is written as `#namespace:path` or `#path`. An
unqualified path resolves inside the declaring package and module; a namespaced
ID can refer to vanilla or another mod. Shapeless recipes accept one to nine
ingredient tokens. Shaped patterns use one to three equal-width rows of one to
three uppercase-letter or space cells, with one item or tag binding for every
used letter and no unused keys. Both families accept an output count of one to
64. Cooking kind codes are `0=smelting`, `1=blasting`, `2=smoking`, and
`3=campfire cooking`. Cooking recipes use one item or tag input, experience
from 0 to 1,000, and cooking time from 1 to 36,000 ticks. Item-output recipe
families share the output-count range of one to 64. Brewing declarations use
fully qualified potion and item IDs; potion declarations use an effect duration
of 1–72,000 ticks and amplifier 0–255. Arbitrary Java recipe serializers are
not provided. Guest-defined WASM serializers are registered
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
live player handle. `sjml_menu_v1.update` replaces the title and content only if
that player still has a menu owned by the calling runtime module:

| Import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_menu_v1.open` | `(player_ptr: i32, player_len: i32, menu_ptr: i32, menu_len: i32) -> i32` | Opens a bounded snapshot menu for that login session. |
| `sjml_menu_v1.update` | `(player_ptr: i32, player_len: i32, menu_ptr: i32, menu_len: i32) -> i32` | Replaces a currently open menu only when owned by the calling module. |

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
Open and update are unavailable from static `mod_register` and client-only
module contexts. Updates require the active screen to be this module's menu and
are limited to eight replacements per module per server tick.

## Editable menu imports

`sjml_editable_menu_v1.open`, `read_slot`, and `transfer` expose a distinct
server-authoritative container menu. Open/read require `entity.read` and
`menu.open`; transfers additionally require `menu.transfer`.

| Import | WebAssembly signature | Behavior |
| --- | --- | --- |
| `sjml_editable_menu_v1.open` | `(player_ptr: i32, player_len: i32, schema_ptr: i32, schema_len: i32) -> i32` | Opens an owner-scoped menu with empty declared input/output slots. |
| `sjml_editable_menu_v1.read_slot` | `(player_ptr: i32, player_len: i32, slot: i32, output_ptr: i32, output_capacity: i32) -> i32` | Writes an optional detached item-ID/count snapshot for a declared slot. |
| `sjml_editable_menu_v1.transfer` | `(player_ptr: i32, player_len: i32, source_kind: i32, source_slot: i32, destination_kind: i32, destination_slot: i32, count: i32) -> i32` | Moves 1–64 existing items between a declared menu slot and a player main-inventory/hotbar slot, or between two declared menu slots. |

The schema is at most 16 KiB and starts with little-endian `u32(1)` version,
`u32` rows (1–6), a length-prefixed UTF-8 title (up to 256 bytes), and a `u32`
slot count (1–54). Each slot has `u32` index, `u32` role (`0=input`,
`1=output`), `u32` item-ID count, and that many canonical length-prefixed
registry IDs. Input slots require 1–32 unique IDs; output slots require zero.
Slot indexes must be unique and fit the declared rows. `read_slot` writes a
schema-v1 optional `EditableMenuStack` containing one registry ID and count.
Transfer endpoint kind is `0=player inventory` (slot 0–35) or `1=menu` (a
declared slot). A move validates both stacks, slot roles/allowlists, full item
and component identity, and destination capacity before changing either side;
it never creates or transforms items. Each module may transfer at most 32
times per server tick. Closing the menu returns remaining stacks to inventory
and drops only exact overflow remainders. Calls require a live player handle
and the owning logical-server thread; another module's or a vanilla menu cannot
be read or changed.

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
128 combined server and client commands, and a command may declare up to 16
arguments; the host stages at most 8,192 commands across all packages.
Client-only modules cannot declare server commands. Local client-command
declarations and their permissions are described in `FORMAT.md`.

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

### Dynamic argument suggestions

An argument entry may set `suggestions = true`. The module must request and
receive `command.suggest` in addition to `content.register`; the command's
`permission_level` is checked before calling the guest. All suggested
arguments of one command use export
`sjml_v1_command_suggestions_<command-name>`, generated by
`#[command_suggestions(name = "...")]`. The host requires this export to have
the normal byte-callback ABI before staging the package. A callback is invoked
synchronously on the owning server thread; recursive calls to the same
callback are suppressed.

The input is `CommandSuggestionRequest` schema version 1. Fields are encoded
in this order: `u32(1)` schema, a length-prefixed command name, a stable
`Option<String>` source player UUID (`u8(0)` or `u8(1)` followed by a string),
`u32` zero-based argument index, a length-prefixed argument name, a
length-prefixed partial argument, then a stable `Vec<CommandSuggestionArgument>`.
The vector count is `u32`; every nested argument record starts with `u32(1)`,
then contains its name, kind (`word`, `string`, `greedy_string`, `integer`, or
`boolean`), and canonical textual value as length-prefixed UTF-8 strings.
Console suggestions have no source-player UUID. The whole callback input is
limited to 1 MiB and each string is limited to 65,536 bytes.

The result is `CommandSuggestions` schema version 1: `u32(1)`, `u32` count,
then that many length-prefixed strings. It accepts at most 64 unique, nonempty
values, each at most 256 UTF-8 bytes, with no control characters. The host
rejects malformed records, filters nonmatching partial text, and verifies
values against the declared parser and integer bounds before passing them to
Brigadier. A malformed callback disables only its module instance and returns
no suggestions.

### Physical-client command suggestions

A client command argument can also set `suggestions = true`; the module must
request and receive `client.command.suggest` in addition to `client.command`.
The host requires the normal byte-callback export
`sjml_v1_client_command_suggestions_<command-name>` before staging the package.
For one Brigadier request, only the selected module/command callback runs,
synchronously on the physical-client main thread. Its SDK callback uses
`#[client_command_suggestions(name = "...")]`.

The input is `ClientCommandSuggestionRequest` schema version 1 and encodes,
in order: `u32(1)`, the length-prefixed command name, the `u32` argument index,
the length-prefixed argument name, length-prefixed partial text, then a stable
`Vec<CommandSuggestionArgument>`. The client request intentionally has no
server command source, player identity, world, or live game object. Partial
text is limited to 256 UTF-8 bytes. Prior arguments are detached canonical
strings, with the same nested layout as server suggestions. The response uses
the existing `CommandSuggestions` schema: at most 64 unique, nonempty values
of up to 256 UTF-8 bytes, with no control characters. The host filters by the
partial text and validates values against the manifest argument type and
integer bounds. A callback trap, error, or malformed result disables only its
client module instance and returns no suggestions.

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

Client-screen click, key, and character callbacks use `client.input`, requested
separately from `client.render`. Mouse-wheel delivery also requires the
separate `client.input.screen_scroll` opt-in. The `sjml_screen_input_v1` import functions are
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
Tag `2` is a typed UTF-16 character unit and modifiers `u8` (16 bytes total).
Tag `3` is a scroll event with cursor X/Y as `f32` and horizontal/vertical
scroll deltas as `f64` (37 bytes total); this tag is sent only to modules that
request and receive `client.input.screen_scroll`. Scroll deltas are finite,
each within ±64, and cannot both be zero; fractional touchpad values are preserved. Mouse
events report Shift, Control, and Alt. A module may hold 256 active input
subscriptions. See [active-screen scrolling](SCREEN_INPUT_SCROLL.md).

## Native client chat imports

ABI 1.34.0 adds client-only `sjml_client_chat_v1` imports. Each operation
requires requested and currently granted `client.chat.modify`, the client main
thread, and runtime phase. Static registration and dedicated-server instances
fail closed. Only the active exclusive controller can query or update geometry.

| Import | WASM signature |
| --- | --- |
| `subscribe` | `(i64 callback_id, i32 output_handle_ptr) -> i32` |
| `unsubscribe` | `(i32 handle_ptr, i32 handle_len) -> i32` |
| `layout` | `(i32 handle_ptr, i32 handle_len, i32 output_layout_ptr) -> i32` |
| `set_layout` | `(i32 handle_ptr, i32 handle_len, i32 layout_ptr, i32 layout_len, i32 output_layout_ptr) -> i32` |
| `get_theme` | `(i32 handle_ptr, i32 handle_len, i32 output_theme_ptr) -> i32` |
| `set_theme` | `(i32 handle_ptr, i32 handle_len, i32 theme_ptr, i32 theme_len, i32 output_theme_ptr) -> i32` |

Zero status means success; nonzero is a stable ABI error. Subscription output
is the existing 24-byte ResourceHandle encoding with handle kind `13`.
Handle inputs must be exactly 24 bytes. Layout inputs and outputs are exactly
16 bytes: four little-endian IEEE-754 `f32` values in order `x,y,width,height`.
Coordinates must be finite and dimensions finite positive. The host returns a
rectangle clamped to the viewport and a 120×56 minimum reduced to fit smaller
viewports. A competing controller returns `REGISTRATION_LIMIT_REACHED` (34).
Runtime closure or unsubscription invalidates the controller and callback route.

Theme inputs and outputs use a fixed 92-byte little-endian representation:
schema `u32=1`, five ARGB `u32` values (panel, header, accent, separator,
title), title byte length `u32`, then a 64-byte UTF-8 title field padded with
zeroes. The title is limited to 64 bytes and control characters are rejected.
Invalid schema, UTF-8, size, or nonzero padding returns `INVALID_TRANSFER`.

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

## Client chat-message observation, filtering, and replacement

ABI 1.37.0 adds `sjml_client_chat_messages_v1`. `subscribe` and `unsubscribe`
require `client.chat.observe`, a physical-client runtime, and the client main
thread. Each module runtime may register one callback; the host supports up to
128 active runtimes and removes each route on unsubscribe, failure, or runtime
close. The callback receives a bounded plain-text snapshot immediately before
NeoForge inserts a message into the local chat. Minecraft's signature, blocked
player, and secure-chat filtering have already run. The guest receives no
signature, style, hover/click event, or Minecraft object. Text is limited to
4,096 UTF-8 bytes and truncated at a code-point boundary.

The guest exports `sjml_v1_client_chat_message_dispatch(pointer, length) ->
i64`; its input is schema `u32=1`, callback ID `u64`, source `u8` (`0=player
chat`, `1=system`), truncation `u8`, and a length-prefixed UTF-8 string. It
returns a stable action byte: `0=Show`, `1=Hide`, or `2=Replace`. Show and Hide
retain the original one-byte ABI encoding. Hide suppresses only this local
display and is honored only when the module separately requests and receives
`client.chat.filter`. Replace requires the separate `client.chat.rewrite` grant
and is followed by a `u32` span count and stable-encoded spans. All strings are
strict UTF-8 with a little-endian `u32` byte length. Each span is encoded in
this order: text string, optional RGB `u32` color (`u32` values above
`0xFFFFFF` are invalid), `u8` style flags, optional hover string, and optional
click action. Optional values use a `u8` tag (`0=none`, `1=present`); present
values immediately follow the tag. Style bits are `1=bold`, `2=italic`,
`4=underline`, `8=strikethrough`, and `16=obfuscated`; all other bits are
invalid. A present click action is `u8` kind plus string: `1=OPEN_URL`,
`2=SUGGEST_COMMAND`, or `3=COPY_TO_CLIPBOARD`. URL values must be valid HTTP(S)
URIs without user information, and Minecraft retains its normal URL
confirmation. Replacements are limited to 64 spans, 4,096 aggregate UTF-8 text
bytes, and 8,192 aggregate hover/click bytes; each hover/click value is at most
512 bytes. The complete callback response is capped at 16 KiB. Click actions
can suggest text in chat input or copy text to the clipboard; command execution
and file opening are not supported.
Command execution and file opening are not supported. Callbacks run in
registration order; later callbacks see the latest replacement as plain text,
and the last authorized replacement supplies the displayed spans. The host
never rewrites or resends signed content or changes server state. See
[client chat messages](CLIENT_CHAT_MESSAGES.md).

The independent `sjml_client_chat_send_v1.send(message_ptr, message_len) -> i32`
import sends one ordinary player chat message through the connected local
player's standard `sendChat` route. It requires `client.chat.send`, a physical
client, and the client main thread. The host keeps Minecraft and NeoForge's
normal signing, send hooks, and acknowledgement handling. The guest cannot send
raw packets or dispatch commands through this import. Text must be nonblank,
single-line, free of controls, not start with `/` after leading whitespace,
at most 256 UTF-16 code units and 1,024 UTF-8 bytes. A module may send once per
second. See [client chat messages](CLIENT_CHAT_MESSAGES.md).

## Interactive client screens

ABI 1.56.0 adds `sjml_client_ui_v1.open_screen(payload_ptr, payload_len) -> i32`.
It requires the declared and administrator-granted `client.ui.screen`
capability, `ExecutionContext.CLIENT`, a physical client, and the client main
thread. The import is denied during static registration and in server contexts.
Only one screen per runtime module may be open; the module may replace its own
screen. If another screen owns the display, the call returns `SERVICE_BUSY`.
Module teardown closes only that module's current screen.

The exact payload is little-endian schema-v1 data:

1. `u32 schema_version` (`1`)
2. stable UTF-8 string `screen_id`
3. stable UTF-8 string `title`
4. `u32 body_line_count`, then that many stable UTF-8 body strings
5. `u32 button_count`, then each stable UTF-8 `button_id` and `label`

The complete payload is at most 4,096 bytes. IDs contain 1–64 lowercase ASCII
letters, digits, or `_.:/-`; button IDs must be unique. The title is 1–128
bytes, there are zero to six body lines of up to 160 bytes each, and there are
one to four buttons with labels of 1–96 bytes. Text must be strict UTF-8 and
cannot contain control characters; empty body lines are allowed. All text is
literal and is rendered by a bounded vanilla `Screen` with ordinary buttons.

ABI 1.57.0 adds schema-v2 form requests to the same `open_screen` import. The
schema-v1 fields remain unchanged; schema v2 appends `u32 field_count` (1–4),
then each text field's stable UTF-8 `field_id`, literal `label`, initial
`value`, and `u32 max_utf8_bytes`. Field IDs are 1–64 lowercase ASCII bytes,
unique among themselves and distinct from button IDs. Labels are 1–96 bytes;
initial values may be empty but cannot exceed the configured 1–128 byte limit.
Values and labels must be strict UTF-8 without control characters, and the
whole request remains capped at 4,096 bytes.

Schema v3 adds a scrollable selection list while keeping the schema-v1 screen
prefix and schema-v2 text-field layout. It appends `u32 field_count` (0–4) and
the bounded text-field entries, then `u32 list_count` (exactly `1`), a stable
list ID, literal list label, and initial selected ID (empty when there is no
initial selection). It then carries `u32 option_count` (1–32) followed by each
option ID and label. Field and list IDs are unique and do not match button IDs;
option IDs are unique, and an initial selection must match an option. List IDs
and option IDs use the 1–64 lowercase ASCII identifier syntax. Labels are
1–96 UTF-8 bytes; all text remains strict UTF-8 without control characters,
and the total payload remains capped at 4,096 bytes.

ABI 1.59.0 adds schema-v4 settings forms. Schema v4 keeps the schema-v3 text
field and selection-list layout, then appends `u32 checkbox_count` (1–4),
followed by each stable UTF-8 checkbox ID, literal label, and initial checked
value as `u32` (`0` or `1`). Checkbox IDs are unique across buttons and all
form elements. Labels are 1–96 bytes; text stays strict UTF-8 without control
characters and the complete request remains capped at 4,096 bytes. Schema-v1,
v2, and v3 requests keep their previous encodings.

When a button is clicked, the screen closes and the owning module receives the
direct callback export `sjml_v1_event_client.screen.action`. Its schema-v1
payload is `u32 schema_version` (`1`), stable UTF-8 `screen_id`, and stable
UTF-8 `button_id`; the callback must return the unit value. This callback is
not routed through `events.subscribe`. The host checks for this export before
opening the screen and returns `INVALID_ARGUMENT` if it is missing. Escape or
external screen closure does not emit a button action. The callback can open
another owner screen after the first screen closes. `Screen` does not pause
integrated-server ticks. See
[interactive client screens](CLIENT_SCREEN.md).

When a schema-v2 form or schema-v3 list-form button is clicked, the host closes
the form and invokes
the owner's direct export `sjml_v1_event_client.screen.form_submit`. Its
schema-v1 payload is `u32 schema_version` (`1`), `screen_id`, `button_id`,
`u32 field_count`, then each field value as a schema-v1 `ClientScreenFormValue`
record containing `field_id` and current `value` as stable UTF-8 strings in
declaration order. For schema-v3, the values include each text field followed
by one record whose ID is the selection-list ID and whose value is the selected
option ID (or an empty string when none is selected). Changing the selection
does not call the guest; the list scrolls locally and submits only when a form
button is clicked. Every button, including one labelled `Cancel`, submits the
current values. Escape and external closure send no callback. The form callback
must be exported before the form is opened; it is not routed via
`events.subscribe`.

If the body exceeds its visible viewport, the client screen clamps wheel
scrolling to the available body text. This is local UI behavior and generates
no callback or network traffic. The screen does not consume wheel input over
form fields, buttons, or outside the body viewport. The visual result and live
wheel behavior still require display-backed client verification.

Schema v6 adds a dedicated multi-select list block after the schema-v1 screen prefix and schema-v2 text-field entries. It appends `u32 list_count` (exactly `1`), a stable list ID and literal label, `u32 initial_selected_count` (0–32) and that many initially selected option IDs, then `u32 option_count` (1–32) and each stable option ID and literal label. The list, field, and button IDs are unique. Initial IDs must be unique and refer to declared options. Toggling rows and scrolling the list are local client UI state; the host sends no callback until a button is clicked. Schemas v1–v5 retain their request bytes.

A schema-v6 submit closes the screen and invokes the distinct direct owner export `sjml_v1_event_client.screen.multi_select_form_submit`. Its payload is `u32 schema_version` (`6`), stable `screen_id`, stable `button_id`, `u32 field_count` (0–4), each text value as a schema-v1 `ClientScreenFormValue`, stable `list_id`, `u32 selected_option_count` (0–32), then the selected option IDs in declaration order. The callback must return the unit value and is not routed through `events.subscribe`. The owner must export it before opening the screen. Existing schema-v1 through v5 forms continue using their original `client.screen.action` or `client.screen.form_submit` callback wires.

## Client toast notifications

ABI 1.37.0 adds `sjml_client_ui_v1.notify(title_ptr, title_len, body_ptr,
body_len, kind, duration_ticks) -> i32`, available only to a physical-client
runtime with requested and granted `client.ui.notify` and on the client main
thread. Title and body are strict UTF-8, nonblank literal strings of at most
128 bytes each and cannot contain control characters. `kind` is `0=info`,
`1=success`, `2=warning`, or `3=error`; duration is 20–600 ticks. Calls are
limited to six per module per 30 seconds, and repeated notices update one
module-owned toast slot. Static-registration and server calls fail closed. See
[client toast notifications](CLIENT_NOTIFICATIONS.md).

## Client-local audio

ABI 1.38.0 adds `sjml_client_audio_v1.play(sound_id_ptr, sound_id_len,
category, volume_bits, pitch_bits) -> i32`. It requires the separately granted
`client.audio.play`, a physical client, and the client main thread. IDs must be
canonical registered sound resources of at most 256 UTF-8 bytes; categories
are the ten vanilla `SoundSource` values (`0=Master` through `9=Voice`), volume
is finite in 0..=4, and pitch is finite in 0.5..=2. Each module is limited to
five requests per two-second window. Playback is local and sends no packets.
See [client-local audio](CLIENT_AUDIO.md).

ABI 1.60.0 adds `sjml_client_audio_v1.play_tracked(sound_id_ptr, sound_id_len,
category, volume_bits, pitch_bits) -> i64`, `stop_tracked(handle) -> i32`, and
`fade_tracked(handle, target_volume_bits, duration_ticks) -> i32`.
The start result packs the status in its upper 32 bits and a nonzero
runtime-owned handle in its lower 32 bits. Tracked playback uses
`client.audio.play`, shares its start rate limit, and is bounded to sixteen
handles per runtime and 512 across the physical client. Stop and runtime cleanup
target only the exact `SoundInstance` created by that runtime. Fade targets are
finite `0..=4`, durations are `1..=1200` client ticks, and an active ramp is
updated on the Minecraft client sound tick. A new fade starts at the current
volume. Disconnect cleanup stops every client-owned tracked sound. If a NeoForge
sound listener cancels or replaces the instance, the host returns
`SERVICE_BUSY` and does not stop the replacement. See [client-local audio](CLIENT_AUDIO.md).

## Client situational music selection

The physical-client imports `sjml_client_music_v1.subscribe() -> i32` and
`unsubscribe() -> i32` require the separately requested and granted
`client.audio.music` capability and the client main thread. They are available
only after the client hook is installed; static-registration and dedicated
server calls fail closed. One callback per active module may be subscribed.
Runtime teardown removes the subscription.

The direct callback export is
`sjml_v1_client_music_selection(input_ptr: i32, input_len: i32) -> i64`,
using the standard callback span and status frame. Its schema-v1 input, encoded
in declaration order, is `u32 version=1`, a stable UTF-8 original sound ID,
an optional stable UTF-8 current sound ID (`u8` tag `0=None`, `1=Some`),
`i32 original_min_delay_ticks`, `i32 original_max_delay_ticks`, and a boolean
byte (`0` or `1`) for `original_replace_current`. IDs must be canonical
`namespace:path` sound IDs no longer than 256 UTF-8 bytes. The callback runs
synchronously before NeoForge's situational music selection on the client
main thread and receives no Minecraft objects.

The schema-v1 result is `u32 version=1`, `u8 action` (`0=keep`, `1=stop`,
`2=replace`), an optional stable UTF-8 replacement sound ID, `u32 minimum and
maximum delay ticks`, and a boolean replace-current byte. Keep and stop require
zero/empty replacement fields. Replace requires a sound successfully
registered by the exact same package and module, an available client sound
resource, and `0 <= minimum <= maximum <= 1200`. The module can request
`unsubscribe()` at any time. This API changes only situational music; it does
not manage tracked one-shot sounds or provide fades.

All active SJML callbacks see the same snapshot and run in ascending runtime
module ID order; the last non-keep result wins. SJML applies its selected
result through `SelectMusicEvent.setMusic` without cancelling the NeoForge
event, so other mods still receive it under NeoForge event-priority rules.
`stop` sets the selected music to `null`. A callback trap, invalid result, or
attempt to select another owner's sound disables only the failing SJML module.
See [client music selection](CLIENT_MUSIC.md) for an example and a
display-backed manual check.

ABI 1.59.0 adds the physical-client `sjml_client_camera_v1` imports
`set_fov_scale(scale: f32) -> i32` and `clear_fov_scale() -> i32`. They require
the `client.camera.fov` grant, the client main thread, and an active client
runtime. Each module's scale must be finite and within `0.25..=4.0`; `1.0` is
neutral. The host multiplies active module scales in runtime activation order,
clamps their product to `0.25..=4.0`, and applies it to NeoForge's already
computed dynamic FOV modifier. The final modifier is clamped to `0.05..=4.0`.
Each module's scale persists until cleared or its runtime closes. This API does
not call guest code during frame-time FOV calculation. Live visual behavior
still needs a display-backed client check. See [client camera controls](CLIENT_CAMERA.md).

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

Mouse-wheel subscriptions use `sjml_client_input_v1.subscribe_scroll` and
`unsubscribe_scroll` with the same 24-byte module-scoped handles. The host
dispatches schema `u32(1)`, callback ID `u64`, and two little-endian `f64`
deltas through `sjml_v1_client_input_scroll_dispatch(pointer, length) -> i64`.
Each delta is finite and bounded to ±64, and at least one must be non-zero.
Callbacks are observational and cannot cancel vanilla scrolling. They share
the `client.input` permission and 256-subscription per-module limit; static
registration and dedicated servers fail closed. See
[mouse-wheel input](CLIENT_INPUT_SCROLL.md).

`sjml_client_input_v1.cursor_position(output_ptr) -> i32` writes a 16-byte
value containing little-endian IEEE-754 `f64` GUI coordinates (`x`, then `y`).
The host validates the full writable span and requires a physical-client
runtime, `client.input`, the client main thread, a live window, and a pointer
inside the viewport. Non-finite, negative, or excessively large coordinates
fail closed. This synchronous call does not move the cursor; see
[cursor position](CURSOR_POSITION.md).

`sjml_client_input_v1.is_key_down(key_code, output_ptr) -> i32` and
`is_mouse_button_down(button, output_ptr) -> i32` write one boolean byte at
`output_ptr` (`0=false`, `1=true`). Key codes are bounded to `0..=348`; mouse
button indices are bounded to `0..=7`. Both calls require a client module with
`client.input.observe`, the client main thread, and the physical-client provider
installed by the `Dist.CLIENT` adapter. They only poll current held state and
do not intercept or mutate keyboard or mouse input. Static registration,
server contexts, missing grants, and off-thread calls fail closed.

## Declarative client keybindings

The `sjml_client_keybinding_v1.register` import is available only during the
module's `mod_register` callback and requires a client-capable module plus
`client.input`. Its signature is
`(path_ptr, path_len, key_type, code, modifier, context) -> i32`. It stages a
bounded named key mapping before NeoForge registry freeze. `key_type` is `0`
for keyboard and `1` for mouse; keyboard codes are -1 through 65,535 and mouse
buttons 0 through 31. Modifier values are `0=None`, `1=Shift`, `2=Control`,
`3=Alt`; context values are `0=InGame`, `1=Gui`, `2=Universal`. Paths are
lowercase ASCII resource paths of at most 48 bytes, with at most 32 mappings
per module and 1,024 staged mappings total.

At runtime, `subscribe(callback_id, path_ptr, path_len, handle_out_ptr) -> i32`
attaches a typed key-down callback to a declared path, and
`unsubscribe(handle_ptr, handle_len) -> i32` revokes its 24-byte handle. The
guest callback is `sjml_v1_client_keybinding_dispatch(pointer, length) -> i64`
with a schema-u32 `1` and callback-ID u64 payload; it returns unit. Up to 128
subscriptions are allowed per module. NeoForge's Controls screen and conflict
handling own rebinding. The mapping bridge is loaded only on `Dist.CLIENT`; the
common static-registration code has no client Minecraft references. See
[client keybindings](KEYBINDINGS.md).

## Logical-server worlds

ABI v1 provides the following synchronous `sjml_world_v1` imports. All parameters
and statuses are `i32`; zero is success and nonzero values use the error table.
World reads require the requested and granted `world.read` capability. Block
and time mutation, scheduled block ticks, weather changes, and particle spawning
additionally require requested and granted `world.write`. Static
registration binds the signatures but every call during `mod_register` returns
`PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `get` | `id_ptr, id_len, out_ptr, out_capacity` | Stable `Option<WorldHandle>`: byte `0`, or byte `1` followed by the 24-byte handle |
| `loaded` | `out_ptr, out_capacity` | Stable `Vec<WorldHandle>`: little-endian u32 count, then 24 bytes per handle |
| `dimension_id` | `handle_ptr, handle_len, out_ptr, out_capacity` | Stable `String`: little-endian u32 UTF-8 length, then bytes |
| `game_time` | `handle_ptr, handle_len, out_ptr` | Little-endian signed i64 |
| `day_time` | `handle_ptr, handle_len, out_ptr` | Little-endian signed i64 |
| `light_at` | `handle_ptr, handle_len, x, y, z, out_ptr, out_capacity` | Stable `Option<LightSnapshot>` in a seven-byte output buffer: presence tag; `Some` appends schema-v1 u32 and sky/block u8 values in 0–15 |
| `fluid_at` | `query_ptr, query_len, out_ptr, out_capacity` | Bounded schema-v1 `Option<FluidStateSnapshot>`; reads only an already-loaded position |
| `daylight_cycle_enabled` | `handle_ptr, handle_len, out_ptr, out_capacity` | Stable bool record: schema-v1 u32, bool u8 |
| `game_rule` | `query_ptr, query_len, out_ptr, out_capacity` | Read an allowlisted boolean gamerule using a 32-byte schema-v1 query and exact 5-byte response; requires `world.read` |
| `set_game_rule` | `mutation_ptr, mutation_len` | Set an allowlisted boolean gamerule from a 33-byte schema-v1 mutation; requires `world.read` and `world.write` |
| `biome_at` | `handle_ptr, handle_len, x, y, z, out_ptr, out_capacity` | Stable `Option<String>`: u32 UTF-8 length (`0` means unavailable), then canonical biome ID bytes |
| `biome_ids` | `query_ptr, query_len, out_ptr, out_capacity` | Enumerate up to 4,000 canonical IDs from this world's biome registry using a schema-v1 query/response; requires `world.read` and never loads chunks |
| `tag_members` | `query_ptr, query_len, out_ptr, out_capacity` | Enumerate up to 1,024 sorted canonical members from one block, item, fluid, or entity-type tag; schema-v1 request and response; a four-byte output capacity probes required response size; requires `world.read` |
| `set_time` | `handle_ptr, handle_len, day_time_low, day_time_high` | Set absolute daylight-cycle time; status only |
| `set_daylight_cycle_enabled` | `handle_ptr, handle_len, enabled` | Set vanilla `doDaylightCycle`; `enabled` must be 0 or 1 |
| `block_states` | `query_ptr, query_len, out_ptr, out_capacity` | Stable ordered `Vec<Option<BlockStateSnapshot>>` |
| `block_properties` | `query_ptr, query_len, out_ptr, out_capacity` | Stable `Option<Vec<BlockProperty>>` at one loaded position |
| `raycast` | `query_ptr, query_len, out_ptr, out_capacity` | Stable `Option<RaycastHit>` for one bounded segment |
| `collides` | `query_ptr, query_len, out_ptr, out_capacity` | Stable `Option<bool>` for one bounded axis-aligned box |
| `set_block` | `handle_ptr, handle_len, x, y, z, block_id_ptr, block_id_len` | Replace one loaded position with the registered block's default state; status only |
| `schedule_block_tick` | `handle_ptr, handle_len, x, y, z, block_id_ptr, block_id_len, delay_ticks` | Schedule one normal tick for the matching registered block at an already-loaded position; status only |
| `notify_neighbors_at` | `handle_ptr, handle_len, x, y, z` | Run the ordinary vanilla neighbor update at a loaded position; status only |
| `spawn_entity` | `query_ptr, query_len` | Spawn one registered, non-player entity from a bounded schema-v1 world/type/position record; status only |
| `spawn_particle` | `handle_ptr, handle_len, x, y, z, particle_id_ptr, particle_id_len` | Spawn one registered simple particle at the center of a loaded block; status only |
| `play_sound` | `handle_ptr, handle_len, x, y, z, sound_id_ptr, sound_id_len, volume_bits, pitch_bits` | Play one registered sound at the center of a loaded block; status only |
| `weather` | `(handle_ptr: i32, handle_len: i32, output_ptr: i32, output_capacity: i32) -> i32` | Write an 18-byte schema-v1 weather snapshot |
| `set_weather` | `(handle_ptr: i32, handle_len: i32, condition: i32, duration_ticks: i32) -> i32` | Set clear, rain, or thunder weather; status only |
| `border` | `handle_ptr, handle_len, out_ptr, out_capacity` | 68-byte schema-v1 world-border snapshot |
| `set_border_center` | `handle_ptr, handle_len, values_ptr, values_len` | Set finite bounded border center; status only |
| `set_border_size` | `handle_ptr, handle_len, values_ptr, values_len` | Set size immediately or begin bounded transition; status only |
| `set_border_warnings` | `handle_ptr, handle_len, values_ptr, values_len` | Set warning lead time and distance; status only |
| `set_border_damage` | `handle_ptr, handle_len, values_ptr, values_len` | Set safe buffer and damage rate; status only |

The gamerule IDs are stable values 0–17 from
[`WorldGameRule`](../sjml-cli/sjml-sdk/src/world.rs), covering the boolean
vanilla rules listed in [world gamerules](WORLD_GAME_RULES.md).
Reads and writes apply only to the selected already-loaded dimension; integer
rules are intentionally excluded. The exact request and response layouts are
schema `u32(1)`, 24-byte world handle, `u32` rule ID, and strict boolean byte
for writes. `schedule_block_tick` requires a delay of 1–72,000 ticks, a
position inside build height, and the canonical ID of the block currently at
that position. It never loads chunks and is limited to 16 requests per module
on each logical-server tick. See [scheduled block ticks](WORLD_SCHEDULED_TICKS.md).
`notify_neighbors_at` additionally requires both `world.read` and `world.write`,
an in-height position inside the world border, and the target plus every
in-height direct-neighbor chunk to already be loaded. It uses non-loading chunk
checks and calls vanilla `ServerLevel.updateNeighborsAt`; it never requests
chunks. A vanilla callback failure returns `UNKNOWN_ERROR_CODE`. See
[explicit vanilla neighbor updates](WORLD_NEIGHBOR_UPDATES.md).

`handle_len` must be 24. `get` takes raw UTF-8, limited to 65,536 bytes,
and requires a canonical `namespace:path` (no implicit `minecraft:` namespace).
Namespace bytes are lowercase ASCII letters, digits, `_`, `.`, or `-`; paths
add `/`. Empty components and malformed IDs return `INVALID_ARGUMENT`;
invalid UTF-8 returns `INVALID_UTF8`. Unknown or unloaded IDs return `None`.

Output buffers belong to the guest and stay live for the synchronous call.
`get` requires capacity at least 25; `loaded` requires at least `4 + 24 * count`;
`dimension_id` requires `4 + utf8_length`; `game_time` and `day_time` need 8
writable bytes. `set_time` reconstructs a little-endian signed `i64` from the
two `i32` words `day_time_low` and `day_time_high`.
`biome_at` takes signed block coordinates and writes a length-prefixed UTF-8
biome ID of at most 256 bytes; a zero length means the position is outside
build height or belongs to an unloaded chunk. It requires `world.read`, checks
the owning logical-server thread and loaded-world handle, and never loads a
chunk. See [world biome lookup](WORLD_BIOMES.md).
`biome_ids` accepts exactly 28 query bytes: little-endian `u32 schema=1` and
the 24-byte world handle. Its complete response is `u32 schema=1`, `u32 count`,
then up to 4,000 canonical, strictly UTF-8 biome IDs as length-prefixed
strings. IDs are sorted lexically, each is at most 256 bytes, and the entire
response must fit in the guest-provided output span or no bytes are written.
The host enumerates the selected world's biome registry only; it does not query
or load chunks. It requires `world.read`, a valid loaded-world handle, and the
logical-server thread. See [biome ID enumeration](WORLD_BIOME_IDS.md).
`light_at` receives a loaded-world handle and signed block coordinates, and
writes a stable optional schema-v1 `LightSnapshot` into a seven-byte output
buffer: `None` is one zero presence byte; `Some` is presence byte `1`, schema
`u32(1)`, then sky-light and block-light bytes, each in `0..=15`. `None` means
outside build height or in an unloaded chunk. It
requires `world.read`, runs on the owning server thread, validates x/z within
±30,000,000, and never loads a chunk. See [world light levels](WORLD_HEIGHT_LIGHT.md).
`fluid_at` accepts exactly 44 query bytes: `u32 schema=1`, 24-byte world
handle, `u32 position schema=1`, and three signed little-endian `i32`
coordinates. Its response is `u32 schema=1`, presence byte, and when present
`u32 snapshot schema=1`, length-prefixed canonical fluid ID, amount byte,
source byte, and finite `f32` intrinsic own-height. The response is capped at
275 bytes. `None` means outside build height or unloaded chunk; a loaded dry
block returns the `minecraft:empty` snapshot. The call requires `world.read`,
runs on the logical-server thread, and never loads a chunk. See
[world fluid state](FLUID_STATE.md).
The complete declared capacity is bounds checked. Only the encoded value is
written; the remaining capacity is padding, not part of its stable encoding.
Insufficient capacity returns `PAYLOAD_TOO_LARGE` before handle allocation.
The SDK allocates 25, 24,580, 65,540, and 8 bytes for each time query. The
string limit remains 65,536 UTF-8 bytes. `weather` requires the 24-byte world handle and an
output capacity of at least 18 bytes. Its output is exactly little-endian
`u32 schema=1`, `u8 raining` (0 or 1), `u8 thundering` (0 or 1), then
`u32 clear_time`, `u32 rain_time`, and `u32 thunder_time`, each timer in ticks.
The SDK rejects returned timer values above `i32::MAX`, matching the host's
non-negative Java `int` timers. `world::weather` requires requested and granted
`world.read`; `set_weather` requires both `world.read` and `world.write`. Its
condition tag is `0=clear`, `1=rain`, or `2=thunder`, and duration is
1–1,728,000 ticks inclusive. Both require an already-loaded server dimension,
run synchronously on its owning server thread, and never load dimensions or
chunks. Static-registration and client calls fail closed with
`PERMISSION_DENIED`. See [world weather](WORLD_WEATHER.md).
`day_time` requires requested and granted `world.read`; `set_time` requires
both `world.read` and `world.write`. The time value must be from 0 through
1,000,000,000,000 inclusive. These operations resolve only an already-loaded
world, run on its owning logical-server thread, and never load dimensions or
chunks. `set_time` changes the daylight-cycle clock, not `game_time` or the
`doDaylightCycle` gamerule. `daylight_cycle_enabled` requires `world.read` and
returns exactly five bytes (`u32 schema=1`, then a boolean byte). The setter
requires both `world.read` and `world.write`, accepts only integer 0 or 1, and
changes the live world's persisted vanilla gamerule without changing the
current day time. See [world time](WORLD_TIME.md) and
[daylight-cycle gamerule](WORLD_DAYLIGHT_CYCLE.md).
`border` requires `world.read` and returns the loaded world's center, current
and target diameter, remaining transition milliseconds, warnings, and border
damage settings. The four setters require both `world.read` and `world.write`.
Their exact schema-v1 value lengths are 20 bytes for center, 20 for size,
12 for warning values, and 20 for damage values. The host bounds center to
±29,999,984, diameter to 1..=59,999,968, transition to at most 604,800,000 ms,
warning time to at most 86,400 seconds, warning distance to at most 59,999,968,
damage buffer to at most 1,024, and damage per block to at most 100. Every
floating-point setter value must be finite. Reads and writes resolve an already
loaded world on its server thread and never load chunks. See
[world border](WORLD_BORDER.md).

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

`block_properties` reads one state at a time using schema `u32(1)`, the
24-byte world handle, a position schema `u32(1)`, and signed `i32` x/y/z. The
response is a versioned `Option<Vec<BlockProperty>>`; each detached property
record contains a canonical lowercase name and a printable UTF-8 value. The
list is sorted by property name and capped at 64 entries, with names limited to
64 bytes and values to 256 bytes. Positions outside build height or in
unloaded chunks return `None`; no chunk is loaded. This call uses the same
`world.read` permission, server context, owner, generation, and thread checks as
the block batch API.

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
| `vitals` | `handle_ptr, handle_len, out_ptr, out_capacity` | 28-byte `PlayerVitals`: u32 schema `1`, `f32 health`, `f32 max_health`, `u32 food_level`, `f32 saturation`, `u32 experience_level`, `f32 experience_progress` |
| `active_effects` | `handle_ptr, handle_len, out_ptr, out_capacity` | Bounded schema-v1 snapshot of at most 64 sorted active effects; requires `entity.read` |
| `equipment` | `handle_ptr, handle_len, out_ptr, out_capacity` | Stable `PlayerEquipment`: six optional slot records in main-hand, offhand, feet, legs, chest, head order |
| `game_mode` | `handle_ptr, handle_len, out_ptr, out_capacity` | Eight bytes: u32 schema `1`, u32 mode (`0=Survival`, `1=Creative`, `2=Adventure`, `3=Spectator`) |
| `set_game_mode` | `handle_ptr, handle_len, mode_u32` | Set the current login session's game mode; status only |
| `motion` | `handle_ptr, handle_len, out_ptr, out_capacity` | 36-byte schema-v1 player motion: u32 schema, three f64 velocity components, f32 yaw, f32 pitch |
| `abilities` | `handle_ptr, handle_len, out_ptr, out_capacity` | 8-byte schema-v1 snapshot: u32 schema and four bool bytes (`may_fly`, `flying`, `invulnerable`, `may_build`) |
| `set_flying` | `handle_ptr, handle_len, enabled` | Set only the flying flag; status only |
| `set_velocity` | `handle_ptr, handle_len, velocity_ptr, velocity_len` | Set three bounded f64 velocity components from a 28-byte schema-v1 record; status only |
| `apply_effect` | `handle_ptr, handle_len, effect_id_ptr, effect_id_len, duration_ticks, amplifier` | Apply one registered effect to the current player for that login session; status only |
| `teleport` | `(handle_ptr: i32, handle_len: i32, destination_ptr: i32, destination_len: i32) -> i32` | Teleport the current player session within its current dimension; status only |
| `action_bar` | `(handle_ptr: i32, handle_len: i32, text_ptr: i32, text_len: i32) -> i32` | Send literal action-bar text; requires `entity.read` and `player.ui.actionbar` |
| `title` | `(handle_ptr, handle_len, title_ptr, title_len, subtitle_ptr, subtitle_len, fade_in_ticks, stay_ticks, fade_out_ticks) -> i32` | Send literal title and optional subtitle; requires `entity.read` and `player.ui.title` |
| `has_item_cooldown` | `handle_ptr, handle_len, item_id_ptr, item_id_len, out_ptr, out_capacity` | Write one stable boolean byte for the live player's cooldown state; requires `entity.read` |
| `set_item_cooldown` | `handle_ptr, handle_len, item_id_ptr, item_id_len, ticks` | Set the vanilla cooldown to 1–72,000 server ticks; requires `entity.read` and `player.cooldown.write` |
| `clear_item_cooldown` | `handle_ptr, handle_len, item_id_ptr, item_id_len` | Remove the vanilla item cooldown; requires `entity.read` and `player.cooldown.write` |
| `advancement_progress` | `handle_ptr, handle_len, advancement_id_ptr, advancement_id_len, out_ptr, out_capacity` | Write a stable schema-v1 snapshot of at most 256 sorted criteria and their achieved flags; requires `entity.read`. |
| `grant_advancement_criterion` | `handle_ptr, handle_len, advancement_id_ptr, advancement_id_len, criterion_id_ptr, criterion_id_len, changed_out_ptr, changed_out_capacity` | Award one exact registered criterion; requires `entity.read` and `player.advancement.write`. |
| `revoke_advancement_criterion` | `handle_ptr, handle_len, advancement_id_ptr, advancement_id_len, criterion_id_ptr, criterion_id_len, changed_out_ptr, changed_out_capacity` | Revoke one exact registered criterion; requires `entity.read` and `player.advancement.write`. |
| `team` | `handle_ptr, handle_len, out_ptr, out_capacity` | Read the live player's optional current scoreboard-team ID; requires `entity.read`. |
| `set_team` | `handle_ptr, handle_len, team_id_ptr, team_id_len` | Move the live player to an existing current-server scoreboard team; requires `entity.read` and `player.team.write`. |
| `clear_team` | `handle_ptr, handle_len` | Remove the live player's current team assignment; requires `entity.read` and `player.team.write`. |

`handle_len` must be 24. UUIDs are canonical lowercase UUID text, and
dimension IDs are canonical `namespace:path`. Each stable string is limited to
65,536 UTF-8 bytes. The SDK reserves `4 + 24 * 1024` bytes for enumeration and
`4 + 3 * (4 + 65,536) + 24` bytes for a snapshot. The complete declared output
capacity is bounds checked; insufficient capacity returns
`PAYLOAD_TOO_LARGE` before new enumeration handles are allocated. `vitals`
requires a 28-byte output buffer; `game_mode` requires eight bytes;
`motion` requires a 36-byte output buffer. `equipment` requires a 1,690-byte
output buffer; it returns a schema-v1 record with six `Option` values in
main-hand, offhand, feet, legs, chest, and head order. Each present stack is a
schema-v1 record containing a length-prefixed item ID followed by count,
maximum stack size, current damage, and maximum damage as little-endian u32s.
The host caps IDs at 256 bytes, stack sizes at 4,096, and durability fields at
1,000,000; client and static-registration calls fail closed.

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
`teleport` requires both requested and granted `entity.read` and
`player.teleport`. Its handle is exactly 24 bytes. `destination_len` must be
exactly 36 and `destination_ptr` points to little-endian `u32 schema=1`,
`f64 x`, `f64 y`, `f64 z`, `f32 yaw`, `f32 pitch`, in that order. Coordinates
must be finite and each axis within ±29,999,984; yaw must be finite within
±36,000 degrees; pitch must be finite within -90..=90 degrees. The host also
checks the current dimension's build height, world border, and that the target
chunk is already loaded. It does not load chunks or change dimensions. Calls
require an active server-player session and the owning server thread. Static
registration and client calls return `PERMISSION_DENIED`. See
[player teleport](PLAYER_TELEPORT.md).
`vitals` returns only a detached read snapshot and requires `entity.read`. The
host validates the same module-owned login-session handle and owning server
thread on every call. Health and maximum health must be finite and at most
1,000,000, maximum health must be positive, food level must be 0–20,
saturation 0–20, experience level at most 1,000,000, and experience progress
0–1. Invalid host-side values return `INVALID_ARGUMENT`; client and static
registration calls fail closed. See [player vitals](PLAYER_VITALS.md).
`game_mode` returns the detached stable mode value and requires `entity.read`.
`set_game_mode` also requires the separately requested and granted
`player.game_mode.write` capability. The host validates the same live player
login-session handle and owning server thread; only the four built-in modes
are accepted. It changes that player's mode without changing the world's
default mode. Static registration and client calls fail closed. See
[player game mode](PLAYER_GAME_MODE.md).
`motion` requires `entity.read` and returns the current velocity and yaw/pitch
rotation of the active login session. `set_velocity` additionally requires the
separately requested and granted `player.motion.write` capability and accepts
only finite components within ±64 blocks per tick. The host synchronizes
motion to the player and validates the owning server thread and live session.
Static registration and client calls fail closed. The Rust SDK's
`PlayerMotion::look_direction()` derives a normalized `[f64; 3]` vector locally
from the decoded yaw and pitch using Minecraft's look-direction convention; it
adds no host import and does not alter the 36-byte schema-v1 record. See
[player motion](PLAYER_MOTION.md).
`abilities` requires `entity.read` and returns four strict boolean bytes after
schema version 1. `set_flying` additionally requires `player.abilities.write`,
accepts only 0 or 1, and cannot enable flight unless the player's existing
`may_fly` flag permits it. It changes no game-mode, invulnerability, or build
permission flags. The host updates the player's ability packet only after a
real change. Static registration and client calls fail closed. See
[player abilities](PLAYER_ABILITIES.md).
`action_bar` additionally requires the separately requested and granted
`player.ui.actionbar` capability alongside `entity.read`. Text is literal,
nonblank, control-free UTF-8 limited to 1,024 bytes and 256 Unicode scalar
values. Calls are server-only, validate the live player login session and
server thread, and are limited to ten per module per five seconds. See
[player action-bar messages](PLAYER_ACTIONBAR.md).
`title` requires the separately requested and granted `player.ui.title`
capability alongside `entity.read`. Title and subtitle are literal, control-free
UTF-8 limited to 1,024 bytes and 256 Unicode scalar values; the title cannot be
blank, while the subtitle may be empty. Each animation timing must be in
`0..=6000` ticks. Calls are server-only, validate the live login session and
server thread, and are limited to ten sends per module per five seconds. See
[player title and subtitle messages](PLAYER_TITLES.md).

Player team imports use `sjml_player_v1`. Reads require `entity.read`; `set_team`
and `clear_team` additionally require `player.team.write`. Team IDs are limited
to 1–64 ASCII letters, digits, `.`, `_`, `+`, or `-`. `team` requires a
73-byte output buffer and returns u32 schema `1`, followed by tag `0` when
unassigned or tag `1`, a little-endian u32 byte length, and the team ID. The
host resolves the handle to the current live `ServerPlayer` and accesses the
active server scoreboard on its owning server thread. `set_team` requires an
already existing team and may replace any current membership; it never creates
a team. `clear_team` removes the current membership and succeeds when already
unassigned. Static registration and client calls fail closed. See
[live player scoreboard teams](PLAYER_TEAMS.md).

Criterion-level advancement imports use `sjml_player_v1`. The optional progress
read requires `entity.read`; the two criterion mutation imports additionally
require `player.advancement.write`. Advancement IDs must resolve in the active
server registry, and criterion IDs must exactly match one of the at most 256
declared keys. `advancement_progress` writes little-endian schema-v1
`PlayerAdvancementProgress`: a versioned length-prefixed advancement ID, a u32
criterion count, then sorted versioned length-prefixed criterion IDs and one
strict achieved byte per criterion. Grant/revoke write a single strict boolean
byte to a one-byte output buffer. These server-thread operations run only for
the current live login session; completing a criterion may invoke vanilla
rewards, while revoking it does not reverse rewards or dependent advancements.
See [player advancements](PLAYER_ADVANCEMENTS.md).

## Registry-backed player statistics

This API uses the separate `sjml_player_statistics_v1` import module and a
typed family selector:

| Import | Parameters | Success output |
| --- | --- | --- |
| `read_registry_stat` | `player_ptr, player_len, family, statistic_id_ptr, statistic_id_len, out_ptr, out_capacity` | Four-byte little-endian nonnegative vanilla counter. |
| `increment_registry_stat` | `player_ptr, player_len, family, statistic_id_ptr, statistic_id_len, amount` | Status only; amount is 0–1,000,000 and zero is a no-op. |

Family codes are `0=BLOCK_MINED` (block ID), `1=ITEM_USED`, `2=ITEM_BROKEN`,
`3=ITEM_CRAFTED` (item IDs), `4=ENTITY_KILLED`, and `5=ENTITY_KILLED_BY`
(entity type IDs). The host validates IDs against the registry selected by the
family and rejects signed-counter overflow. Reads require `entity.read`;
increments also require `player.statistics.write`. Calls resolve the active
server-player session on its owning server thread. `ITEM_PICKED_UP` and
`ITEM_DROPPED` are not exposed. The `player` module separately covers
`Stats.CUSTOM`. See [player statistics](PLAYER_STATISTICS.md).

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
| `riding_snapshot` | `handle_ptr, handle_len, out_ptr, out_capacity` | Schema-v1 `EntityRidingSnapshot` with an optional vehicle identity and up to 16 sorted passenger identities |
| `start_riding` | `passenger_ptr, passenger_len, vehicle_ptr, vehicle_len, out_ptr, out_capacity` | Writes little-endian `u32` `0`/`1` for refused/successful non-forced mount |
| `stop_riding` | `passenger_ptr, passenger_len, out_ptr, out_capacity` | Writes little-endian `u32` `0`/`1` for no change/successful dismount |
| `navigate_to` | `request_ptr, request_len, out_ptr, out_capacity` | Schema-v1 44-byte request (`u32` version, 24-byte entity handle, `i32 x/y/z`, raw `f32` speed); writes schema-v1 `u32` and `u8` accepted flag |
| `apply_effect` | `handle_ptr, handle_len, effect_id_ptr, effect_id_len, duration_ticks, amplifier` | Apply one registered effect to a loaded living non-player entity; status only |
| `remove_effect` | `handle_ptr, handle_len, effect_id_ptr, effect_id_len` | Remove one registered active effect from a loaded living non-player entity; status only |
| `discard` | `(handle_ptr: i32, handle_len: i32) -> i32` | Discard one loaded non-player entity and revoke its handle; status only |
| `damage` | `(handle_ptr: i32, handle_len: i32, amount_bits: i32) -> i32` | Apply bounded generic damage to a loaded living non-player entity; requires `entity.read` and `entity.damage` |
| `rotation` | `handle_ptr, handle_len, out_ptr, out_capacity` | Write schema-v1 `u32(1)`, `f32 yaw`, `f32 pitch`; requires `entity.read` |
| `set_rotation` | `handle_ptr, handle_len, rotation_ptr, rotation_len` | Set bounded yaw and pitch from an exact 12-byte schema-v1 record; requires `entity.read` and `entity.rotation.write` |

`handle_len` must be 24. `uuid` is canonical lowercase UUID text;
`entity_type_id` and `dimension_id` are canonical `namespace:path` IDs. Each
string is limited to 65,536 UTF-8 bytes. The SDK reserves `4 + 24 * 1024` bytes
for enumeration and `4 + 3 * (4 + 65,536) + 24` bytes for a snapshot. The complete declared output
capacity is bounds checked; insufficient capacity returns
`PAYLOAD_TOO_LARGE` before allocating handles. Effect application additionally
requires `entity.read` and `entity.effect.apply`; effect removal separately
requires `entity.read` and `entity.effect.remove`, and an inactive effect is a
successful no-op. Both operations are denied during static registration. The
ID is strict UTF-8 in canonical `namespace:path` form and at
most 256 bytes. Duration is 1–72,000 ticks and amplifier is 0–255. The host
revalidates the handle and resolves the effect from the active registry on the
owning server thread. Non-living targets and unknown IDs return
`INVALID_ARGUMENT`; this operation cannot target players.
`damage` accepts a finite amount in `(0, 64]` encoded as raw `f32` bits. It
uses Minecraft's generic damage source and normal `LivingEntity.hurt` path;
the guest cannot choose an attacker or damage type, and vanilla mitigation
and cancellation remain in effect. See
[generic entity damage](ENTITY_DAMAGE.md).
Riding reads require `entity.read`. `riding_snapshot` uses a little-endian
schema-v1 record: `u32` version, `u8` vehicle-presence tag, the optional
relationship record, `u32` passenger count, then sorted relationship records;
each relationship contains schema `u32(1)`, a length-prefixed canonical UUID,
a length-prefixed canonical entity type ID, and an `is_player` byte. The guest
response is bounded to 16 passenger records. Mount and dismount imports require
both `entity.read` and `entity.mount.write`; handles must identify loaded
non-player entities in the same world. Mounts use `force=false`, and an already
mounted passenger is not silently moved. No operation loads chunks. See
[entity riding](ENTITY_RIDING.md).
`navigate_to` requires both `entity.read` and `entity.navigation.write`. Its
exact 44-byte schema-v1 request contains the entity handle, integer target
position, and finite speed multiplier in `0.01..=4.0`; the exact five-byte
response contains schema `u32(1)` and a strict `u8` boolean. The target must be
within 32 blocks, inside the current dimension build height and world border,
and its bounded one-chunk-margin path corridor must already be loaded. The
operation is server-thread-only, accepts loaded non-player Mobs, and never asks
Minecraft to load a chunk. It returns `false` if the Mob's navigator rejects
the path, including when AI is disabled. Static registration and client calls
are denied. See [bounded entity navigation](ENTITY_NAVIGATION.md).
Effect removal applies only to loaded living non-player entities on the owning
server thread. Stale handles, players, nonliving targets, and unknown IDs are
rejected. See [effect removal](ENTITY_EFFECT_REMOVAL.md).

`rotation` reads detached yaw and pitch from any live loaded non-player entity.
Its exact 12-byte result is little-endian `u32(1)`, `f32 yaw`, `f32 pitch`.
`set_rotation` accepts the same record and requires both `entity.read` and
`entity.rotation.write`. Yaw must be finite within ±36,000 degrees and pitch
finite within -90..=90; invalid values or a wrong schema/request size return
`INVALID_ARGUMENT`. Both imports revalidate the module-owned handle and run on
the owning server thread. The host checks current loaded entity membership
without loading chunks; static-registration and client calls fail closed. See
[entity rotation](ENTITY_ROTATION.md).

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
`discard` requires both requested and granted `entity.read` and
`entity.remove`; it takes an exact 24-byte module-scoped entity handle. The
operation runs on the logical-server thread, revalidates that the entity is
still loaded and not a player, then calls Minecraft's discard operation. It
does not execute death behavior or produce drops. The handle is revoked
immediately, so later use returns `STALE_HANDLE`. It never loads chunks. Static
registration and client calls return `PERMISSION_DENIED`. See
[entity removal](ENTITY_REMOVAL.md).

## Live entity state flags

`sjml_entity_v1.is_glowing` and `is_silent` read two stable flags from loaded
non-player entities. Both require `entity.read`. `set_glowing` and `set_silent`
also require the separate `entity.state.write` grant and accept only a little-
endian `i32` boolean (`0` or `1`). Reads write a little-endian `u32` boolean to
a checked four-byte output span. All calls validate the module-owned 24-byte
entity handle and run on the owning server thread. Static-registration and
client calls fail closed. See [entity state flags](ENTITY_STATE_FLAGS.md).

| Import | Parameters | Success output |
| --- | --- | --- |
| `is_glowing`, `is_silent` | `handle_ptr, handle_len, out_ptr, out_capacity` | Write little-endian `u32(0 or 1)`; status only |
| `set_glowing`, `set_silent` | `handle_ptr, handle_len, value` | Change the requested flag; status only |

## Literal non-player entity custom names

`sjml_entity_v1.custom_name` reads an optional plain-text custom name from a
loaded non-player entity and requires `entity.read`. `set_custom_name` and
`clear_custom_name` also require the separate `entity.custom_name.write` grant.
Names must be nonempty, control-free UTF-8 up to 1,024 bytes and 256 Unicode
scalar values; clearing uses the dedicated clear import. The host constructs a
literal text component and never parses markup or formatting codes.

| Import | Parameters | Success output |
| --- | --- | --- |
| `custom_name` | `handle_ptr, handle_len, out_ptr, out_capacity` | Optional schema-v1 string in a bounded response of at most 1,033 bytes; status only |
| `set_custom_name` | `handle_ptr, handle_len, name_ptr, name_len` | Set plain text; status only |
| `clear_custom_name` | `handle_ptr, handle_len` | Clear the current name; status only |

All operations require the exact 24-byte module-owned entity handle and run on
the owning logical-server thread. Static-registration and client calls fail
closed. Player names and arbitrary `Component` construction are not exposed.

## Live non-player entity attributes

The `sjml_entity_v1.attribute` and `set_attribute_base` imports expose the
SDK's 31 allowlisted vanilla attributes for loaded living non-player entities.
Reads require `entity.read`; writes also require `entity.attributes.write`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `attribute` | `handle_ptr, handle_len, attribute_id_ptr, attribute_id_len, out_ptr, out_capacity` | 20 bytes: schema `u32(1)`, base `f64`, effective `f64` |
| `set_attribute_base` | `handle_ptr, handle_len, attribute_id_ptr, attribute_id_len, value_ptr, value_len` | Set an allowlisted base value from a 12-byte schema-v1 record; status only |
| `add_attribute_modifier` | `handle_ptr, handle_len, attribute_id_ptr, attribute_id_len, key_ptr, key_len, request_ptr, request_len` | Add a module-owned transient modifier from a 16-byte schema-v1 request; status only |
| `remove_attribute_modifier` | `handle_ptr, handle_len, key_ptr, key_len, out_ptr, out_capacity` | 5 bytes: schema `u32(1)`, removed `u8(0|1)` |

The handle is exactly 24 bytes; IDs are canonical UTF-8 and are restricted to
the SDK's fixed `LivingAttribute` set. The request carries schema `u32(1)` and
a finite `f64` within the attribute-specific range. The host checks the live
entity handle, logical-server context, owning server thread, living entity
type, attribute presence, and capability grants on every call. A base-value
write changes only that value. Transient modifiers accept finite amounts in
-1,024..=1,024 and operation IDs `0=add value`, `1=add multiplied base`, and
`2=add multiplied total`. Modifier keys match `[a-z0-9_.-]{1,64}`; quotas are
16 per entity and 256 per module. Vanilla modifier IDs are derived from the
package/module owner and the guest key, so a module cannot mutate another
module's modifier. The host removes owned modifiers on explicit removal,
entity/world unload, and module shutdown; they are not saved to entity NBT.
Static registration and client calls fail closed. See [live entity attributes](ENTITY_ATTRIBUTES.md).

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

## Read-only inventory snapshots

ABI v1 provides the synchronous `sjml_inventory_v1.read` import. All parameters
and the status are `i32`; zero is success and nonzero values use the error table.
The module must request and receive both `world.read` and `inventory.read`.
Static registration binds the authorized signature but every call returns
`PERMISSION_DENIED`.

| Import | Parameters | Success output |
| --- | --- | --- |
| `read` | `world_ptr, world_len, position_ptr, position_len, side_ptr, side_len, out_ptr, out_capacity` | Stable `Option<InventorySnapshot>`: byte `0`, or byte `1`, snapshot schema version `1`, u32 slot count, then one `Option<ItemStackDescriptor>` per slot |
| `read_player` | `player_ptr, player_len, out_ptr, out_capacity` | Stable `InventorySnapshot`: schema version `1`, u32 slot count, then one `Option<ItemStackDescriptor>` per slot |

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

`read_player` accepts a module-owned 24-byte live-player handle and requires
both `entity.read` and `inventory.read`. It validates the login session and
owning server thread, then returns detached slots in vanilla order (hotbar,
main storage, armor, offhand). The response is limited to 128 slots and 1 MiB;
it contains no components, NBT, or mutable stack references. It never mutates
or synchronizes inventory. Static-registration and client calls fail closed.
See [player inventory reads](PLAYER_INVENTORY.md).

## Player inventory slot and durability writes

These server-only imports require `entity.read` and `inventory.write`. They
accept a module-owned live player handle and a slot from `0` through `40`.
`set_player_slot` accepts a canonical registered non-air item ID plus count, or
an empty ID and zero count to clear a slot. New stacks use the item's default
components; arbitrary components and NBT are not accepted. Armor slots accept
one item.

`set_player_damage(player_ptr, player_len, slot, damage) -> i32` changes the
durability of a nonempty damageable stack already in that slot. It preserves
the stack's other components, accepts values from `0` through `65,535`, and
rejects a value at or above the item's maximum damage so this operation cannot
break the item. Both imports run on the owning server thread and broadcast the
change through the player's active menu. Stale handles, empty or
nondamageable stacks, client/static contexts, and invalid values are rejected.

## Module-owned components on player inventory stacks

Player component reads require `entity.read` and `inventory.read`; writes
require `entity.read` and `inventory.write`. Both are available only to server
and `both` modules, run on the owning server thread, and resolve the live player
session for every call. The component path must identify a type registered by
the calling module. Empty slots or absent values return `None`; writes require
a non-empty stack and update only the registered component.

| Import | WebAssembly signature | Capability and behavior |
| --- | --- | --- |
| `read_player_integer_component` | `(player_ptr, player_len, slot, component_path_ptr, component_path_len, out_ptr, out_capacity) -> i32` | Requires `entity.read` and `inventory.read`; writes `Option<i32>` as tag `0`, or tag `1` followed by a little-endian `i32`. |
| `write_player_integer_component` | `(player_ptr, player_len, slot, component_path_ptr, component_path_len, value) -> i32` | Requires `entity.read` and `inventory.write`; the value must fit the registered inclusive bounds. |
| `read_player_boolean_component` | `(player_ptr, player_len, slot, component_path_ptr, component_path_len, out_ptr, out_capacity) -> i32` | Requires `entity.read` and `inventory.read`; writes `Option<bool>` as tag `0`, or tag `1` followed by byte `0` or `1`. |
| `write_player_boolean_component` | `(player_ptr, player_len, slot, component_path_ptr, component_path_len, value) -> i32` | Requires `entity.read` and `inventory.write`; value must be `0` or `1`. |

All writes copy the existing stack, preserve unrelated data, and synchronize
the active menu. See [player inventory components](PLAYER_INVENTORY_COMPONENTS.md)
and [boolean item components](BOOLEAN_ITEM_COMPONENTS.md).

## Enchantments on player inventory stacks

Enchantment reads require `entity.read` and `inventory.read`; set and remove
require `entity.read` and `inventory.write`. These synchronous imports are
available only to server and `both` modules on the owning server thread. Every
call resolves the enchantment ID through the active world's enchantment
registry and operates on one non-empty player inventory stack. Unknown IDs,
empty slots, and IDs over 256 UTF-8 bytes are rejected.

| Import | WebAssembly signature | Capability and behavior |
| --- | --- | --- |
| `read_player_enchantment_level` | `(player_ptr, player_len, slot, enchantment_id_ptr, enchantment_id_len, out_ptr, out_capacity) -> i32` | Requires `entity.read` and `inventory.read`; writes `Option<u32>` as byte `0`, or tag `1` followed by a little-endian u32 level. |
| `set_player_enchantment_level` | `(player_ptr, player_len, slot, enchantment_id_ptr, enchantment_id_len, level) -> i32` | Requires `entity.read` and `inventory.write`; the host checks the registry entry's minimum/maximum, whether the enchantment applies to the item, and conflicts with other enchantments. |
| `remove_player_enchantment` | `(player_ptr, player_len, slot, enchantment_id_ptr, enchantment_id_len) -> i32` | Requires `entity.read` and `inventory.write`; removes only the named enchantment, and succeeds without mutation if it is already absent. |

IDs must be canonical UTF-8 registry IDs. Set levels are `1..=255` at the
ABI boundary and must also fit the selected enchantment's registered bounds.
The host copies the existing stack, updates only its enchantment component,
marks inventory changed, and synchronizes the active menu. Enchanted books use
their stored-enchantment component; other items use their normal enchantment
component. Item identity, count, damage, unrelated components, and all other
enchantments are preserved. Client-side, static-registration, stale-handle,
and off-thread calls fail closed. See
[player inventory enchantments](PLAYER_INVENTORY_ENCHANTMENTS.md).

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

## Module-owned integer components on inventory items

Integer components are registered by the module with an inclusive signed
`i32` minimum and maximum. The declaration phase can assign an in-range default
to a same-module item; runtime imports can read or write one existing stack in
a loaded block `Container`. The host accepts only components owned by the
calling module and copies the stack before replacement, preserving its count
and unrelated components.

| Import | WebAssembly signature | Capability and behavior |
| --- | --- | --- |
| `read_integer_component` | `(world_ptr, world_len, position_ptr, position_len, side_ptr, side_len, slot, component_path_ptr, component_path_len, out_ptr, out_capacity) -> i32` | Requires `world.read` and `inventory.read`. Writes `Option<i32>` as byte `0`, or byte `1` followed by little-endian signed `i32`. |
| `write_integer_component` | `(world_ptr, world_len, position_ptr, position_len, slot, component_path_ptr, component_path_len, value) -> i32` | Requires `world.read` and `inventory.write`; value must fit the registered inclusive bounds. Zero status means the copied stack was updated. |

The read uses the optional sided item-handler capability and returns `None`
for an absent inventory, empty slot, or unset value. The write is unsided,
rejects empty slots and positions without a `Container`, and preserves the
existing item, count, and unrelated components. Both calls are server-thread
only, reject unloaded chunks instead of loading them, and are unavailable to
client-side modules. Minecraft's persistent and network codecs enforce the
same component bounds. See [integer item components](INTEGER_ITEM_COMPONENTS.md).

Boolean components use `register_boolean_component(path)` and
`set_item_boolean_component(item_path, component_path, value)`. Registration,
defaults, and runtime access are scoped to the declaring module. Their data
codec is a strict boolean, and the static default takes an integer ABI value of
exactly `0` or `1`.

| Import | WebAssembly signature | Capability and behavior |
| --- | --- | --- |
| `read_boolean_component` | `(world_ptr, world_len, position_ptr, position_len, side_ptr, side_len, slot, component_path_ptr, component_path_len, out_ptr, out_capacity) -> i32` | Requires `world.read` and `inventory.read`. Writes `Option<bool>` as byte `0`, or tag `1` followed by boolean byte `0` or `1`. |
| `write_boolean_component` | `(world_ptr, world_len, position_ptr, position_len, slot, component_path_ptr, component_path_len, value) -> i32` | Requires `world.read` and `inventory.write`; value must be `0` or `1`. |

Block reads use an optional sided item handler and return `None` for an absent
inventory, empty slot, or unset value. Writes require a non-empty slot in an
already-loaded block `Container`, preserve unrelated stack data, and do not
load chunks. The calls are server-side and run on the owning server thread.
See [boolean item components](BOOLEAN_ITEM_COMPONENTS.md).

The `transfer` import requires both `world.read` and `inventory.write` and moves
items between two loaded block item-handler slots in one host call. Its 15
`i32` arguments contain the 24-byte world handle, source position and optional
side, source slot, destination position and optional side, destination slot,
requested count, and output span. Count is limited to 1–64. The output is a
little-endian `u32` count of moved items. The host preflights extraction and
insertion with simulation, then performs the transfer on the owning server
thread. It never loads chunks or exposes an item stack to the guest. See
[inventory mutation details](INVENTORY_MUTATIONS.md) for limits and rollback
behavior.

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
| `get_data` | `request_ptr, request_len, out_ptr, out_capacity` | Schema-v1 optional Boolean/i32/String value for one typed key |
| `set_data` | `request_ptr, request_len` | Status only; sets one owner-private persistent typed value |
| `remove_data` | `request_ptr, request_len` | Status only; removes one owner-private persistent value |

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

Typed persistent data is a separate API for `SjmlRegisteredBlockEntity` values
created by the calling module. The request is schema-v1: `u32` version, `u8`
operation (`0=get`, `1=set`, `2=remove`), 24-byte block-entity handle,
length-prefixed UTF-8 key, `u8` value kind (`0=absent`, `1=boolean`,
`2=i32`, `3=string`), and the typed value for `set`. The `get_data` response is
`u32` version, `u8` option tag, and when present a kind byte plus its value.
Boolean tags accept only zero or one. Keys are 1–64 lowercase ASCII letters,
digits, or underscores; strings are printable UTF-8 up to 2,048 bytes. A block
entity stores at most 64 keys/16 KiB, and each module has at most 64 writes or
removals per server tick. Reads require `block_entity.data.read`; writes and
removals require `block_entity.data.read`, `block_entity.data.write`, and
`world.write`. The handle must identify the module's own registered block in
an already loaded chunk. Generic NBT snapshots remove the private data tag.
See [persistent block-entity data](BLOCK_ENTITY_PERSISTENT_DATA.md).

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

## Existing objective display slots (NeoForge adapter)

The Rust guest SDK binds `set_display_slot` and `clear_display_slot` in
`sjml_scoreboard_v1`; both require `scoreboard.write`, a live logical-server
scoreboard, and its owning server thread. `set_display_slot` takes
`objective_ptr, objective_len, slot_code`; the objective must already exist.
`clear_display_slot` takes only `slot_code` and is idempotent. Neither operation
creates or changes an objective. The bounded stable codes are `List` 0,
`Sidebar` 1, `BelowName` 2, `TeamBlack` 3, `TeamDarkBlue` 4, `TeamDarkGreen` 5,
`TeamDarkAqua` 6, `TeamDarkRed` 7, `TeamDarkPurple` 8, `TeamGold` 9,
`TeamGray` 10, `TeamDarkGray` 11, `TeamBlue` 12, `TeamGreen` 13, `TeamAqua`
14, `TeamRed` 15, `TeamLightPurple` 16, `TeamYellow` 17, and `TeamWhite` 18.
Unknown or negative codes return `INVALID_ARGUMENT`. The host maps this fixed
wire range explicitly rather than accepting arbitrary Java enum ordinals.

## Module-owned scoreboard teams (NeoForge adapter)

The Rust guest SDK binds the following imports in `sjml_scoreboard_v1`.
They operate only on the active logical-server scoreboard and require the
module's `scoreboard.read` or `scoreboard.write` permission as shown:

| Import | Parameters | Permission | Behavior |
| --- | --- | --- | --- |
| `team` | `id_ptr, id_len, output_ptr, output_capacity` | `scoreboard.read` | Read the current options of this module's team ID into a bounded snapshot. |
| `create_team` | `id_ptr, id_len` | `scoreboard.write` | Create an empty team or succeed for the same persisted owner and local ID. |
| `remove_team` | `id_ptr, id_len` | `scoreboard.write` | Remove only this module's team and its memberships. |
| `set_team_options` | `id_ptr, id_len, display_ptr, display_len, prefix_ptr, prefix_len, suffix_ptr, suffix_len, color, name_visibility, death_visibility, collision, friendly_fire, see_invisibles` | `scoreboard.write` | Set bounded literal text and validated vanilla options. |
| `add_player_to_team` | `id_ptr, id_len, uuid_ptr, uuid_len` | `scoreboard.write` | Add an online player that has no foreign team assignment. |
| `remove_player_from_team` | `id_ptr, id_len, uuid_ptr, uuid_len` | `scoreboard.write` | Remove an online player only from this exact team. |

All pointer/length pairs are strict UTF-8 byte spans. Local team IDs are
1–64 lowercase ASCII letters, digits, `_`, or `-`, beginning with a letter.
The host derives a 16-character vanilla-safe name from the exact
package/module/local-ID tuple and persists the tuple in world SavedData. An
existing vanilla team or team without an exact owner record is never adopted
or changed. The saved ownership record contains strings only and is resolved
again for each operation; teams are accessed and changed on the owner server
thread. Corrupt ownership data disables team access until the save data is
repaired rather than weakening the ownership check.

Display name, prefix, and suffix are literal components, bounded individually
to 128 UTF-8 bytes and 64 Unicode code points, with controls rejected. Display
name is nonempty; prefix/suffix can be empty. Color is a u32 code in this order:
`Reset`, `Black`, `DarkBlue`, `DarkGreen`, `DarkAqua`, `DarkRed`, `DarkPurple`,
`Gold`, `Gray`, `DarkGray`, `Blue`, `Green`, `Aqua`, `Red`, `LightPurple`,
`Yellow`, `White` (0–16). Name-tag and death-message visibility codes are
`Always`, `Never`, `HideForOtherTeams`, `HideForOwnTeam` (0–3). Collision codes
are `Always`, `Never`, `PushOtherTeams`, `PushOwnTeam` (0–3). Boolean options
must be exactly 0 or 1. Player UUIDs use canonical lowercase 8-4-4-4-12
hexadecimal spelling; the addressed player must be online. Moving someone
from vanilla or another module's team is denied.

The `team` response uses schema 1: little-endian u32 schema, then three
length-prefixed UTF-8 strings (display name, prefix, suffix), then u32 color,
name-tag visibility, death-message visibility, and collision codes, followed
by one byte each for friendly fire and friendly-invisible visibility. The
output buffer is limited to 1,024 bytes and callers initialize unused bytes to
zero. Unknown schema/enum values, invalid booleans, invalid UTF-8, and trailing
nonzero bytes are rejected by the SDK decoder. The API does not expose
`PlayerTeam`, `Scoreboard`, or other Java references to guests.

## Client localization lookup

The `sjml_client_localization_v1.translate` import accepts
`key_ptr, key_len, output_ptr, output_capacity, output_length_ptr` and returns
an ABI status code. It requires the `client.localization.read` permission,
`ExecutionContext.CLIENT`, and the physical client's main thread. Key spans are
strict UTF-8 and limited to 256 bytes; the returned UTF-8 text is limited to
4,096 bytes. On success the host writes the number of output bytes as one
little-endian u32 to `output_length_ptr` and copies exactly those bytes to the
output span. Static registration and dedicated-server contexts fail closed.
The result is detached literal text; no language or resource-manager object
crosses the ABI.

## Loaded-only structure query

`sjml_world_v1.structures_at(query_ptr, query_length, output_ptr,
output_capacity)` requires `world.read` and the logical server owner thread.
The schema-v1 query is little-endian `u32(1)`, a 24-byte `WorldHandle`, then a
schema-v1 `BlockPos` (`u32(1)` and three little-endian `i32` coordinates).
The response is `u32(1)`, a 0/1 availability byte, a little-endian `u32` count,
then that many length-prefixed canonical UTF-8 structure registry IDs. IDs are
lexically sorted, unique, limited to 256 bytes each and 128 total. The host
examines at most 4,096 structure references. Unavailable position/reference
chunks produce availability 0; a loaded position with no matching generated
structure piece produces availability 1 with zero IDs. No chunk is loaded.

## World registry tag members

`sjml_world_v1.tag_members(query_ptr, query_length, output_ptr,
output_capacity)` requires `world.read`, a live module-owned world handle, and
the owning logical-server thread. It is unavailable during static registration
and to physical-client modules. The schema-v1 query is little-endian
`u32(1)`, the 24-byte world handle, a `u32` registry selector, and a
length-prefixed canonical tag ID. Registry selectors are stable ABI values:
`0=block`, `1=item`, `2=fluid`, and `3=entity type`.

An output capacity of exactly four bytes performs a size probe and writes a
little-endian `u32` with the encoded response size. The caller then retries
with a buffer of that size plus the four-byte length prefix. The final output
starts with a little-endian `u32` containing the number of response bytes that
follow. The response is `u32(1)`, a 0/1 tag-availability byte, a
little-endian `u32` member count, then length-prefixed canonical registry IDs.
IDs are sorted lexically and unique; each ID is at most 256 UTF-8 bytes and the
count is at most 1,024. An absent tag has availability 0 and count 0. A
registered empty tag has availability 1 and count 0. If the complete response
does not fit the guest's output buffer, the host returns `PAYLOAD_TOO_LARGE`
without writing a partial result. The host reads the active server registry
and never accesses or loads chunks. Subsequent calls observe registry data
after a resource reload. See [registry tag members](WORLD_TAG_MEMBERS.md).

## Bonemeal fertilization attempt

The synchronous cancellable
`sjml_v1_event_block.fertilize_attempt` callback requires both
`events.subscribe` and `events.cancel`. Its schema-v1 `BlockFertilizeAttempt`
contains an optional player UUID, dimension ID, `BlockPos`, target block ID,
fertilizer item ID/count, and whether vanilla considered the target valid.
Identifiers and payload bounds are documented in
[block fertilization attempts](BLOCK_FERTILIZE_ATTEMPTS.md). Returning true
sets the NeoForge `BonemealEvent` unsuccessful and cancels fertilizer use;
false preserves the event's current result. Guest callbacks are synchronous
and owner-scoped; canceled events are not dispatched.

## Fluid block placement attempt

The synchronous cancellable `sjml_v1_event_block.fluid_place_attempt`
callback requires both `events.subscribe` and `events.cancel`. Its schema-v1
`BlockFluidPlaceAttempt` consists of little-endian `u32(1)`, a length-prefixed
`dimension_id`, schema-v1 target and liquid-source `BlockPos` values, then
length-prefixed `fluid_id`, `original_block_id`, and `result_block_id`. Resource
locations must be canonical lowercase IDs, each string is limited to 256
bytes, and the complete payload is capped at 2 KiB. The payload contains no
live Minecraft objects or block-state properties. NeoForge's cancellable
`BlockEvent.FluidPlaceBlockEvent` is captured on the owning logical-server
thread; a true guest result vetoes only that attempted block-state placement.
The fluid may continue flowing and later ticks are independent. See
[fluid placement attempts](BLOCK_FLUID_PLACE_ATTEMPTS.md).

## Player attack attempt

The synchronous cancellable `sjml_v1_event_player.attack_attempt` callback
requires both `events.subscribe` and `events.cancel`. The schema-v1
`PlayerAttackAttempt` payload is little-endian `u32(1)`, then length-prefixed
UTF-8 `player_id`, `target_id`, `target_type_id`, and `dimension_id`, followed
by little-endian `f64` `target_x`, `target_y`, and `target_z`. The first two
strings are canonical UUIDs; the other strings are canonical lowercase
resource locations. Strings are limited to 256 bytes, positions must be finite
and within ±30,000,000, and the complete payload is capped at 2 KiB. The
snapshot contains no live Minecraft objects or mutable event fields. A true
result cancels NeoForge's `AttackEntityEvent` before vanilla weapon hooks or
damage handling; the hook runs on the owning logical-server thread and
already-canceled events are not dispatched. See
[player attack attempts](PLAYER_ATTACK_ATTEMPTS.md).
