# SJML Security Boundaries

## WebAssembly execution

`sjml-core` parses modules with Endive and compiles every function to JVM
bytecode. Interpreter fallback is disabled. The portable core starts with no
host imports; `ModuleImports` binds only explicitly named numeric functions.
The current NeoForge adapter binds `sjml.abi_version` and `sjml.log`
unconditionally. Other optional APIs require both a manifest request and an
administrator grant. WASI is not installed. Defined linear memory is limited
to 256 wasm32 pages (16 MiB), and modules larger than 16 MiB are rejected.
The current host runtime also accepts at most 16 defined tables, with at most
65,536 initial entries in any table and 262,144 initial entries across all
tables. Growth is capped at 65,536 entries per table. These table bounds are
host resource limits, not ABI v1 guarantees.

This is an in-process runtime, not an operating-system sandbox. Compilation
does not provide a hard CPU deadline, process isolation, or protection from a
defect in Endive, the JVM, or a host function. CPU cancellation is cooperative
and best-effort. Host functions remain trusted code and must validate their
arguments, permissions, handle scope, and thread affinity before touching
Minecraft state.

Optional capabilities are enabled only when both declared in the module
manifest and granted by the host. Host API entry points must check those
capabilities before resolving or using a host resource. Portable-core handles
are scoped to one `ModuleInstance` and one execution context; the portable
`ModuleInstance` closes its handle table and task group during disable, failure,
and shutdown. The NeoForge adapter does not use that portable handle table or
task group. Its event-subscription, world, and player handles are non-transferable and scoped to
one `ActiveModule` and execution context; `ActiveModule.close()` closes the
event subscription, world, and player handle registries during module teardown after any eligible
`mod_shutdown` callback and compiled-runtime close.

Client input is split by effect: `client.input.observe` exposes detached
global-key notices after Minecraft has processed them, so callbacks cannot
cancel gameplay keys. The same grant allows bounded polling of held GLFW keys
and mouse buttons on the physical client's main thread; results are detached
booleans and do not consume or mutate input. `client.input.screen_cancel` can
synchronously veto a key press delivered to an active screen. The older
`client.input` grant continues to govern the existing key/mouse callbacks, and
a mouse-button press can be consumed only through its documented hook. Input
subscriptions are module-owned and removed when their runtime closes.

The physical-client `client.action_attempt` callback runs before vanilla attack,
use-item, or pick-block keymapping handling. It requires
`events.subscribe`, `events.cancel`, and `client.input.action_cancel`. Callbacks
receive only the action and hand, run synchronously on the client main thread,
and are capped at 64 per input event. An existing cancellation is preserved.

The `client.item.tooltip` event passes only a registry ID, stack count, and
durability values to a synchronous client callback. Appended output is bounded
literal text and requires the separate `client.item.tooltip.modify` grant.
Callbacks run on the client main thread because Minecraft consumes their
result during tooltip construction; keep handlers short and avoid unrelated
work in them. The host never exposes an `ItemStack`, components, or item NBT
through this event.

Outgoing client chat can be vetoed with the synchronous
`client.chat.send_attempt` callback. It receives the original plain text from
NeoForge's `ClientChatEvent`, requires the explicit `client.chat.send.observe`
grant to inspect that text, and requires `events.subscribe` plus `events.cancel`
to subscribe and veto. SJML never rewrites the message; allowed messages
continue through Minecraft's regular signed-chat path. Callbacks run on the
client main thread, and nested chat sends from a guest callback are canceled to
prevent recursion or bypassing a veto.

`client.ui.screen` opens only a bounded vanilla screen for the owning physical-
client module. Literal text, ID syntax, button count, and payload size are
validated on both sides of the ABI; the screen exposes no Minecraft objects or
arbitrary widget callbacks. A button action is dispatched only to the module
that owns that screen, and module teardown closes only its screen instance.
The API refuses to replace another screen, does not pause integrated-server
ticks, and does not require `events.subscribe` for the direct action callback.
Schema-v2 forms add at most four labelled text fields with per-field UTF-8 byte
limits. The host rejects control characters and returns values only to the
owning client module after a button click; it does not send form values to the
server. A module must explicitly use a separate authorized networking API to
transmit any value elsewhere.
The body viewport's wheel scrolling is local-only and clamped; it produces no
callback or network traffic. Wheel input over form fields, buttons, or outside
the body viewport is left for the active control or Minecraft to handle.
Schema-v3 adds a local selection list with at most 32 options. Changing or
scrolling the list does not call guest code; the selected option ID is returned
only to the owning module when a form button is submitted. The list itself does
not send values to a server.

