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

The current host is `0.77.0`, the Rust SDK/macros are `0.75.0`, and the ABI is
`1.67.0`. ABI compatibility remains major-version based: host 0.77.0 accepts ABI
`1.0.x` through `1.67.0`; it rejects later contracts it does not know. Packages
using newer APIs declare ABI `^1.67.0` and host range `^0.77.0`.

Package manifests use SemVer ranges for `abi`, `minecraft`, and `sjml`; those
ranges are checked during package validation and activation.

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
./gradlew --no-configuration-cache :test build
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
GameTests now exercise these server-backed APIs; client command display and
execution still need a display-backed Minecraft client check.

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
world neighbor notifications.

Host 0.71.0 / ABI 1.61.0 / SDK/macros 0.69.0 adds criterion-level advancement
progress and mutation, six typed vanilla statistic families, periodic keyframe
rotations, and schema-v4 client forms with up to eight checkboxes.

Host 0.72.0 / ABI 1.62.0 / SDK/macros 0.70.0 adds the successful-crafting
`player.item_crafted` event and bounded live reads/base-value updates for all 31
built-in vanilla living-entity attributes. All writes retain per-attribute
ranges and require `entity.attributes.write`. Client visuals, actual crafting
interaction, and remote multiplayer remain for live assembly testing.

Host 0.73.0 / ABI 1.63.0 / SDK/macros 0.71.0 adds bounded dynamic server-command suggestions, same-module registered item-use callbacks, cropped package sprite rendering, vanilla brewing mixes and one-effect potions, player-inventory enchantment read/set/remove, exact tracked-sound fades and owner-checked client music selection, and module-owned scoreboard teams. It also adds synchronous cancellable `entity.breed_attempt`, `block.grow_attempt`, and `player.fished_attempt` events. Server GameTests cover the new event, brewing, inventory, and team contracts; client music, audio, and rendering still need display-backed verification. The CLI crate is 0.1.3 locally; no crates.io release or public-repository sync was made.

Host 0.74.0 / ABI 1.64.0 / SDK/macros 0.72.0 adds `client_localization::translate` for the active client language, bounded loaded-only `world::structures_at` queries, and the cancellable `block.fertilize_attempt` bonemeal callback. Focused Java/Rust tests and dedicated-server GameTests cover these contracts; client-language display behavior still needs a live client check. The CLI crate is 0.1.4 locally; no crates.io release or public-repository sync was made.

Host 0.75.0 / ABI 1.65.0 / SDK/macros 0.73.0 adds loaded-only `world::block_has_tag` queries and current-registry `world::item_has_tag` membership checks for canonical tag IDs. Block queries distinguish a loaded non-member from an unavailable position without loading chunks; item queries do not inspect inventory or load chunks. Codec, host import, Rust contract, and dedicated-server GameTests cover both contracts. The CLI crate is 0.1.5 locally; no crates.io release or public-repository sync was made.

Host 0.76.0 / ABI 1.66.0 / SDK/macros 0.74.0 adds sorted bounded enumeration of current block, item, fluid, and entity-type tag members through `world::tag_members` and typed convenience functions. Results distinguish absent tags from registered empty tags and cap at 1,024 entries; all calls require `world.read` and never access chunks. The private CLI crate is 0.1.6 locally; no crates.io publication or public-repository sync was made.

Host 0.77.0 / ABI 1.67.0 / SDK/macros 0.75.0 adds fixed full-cube, empty, and single-cuboid selection, collision, occlusion, and support shapes for registered blocks through `registry::register_block_with_shape` and `registry::set_block_shape`. Shapes are bounded, validated, and immutable after registration; GameTests exercise the selected and collision bounds. The private CLI crate is 0.1.7 locally; no crates.io publication or public-repository sync was made.
