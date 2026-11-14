# NeoForge Bootstrap and Diagnostics

SJML is a regular JavaFML mod with mod id `sjml`. NeoForge does not treat
`.sjml` files as FML containers, and SJML does not register virtual SJML
packages as NeoForge mods.

## Discovery phase

The common `SjmlMod` entrypoint reads `FMLPaths.MODSDIR` during its constructor,
before NeoForge registry events. It considers direct child files with a
`.sjml` extension, in deterministic filename order. It does not recursively
scan arbitrary directories or follow archive symlinks. Each candidate is sent
through `SjmlPackageReader`, which validates ZIP bounds and paths, strict TOML
manifest contents, declared WASM entries, and declared checksums without
extracting files.

Install each `.sjml` archive directly in Minecraft's `mods/` directory,
alongside ordinary `.jar` mods. A nested folder is not scanned.

The scan is bounded to 256 candidate archives, 1 GiB of compressed package
input, and 512 MiB of retained uncompressed package data by default, as well as
the per-archive limits in [the format documentation](FORMAT.md). The report
keeps one result per scanned archive and retains validated package bytes for
later bootstrap stages. Candidate overflow and directory access problems are
reported separately.

`VALIDATED` is the archive-scan result: the archive and manifest passed package
validation and exact module bytes remain retained. Before registry events,
SJML separately resolves packages for client and logical-server contexts,
compiles selected modules, and invokes an exported `sjml_v1_mod_register` once
per module identity. A registration failure rolls back that package's staged
items, blocks, fluids, and recipes. Static declarations include items, blocks,
block entities, entities, fluids, vanilla shaped, shapeless, and cooking recipes, guest-defined callback
recipe serializers, effects, sounds,
tabs, and string data components. Recipe JSON is supplied by an
always-enabled generated server data pack and loaded through Minecraft's
vanilla and guest-defined recipe serializers on resource reload.
Fluid and block registry events can arrive in either order; fluid buckets are
emitted as items only after both have completed.
After registry events, `FMLCommonSetupEvent.enqueueWork`
creates and retains one compiled instance per module and execution context,
then invokes `sjml_v1_mod_init` when present. Failed initialization closes
that package's context-specific instances and prevents required dependent
packages from activating. Non-cancellable server/player/entity lifecycle
notifications and synchronous cancellable block-break/place, player
interaction/drop, entity death/damage/target, projectile-impact, and explosion
attempt callbacks are available through explicit runtime subscriptions. The
`player.advancement_earned` notification reports completed vanilla progress.
An event macro only exports its callback; a module must call
`events::subscribe` from its retained server instance. Subscribed handlers run
on the owning logical-server thread in priority order. Cancellable callbacks
must remain synchronous so they can affect the originating game operation.
Manifest commands are wired under `/sjml guest`. With requested and granted
`world.read`, modules can look up/enumerate loaded server dimensions and read
dimension ID/game time. The logical server is bound before integrated activation
and before server-start callbacks; dedicated early initialization sees no worlds.
Each module owns a 1,024-entry world handle registry. `LevelEvent.Unload` revokes
only that level, and module/server close revokes the remaining handles.
With requested and granted `entity.read`, modules can enumerate online players
and read schema-v1 identity, current dimension, and coordinate snapshots.
Player handles are module- and login-session-scoped, survive respawn and
dimension changes, and are revoked before the `player.left` callback. Logout
exclusion prevents callbacks from reacquiring the departing session. Each
module has a 1,024-player handle quota; client and pre-bind enumeration is empty.
The same `entity.read` grant also enables `entity::loaded` and
`entity::snapshot` for non-player entities across all loaded server dimensions.
Entity handles are tied to one live instance and are revoked on
`EntityLeaveLevelEvent` (including chunk unload and dimension transfer), level
unload, or module/runtime close. Enumeration is UUID-sorted, complete or an
error at the 1,024-handle quota, and does not load chunks. Client and pre-bind
enumeration is empty.
With both `world.read` and `inventory.read`, modules can query the block
item-handler capability at a position in an already loaded dimension and read
bounded detached slot snapshots. Queries do not load chunks or expose live item
stacks, and run on the owning server thread.
Rejected scan packages retain their individual diagnostic.