Task groups bound the number of queued and running operations per module and
bound each requested timeout. A timeout completes the operation with a timeout
error and requests cooperative cancellation. It cannot guarantee termination
of guest CPU work. A running uncooperative operation retains its admission slot
until it returns; the host must not treat timeout as proof that execution has
stopped. SJML does not rely on `Thread.interrupt()` to stop WebAssembly code.
Runtime shutdown may wait for an active guest invocation, so platform adapters
must perform lifecycle cleanup on a control executor rather than a latency-
sensitive game tick.

## Compiler cache

The NeoForge adapter stores Endive generated class archives under
`config/sjml/compiler-cache`. Each filename is scoped by the WASM SHA-256 digest,
Endive version, SJML ABI version, cache format, compiler options, Java target,
and Endive's own module key. Records are bounded, written through a private
temporary file and atomic rename, and authenticated with HMAC-SHA-256.
Truncated, oversized, or unauthenticated records are removed and treated as
cache misses. `/sjml profiler` reports aggregate cache hits, misses, writes,
authentication failures, malformed/oversized entries, and evictions for the
current runtime process.

The random 256-bit HMAC key is stored in a separate file. The cache and key
directories and files must have owner-only POSIX permissions or an ACL that
grants access only to the owner. If the filesystem cannot establish and verify
those permissions, the cache stays disabled and compilation proceeds without
disk reuse. Defaults cap the cache at 128 records, 64 MiB total, and 8 MiB per
record; hosts can choose stricter bounds.

Authentication protects cache integrity only while the private key remains
private. It does not protect against the local account owner or a compromised
game process, and cached bytecode is not encrypted.


## World access

World reads require the module's explicit `world.read` request and administrator
grant. `set_block`, `spawn_particle`, and `play_sound` additionally require `world.write`;
entity creation requires the separate `entity.spawn` grant. Static registration
binds authorized signatures but denies calls. Live handles store `world.read`
as a required capability and each operation validates it together with module
owner, execution context, kind and generation. Direct Minecraft references
remain host-side. `get` and `loaded` never load dimensions, and world access
runs only on the owning logical server thread. Clients cannot see server
worlds; pre-bind dedicated lookup/enumeration return empty results.

The grant is module-wide and exposes every loaded server dimension, including
modded dimensions; there is no per-dimension ACL. Each module is limited to 1,024
live world handles. Enumeration preflights quota and output capacity, returning
a full sorted snapshot or an error without leaving partial allocations.
`LevelEvent.Unload` excludes and revokes just that level before it closes;
module/runtime teardown closes the remaining registry. Reloads cannot revive
old handle generations. Block-state batch reads return detached registry IDs
and state text; they check build height and chunk availability without loading
chunks, and the batch payload is bounded to 1 MiB. Block updates, particle
emission, and sound playback require both world capabilities, use registered content, enforce
build-height and loaded-chunk checks before the operation, and never load
chunks. There is
no per-dimension or per-position ACL for `world.write`; grant it only to
modules trusted to edit any loaded dimension on that logical server.

`world::best_neighbor_signal` is a read-only vanilla query under `world.read`.
Before calling `Level.getBestNeighborSignal`, the host checks the query's
neighboring/direct-signal neighborhood with `hasChunk`; if any required chunk
is absent, the API returns `None` without loading it.

## Player sessions and effect application

Every `sjml_player_v1` import requires the manifest's `entity.read` request and
an administrator grant. Static registration binds signatures but denies calls.
Live handles require `entity.read` and validate module owner, execution context,
kind, slot, and generation. A stable login-session identity survives respawn
and dimension changes. Logout marks the UUID unavailable and revokes handles
before guest callbacks, so callbacks cannot reacquire the departing session.
Calls and enumeration run on the owning logical-server thread; clients and
pre-bind server contexts see an empty list. The per-module quota is 1,024, and
enumeration preflights quota and output capacity before allocating handles.
Snapshots expose only UUID, profile name, current dimension ID, and coordinates.
Effect application additionally requires `entity.effect.apply`, validates the
registered effect ID and bounded duration/amplifier, and resolves the current
player for the same login session on the server thread. It does not require
`world.read` and does not expose inventory or a mutable player reference.

