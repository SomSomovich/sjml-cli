# SJML CLI

`sjml-cli` is a Rust command-line tool for creating, checking, building, testing,
and packaging Rust/WebAssembly mods for Somuch Java Mod Loader.

This repository is a Cargo workspace. The root package is `sjml-cli`,
`sjml-sdk/` contains the `sjml-sdk` guest API crate (imported in Rust as `sjml`),
and `sjml-macros/` contains the procedural macros re-exported by the SDK. The
macros are a separate package because Rust requires procedural macros to be
compiled as their own crate.

## Install and create a project

The CLI requires Rust and Cargo. Install the WebAssembly target once:

```sh
rustup target add wasm32-unknown-unknown
```

From this repository, install the binary with:

```sh
cargo install --path .
```

The CLI is published on crates.io as `sjml-cli`; install it with
`cargo install sjml-cli`. To publish the guest SDK as well, publish
`sjml-macros` first, then `sjml-sdk`; the SDK depends on the macro crate. The
CLI can be released independently.

Create a project in a new directory named after the mod:

```sh
sjml-cli init mymod
```

Or choose a relative destination from the current directory:

```sh
sjml-cli init home/mymod
```

The final path component becomes the default SJML package ID. Use `--id` to
choose a different ID. The generated project contains `Cargo.toml`,
`sjml.toml`, `sjml-project.toml`, `src/lib.rs`, and `.gitignore`. `init` also
creates a Git repository with an initial `master` branch; Git must be installed.

When developing against a local SJML checkout, pass the path to its SDK crate:

```sh
sjml-cli init mymod --sdk-path /path/to/sjml-cli/sjml-sdk
```

Without `--sdk-path`, the generated Cargo manifest depends on `sjml-sdk =
"0.94.0"`; that crate must be available from a configured Cargo registry. This
private target version is not published yet, so use `--sdk-path` to compile the
generated project until the matching SDK crates are released.

## Documentation