## NeoForge hooks

- `SjmlMod` performs archive discovery in the mod constructor, before registry
  registration events can freeze static content.
- `PackageContentRegistrationService` resolves side-specific dependencies and
  invokes static registration callbacks before the registry events.
  `ContentRegistrationStaging` submits blocks and items from their matching
  `RegisterEvent`; generated `BlockItem` instances reference the registered
  block object.
- Registry imports require `content.register` in the manifest and an
  administrator grant in `config/sjml-permissions.toml`. A missing file grants
  no capabilities. For example:

  ```toml
  [[grant]]
  module = "ruby_pack/content"
  capabilities = ["content.register"]
  ```
- Simple guest particle types register through the matching NeoForge registry
  event. SJML attaches a shared client renderer and an always-enabled generated
  client resource pack that points at the vanilla `generic_0` sprite.
- Validated `.sjml` archives can provide client assets under their own
  `assets/<package-id>/` namespace. SJML mounts those files from a generated
  resource pack, including `sounds.json` and OGG files for registered sounds,
  particle atlas PNGs, and colored/textured entity and block-entity model
  textures.
- `FMLCommonSetupEvent.enqueueWork` activates retained WASM instances after
  registries are complete, keeping callbacks out of parallel mod setup
  dispatch. The physical-client context starts there; each physical server
  context owns its own activation.
- `ServerStartingEvent` creates a fresh integrated-server activation for every
  world session. SJML tracks joined players independently for each logical
  server context. `ServerStoppedEvent` closes that session's modules after its
  stopping callbacks have drained remaining players, so a later world gets
  fresh WASM memories and runs `mod_init` again.
- `ServerStartedEvent` dispatches subscribed handlers for the unit-valued
  `sjml_v1_event_server.started` export synchronously on the server thread.
  The module must request `events.subscribe` and receive an administrator grant.
  This event is non-cancellable; a callback failure disables only that module.
- `ServerStoppingEvent` dispatches subscribed handlers for the unit-valued
  `sjml_v1_event_server.stopping` export synchronously. An integrated-server
  player's logout callback can occur before this event. Immediately after
  `server.stopping`, SJML dispatches `player.left` for each still-tracked player
  and drains that context's player tracker; later logout callbacks for those
  players are suppressed. The activation remains alive until `ServerStoppedEvent`
  closes the matching logical-server runtime.
- `PlayerLoggedInEvent` dispatches subscribed handlers for
  `sjml_v1_event_player.joined` synchronously
  on the owning server thread. Its schema-v1 payload contains the player's
  canonical UUID string followed by the profile name as stable UTF-8 strings;
  the callback cannot access a Minecraft player object. It uses the same
  manifest request and administrator grant as `server.started`, is
  non-cancellable, and disables only the failing module.
- `PlayerLoggedOutEvent` dispatches subscribed handlers for
  `sjml_v1_event_player.left` synchronously
  on the owning server thread for a player whose join is still tracked, with
  the same bounded identity record and capability checks as `player.joined`.
  The callback cannot access a Minecraft player object; a failure disables
  only that module. Tracking prevents duplicate notifications when an
  integrated-server logout precedes `server.stopping` or a later logout follows
  shutdown draining.
- `LivingDeathEvent` dispatches `sjml_v1_event_entity.death` synchronously on
  the owning server thread. Its schema-v1 `EntityDeath` snapshot contains the
  entity UUID, registered type ID, dimension ID, and finite `f64` position
  coordinates. The guest callback is a notification and cannot cancel the
  NeoForge event; only modules with the requested and granted
  `events.subscribe` capability receive it.
- `AdvancementEvent.AdvancementEarnEvent` reports an earned advancement through
  `player.advancement_earned` after Minecraft awards it. Its detached payload
  contains player identity, dimension, advancement ID, and display text. The
  notification requires `events.subscribe`; it cannot grant, revoke, or undo
  advancement progress.