Player item-cooldown reads use the live session's vanilla cooldown manager and
require `entity.read`; setting or clearing cooldowns also requires
`player.cooldown.write`. These imports are server-only, accept registered
non-air item IDs, and bound new cooldowns to 1–72,000 ticks.

Respawn-point reads require `entity.read`; set and clear additionally require
`player.respawn.write`. The host accepts only a loaded destination dimension,
valid build height, and an in-border position. It updates vanilla respawn
metadata without loading or checking the target chunk.

Teleportation additionally requires `player.teleport`. The host bounds every
coordinate and rotation, revalidates the live player session, and checks the
current dimension's build height, world border, and loaded chunk before moving
the player. It cannot transfer dimensions or load/generate chunks. Client and
static-registration calls are denied.

## Guest menus and action callbacks

`sjml_menu_v1.open` and `sjml_menu_v1.update` require both `entity.read` and
`menu.open`, a live
module-owned player handle, and the owning logical-server thread. The host
accepts only bounded slot, item-ID, count, title, label, and optional action-ID
data; it resolves IDs through the item registry and rejects unknown items or
counts above the registered stack limit. The menu contains detached display
stacks. Its server-side container ignores all item movement and quick-move
requests. Only a left-click on an explicitly marked action slot can call the
owner module's synchronous `menu.action` handler, and each module is limited to
eight callbacks per server tick. Updates can only replace an active SJML menu
whose opaque owner token matches the calling runtime module; they cannot modify
vanilla or another module's menu and are limited to eight replacements per
module per server tick. The callback requires a live event
subscription plus `entity.read` and `menu.open`; it does not load client classes
on dedicated servers and is unavailable during static registration or on
physical-client module instances.

Editable menus use a separate `sjml_editable_menu_v1` import. Opening and reading require `entity.read` and `menu.open`; transfer calls also require `menu.transfer`. Each menu declares one to 54 input/output slots. Inputs accept only explicitly allowlisted registered IDs, outputs reject player insertion, and a transfer preflights both endpoints before moving an existing item stack without changing its components. Each module is limited to 32 transfers per server tick. The menu returns remaining stacks to the player on close and drops only exact overflow remainders. It cannot create items or turn one item into another.

## Entity reads and attachments

`sjml_entity_v1.snapshot` requires the manifest's `entity.read` request and an
administrator grant. `loaded` is available when a module has `entity.read`,
`entity.data.read`, or `entity.data.write`, so a data-only module can obtain
opaque handles without receiving snapshot access. Enumeration exposes loaded
non-player entities across server dimensions; it does not require a world
handle or `world.read`. It scans only already-loaded server entities and never
requests a chunk. Calls run synchronously on the owning server thread. Client
modules and pre-bind server contexts receive an empty list. Static registration
binds authorized signatures but denies calls.

Entity handles are scoped to the active module and execution context, and each
snapshot validates capability, kind, owner, context, slot, and generation. A
handle is revoked when NeoForge removes its exact entity instance from a level,
when its world unloads, or when the module/runtime closes. This includes chunk
unload and dimension transfer; a later entity instance with the same UUID gets
a new handle. A module may hold at most 1,024 live entity handles. Enumeration
preflights both the quota and output capacity and returns a complete UUID-sorted
list or an error, never a partial list. Snapshots contain only UUID, registry
entity type ID, dimension ID, and coordinates; they expose no entity object,
NBT, or components.

`entity::within` requires both `entity.read` and `world.read`, a live
module-owned world handle, and the logical-server thread. The provider scans
only loaded entities in the selected level, excludes players, bounds the
radius to 64 blocks and results to 256 handles, and sorts by UUID. If the
result would exceed the limit, the query fails without allocating partial
handles. It never loads a chunk or dimension.

Mutating a living entity with a status effect additionally requires
`entity.effect.apply`. The host verifies both capabilities, the exact live
handle, server-thread ownership, a canonical registered effect ID, and bounded
duration/amplifier values. The API excludes players and cannot load chunks.

