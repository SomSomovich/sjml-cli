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

The current host is `0.52.0` with ABI `1.42.0`. ABI compatibility is major-version based:
the host accepts a requested ABI with the same major version when the host's
minor/patch version is equal or newer. A host at ABI `1.42.0` therefore accepts
`1.0.x` through `1.42.0` (including earlier ABI 1.32–1.41 guests), but rejects
a later contract it does not know.
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
