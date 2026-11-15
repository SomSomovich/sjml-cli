# Compatibility and supported runtime

## Host requirements

The current mod targets Java 21, Minecraft 1.21.1, and the NeoForge version
declared by `neo_version` in `gradle.properties`. Use the checked-in Gradle
wrapper. The Rust SDK workspace declares Rust 1.85 as its minimum toolchain and
supports the `wasm32-unknown-unknown` target. Guest crates must use the SJML
ABI; WASI modules and ambient filesystem, process, and socket APIs are not
supported.

SJML compiles WebAssembly with Endive to JVM bytecode and fails module
activation if compilation fails. It does not interpret a module as a fallback.
The host caps module size at 16 MiB and defined linear memory at 256 wasm32
pages. See [Security](SECURITY.md) for table and operation limits.

## Version compatibility

The current target is host `0.115.0`, Rust SDK/macros `0.113.0`, private CLI
`0.1.45`, and ABI `1.105.0`. ABI compatibility remains major-version based: host
0.115.0 accepts ABI `1.0.x` through `1.105.0`; it rejects later contracts it does
not know. Packages using these APIs declare ABI `^1.105.0` and host range
`^0.115.0`. The private CLI version is documented for the Codex workspace; no
crates.io publication is implied.

Package manifests use SemVer ranges for `abi`, `minecraft`, and `sjml`; those
ranges are checked during package validation and activation.

### APIs added in ABI 1.105.0

This version adds seven synchronous cancellable logical-server callbacks
(`item.expire_attempt`, `entity.enderman_anger_attempt`,
`entity.swap_items_attempt`, `player.arrow_nock_attempt`,
`player.arrow_loose_attempt`, `block.note_block_attempt`, and
`player.spawn_phantoms_attempt`) plus three observation notifications
(`player.anvil_repair`, `world.sleep_finished_time`, and `world.tags_updated`).
All payloads are bounded schema-v1 records; the vetoes reject asynchronous
handlers.

### APIs added in ABI 1.104.0

This version adds seven synchronous cancellable logical-server callbacks
(`entity.travel_dimension_attempt`, `entity.finalize_spawn_attempt`,
`entity.mob_griefing_attempt`, `entity.despawn_attempt`,
`player.critical_hit_attempt`, `block.brew_attempt`, and
`player.sweep_attack_attempt`) plus five observation notifications
(`player.container_open`, `player.container_close`, `player.wake_up`,
`world.chunk_watch`, and `world.chunk_unwatch`). Container notifications also
require `player.container.observe`. All payloads are bounded schema-v1 records;
the vetoes reject asynchronous handlers.

### APIs added in ABI 1.103.0

This version adds the synchronous cancellable `player.command_attempt` and
`entity.mob_split_attempt` callbacks. Both use bounded detached payloads,
require both event capabilities, and reject asynchronous handlers.

### APIs added in ABI 1.102.0

This version adds the synchronous cancellable `block.piston_attempt` and
`player.anvil_update_attempt` callbacks. Both use bounded detached payloads,
require both event capabilities, and reject asynchronous handlers.

### APIs added in ABI 1.101.0

This version adds the synchronous cancellable `entity.lightning_strike_attempt`
veto and `world::set_block_state` under `world.write`, which parses full vanilla
block states without NBT at loaded positions and applies at most 32 changes per
tick.

### APIs added in ABI 1.100.0

This version adds the synchronous cancellable `sound.play_attempt` callback,
which can suppress a logical-server level sound after a subscription precheck,
and `player.spawn_point_attempt`, which can veto a respawn-point change. Both
use bounded schema-v1 payloads and require both event capabilities.

### APIs added in ABI 1.99.0

This version adds the synchronous cancellable `entity.convert_attempt`
callback for living-entity replacement attempts and the bounded
`world::destroy_block` operation under `world.write`, which destroys one loaded
non-air block with optional vanilla drops and a 32-block-per-tick quota. Both
contracts preflight only already-loaded chunks and fail closed on client and
static-registration contexts.

### APIs added in ABI 1.98.0

This version adds synchronous cancellable `player.chat_attempt`,
`entity.tame_attempt`, and `entity.experience_drop_attempt` server callbacks
plus `world::create_explosion` and `world::strike_lightning` behind the
separate `world.explosion` and `world.lightning` capabilities. Explosions and
lightning are limited per module per tick, preflight only already-loaded
chunks, and are denied to client and static-registration contexts. The maximum
combined server and client command declarations per module rises from 128 to
256. See the focused guides and the coverage matrix for exact bounds.

### APIs added in ABI 1.97.0

This version adds persistent physical-client fog overrides, the observation-only
`player.item_consumed` event, a typed furnace-fuel data-map declaration for
same-module items, and loaded-Mob navigation snapshots and path cancellation.
The API bounds and lifecycle checks are documented in the focused guides and
the coverage matrix.

### APIs added in ABI 1.96.0

This version adds age reads/writes for loaded ageable Mobs, live module-owned
villager-offer CRUD, bounded fluid-type properties and placed ore worldgen,
batch particle spawning, Arrow and Spectral Arrow spawning, post-application
enchantment notifications, cancellable random fire-spread attempts, and
physical-client discovery of package-owned asset paths. These operations stay
bounded and use their documented registration phase, side, ownership, and
capability checks. See the focused guides and API coverage matrix for limits.

### APIs added in ABI 1.95.0

This version adds exact-state block piston reactions, fire-spread speed and
flammability; an explicit safe `suppress_drops` block-update option; registered
item repair tags and per-stack player-inventory glint overrides; bounded
non-player entity orientation, equipment, and equipment-drop-chance controls;
raw physical-client cursor motion; and observational client sound-start events.
All mutations retain module ownership, live-handle, registration-phase,
side/thread, and existing capability checks. See the API coverage matrix and
focused guides for the exact contracts.

### APIs added in ABI 1.94.0

This version adds player ability writes and living-mob persistence/loot-pickup
controls; loaded-only chunk lifecycle notifications; client resource-reload and
local chat-history APIs; observed world-save and entity-left-level events;
bounded reads and changes to live villager offers; and player statistic-change
notifications. The client APIs stay on the physical client, world notifications
do not load chunks, and mutating operations validate side, thread, ownership,
capability, and live entity state. See the focused API guides and coverage
matrix for their exact limits.

### APIs added in ABI 1.93.0