Removing an active effect instead requires `entity.effect.remove` with
`entity.read`; the grant is independent from `entity.effect.apply`. The host
uses the exact module-owned live handle, resolves only registered effect IDs,
and removes effects only from loaded living non-player entities on the server
thread. An absent effect is a successful no-op; client/static calls, players,
nonliving entities, and stale handles are rejected.

`world.schedule_block_tick` requires `world.read` plus `world.write`, checks
that the position is in build height and its chunk is already loaded, and
requires the requested registered block to match the current state. The host
queues a normal Minecraft scheduled tick and limits each module to 16 requests
per server tick; it never forces chunk loading.

`world::spawn_projectile` requires `world.read` and `projectile.spawn`, a live
module-owned world handle, and the server thread. It accepts only vanilla eggs
and snowballs, finite positions inside the build height and world border, and
velocity between 0.1 and 4 blocks per tick. Every intersected chunk must already
be loaded. A module may launch at most four projectiles per server tick across
all dimensions. Vanilla egg hatching and snowball Blaze damage remain enabled.

`world::spawn_item` requires both `world.read` and `world.write`. IDs resolve
only through the current item registry, stack count must fit the item's
registered stack limit, and only the default item stack is created; guests
cannot attach NBT or arbitrary data components. Position bounds, loaded chunks,
world border, and build height are checked before insertion. A per-module
16-spawn server-tick quota and finite 4-block/tick velocity limit bound the
operation. No chunk-loading accessor is used.


Typed block-entity persistence is restricted to a module's own registered block
entities and requires `block_entity.data.read`; writes/removals also require
`block_entity.data.write` and `world.write`. It stores only Boolean, i32, and
bounded printable string values, with 64 keys and 16 KiB per block entity and 64
write/removal attempts per module/server tick. Generic NBT snapshots redact the
private SJML data compound, so `block_entity.nbt.read` cannot bypass these
permissions. The API never loads chunks or writes another module's entity.

`player.interaction` is synchronous and may veto the current right-click
action. It requires both `events.subscribe` and `events.cancel`. Callbacks run
on the owning server thread and receive only detached identifiers/coordinates;
`events.cancel` also gates the existing `block.break` veto path.

`block.place_attempt` is another synchronous veto path and requires both event
capabilities. It receives detached placement identifiers and coordinates.
NeoForge restores the block snapshots when a handler vetoes; the successful
`block.place` notification remains unchanged. Any matching authorized veto
cancels the full placement, including a multi-place operation.

`block.fluid_place_attempt` uses the same two event capabilities and runs
synchronously on the logical-server thread. It receives only bounded registry
IDs and block positions, can veto the one `FluidPlaceBlockEvent`, and cannot
mutate the target state or stop later fluid flow.

`player.attack_attempt` uses the same two event capabilities and is dispatched
synchronously on the owning logical-server thread. It gives guest code detached
player/target IDs, target type and dimension IDs, and bounded target
coordinates; a true result cancels before the weapon hook and damage code. It
does not allow guest code to edit attack damage or any live event field.

`entity.death_attempt` requires the same two capabilities and runs synchronously
on the logical-server thread. A guest receives a detached identity and position
snapshot; returning `true` cancels the death transition and the host restores the
victim to at least one health so the entity does not remain in a zero-health
state. The ordinary `entity.death` notification is skipped for this vetoed
attempt. Invalid callbacks disable their module and fail open.

Discarding a loaded non-player entity requires both `entity.read` and the
separate `entity.remove` request/grant. The host performs the discard on the
logical-server thread and immediately revokes the module's handle. Static
registration links the authorized import but always returns
`PERMISSION_DENIED`; removing an entity cannot run during registry setup.

Entity attachments use separate `entity.data.read` and `entity.data.write`
requests/grants. The write grant covers both setting and removing byte values;
the read grant does not imply `entity.read`. State is stored under SJML's fixed
`sjml:entity_attachments` key in persistent entity NBT, divided by package,
module, and local key. Guest code cannot select the host namespace or exchange
Java objects. Keys are at most 128 lowercase UTF-8 bytes; each value is at most
16 KiB; a module may store 128 entries/64 KiB per entity, and aggregate SJML
state is capped at 1,024 entries/256 KiB per entity. Limits include key and
owner-name overhead and writes are atomic. These bytes follow entity save/load
and dimension transfer. They differ from per-save entity-scope storage, which
is a mod-level UUID-keyed database rather than entity NBT.