- [Rust SDK guide](docs/SDK_GUIDE.md), [registered block scheduled ticks](docs/BLOCK_TICK.md), [persistent block-entity data](docs/BLOCK_ENTITY_PERSISTENT_DATA.md), [bounded vanilla projectile launches](docs/VANILLA_PROJECTILES.md), [registered entity AI goals](docs/ENTITY_AI_GOALS.md), [client action attempts](docs/CLIENT_ACTION_ATTEMPTS.md), [block tag membership](docs/WORLD_BLOCK_TAGS.md), [item tag membership](docs/WORLD_ITEM_TAGS.md), [registry tag members](docs/WORLD_TAG_MEMBERS.md), [registered block-state properties](docs/BLOCK_STATE_PROPERTIES.md), [block-state shapes](docs/BLOCK_STATE_SHAPES.md), [block-state light](docs/BLOCK_STATE_LIGHT.md), [state-aware block behavior](docs/BLOCK_STATE_BEHAVIOR.md), [entity health](docs/ENTITY_HEALTH.md), [entity equipment](docs/ENTITY_EQUIPMENT.md), [player boss bars](docs/PLAYER_BOSS_BARS.md), [player tab-list text](docs/PLAYER_TAB_LIST.md), [ordinary item-stack spawning](docs/WORLD_ITEM_SPAWN.md), [world game-event dispatch](docs/WORLD_GAME_EVENTS.md), [live villager-trade notifications](docs/PLAYER_VILLAGER_TRADES.md), [module-owned scoreboard objectives](docs/SCOREBOARD_OBJECTIVES.md), [safe block update options](docs/WORLD_BLOCK_UPDATE_OPTIONS.md), [player inventory item grants](docs/PLAYER_INVENTORY_GIVE.md), [exact player statistic setters](docs/PLAYER_STATISTICS.md), [model translation and scale keyframes](docs/MODEL_ANIMATIONS.md), [interactive client screens and owner-scoped close](docs/CLIENT_SCREEN.md), [per-state block friction](docs/BLOCK_STATE_FRICTION.md), [loaded-only fluid tick scheduling](docs/WORLD_FLUID_TICKS.md), [sampled cursor movement](docs/CLIENT_CURSOR_MOVED.md), and [SDK API reference on docs.rs](https://docs.rs/sjml-sdk/0.92.0/sjml/)
- [Registered block shapes](docs/WORLD_BLOCK_SHAPES.md)
- [Registered block behavior](docs/WORLD_BLOCK_BEHAVIOR.md)
- [Registered block use](docs/BLOCK_USE.md)
- [Registered item use on blocks](docs/ITEM_USE_ON_BLOCK.md)
- [Registered item use on entities](docs/ITEM_USE_ON_ENTITY.md)
- [Bounded loaded-Mob navigation](docs/ENTITY_NAVIGATION.md)
- [WebAssembly ABI v1](docs/ABI.md)
- [Project configuration and capability grants](docs/PROJECT_CONFIGURATION.md)
- [Complete manifest schema](docs/MANIFEST_SCHEMA.md) and [`.sjml` archive format](docs/FORMAT.md)
- [Compatibility and host support](docs/COMPATIBILITY.md)
- [Minecraft API coverage matrix](docs/minecraft-api-coverage.md)
- [Cancellable player item-pickup attempts](docs/PLAYER_ITEM_PICKUP_ATTEMPT.md)
- [Totem-use attempts](docs/PLAYER_TOTEM_USE_ATTEMPT.md), [shield-block attempts](docs/ENTITY_SHIELD_BLOCK_ATTEMPT.md), and [entity teleport attempts](docs/ENTITY_TELEPORT_ATTEMPT.md)
- [Registered item behavior](docs/CUSTOM_ITEM_BEHAVIOR.md), [player inventory enchantments](docs/PLAYER_INVENTORY_ENCHANTMENTS.md), [scoreboard access](docs/SCOREBOARD.md), and [module-owned scoreboard objectives](docs/SCOREBOARD_OBJECTIVES.md)
- [Client music selection](docs/CLIENT_MUSIC.md), [breeding](docs/ENTITY_BREED_ATTEMPTS.md), and [crop growth](docs/BLOCK_GROW_ATTEMPTS.md)
- [Client localization lookups](docs/CLIENT_LOCALIZATION.md)
- [Networking](docs/NETWORKING.md), [storage](docs/STORAGE.md), [async services](docs/ASYNC_MODEL.md), and [client rendering/OpenGL](docs/RENDERING.md)
- [Security boundaries](docs/SECURITY.md), [module lifecycle](docs/MODULE_LIFECYCLE.md), [block entities](docs/BLOCK_ENTITIES.md), [player movement](docs/PLAYER_MOVED_EVENT.md), [player respawn events](docs/PLAYER_RESPAWNED_EVENT.md), [player vitals](docs/PLAYER_VITALS.md), [player active effects](docs/PLAYER_ACTIVE_EFFECTS.md), [player game mode](docs/PLAYER_GAME_MODE.md), [player motion](docs/PLAYER_MOTION.md), [player teleport](docs/PLAYER_TELEPORT.md), [entity removal](docs/ENTITY_REMOVAL.md), [world biomes](docs/WORLD_BIOMES.md), [world time](docs/WORLD_TIME.md), [world weather](docs/WORLD_WEATHER.md), [stonecutting recipes](docs/STONECUTTING.md), [smithing transform recipes](docs/SMITHING_TRANSFORM.md), [brewing recipes](docs/BREWING_RECIPES.md), [inventory transfers](docs/INVENTORY_MUTATIONS.md), [particles](docs/PARTICLES.md), and [worker compute](docs/WORKER_COMPUTE.md)
- [Player equipment](docs/PLAYER_EQUIPMENT.md), [player abilities](docs/PLAYER_ABILITIES.md), [daylight-cycle gamerule](docs/WORLD_DAYLIGHT_CYCLE.md), [world border](docs/WORLD_BORDER.md), [client chat messages](docs/CLIENT_CHAT_MESSAGES.md), [client audio](docs/CLIENT_AUDIO.md), [client key events](docs/CLIENT_KEY_EVENTS.md), [player titles](docs/PLAYER_TITLES.md), [player inventory components](docs/PLAYER_INVENTORY_COMPONENTS.md), [item rarity and enchantability](docs/ITEM_RARITY.md), [entity fire ticks](docs/ENTITY_FIRE_TICKS.md), [entity state flags](docs/ENTITY_STATE_FLAGS.md), [menu updates](docs/MENU_UPDATES.md), [player drop attempts](docs/PLAYER_DROP_ATTEMPTS.md), [block break attempts](docs/BLOCK_BREAK_ATTEMPTS.md), and [client toast notifications](docs/CLIENT_NOTIFICATIONS.md)
- [Loaded chunks](docs/WORLD_LOADED_CHUNKS.md), [entity raycasts](docs/ENTITY_RAYCAST.md), [cancellable entity death attempts](docs/ENTITY_DEATH_ATTEMPTS.md), [local client commands and suggestions](docs/CLIENT_COMMANDS.md), [integer game rules](docs/WORLD_INTEGER_GAME_RULES.md), [block-property writes](docs/WORLD_BLOCK_PROPERTIES_WRITE.md), [block replacement update options](docs/WORLD_BLOCK_UPDATE_OPTIONS.md), [item-stack snapshots](docs/ITEM_STACK_SNAPSHOT.md), [player inventory and durability writes](docs/PLAYER_INVENTORY_WRITES.md), [giving items to a player](docs/PLAYER_INVENTORY_GIVE.md), [block-placement vetoes](docs/BLOCK_PLACE_ATTEMPTS.md), and [living entity and player attributes](docs/ENTITY_ATTRIBUTES.md), [villager trades](docs/VILLAGER_TRADES.md), [damage-attempt vetoes](docs/ENTITY_DAMAGE_ATTEMPTS.md), [cross-dimension teleport](docs/PLAYER_DIMENSION_TELEPORT.md), and [shared game time](docs/WORLD_GAME_TIME.md)
- [Editable menus](docs/EDITABLE_MENUS.md), [item food properties](docs/ITEM_FOOD.md), [registered item attribute modifiers](docs/ITEM_ATTRIBUTE_MODIFIERS.md), [static item crafting remainders](docs/ITEM_CRAFTING_REMAINDER.md), [non-player entity custom names](docs/ENTITY_CUSTOM_NAMES.md), [projectile-impact vetoes](docs/PROJECTILE_IMPACT_ATTEMPT.md), [fishing-catch vetoes](docs/PLAYER_FISHED_ATTEMPTS.md), and [boolean item components](docs/BOOLEAN_ITEM_COMPONENTS.md)

## Commands

| Command | Purpose |
| --- | --- |
| `init <name-or-path>` | Scaffold a Rust guest project. |
| `build` | Compile mapped Cargo packages for `wasm32-unknown-unknown` and copy validated artifacts into module paths. |
| `check` | Validate the package manifest and crate mappings, then type-check for WebAssembly. |
| `test` | Run native Cargo workspace tests. |
| `package` | Build a deterministic, checksummed `.sjml` archive. |
| `dev` | Build/package once, then watch project files and rebuild the archive when they change. |
| `inspect <archive.sjml>` | Validate an archive without extracting it and list its contents and hashes. |

`build`, `check`, `test`, `package`, and `dev` accept `--project <path>` (default
`.`), `--release`, `--offline`, and `--locked`. `package` and `dev` also accept
`--output <file.sjml>` and `--no-build`. `dev` supports `--once` and
`--interval-ms` (50–60000, default 500). Stop the watcher with Ctrl-C.

For multi-crate Cargo workspaces, `sjml-project.toml` maps each SJML module ID
to its Cargo package. A one-module, one-package project needs no mapping file.
The CLI validates package paths and WASM, writes packages deterministically,
and refuses to overwrite project inputs with an archive. It does not launch
Minecraft or hot-reload an archive in a running game; restart Minecraft after
replacing a loaded package.

To validate every package in this repository, run `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo check -p sjml-sdk --examples --target wasm32-unknown-unknown`.

Run `sjml-cli --help` or `sjml-cli <command> --help` for the full option list.