This version adds loaded-only block collision boxes and fluid-flow vectors;
detached world bounds for loaded non-player entities; living entity air-supply
reads and guarded writes; player air-supply and movement-state snapshots,
guarded air writes, and fall-distance reset; separate no-gravity and
invulnerability controls; and static fire-resistance and no-repair properties
for module-owned items. The world reads never load chunks. Player and entity
mutations require separate capabilities and validate the current live handle.
See [block collision boxes](BLOCK_COLLISION_BOXES.md), [fluid flow](FLUID_FLOW.md),
[entity bounds](ENTITY_BOUNDS.md), [entity air supply](ENTITY_AIR_SUPPLY.md),
[entity state writes](ENTITY_STATE_WRITES.md), [player air supply](PLAYER_AIR_SUPPLY.md),
[player movement state](PLAYER_MOVEMENT_STATE.md), [fall-distance reset](PLAYER_FALL_DISTANCE.md),
[item fire resistance](ITEM_FIRE_RESISTANCE.md), and [item no-repair](ITEM_NO_REPAIR.md).

### APIs added in ABI 1.92.0

This version adds loaded-only biome climate and block destroy-speed reads, a
detached entity movement/visibility snapshot, synchronous player break-speed
callbacks, bounded text read/write access for vanilla signs, registered-item
glint defaults, exact-state static redstone outputs, an exact-type look-at goal,
and passive screen progress bars. The client also gains an active-screen mouse
drag callback. See [biome climate](BIOME_CLIMATE.md),
[block destroy speed](BLOCK_DESTROY_SPEED.md), [entity state flags](ENTITY_STATE_FLAGS.md),
[player break speed](PLAYER_BREAK_SPEED.md), [sign text](SIGN_TEXT.md),
[item glint](ITEM_GLINT_OVERRIDE.md), [redstone outputs](BLOCK_REDSTONE_OUTPUTS.md),
[entity AI goals](ENTITY_AI_GOALS.md), [client screens](CLIENT_SCREEN.md), and
[screen input](SCREEN_INPUT_MOUSE_RELEASE.md).

### APIs added in ABI 1.91.0

Modules can observe a broken item with `player.item_broken`, set a vanilla use
animation and bounded hold duration on their registered items, and register
finite bounded float data components for default item stacks and existing block
or player inventory stacks. Client modules can replace the complete definition
of their currently open owner-scoped screen. The screen ID and runtime owner
must match the active session; callback exports for the replacement are checked
before the update is applied. See [player item broken](PLAYER_ITEM_BROKEN.md),
[item use behavior](ITEM_USE_BEHAVIOR.md),
[float item components](FLOAT_ITEM_COMPONENTS.md), and
[interactive client screens](CLIENT_SCREEN.md).

### APIs added in ABI 1.90.0

`world.tick` reports each logical server level after its tick, including the
dimension ID, game time, and day time. It is non-cancellable and supports
subscriptions every 1–1,200 game ticks. It requires `events.subscribe` and
never loads chunks. See [world ticks](WORLD_TICK.md).

### APIs added in ABI 1.89.0

Registered SJML mobs can use `EntityAiGoal::AvoidEntityType` and
`EntityAiGoal::TargetEntityType` to avoid or select one fully qualified living
entity type. The host keeps vanilla pathfinding and target checks, then matches
the exact registered entity type. The new registration import is restricted to
`content.register` during `mod_register`; existing goal import versions are
unchanged. Unknown registry IDs are skipped with a warning when the mob's goals
are installed. Vanilla selectors only consider living entities, so non-living
registered types never match. See [registered entity AI goals](ENTITY_AI_GOALS.md).

### APIs added in ABI 1.87.0

`entity.mount_attempt` synchronously reports an entity mount or dismount before
the riding relation changes. A handler returning true cancels the attempt.
Its detached snapshot includes the acting entity's identity, dimension,
mount/dismount direction, and an optional target identity. Both event grants are
required, and async handlers are rejected. See
[entity mount attempts](ENTITY_MOUNT_ATTEMPTS.md).

### APIs added in ABI 1.86.0

Logical-server modules can now synchronously cancel eligible farmland trampling through `block.farmland_trample_attempt`. Its detached bounded snapshot includes the farmland position, dimension and block IDs, entity UUID/type, and fall distance. The callback uses both event capabilities; a veto preserves farmland and does not prevent the entity landing. `entity.fall_attempt` also allows a synchronous guest veto before a living entity takes fall damage; it reports detached identity, position, fall distance, and multiplier values without allowing them to be changed.

### APIs added in ABI 1.85.0

Three synchronous cancellable server callbacks are available to modules with
`events.subscribe` and `events.cancel`. `player.totem_use_attempt` runs before a
player's totem effect; cancellation prevents the save and leaves pending damage
in place. `entity.shield_block_attempt` is emitted only when the vanilla shield
eligibility check passed; cancellation disables that block and incoming damage
continues. `entity.teleport_attempt` covers the documented server
`EntityTeleportEvent` subclasses and captures source/target positions without
loading the destination chunk. Each callback runs on the owning server thread,
uses detached bounded values, and cannot be handled asynchronously. See
[player totem attempts](PLAYER_TOTEM_USE_ATTEMPT.md), [shield block attempts]
(ENTITY_SHIELD_BLOCK_ATTEMPT.md), and [entity teleport attempts]
(ENTITY_TELEPORT_ATTEMPT.md).

### APIs added in ABI 1.84.0

Registered blocks can declare a shape as the union of 1–8 normalized, block-local
AABBs. Coordinates must be finite and within 0..=1, with strictly ordered bounds;
the union provides selection, collision, occlusion, and support geometry. Shapes
are static and same-module registration is required. `entity::target_snapshot`
returns detached UUID/type/dimension/coordinate data for a loaded Mob's target;
it returns `None` if the target is absent or a player and never creates a handle.
The legacy `entity::target` still returns its `Option<EntityHandle>`. The
synchronous logical-server `player.potion_use_attempt` event covers only
`PotionItem` use before it starts, not other consumables or already-active use.
Its snapshot bounds count to 1..=64, use duration to 1..=72,000 ticks, item IDs
to 256 UTF-8 bytes, and dispatch to 64 authorized callbacks; async handlers
cannot veto it. Active-screen mouse-release callbacks run on the client main
thread, accept button IDs 0..=31, and can veto only the active screen's release
handler under `client.input.screen_cancel`; global releases remain
observational. Each API retains its side, thread, callback, payload, handle,
and capability limits. See [block shapes](WORLD_BLOCK_SHAPES.md),
[entity targets](ENTITY_TARGETS.md), [potion-use attempts](PLAYER_POTION_USE_ATTEMPT.md),
and [active-screen mouse releases](SCREEN_INPUT_MOUSE_RELEASE.md).

### APIs added in ABI 1.83.0