## Block inventory reads

Every `sjml_inventory_v1.read` import requires the module to request and receive
both `world.read` and `inventory.read`. The host validates the supplied world
handle, including owner, execution context, kind, generation, required world
permission, and server-thread affinity, before querying slots. Static
registration binds the signature but denies invocation. Capability lookup only
uses an already loaded chunk; inventory results are detached item registry IDs
and u32 counts/limits, without `ItemStack`, NBT, or arbitrary data components.
Reads are synchronous, limited to 4,096 slots and 1 MiB, and produce no partial
output. No capability or an unloaded position returns `None`; the API never
loads a chunk and cannot mutate a handler.

`sjml_inventory_v1.read_string_component` uses the same world and inventory
read grants and can address only bounded string components registered by the
calling module. `write_string_component` additionally requires the separate
`inventory.write` grant. Writes are restricted to loaded block entities that
implement `Container`; the host copies the stack and changes only that
module-owned component through `Container.setItem`. Writes are unsided and
address the whole container instead of using sided item-handler capabilities.
Both operations validate the world handle and run on
the server thread. Neither loads chunks or exposes a live `ItemStack` or item
handler to guest code. `inventory.write` has no per-block or per-item ACL, so
grant it only to modules trusted to mutate containers reachable through their
`world.read` handle.

`read_block_enchantment_level`, `set_block_enchantment_level`, and
`remove_block_enchantment` use the same world-handle and server-thread checks.
They address only a single existing stack in an already loaded `Container`,
reject unresolved random-loot containers, and never load a chunk. Mutations
copy the existing stack and commit only if the container retains its item,
count, and complete component set apart from the requested enchantment. These
operations inherit the broad `inventory.write` container authority; there is
no per-block or per-item ACL.

`inventory::read_stack` exposes only bounded detached fields for one slot in
an already-loaded item handler: registry ID, counts, slot limit, optional
durability, and a sanitized plain-text custom name. It requires
`world.read` plus `inventory.read`, runs on the owning server thread, and never
exposes NBT or arbitrary data components. `player_inventory::set_slot` instead
requires `entity.read` plus `inventory.write` and a live player session. It
accepts only a canonical registered item ID and a count valid for both the
item and target slot, creates a fresh default stack, and synchronizes through
the active menu. `player_inventory::set_damage` uses the same grant to change
durability on an existing damageable item, preserves its other components, and
rejects values that would break the item. Guest code cannot set arbitrary item
components or NBT.

`player_inventory::move_stack` uses the same live-session and server-thread boundary and requires
`entity.read` plus `inventory.write`. It moves an exact count only when the source has enough items
and the destination is empty or has the same item and complete component set. Item and slot limits
are preflighted; the host changes counts only, verifies both slots, and restores both originals on
a failed commit.

`player_inventory::set_item_custom_name` and `clear_item_custom_name` require
`entity.read` plus `inventory.write`. They copy an existing player stack and
change only its literal custom-name component, preserving its item, count,
durability, and other components before synchronizing the active menu. They do
not create stacks or expose arbitrary components.

Module-owned boolean item components follow the same boundary as integer
components: `content.register` creates only the caller's own component type,
and runtime block/player inventory reads and writes resolve that type through
the caller's package/module identity. Reads and writes have separate inventory
grants, reject empty slots, and copy existing stacks before changing one value.
The boolean ABI accepts only `0` and `1`; it does not expose arbitrary vanilla
components or NBT. See [boolean item components](BOOLEAN_ITEM_COMPONENTS.md).

Player attribute reads and writes accept a live server-player handle and the
existing attribute allowlist. Reads require `entity.read`; writes additionally
require `entity.attributes.write`. Every call checks the login session and
server thread, and base-value writes enforce the same per-attribute ranges as
non-player entities.

`entity::fire_ticks` reads a bounded effective duration from a live loaded
non-player entity and requires `entity.read`. `entity::set_fire_ticks` also
requires the separately granted `entity.fire.write` capability; both recheck
the live module-scoped handle and logical-server thread. Values are limited to
`0..=72_000`, and a negative vanilla extinguished sentinel is reported as zero.

`entity::custom_name` reads a bounded plain-text name from a live loaded non-player entity under `entity.read`. Set and clear calls also require `entity.custom_name.write`; the host builds `Component.literal` text, rejecting styling/control payloads and oversized names. Player names and arbitrary text components are outside this API.