- `ExplosionEvent.Start` dispatches `world.explosion_attempt` on the owning
  server thread before explosion effects are applied. A callback can veto the
  explosion only with both `events.subscribe` and `events.cancel`. Its bounded
  payload contains source identity when present, dimension, center, radius, and
  block-interaction mode; guest code cannot reshape the explosion.
- `EntityTickEvent.Post` checks active effects on server-side living entities.
  Effects declared with `registry::register_effect_tick` dispatch an
  `EffectTickEvent` after the configured interval on the owning server thread.
  The module must separately subscribe to `entity.effect.tick` and have the
  requested and granted `events.subscribe` capability. Intervals are bounded
  to 1–1,200 entity ticks.
- `BlockEvent.BreakEvent` dispatches subscribed handlers for
  `sjml_v1_event_block.break` on the owning
  server thread with a schema-v1 snapshot: player UUID, dimension ID, block
  ID, and signed block coordinates. The module must request and receive both
  `events.subscribe` and `events.cancel`; notification subscriptions alone do
  not grant veto authority. A return must be exactly one stable boolean byte.
  Any authorized `true` vetoes the break; an invalid result or callback error
  disables only that module and does not veto. SJML only sets the canceled
  state to `true`, so cancellation from another handler is preserved.
  To grant both capabilities, use a single record scoped to the module:

  ```toml
  [[grant]]
  module = "ruby_pack/content"
  capabilities = ["events.subscribe", "events.cancel"]
  ```
- The event import module is `sjml_events_v1`. `subscribe(event_ptr, event_len,
  priority, handle_output_ptr) -> i32` registers one exported callback and writes
  a 24-byte handle; `unsubscribe(handle_ptr, handle_len) -> i32` removes it and
  requires a 24-byte handle. Both return zero on success or a stable ABI error
  code. Handles are scoped to the module instance and execution context and
  cannot be transferred. Priorities are `0=Highest`, `1=High`, `2=Normal`,
  `3=Low`, and `4=Lowest`; lower values run first, and ties preserve module
  activation order and registration order. An active module instance can hold
  up to 256 subscriptions.
- Subscription imports are exposed only when `events.subscribe` is requested
  in the module manifest and granted by the administrator. They can be called
  only from integrated-server or dedicated-server contexts; a physical-client
  instance of a `side = "both"` module cannot subscribe. Cancellable event
  subscriptions also require the separate `events.cancel` request and grant.
  Callback failures disable only the affected module and do not veto. SDK
  `#[event_async]` callbacks can queue work only for
  non-cancellable events; they return at the ABI boundary and use the existing
  bounded task runtime.
- `ServerStoppedEvent` closes only the matching logical-server runtime and
  invokes its initialized modules' `mod_shutdown` callbacks in reverse load
  order. The JVM shutdown hook closes only the physical-client runtime; it does
  not close integrated- or dedicated-server activations ahead of their
  `ServerStoppedEvent` callbacks.
- `RegisterCommandsEvent` installs `/sjml status` on the NeoForge game event
  bus. Permission level 2 is required. The command lists up to 16 individual
  package results; full diagnostics for all scanned packages remain in the
  startup log.
- `SjmlModClient` is annotated for the physical client distribution and only
  subscribes to `FMLClientSetupEvent`. The common entrypoint does not reference
  Minecraft client classes.

The registration callback compiles and closes its temporary instance after
staging. `mod_init` runs in the later retained runtime instance. `side = both`
registrations are staged once on a physical client even though the client and
integrated-server runtime contexts receive separate instances. Stopping an
integrated server closes its own instances without closing the physical-client
instances.

## Packaging

The development run classpath includes `sjml-core` and Endive runtime
dependencies. The distributable NeoForge mod uses ModDevGradle Jar-in-Jar for
the core and non-mod runtime dependencies. Build output should be checked for
`META-INF/jarjar/metadata.json` and nested dependency jars after changing
runtime dependency declarations.