Loaded block inventories support bounded stack-count and durability changes;
player inventories support stack-count updates and transactional slot-to-slot
moves that preserve components. Loaded non-player entities can be teleported
without loading destination chunks, and their vanilla custom-name visibility
flag can be read or changed. Static registration can associate a vanilla
`SpawnEggItem` with a same-module SJML mob. The synchronous
`block.drops_attempt` event exposes bounded detached drop snapshots and can
suppress computed item/experience drops while leaving the actual block break
uncancelled. Player XP-point and level-change attempts can be synchronously
passed, cancelled, or bounded to a replacement amount. `player::send_rich_system_message`
builds literal component trees with bounded styles, hover text, and safe URL,
command-suggestion, and clipboard actions; arbitrary JSON and command execution
are not exposed. Each operation keeps its own capability, server-thread, live
handle, loaded-chunk, and payload checks. See [block drops](BLOCK_DROPS_ATTEMPT.md),
[block inventory count](BLOCK_INVENTORY_SET_COUNT.md), [block inventory durability](BLOCK_INVENTORY_DURABILITY.md),
[entity teleport](ENTITY_TELEPORT.md), [registered spawn eggs](ENTITY_SPAWN_EGGS.md),
[player inventory writes](PLAYER_INVENTORY_WRITES.md), [player XP change events](PLAYER_EXPERIENCE_CHANGE_ATTEMPT.md),
[rich system messages](PLAYER_RICH_SYSTEM_MESSAGES.md), and [entity custom-name visibility](ENTITY_CUSTOM_NAME_VISIBILITY.md).

### APIs added in ABI 1.82.0

`inventory::read_enchantment_level`, `set_enchantment_level`, and `remove_enchantment` operate on existing stacks in already-loaded block inventories. Reads require `world.read` and `inventory.read`; writes require `world.read` and `inventory.write`. The synchronous owner-routed `item.attack_attempt` event requires `events.subscribe` and `events.cancel` and applies only to the calling module's registered main-hand item. Villager trade declarations can set an initial demand from -1,000 to 1,000 through `content.register`; ABI 1.81 clients using the earlier registration import continue to register offers with demand 0. Entity-scoped storage now returns `STALE_HANDLE` for missing or unloaded entities and never loads their chunks. See [block/player enchantments](PLAYER_INVENTORY_ENCHANTMENTS.md), [item attack callbacks](ITEM_ATTACK_EVENTS.md), [villager offers](VILLAGER_TRADES.md), and [storage scopes](STORAGE.md).

### New permissions and block-behavior schema in ABI 1.80.0

Fluid tick scheduling requires the additional `world.fluid_tick.schedule` grant
alongside `world.read` and `world.write`; requests are limited to an already
loaded chunk. Exact-state block behavior payload schema 2 adds an optional
friction value while the host continues to accept schema 1. Cursor movement uses
the existing `client.input.observe` grant and is sampled once per client tick; it
is not a raw mouse-move event.

### Entity glowing permission migration for ABI 1.79.0