`entity::is_glowing` and `entity::is_silent` read only two fixed vanilla flags
from live loaded non-player entities under `entity.read`. `set_glowing` also
requires `entity.glowing.write`; `set_silent` requires `entity.state.write`. The
host rejects client, static-registration, and stale-handle calls. These flags do
not expose general entity metadata or NBT.

`entity::is_ai_enabled` and `entity::set_ai_enabled` operate only on loaded
vanilla `Mob` instances. Reads require `entity.read`; writes additionally
require `entity.ai.control`. The setter changes only Minecraft's existing
no-AI flag on the logical-server thread; it does not expose goal selectors or
arbitrary guest AI code. A separate bounded `entity.navigate_to` operation
requires `entity.read` plus `entity.navigation.write`, validates a short target
in the current loaded dimension, and invokes the vanilla Mob path navigator
only after checking that the target corridor is loaded. It does not expose
path nodes or allow a guest callback from inside pathfinding.

`entity::target_snapshot` returns a bounded detached snapshot of the loaded Mob's current
non-player living target under `entity.read`; it does not create or leak an entity
handle. The original `entity::target` retains its module-scoped handle result.
Setting or clearing a target also requires `entity.target.write`; the Mob
and target must be live in the same loaded server dimension, and the target must
be alive and within 128 blocks. Target reads and writes run on the server thread,
do not load chunks, and do not expose the Mob's goal selector.

`entity::teleport` requires `entity.read` plus `entity.teleport` and accepts
only a live, unmounted non-player entity. It validates finite bounded
coordinates, the full destination bounding box, world border, build height,
and every chunk touched by that box before calling Minecraft's same-dimension
teleport path. Unloaded destinations fail without chunk loads or tickets. The
host rechecks the final position and loaded-level membership after the call.

`entity::equipment` reads the six fixed vanilla equipment slots of a live loaded non-player `LivingEntity` under `entity.read`. It returns detached canonical item ID/count/damage/max-damage values only; no NBT, data components, or mutable item handles cross the ABI. The call checks the live module-owned handle and owning server thread without loading chunks. See [loaded entity equipment](ENTITY_EQUIPMENT.md).

`entity.projectile_impact_attempt` runs synchronously on the logical server before a projectile applies its hit. It requires `events.subscribe` and `events.cancel`; callbacks receive detached projectile and hit data, and only a successful guest veto cancels the impact. Block hit details are read only from an already-loaded chunk.

`player.drop_attempt` is synchronous and cancellable only when the host can
restore the entire tossed stack to the player's inventory. It requires both
event capabilities. If the inventory is full, or a callback changes it so the
whole stack no longer fits, the original toss continues (with only any
unrestored remainder left in the world). The host never cancels a toss after a
partial restore, preventing item loss or duplication.

Food properties are declared during static registration under `content.register`. The host accepts bounded vanilla nutrition/saturation values and the `fast` / `always_eat` flags; it does not grant arbitrary item behavior or client-side item hooks.

Integer game-rule writes use a fixed four-rule allowlist with narrow ranges;
they require both `world.read` and `world.write`. `world::set_block_properties`
also requires both capabilities, an already-loaded block position, the exact
expected current block ID, and registered property values. It changes only
those properties and never loads chunks or replaces the block type.

Player selected-hotbar reads require `entity.read`; changing the selected index
also requires `inventory.write`, is limited to slots `0..=8`, and does not
change stack contents. Active-effect snapshots require `entity.read` and return
at most 64 detached records from a live loaded living entity. Scoreboard imports
require `scoreboard.read` or `scoreboard.write`, run only on the owning server
thread, and operate on existing objectives; writes cannot create objectives or
change read-only criteria. `player.sleep_attempt` requires both event grants and
can only replace an allowed sleep result with a denial; an existing vanilla or
other-mod denial is preserved.

Declared network rate limits are enforced by the host. On a server, inbound
client-to-server traffic is bucketed per authenticated player and channel, so
one player cannot spend another player's rate allowance. The host caps retained
peer buckets at 4096 per module, prunes idle entries only after their burst has
had time to refill, and clears them on module close. Valid over-limit packets
are dropped before guest callbacks; they do not bypass the channel direction,
schema, session, or payload-size checks.
