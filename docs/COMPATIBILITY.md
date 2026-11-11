# Compatibility and supported runtime

## Host requirements

The current host targets Java 21, Minecraft 1.21.1, and the NeoForge version
declared by `neo_version` in its Gradle project. The SDK and macro packages
declare Rust 1.85 as their minimum toolchain; the `sjml-cli` package requires
Rust 1.88. Guest crates must use `wasm32-unknown-unknown` and the SJML ABI;
WASI modules and ambient filesystem, process, and socket APIs are not supported.

SJML compiles WebAssembly with Endive to JVM bytecode and fails module
activation if compilation fails. It does not interpret a module as a fallback.
The host caps module size at 16 MiB and defined linear memory at 256 wasm32
pages. See [Security](SECURITY.md) for table and operation limits.

## Version compatibility

The current host ABI is `1.36.0`. ABI compatibility is major-version based:
the host accepts a requested ABI with the same major version when the host's
minor/patch version is equal or newer. A host at ABI `1.36.0` therefore accepts
`1.0.x` through `1.36.0` (including earlier ABI 1.32–1.35 guests), but rejects a
later contract it does not know.
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

SDK 0.42.0 added generic native chat panel styling under ABI 1.34.0, plus the
server-authoritative `player.moved` notification and bounded slot-to-slot block
inventory transfers. Chat uses `client.chat.modify`; transfers require
`world.read` and `inventory.write`. See [Native client chat](CHAT.md),
[movement event](PLAYER_MOVED_EVENT.md), and
[inventory mutations](INVENTORY_MUTATIONS.md). Live-client rendering and remote
multiplayer behavior still require verification.

SDK 0.43.0 added bounded same-dimension player teleports, capability-gated
non-player entity removal, weather snapshots and mutations for loaded worlds,
and vanilla stonecutting recipes. At that checkpoint, the host was 0.45.0 / ABI 1.35.0.

SDK 0.44.0 adds read-only player vitals, bounded player game-mode changes,
loaded-world biome and daylight-cycle time access, bounded server-player motion
reads and velocity changes, smithing-transform recipe registration, and the non-cancellable
`player.respawned` event. The current host is 0.46.0 / ABI 1.36.0. Game-mode
changes require `entity.read` and `player.game_mode.write`; setting daylight
time requires `world.read` and `world.write`. See [player vitals](PLAYER_VITALS.md),
[player game mode](PLAYER_GAME_MODE.md), [world biomes](WORLD_BIOMES.md),
[world time](WORLD_TIME.md), [player motion](PLAYER_MOTION.md),
[smithing transform recipes](SMITHING_TRANSFORM.md), and [player respawn events](PLAYER_RESPAWNED_EVENT.md).

Client modules have HUD, bounded typed OpenGL, camera-relative world-render,
active-screen input, native chat layout control, and global no-screen input APIs
loaded through client-only integration. Guest entities and block entities can
render bounded colored or package-textured cuboid models with a host-evaluated
sine rotation. Package
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

## Checking this workspace

These commands validate the CLI, SDK, macros, and example guests:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check -p sjml-sdk --examples --target wasm32-unknown-unknown --locked
```

These Rust checks do not build the private NeoForge host or prove live
Minecraft compatibility. Client rendering, GPU behavior, multiplayer
handshakes, and tick-rate impact require runtime tests with the matching host.