`entity::set_glowing` now requires `entity.read` and the dedicated
`entity.glowing.write` capability. `entity.state.write` continues to authorize
`entity::set_silent` and `entity::set_no_gravity`; invulnerability changes use
the separate `entity.invulnerability.write` capability. Update both the
module's manifest and the server's `config/sjml-permissions.toml` grant before upgrading a package that calls
`set_glowing`; the host will not bind that import without the new permission.
Read operations are unchanged. See [entity state flags](ENTITY_STATE_FLAGS.md),
[entity state writes](ENTITY_STATE_WRITES.md), and [project capability
grants](PROJECT_CONFIGURATION.md#capability-requests-and-server-grants).

## Logical side and current API coverage

Server-side world, player, entity, inventory, block-entity, event, storage,
network, command, async service, and server-menu operations run against their
owning integrated or dedicated logical server and enforce the corresponding
thread and permission checks. Save, dimension, player, entity, and block-entity
storage scopes are available to logical-server modules; physical-client modules
have a separate per-mod `client_profile` scope. Opaque non-player entity bytes
use separate `entity.data.read` and `entity.data.write` grants and are stored in
the entity's persistent NBT. World reads do not load dimensions or chunks;
block-state batches are limited to 1,024 positions and 1 MiB of encoded output.
Physical-client commands use the local command dispatcher; dynamic argument
suggestions require `client.command.suggest`, run on the physical-client main
thread, and receive only bounded local text and prior arguments without server
source or world access. They do not change server command trees or state.
Physical-client modules can also open owner-scoped literal screens and bounded
text-entry forms with `client.ui.screen`; form values return only through the
owning module's direct callback. Long body text has local, clamped mouse-wheel
scrolling inside its viewport; it emits no callback or network traffic and
does not consume wheel input over fields, buttons, or outside the body. This
API now also supports one bounded selection list per form. Browsing and changing
the selection remain local; the chosen ID returns with text values only when a
form button is clicked. Schema-v3 list forms are additive; schema-v1 button-only
screens and schema-v2 text-only forms remain compatible. This does not replace
a general widget-tree or custom-render API, and still requires a display-backed
client check.

Rust SDK 0.42.0 adds generic native chat panel themes under ABI 1.34.0 and keeps
the exclusive geometry controller under `client.chat.modify`; local layout and
theme persistence use the existing client-profile storage API.
[Chat documentation](CHAT.md) defines the bounds and lifetime. Native chat
wrapping, hit testing, input, and suggestions still require live-client
verification.

SDK 0.42.0 also adds a bounded server-authoritative `player.moved` notification
and an atomic slot-to-slot block inventory transfer. See
[movement event](PLAYER_MOVED_EVENT.md) and the inventory API documentation for
their thresholds, limits, permissions, and tests.

Host 0.45.0 / ABI 1.35.0 adds bounded player teleportation, non-player entity
removal, loaded-world weather reads and writes, and one-input vanilla
stonecutting recipes. Teleports remain in the player's current dimension and
reject unloaded chunks; entity removal requires the separate `entity.remove`
grant. See the focused API documents for limits and tests.

Host 0.46.0 / ABI 1.36.0 adds read-only player health, hunger, and experience
snapshots; server-player game-mode access; biome lookup and daylight-cycle time
access for already-loaded worlds; and the non-cancellable `player.respawned`
notification. Player reads require
`entity.read`; changing daylight time requires `world.read` and `world.write`.
Changing a player's game mode additionally requires
`player.game_mode.write`. SDK 0.44.0 also exposes bounded player motion reads,
player velocity writes under `player.motion.write`, and vanilla smithing
transform recipe registration. See [player vitals](PLAYER_VITALS.md),
[player game mode](PLAYER_GAME_MODE.md), [world biomes](WORLD_BIOMES.md),
[world time](WORLD_TIME.md), [player motion](PLAYER_MOTION.md),
[smithing transform recipes](SMITHING_TRANSFORM.md), and
[player respawn events](PLAYER_RESPAWNED_EVENT.md).

Host 0.47.0 / ABI 1.37.0 adds detached six-slot player equipment snapshots,
read/write access to the vanilla `doDaylightCycle` gamerule, client-local chat
message observation with separately gated local display filtering, and bounded
client toast notifications. Equipment reads require `entity.read`; gamerule
reads and writes require `world.read` and both `world.read`/`world.write`;
message observation and filtering require `client.chat.observe` and the separate
`client.chat.filter`; toast display requires `client.ui.notify`. Client UI APIs
remain unavailable to dedicated-server and static-registration contexts. Chat
filtering only hides a local insertion after Minecraft's secure-chat checks; it
does not rewrite signed messages or affect server state. See [player
equipment](PLAYER_EQUIPMENT.md), [daylight-cycle gamerule](WORLD_DAYLIGHT_CYCLE.md),
[client chat messages](CLIENT_CHAT_MESSAGES.md), and [client toast
notifications](CLIENT_NOTIFICATIONS.md).

Host 0.48.0 / ABI 1.38.0 adds a detached player-abilities snapshot and a
guarded current-flight toggle under `player.abilities.write`, normal signed
client chat sending under `client.chat.send`, bounded client-local sound
playback under `client.audio.play`, and loaded-world border reads and setters
under `world.read`/`world.write`. Flight changes cannot grant `may_fly` or
change invulnerability/build permissions. Chat follows Minecraft's ordinary
signed send route and cannot dispatch commands or raw packets. Audio is local
to the physical client. See [player abilities](PLAYER_ABILITIES.md), [client
chat messages](CLIENT_CHAT_MESSAGES.md), [client audio](CLIENT_AUDIO.md), and
[world border](WORLD_BORDER.md).

Client modules have HUD, bounded typed OpenGL, camera-relative world-render,
active-screen input, native chat layout control, global no-screen input, local
chat-message and toast APIs
loaded through client-only integration. Guest entities and block entities can render bounded colored or
package-textured cuboid models with a host-evaluated sine rotation. Package
assets beneath `assets/<package-id>/` are mounted through an always-enabled
generated resource pack, including `sounds.json`/OGG files, particle sprites,
and model textures. Unit and host tests cover the render contracts, but no
display-backed Minecraft client run has verified visual behavior. On the
logical server, pure compute work can run in an isolated worker WASM instance
when `task.compute` is granted. Chest menus reject item movement, while
declared action slots dispatch subscribed guest callbacks. A connected
multiplayer handshake and payload session has also not been verified against a
live client and server. These limits are not implied to work by successful
compilation or unit tests.

## Checking a specific checkout

Run the Java suite, Rust tests, lint, and WebAssembly example build before
changing supported versions:

```sh
./gradlew :sjml-core:test
./gradlew --configuration-cache test build runGameTestServer
cargo fmt --manifest-path sjml-cli/Cargo.toml --all -- --check
cargo clippy --manifest-path sjml-cli/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo test --manifest-path sjml-cli/Cargo.toml --workspace --locked
cargo build --manifest-path sjml-cli/Cargo.toml -p sjml-sdk --example guest \
  --target wasm32-unknown-unknown --locked
```

The CI matrix runs these Java and Rust checks on Linux, Windows, and macOS.
Minecraft client, multiplayer, GPU, tick-rate, and frame-rate compatibility
still requires a live run on each platform being claimed.

Host 0.49.0 / ABI 1.39.0 adds observational global mouse-wheel input, read-only player-inventory snapshots, bounded live reads and base-value writes for six vanilla living-entity attributes, and loaded block-state property snapshots. These require `client.input`, `entity.read` plus `inventory.read`, `entity.read` plus `entity.attributes.write`, and `world.read`, respectively. SDK/macros 0.47.0 provides the Rust wrappers. See the focused API documents for each contract.

Host 0.50.0 / ABI 1.40.0 adds reads and writes for 18 allowlisted boolean
world gamerules, fixed-source generic damage for living non-player entities,
literal player action-bar messages, and declarative client keybindings. The
active-screen input ABI also supports scroll events, but dispatch is gated by
the separate `client.input.screen_scroll` grant to preserve compatibility
with earlier screen-input callbacks. Rust SDK/macros 0.48.0 supplies the
guest APIs. See the focused API documents and source-based coverage matrix for
their capability and runtime boundaries.

Host 0.51.0 / ABI 1.41.0 adds quota-limited scheduled block ticks that never
load chunks, live status-effect removal under its own capability, synchronous
vetoes for right-click block/entity/item interactions, and physical-client
cursor coordinates. These APIs are bound to the logical-server or physical-
client main thread and return bounded values rather than Minecraft objects.
The new scheduled-tick GameTest and Java/Rust boundary tests cover the
server-backed contracts; cursor positions and interaction vetoes still need a
visible-client and interactive-client run, respectively.

Host 0.52.0 / ABI 1.42.0 adds loaded non-player spatial queries, allowlisted
integer gamerule reads and writes, expected-block-guarded block-state property
updates, detached item-stack snapshots, player inventory slot writes, and
server-player attribute access. It also adds synchronous cancellation for
block placement attempts before the placed state is committed. These operations
use their existing narrow `world.read`, `world.write`, `entity.read`,
`entity.attributes.write`, `inventory.read`, `inventory.write`, and
`events.cancel` capabilities. A dedicated-server GameTest verifies the block
placement veto, inventory slot mutation, and the end-to-end paths for spatial
queries, integer gamerules, item snapshots, player attributes, and block-state
properties.

Host 0.53.0 / ABI 1.43.0 adds bounded durability updates to existing player
inventory stacks, loaded-chunk enumeration, nearest loaded-entity raycasts,
synchronous cancellable `entity.death_attempt`, and manifest-declared local
client commands. Durability uses `entity.read` plus `inventory.write`; chunk
queries use `world.read`; entity raycasts use `entity.read` plus `world.read`;
death cancellation requires `events.subscribe` and `events.cancel`; local
commands require a requested and locally granted `client.command`. The host
GameTests exercise the server-backed APIs; client command display and execution
still need a display-backed Minecraft client check.

Host 0.54.0 / ABI 1.44.0 / SDK 0.52.0 adds bounded static villager offers,
pre-mitigation cancellable `entity.damage_attempt`, cross-dimension server-player
teleport into an already-loaded chunk, and shared elapsed `game_time` writes.
These require `content.register`, `events.subscribe` plus `events.cancel`,
`entity.read` plus `player.teleport` plus `world.read`, and `world.read` plus
`world.write`, respectively. Focused host/SDK tests and GameTests exercise the
live trade table, damage veto, dimension transfer, and game-time mutation.

Host 0.55.0 / ABI 1.45.0 / SDK 0.53.0 adds local rich chat-message replacement
with bounded literal spans, formatting, hover text, and safe click actions.
Replacement requires `client.chat.observe` and the separately granted
`client.chat.rewrite`; local filtering remains behind `client.chat.filter`.
Legacy one-byte Show/Hide callbacks remain compatible. Host/SDK contract tests
cover bounds, permission revocation, action decoding, and subscriber ordering;
visible behavior still needs a display-backed Minecraft client.

Host 0.56.0 / ABI 1.46.0 / SDK 0.54.0 adds bounded registry-backed biome ID
enumeration, module-owned bounded integer components for existing block
inventory stacks, and cancellable `entity.target_attempt` events from
NeoForge's `LivingChangeTargetEvent`. The biome API never loads chunks;
component reads/writes require their separate inventory grants; target vetoes
require `events.subscribe` plus `events.cancel`. Host/SDK contract tests cover
the contracts, and a dedicated-server GameTest verifies a target change is
vetoed. See [biome IDs](WORLD_BIOME_IDS.md), [integer item components](INTEGER_ITEM_COMPONENTS.md),
and [entity target attempts](ENTITY_TARGET_ATTEMPTS.md).


Host 0.57.0 / ABI 1.47.0 / SDK 0.55.0 adds module-owned integer-component access on existing player inventory stacks, static item rarity declarations, bounded fire-tick access for loaded non-player entities, synchronous cancellable block-break attempts, observe-only global keyboard notices, active-screen key vetoes, module-owned client-loop sound handles, and literal player title/subtitle packets. Each operation uses its distinct capability and remains bounded to the owning module, active session, logical side, and required game thread. See [player inventory components](PLAYER_INVENTORY_COMPONENTS.md), [item rarity](ITEM_RARITY.md), [entity fire ticks](ENTITY_FIRE_TICKS.md), [block-break attempts](BLOCK_BREAK_ATTEMPTS.md), [client key events](CLIENT_KEY_EVENTS.md), [client audio](CLIENT_AUDIO.md), and [player titles](PLAYER_TITLES.md).


Host 0.58.0 / ABI 1.48.0 / SDK 0.56.0 adds inventory-safe cancellable player drop attempts, module-owned updates to its active read-only menu, bounded entity glowing/silent flags, and static item enchantment values. Drop vetoes require `events.subscribe` and `events.cancel` and are declined when the complete stack cannot be restored; menu updates require `entity.read` and `menu.open`; flag writes require `entity.state.write` plus `entity.read`; item declarations require `content.register`. See [drop attempts](PLAYER_DROP_ATTEMPTS.md), [menu updates](MENU_UPDATES.md), [entity state flags](ENTITY_STATE_FLAGS.md), and [item properties](ITEM_RARITY.md).

Host 0.59.0 / ABI 1.49.0 / SDK 0.57.0 adds cancellable projectile-impact attempts, literal custom names on loaded non-player entities, bounded food properties for registered items, and editable server menus. Menu slots use explicit item allowlists, transfer only existing stacks, preserve count and components, and return remaining stacks on close; each module is limited to 32 transfers per server tick. The new event uses `events.subscribe` and `events.cancel`; entity custom-name writes use `entity.read` and `entity.custom_name.write`; food declarations use `content.register`; editable-menu reads use `entity.read` and `menu.open`, while transfers also need `menu.transfer`. See [projectile impacts](PROJECTILE_IMPACT_ATTEMPT.md), [custom names](ENTITY_CUSTOM_NAMES.md), [food properties](ITEM_FOOD.md), and [editable menus](EDITABLE_MENUS.md).

Host 0.60.0 / ABI 1.50.0 / SDK 0.58.0 adds vanilla max durability for registered standalone items, server-side player item cooldown reads/writes, earned-advancement notifications, and cancellable explosion-start attempts. Durability declarations require `content.register` and a same-module item with stack size one. Cooldown reads require `entity.read`, while changes also require `player.cooldown.write` and accept 1–72,000 ticks. Advancement observation uses `events.subscribe`; explosion vetoes require both `events.subscribe` and `events.cancel` and run before damage or block changes. See [item durability](ITEM_DURABILITY.md), [player item cooldowns](PLAYER_COOLDOWNS.md), and [gameplay events](GAMEPLAY_EVENTS.md).

Host 0.61.0 / ABI 1.51.0 / SDK 0.59.0 adds server-side advancement grants/revocations and custom-stat reads/increments; difficulty and saved Overworld spawn-point control; cancellable entity heal, effect-application, and join attempts; bounded names on block-inventory item stacks and transfers; effects applied by registered food; independent X/Y/Z model animation; and exact client GUI-layer suppression. Host calls validate IDs and bounds, enforce dedicated permissions and side/thread checks, and avoid loading chunks. The dedicated-server GameTest suite passed all 70 required cases, including the new server paths. See [advancements](PLAYER_ADVANCEMENTS.md), [statistics](PLAYER_STATISTICS.md), [world difficulty](WORLD_DIFFICULTY.md), [spawn points](WORLD_SPAWN_POINT.md), [entity events](ENTITY_HEAL_ATTEMPTS.md), [inventory names](BLOCK_INVENTORY_ITEM_NAMES.md), [inventory transfers](BLOCK_INVENTORY_MOVE.md), [food effects](ITEM_FOOD_EFFECTS.md), [model animation](MODEL_ANIMATIONS.md), and [GUI-layer suppression](GUI_LAYER_SUPPRESSION.md).

Host 0.62.0 / ABI 1.52.0 / SDK 0.60.0 adds loaded-Mob AI toggles, player respawn-point reads and updates, literal custom names on existing player-inventory stacks, and a loaded-only vanilla neighboring-redstone-signal query. Respawn updates validate a loaded destination dimension and its bounds without loading its chunk. AI control requires `entity.read` and `entity.ai.control`; respawn writes require `entity.read` and `player.respawn.write`; player stack naming requires `entity.read` and `inventory.write`; redstone reads use `world.read`. The dedicated-server GameTest suite passed all 74 required cases. See [entity AI control](ENTITY_AI_CONTROL.md), [player inventory item names](PLAYER_INVENTORY_CUSTOM_NAMES.md), [player respawn points](PLAYER_RESPAWN_POINT.md), and [redstone signal reads](WORLD_REDSTONE_SIGNAL.md).

Host 0.63.0 / ABI 1.53.0 / SDK 0.61.0 adds selected hotbar-slot reads/writes, bounded active-effect snapshots for loaded living entities, reads and writes for entries in existing scoreboard objectives, and synchronous cancellable `player.sleep_attempt`. Hotbar changes require `entity.read` and `inventory.write`; effect snapshots require `entity.read`; scoreboard operations require `scoreboard.read` or `scoreboard.write` and never create objectives; sleep vetoes use NeoForge's `CanPlayerSleepEvent` result and preserve an existing vanilla or mod denial. The dedicated-server GameTest suite now covers these paths. See [player hotbar](PLAYER_HOTBAR.md), [active entity effects](ENTITY_ACTIVE_EFFECTS.md), [scoreboard](SCOREBOARD.md), and [sleep attempts](PLAYER_SLEEP_ATTEMPT.md).

Host 0.64.0 / ABI 1.54.0 / SDK 0.62.0 adds synchronous client item-tooltip callbacks. A client module receives only an item registry ID, count, and durability snapshot and may append bounded literal lines under `events.subscribe` and `client.item.tooltip.modify`. Server contexts cannot subscribe, and callbacks cannot mutate item data. Codec, adapter, subscription-context, SDK, and example tests pass; visible rendering still needs a display-backed client. See [client item-tooltip events](ITEM_TOOLTIP_EVENTS.md).

Host 0.65.0 / ABI 1.55.0 / SDK 0.63.0 adds `client.chat.send_attempt`, a physical-client callback for the original text about to be sent. A module may veto sending synchronously, but cannot rewrite the message. Reading text requires `client.chat.send.observe`; vetoing also requires `events.subscribe` and `events.cancel`. This leaves Minecraft's signed-chat path intact. Codec, adapter, permissions, client-side routing, and SDK tests pass; a display-backed client check remains outstanding. See [outgoing client chat veto](CLIENT_CHAT_SEND.md).

Host 0.66.0 / ABI 1.56.0 / SDK 0.64.0 adds bounded interactive client screens. A physical-client module with `client.ui.screen` can open a vanilla screen containing literal text and up to four buttons. Button actions route directly to the owning module's `client.screen.action` callback; they do not use global event subscriptions. The host rejects screens when another screen owns the display, keeps the screen non-pausing, and closes it when its module unloads. Codec, capability/context, screen ownership, cleanup, SDK, and example tests pass; visible layout and input still need a display-backed client. See [interactive client screens](CLIENT_SCREEN.md).

Host 0.67.0 / ABI 1.57.0 / SDK 0.65.0 adds bounded text-entry forms to client screens. A form has up to four labeled fields with per-field UTF-8 byte limits; button clicks close it and send the values only to the owning module through `client.screen.form_submit`. Existing schema-v1 screens and `client.screen.action` callbacks remain compatible. Form codecs, host routing, SDK serialization, and the example are covered by tests; visible layout and keyboard input still need a display-backed client. See [interactive client screens](CLIENT_SCREEN.md).

Host 0.67.1 keeps ABI 1.57.0 and adds local, clamped mouse-wheel scrolling for long body text in client screens. It preserves paragraph breaks, renders only visible wrapped lines, and does not consume wheel input over fields, buttons, or outside the body viewport. It emits no callback or network traffic. Headless viewport tests pass; live visual and wheel behavior still need a display-backed client. See [interactive client screens](CLIENT_SCREEN.md).

Host 0.68.0 / ABI 1.58.0 / SDK 0.66.0 adds schema-v3 client list forms with one bounded selection list of up to 32 options, alongside up to four text fields. List browsing and selection stay local; the selected ID is returned only to the owning module when a form button is clicked. Schema-v1 button-only screens and schema-v2 text-only forms remain compatible. Headless contract tests pass, but visible list layout, selection, and wheel interaction still need verification on a display-backed client. See [interactive client screens](CLIENT_SCREEN.md).

Host 0.69.0 / ABI 1.59.0 / SDK 0.67.0 adds schema-v4 checkbox settings
forms, module-owned boolean item components on existing block/player inventory
stacks, detached `world::fluid_at` snapshots, and persistent physical-client
FOV scaling. It also adds non-cancellable `player.tick` and `client.tick`
notifications with bounded per-subscription intervals. Declared network
channels accept optional `messages_per_second` and `burst` limits; these are
local per-module/channel/direction token buckets and do not change the wire
handshake. It adds the cancellable logical-server `player.item_use_attempt`
callback before `Item.use`, with a detached hand/item/count snapshot and no stack
mutation. Source-water, dry-block, light-level, unloaded-chunk, and build-height
GameTests are included. Headless tests do not verify visible FOV or screen
layout; those still need a display-backed client. See the relevant focused API
guides for contracts and bounds.

Host 0.70.0 / ABI 1.60.0 / SDK/macros 0.68.0 adds bounded input-state reads,
tracked client one-shot sounds, loaded-entity riding snapshots and guarded
non-forced mount/dismount operations, observational `entity.tick`, and loaded-only
world neighbor notifications. The server-side inbound network quota now uses
per-authenticated-player token buckets with a bounded bucket count. The CLI parser
accepts up to 128 combined server/client commands. Dedicated-server GameTests
cover entity riding, entity ticks, and neighbor updates; actual display-backed
input/audio and remote multiplayer behavior still need the user's Minecraft
client/server.

Host 0.71.0 / ABI 1.61.0 / SDK/macros 0.69.0 adds criterion-level advancement
progress snapshots and single-criterion grant/revoke, typed reads/increments
for vanilla block-mined, item-used/broken/crafted, and entity-killed/killed-by
statistics, and periodic keyframe rotations for registered entity and block-
entity models. Client settings forms accept up to eight checkboxes and return
up to thirteen bounded values to their owning module. Dedicated-server
GameTests exercise player APIs and model registration; headless checks do not
establish visible animated-model or screen behavior.

Host 0.72.0 / ABI 1.62.0 / SDK/macros 0.70.0 adds `player.item_crafted`, a
bounded recipe-aware notification for successful crafting, and expands live
attribute reads/base-value updates from six to all 31 built-in vanilla
attributes. Attribute writes retain per-ID value bounds and require
`entity.attributes.write`. The GameTest guest exercises player block
interaction range, while codec and host tests cover crafted recipe/output
snapshots; actual crafting interaction still needs the client/server assembly.

Host 0.73.0 / ABI 1.63.0 / SDK/macros 0.71.0 adds bounded dynamic server-command suggestions, same-module registered item-use callbacks, cropped package sprite rendering, vanilla brewing mixes and one-effect potions, player-inventory enchantment read/set/remove, exact tracked-sound fades and owner-checked client music selection, and module-owned scoreboard teams. It also adds synchronous cancellable `entity.breed_attempt`, `block.grow_attempt`, and `player.fished_attempt` events. Server GameTests cover the new event, brewing, inventory, and team contracts; client music, audio, and rendering still need display-backed verification. The CLI crate is 0.1.3 locally; no crates.io release or public-repository sync was made.

Host 0.74.0 / ABI 1.64.0 / SDK/macros 0.72.0 adds `client_localization::translate` for the active client language, bounded loaded-only `world::structures_at` queries, and the cancellable `block.fertilize_attempt` bonemeal callback. Focused Java/Rust tests and dedicated-server GameTests cover these contracts; client-language display behavior still needs a live client check. The CLI crate is 0.1.4 locally; no crates.io release or public-repository sync was made.

Host 0.75.0 / ABI 1.65.0 / SDK/macros 0.73.0 adds loaded-only `world::block_has_tag` queries and current-registry `world::item_has_tag` membership checks for canonical tag IDs. Block queries distinguish a loaded non-member from an unavailable position without loading chunks; item queries do not inspect inventory or load chunks. Codec, host import, Rust contract, and dedicated-server GameTests cover both contracts. The CLI crate is 0.1.5 locally; no crates.io release or public-repository sync was made.

Host 0.76.0 / ABI 1.66.0 / SDK/macros 0.74.0 adds sorted bounded enumeration of current block, item, fluid, and entity-type tag members through `world::tag_members` and typed convenience functions. Results distinguish absent tags from registered empty tags and cap at 1,024 entries; all calls require `world.read` and never access chunks. The private CLI crate is 0.1.6 locally; no crates.io publication or public-repository sync was made.

Host 0.77.0 / ABI 1.67.0 / SDK/macros 0.75.0 adds fixed full-cube, empty, and single-cuboid selection, collision, occlusion, and support shapes for registered blocks through `registry::register_block_with_shape` and `registry::set_block_shape`. Shapes are bounded, validated, and immutable after registration; GameTests exercise the selected and collision bounds. The private CLI crate is 0.1.7 locally; no crates.io publication or public-repository sync was made.

Host 0.78.0 / ABI 1.68.0 / SDK/macros 0.76.0 adds bounded static movement and physical behavior for registered blocks: friction, speed/jump factors, light emission, lava ignition, piston reaction, and allowlisted vanilla sound groups. `registry::register_block_with_behavior` and `registry::set_block_behavior` stage the settings during the block registration phase. Java boundary tests and a dedicated-server GameTest exercise the values. The private CLI crate is 0.1.8 locally; no crates.io publication or public-repository sync was made.

Host 0.79.0 / ABI 1.69.0 / SDK/macros 0.77.0 adds the synchronous,
module-owned `block.use` callback. It receives a bounded detached snapshot for
right-clicks on the module's registered blocks and can pass through, handle the
click, or consume one held item when its registry ID and count remain
unchanged. The GameTest server verifies live event routing and item consumption.
The private CLI crate is 0.1.9 locally; no crates.io publication or
public-repository sync was made.

Host 0.80.0 / ABI 1.70.0 / SDK/macros 0.78.0 adds the synchronous,
owner-scoped `item.use_on_block` callback. After the clicked block's own
`block.use` callback passes, the callback goes only to the module that declared
the held item; it receives a bounded detached snapshot and can pass through,
handle the click, or consume one unchanged held item before vanilla
`Item.useOn`. The dedicated-server GameTest checks real routing and
consumption. The private CLI crate is 0.1.10 locally; nothing was published to
crates.io or synchronized to the public CLI repository.

Host 0.81.0 / ABI 1.71.0 / SDK/macros 0.79.0 adds the synchronous, owner-scoped `item.use_on_entity` callback for registered items. It runs on the general `EntityInteract` hook only after the more specific `interactAt` stage passes and after the `player.interaction` veto; it can pass through, handle the interaction, or consume one unchanged item. The dedicated-server GameTest posts the NeoForge event bus hook with a live cow and registered item. The private CLI crate is 0.1.11 locally; nothing was published to crates.io or synchronized to the public CLI repository.


Host 0.82.0 / ABI 1.72.0 / SDK/macros 0.80.0 adds bounded static attribute modifiers for registered items. Modifiers support fully qualified vanilla or modded attribute IDs, package/module-owned IDs, vanilla operations, and all vanilla equipment-slot groups. Java boundary tests and a dedicated-server GameTest verify registration and the equipped player's live attack-damage value. The private CLI crate is 0.1.12 locally; nothing was published to crates.io or synchronized to the public CLI repository.

Host 0.83.0 / ABI 1.73.0 / SDK/macros 0.81.0 adds bounded Boolean and integer state properties to module-registered blocks. Properties are immutable after registry freeze; loaded-world reads and writes use the existing block-property APIs. Java tests check staging limits and rollback, and a dedicated-server GameTest exercises defaults, snapshots, mutation, and restoration. The private CLI crate is 0.1.13 locally; nothing was published to crates.io or synchronized to the public CLI repository.

Host 0.84.0 / ABI 1.74.0 / SDK/macros 0.82.0 adds exact-state geometry overrides for registered blocks with declared properties. Each override is a fixed full cube, empty shape, or one normalized cuboid and is selected from the immutable Minecraft block state without calling guest code at runtime. Codec, staging, rollback, and live GameTest coverage verify collision behavior. The private CLI crate is 0.1.14 locally; nothing was published to crates.io or synchronized to the public CLI repository.

Host 0.85.0 / ABI 1.75.0 / SDK/macros 0.83.0 adds module-owned vanilla boss bars for live players, detached non-player living-entity health snapshots with bounded healing, exact block-state emitted-light overrides, and cancellable `entity.knockback_attempt` vetoes. Each API has an explicit ABI boundary, capability checks, Rust wrapper, focused tests, and dedicated-server GameTest coverage. The `/sjml profiler` command also reports bounded inter-module service call, failure, elapsed-time, and queue metrics. The private CLI crate is 0.1.15 locally; nothing was published to crates.io or synchronized to the public CLI repository.


Host 0.86.0 / ABI 1.76.0 / SDK/macros 0.84.0 adds module-owned player tab-list contributions and dummy scoreboard objectives, completed-trade notifications, ordinary item spawning into loaded chunks, exact-state block behavior predicates and sound. It also splits the compatibility test guest to keep each export below the instruction-fuel limit. The integrated Gradle `--configuration-cache test build runGameTestServer` run passed 1,152 host tests, 434 core tests (one skipped), and all 125 required GameTests. The Cargo workspace passed 26 CLI, 43 macro, and 347 SDK tests plus doc tests and strict Clippy. The SDK and root, multi-module, world-render, and OpenGL-geometry example workspaces passed `wasm32-unknown-unknown` checks; those example workspaces also passed offline tests. The private CLI crate is 0.1.16 locally; this did not publish to crates.io or synchronize the public CLI repository.

Host 0.87.0 / ABI 1.77.0 / SDK/macros 0.85.0 adds bounded block update options, host-evaluated translation/scale model keyframes, registered-item grants to live player inventories with returned overflow, and exact-value player statistic setters. These APIs use explicit permissions, bounds, and typed host bindings. Gradle `--configuration-cache --offline test build runGameTestServer` passed 1,164 host tests, 434 core tests (one skipped), and all 129 required GameTests. The private CLI 0.1.17 workspace passed 26 CLI, 43 macro, 355 SDK unit, 157 SDK integration, and 12 documentation tests, strict Clippy, SDK/example WebAssembly checks, and offline tests for all four example workspaces. The private CLI crate is 0.1.17 locally; no crates.io publication or public CLI sync was made.

Host 0.88.0 / ABI 1.78.0 / SDK/macros 0.86.0 adds owner-scoped client-screen closing, guarded server player rotation writes, detached equipment snapshots for loaded non-player living entities, and static crafting remainders for registered standalone items. The APIs use explicit capability, context, owner, registry, and size checks. Gradle passed 1,176 host tests, 436 core tests (one skipped), and all 131 required dedicated-server GameTests; the CLI workspace passed 26 CLI, 43 macro, 358 SDK unit, 162 SDK integration, and 12 documentation tests plus strict Clippy and wasm32/example checks. This did not verify visible client behavior or a remote multiplayer session; no release was made to crates.io or the public CLI repository.

Host 0.89.0 / ABI 1.79.0 / SDK/macros 0.87.0 adds criterion-level advancement progress notifications, loaded-world game-event dispatch, and a separate permission for glowing-flag writes. It also completes item-pickup vetoes and plain-text player inventory lore with live GameTest coverage. The private CLI's generated-project template and examples now declare the matching versions. Gradle passed 1,190 host tests, 449 core tests (one skipped), and all 136 required GameTests. The private CLI 0.1.19 workspace passed 26 CLI, 45 macro, 363 SDK unit, 167 SDK integration, and 12 documentation tests; strict Clippy, SDK/example wasm32 checks, and offline tests for all example workspaces passed. Visible client behavior and remote multiplayer remain unverified; no release was made to crates.io or the public CLI repository.

Host 0.90.0 / ABI 1.80.0 / SDK/macros 0.88.0 adds per-state friction overrides for registered blocks, bounded loaded-only fluid-tick scheduling, and sampled physical-client cursor movement. The fluid scheduler requires `world.fluid_tick.schedule` and never loads chunks. NeoForge 21.1.265 has no raw global mouse-move event, so cursor deltas are sampled at most once per client tick and reset when the screen/window/cursor context changes.

Host 0.90.0 / ABI 1.80.0 / SDK/macros 0.88.0 adds per-state friction overrides for registered blocks, bounded loaded-only fluid-tick scheduling, and sampled physical-client cursor movement. The fluid scheduler requires `world.fluid_tick.schedule` and never loads chunks. NeoForge 21.1.265 has no raw global mouse-move event, so cursor deltas are sampled at most once per client tick and reset when the screen/window/cursor context changes. Gradle `--no-daemon --configuration-cache --offline test build runGameTestServer` passed 1,197 host tests and 454 core tests (one skipped), with all 137 required GameTests passing. Cargo passed 26 CLI, 45 macro, 365 SDK unit, 167 SDK integration, and 12 doc tests; strict Clippy, SDK wasm32 compilation, and native/Clippy/wasm32 checks for the root, multi-module, world-render, and OpenGL-geometry example workspaces passed. These headless checks do not verify visible cursor behavior or remote multiplayer; no crates.io publication or public CLI sync was made.

Host 0.93.0 / ABI 1.83.0 / SDK/macros 0.91.0 adds bounded stack-count and durability updates, component-preserving player inventory moves, loaded-only entity teleports, custom-name visibility, registered-mob spawn eggs, block-drop suppression, XP-point and level-change callbacks, and bounded rich system messages. Gradle `--no-daemon --configuration-cache --offline test build runGameTestServer` passed 1,274 host tests, 474 core tests (one skipped), and all 146 required GameTests. Cargo passed 26 CLI, 47 macro, 389 SDK unit, 172 SDK integration, and 12 documentation tests; strict workspace Clippy, SDK wasm32, and native test/Clippy/wasm32 checks for all four example workspaces passed. These headless checks do not verify visible client behavior or remote multiplayer; no crates.io publication or public CLI sync was made.

Host 0.94.0 / ABI 1.84.0 / SDK/macros 0.92.0 adds multi-box registered-block shapes, detached entity target snapshots, cancellable potion-use attempts, and active-screen mouse-release vetoes. Gradle `--no-daemon --configuration-cache --offline test build runGameTestServer` passed 1,286 host tests, 478 core tests (one skipped), and all 147 required GameTests. Cargo passed 26 CLI, 47 macro, 394 SDK unit, 172 SDK integration, and 12 documentation tests; strict workspace Clippy, SDK wasm32, and native test/Clippy/wasm32 checks for all four example workspaces passed. These headless checks do not verify visible mouse-release behavior or remote multiplayer; no crates.io publication or public CLI sync was made.

- On 2026-11-13, host 0.95.0 / ABI 1.85.0 / SDK/macros 0.93.0 / private CLI 0.1.25 passed Gradle `--no-daemon --configuration-cache --offline test build runGameTestServer`: 1,295 host tests, 489 core tests (one skipped), and all 150 required GameTests passed. The live guest exercised totem-use cancellation, shield-block veto, and teleport veto. Cargo passed 26 CLI, 49 macro, 401 SDK unit, 175 SDK integration, and 12 documentation tests (663 total); strict workspace Clippy and SDK wasm32 example compilation passed. The root, multi-module, world-render, and OpenGL-geometry example workspaces passed formatting, offline tests, strict Clippy, and wasm32 builds. Headless checks do not verify visible client behavior or remote multiplayer; no crates.io release or public CLI sync was made.
