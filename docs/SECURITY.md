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

The client-only `client.input` capability exposes detached key and mouse-button
notifications on the client main thread when no screen is open. NeoForge does
not allow global key events to be canceled; only a mouse-button press can be
consumed. Input subscriptions are module-owned and removed when their runtime
closes.

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

## Guest menus and action callbacks

`sjml_menu_v1.open` requires both `entity.read` and `menu.open`, a live
module-owned player handle, and the owning logical-server thread. The host
accepts only bounded slot, item-ID, count, title, label, and optional action-ID
data; it resolves IDs through the item registry and rejects unknown items or
counts above the registered stack limit. The menu contains detached display
stacks. Its server-side container ignores all item movement and quick-move
requests. Only a left-click on an explicitly marked action slot can call the
owner module's synchronous `menu.action` handler, and each module is limited to
eight callbacks per server tick. The callback requires a live event
subscription plus `entity.read` and `menu.open`; it does not load client classes
on dedicated servers and is unavailable during static registration or on
physical-client module instances.

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

Mutating a living entity with a status effect additionally requires
`entity.effect.apply`. The host verifies both capabilities, the exact live
handle, server-thread ownership, a canonical registered effect ID, and bounded
duration/amplifier values. The API excludes players and cannot load chunks.

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
