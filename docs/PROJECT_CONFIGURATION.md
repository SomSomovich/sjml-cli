# Mod project and configuration

This guide describes the files produced by `sjml-cli init` and the settings
that connect a Rust guest to its `.sjml` package and to the host. The complete
key-by-key manifest schema is in [manifest schema](MANIFEST_SCHEMA.md);
limits, commands, data-pack resources, and archive rules are in the
[package format reference](FORMAT.md). The raw WebAssembly contract is
in the [ABI v1 reference](ABI.md); the Rust wrappers are described in the
[SDK guide](SDK_GUIDE.md).

## Create a project

```sh
sjml-cli init mymod
```

The command creates the project without prompts. The final path component is
used as the default package and module ID. `--id <id>` overrides it; a local
SDK checkout can be selected with `--sdk-path <path>`.

The generated tree is:

```text
mymod/
├── .git/                 # initialized on master; no commit is created
├── .gitignore
├── Cargo.toml
├── sjml-project.toml
├── sjml.toml
└── src/lib.rs
```

`Cargo.toml` builds a Rust library as a WebAssembly `cdylib` and also enables a
native `rlib` for tests. The default registry dependency is `sjml-sdk =
"0.95.0"`; `--sdk-path` instead writes a local path dependency. Keep the
generated `[workspace]` table so a project created inside another Cargo
workspace remains an independent root.

`--sdk-path` must point to the directory containing the `sjml-sdk` crate's
`Cargo.toml`. The CLI resolves it to an absolute path in the generated Cargo
manifest, so that development project depends on that local checkout and the
manifest is not portable to another machine until its dependency is changed.

Install the target once and build the guest with:

```sh
rustup target add wasm32-unknown-unknown
sjml-cli build --project mymod
```

## `sjml.toml`

This is the host package manifest. The CLI template starts with:

```toml
[mod]
id = "mymod"
name = "mymod"
version = "0.1.0"

[runtime]
abi = "^1.87.0"
minecraft = "~1.21.1"
sjml = "^0.97.0"

[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = []
```

The module `path` is where `sjml-cli build` copies the compiled WASM and where
the package loader expects it. The current CLI template explicitly selects
`side = "server"`; the manifest parser itself defaults an omitted side to
`both`.

`side` is per module and chooses where its code runs:

| Value | Execution |
| --- | --- |
| `server` | Logical server, whether dedicated or integrated. |
| `client` | Physical client only. Client-only modules may declare client commands, but not server commands. |
| `both` | Separate module instances on client and server when each context exists. |

`[mod].distribution` is a separate package-wide setting. It controls whether
the other multiplayer peer must install the package; it does not choose where
module code runs:

| Value | Installation requirement |
| --- | --- |
| `optional` | No matching package is required on the peer. This is the default. |
| `server_only` | A server normally does not require clients to install SJML or the package. Server modules that declare network channels or request `content.register` are conservatively made peer-required. |
| `client_only` | The client package does not require the server to install SJML or the package. |
| `both_required` | The peer must have the same package version. |

Typical pairings are `side = "server"` with `distribution = "server_only"`
for independent server behavior, `side = "client"` with `distribution =
"client_only"` for a client feature, and `side = "both"` with
`distribution = "both_required"` when both peers need the same package.
`side = "server"` alone does not set `server_only`. In particular, a plain
server module can run for clients without SJML, but network channels or
`content.register` can make its package required at login. See
[package distribution](FORMAT.md#package-distribution) for the exact rules.

Other `[mod]` fields are `authors`, `description`, `license`, arbitrary bounded
`metadata`, and `distribution`. `runtime.abi` is required; `runtime.minecraft`
and `runtime.sjml` are optional compatibility ranges. SemVer ranges such as
`^1.41.0` and `~1.21.1` are checked by the host. Event vetoes use the
separately granted `events.cancel` capability; effect removal uses
`entity.effect.remove` in addition to `entity.read`.

Each `[[modules]]` entry can also declare:

- `depends`: IDs of modules in this package that must load first.
- `permissions`: host capabilities requested by this module.
- `provides` and `exports`: versioned service declarations and visibility.
- `commands`: typed server command callbacks.
- `channels`: named network channels with schema versions and directions.

Package-level `[[dependencies]]` declares other SJML packages, `conflicts`
declares incompatible package IDs, and `[[files]]` explicitly lists payload
files. `sjml-cli package` regenerates the `[[files]]` checksums. The optional
`[signature]` table is metadata only; CLI inspection does not authenticate a
publisher signature. See [the full format reference](FORMAT.md) before using
these advanced fields.

## `sjml-project.toml`

This file maps host module IDs to Cargo workspace packages. A one-module,
one-crate project needs no mapping file, but the CLI template includes an
explicit mapping. For a multi-crate project, add packages to the Cargo
workspace and map each module:

```toml
[[crates]]
module = "content"
package = "mymod-content"

[[crates]]
module = "logic"
package = "mymod-logic"
```

Each selected package must have one `cdylib` target. `module` must name an
existing `[[modules]].id`; `package` must match a Cargo package name. A module
can intentionally map to the same crate as another module. See the CLI section
in the [repository README](../README.md) for workspace matching and build
behavior.

This file accepts only `[[crates]]` entries, each with the required string
keys `module` and `package`. Entries are optional when a project has one
manifest module and one Cargo workspace package. Otherwise, the CLI uses
explicit entries first, then matches normalized Cargo package and module names
(`-` and `_` are treated alike). If that does not identify exactly one
workspace package, add an explicit mapping. Duplicate module entries and
unknown module IDs fail validation; each selected Cargo package must contain
exactly one `cdylib` target.

## Capability requests and server grants

A capability must be declared in the module's `permissions` array and granted
by the administrator for that Minecraft instance. Neither side alone is enough.
The grant file is `config/sjml-permissions.toml` in the game instance, not
inside the mod project. Client-only permissions are granted on each client
that runs the module:

```toml
[[grant]]
module = "mymod/main"
capabilities = ["events.subscribe", "world.read"]
```

`module` is the combined `<package-id>/<module-id>` identity. A missing file
grants no optional capabilities. The host grants only the intersection of the
module's manifest request and the administrator's list.

For `entity::set_glowing`, request both `entity.read` and
`entity.glowing.write` in the module's `permissions` array, then grant both to
that same module in `config/sjml-permissions.toml`. The older
`entity.state.write` grant still controls `entity::set_silent` and no longer
authorizes glowing changes.

For example, the module manifest requests:

```toml
permissions = ["entity.read", "entity.glowing.write"]
```

The administrator grants the same two capabilities to that module:

```toml
[[grant]]
module = "mymod/main"
capabilities = ["entity.read", "entity.glowing.write"]
```

| Capability | Enables |
| --- | --- |
| `content.register` | Static registry declarations, including sound events and bounded item food/rarity/enchantment/durability values, and server guest commands. |
| `command.suggest` | Invoking bounded synchronous guest suggestions for declared server-command arguments; also requires `content.register` and the command's Brigadier permission level. |
| `client.command` | Register and execute bounded local commands on the physical client; does not send commands to the server. |
| `client.command.suggest` | Invoke bounded synchronous suggestions for declared physical-client command arguments; also requires `client.command` and the local client thread. |
| `events.subscribe` | Subscribing to supported host events. |
| `client.item.tooltip.modify` | Appending bounded literal lines from synchronous `client.item.tooltip` callbacks on the physical client; also requires `events.subscribe`. |
| `client.chat.send.observe` | Reading original outgoing chat text from the local client's synchronous `client.chat.send_attempt` callback; also requires `events.subscribe` and `events.cancel`. |
| `events.cancel` | Vetoing cancellable block break/place/growth, fluid block placement, right-click `player.interaction`, `player.drop_attempt`, `player.fished_attempt`, `player.attack_attempt`, `client.chat.send_attempt`, `client.action_attempt`, `entity.death_attempt`, `entity.damage_attempt`, `entity.knockback_attempt`, `entity.heal_attempt`, `entity.effect_application`, `entity.join_attempt`, `entity.target_attempt`, `entity.breed_attempt`, `entity.projectile_impact_attempt`, and `world.explosion_attempt`; also permits result callbacks for module-owned `block.use`, `item.use`, `item.use_on_block`, and `item.use_on_entity`, which also require `events.subscribe`. A drop veto is skipped if the full stack cannot be restored. `client.action_attempt` additionally requires `client.input.action_cancel`. |
| `world.read`, `world.write` | Reading loaded server-world state; writes include blocks and properties, scheduled block ticks, time, weather, difficulty, Overworld spawn point, allowlisted boolean/integer game rules, and world-border settings, and require both. |
| `entity.read`, `entity.spawn`, `entity.effect.apply`, `entity.effect.remove` | Reading player/entity snapshots including bounded living-entity equipment, spawning registered entities, or applying/removing registered effects; effect mutation also requires `entity.read`. |
| `entity.equipment.write` | Adding one registered item's default stack to an empty slot of a loaded living non-player entity; also requires `entity.read`. Occupied slots are left unchanged. |
| `entity.mount.write` | Starting or stopping a non-forced ride between two loaded non-player entities; also requires `entity.read` and same-world live handles. |
| `entity.glowing.write` | Changing the glowing flag on a loaded non-player entity; also requires `entity.read`. |
| `entity.remove`, `entity.velocity.write`, `entity.attributes.write`, `entity.fire.write`, `entity.state.write`, `entity.custom_name.write`, `entity.ai.control`, `entity.navigation.write` | Removing loaded non-player entities, changing bounded velocity, fire duration, the silent flag, literal custom names, or the enabled-AI flag on loaded Mob entities, starting a bounded path for a loaded Mob, and reading/writing allowlisted living-entity and server-player attribute base values; entity writes also require `entity.read`. Navigation also requires a loaded target corridor and server execution. |
| `player.teleport`, `player.respawn.write`, `player.vitals.write`, `player.game_mode.write`, `player.motion.write`, `player.rotation.write`, `player.abilities.write`, `player.cooldown.write`, `player.advancement.write`, `player.statistics.write` | Teleporting a live player, changing their saved respawn position, bounded vitals, game mode, velocity, yaw/pitch, flight state, vanilla item cooldowns, advancement progress, or vanilla custom statistics; player writes also require `entity.read` and logical-server execution. |
| `entity.data.read`, `entity.data.write` | Reading or changing module-owned data attached to non-player entities. |
| `inventory.read`, `inventory.write` | Reading loaded block/player inventories and detached item-stack snapshots, or changing registered block-inventory stacks, literal custom names on existing block- and player-inventory stacks, bounded block/player slot transfers, player slots/durability and ordinary item grants, and same-module integer components. Player operations also require `entity.read`; block operations require `world.read` and an inventory capability. |
| `block_entity.nbt.read` | Reading bounded block-entity snapshots and NBT. |
| `block_entity.data.read`, `block_entity.data.write` | Reading or writing typed Boolean, i32, and string data on the calling module's own registered block entities; writes also require `world.write`. |
| `network.mod`, `network.external` | Declaring/using SJML network channels; cross-module channels need `network.external`. |
| `storage.read`, `storage.write` | Reading or changing persistent typed records. |
| `service.call` | Calling versioned server-side guest services; client service calls are unavailable. |
| `task.compute` | Running bounded server-side worker computations. |
| `menu.open` | Opening/updating a bounded read-only server menu or opening an allowlisted editable menu; updates are limited to eight per module per server tick. Editable-menu reads also require `entity.read`; action callbacks require `entity.read` and `events.subscribe`. |
| `menu.transfer` | Moving 1–64 existing items between declared editable-menu slots and the player inventory; also requires `entity.read` and `menu.open`, with at most 32 transfers per module per server tick. |
| `client.input` | Receiving bounded client screen/global input events, named keybindings, and scaled cursor coordinates. |
| `client.input.observe` | Receiving observe-only global keyboard notices after Minecraft processes the key and polling bounded held key/mouse-button state. |
| `client.input.screen_cancel` | Vetoing supported key presses delivered to the active screen; does not cancel global gameplay keys. |
| `client.input.action_cancel` | Vetoing vanilla attack, use-item, or pick-block mappings through synchronous `client.action_attempt` callbacks on the physical-client thread; also requires `events.subscribe` and `events.cancel`. |
| `client.input.screen_scroll` | Receiving/consuming active-screen scroll callbacks; requires `client.input`. |
| `player.ui.actionbar` | Sending bounded literal action-bar text to a live player; also requires `entity.read`. |
| `player.ui.title` | Sending bounded literal title and subtitle packets to a live player; also requires `entity.read`. |
| `client.ui.screen` | Opening a bounded literal vanilla screen with buttons, text/list forms, checkboxes, or one numeric slider on the physical client; button actions are dispatched directly to their owning module. |
| `client.chat.observe`, `client.chat.filter`, `client.chat.rewrite`, `client.chat.send`, `client.chat.send.observe` | Observing bounded incoming chat text, hiding its display insertion, replacing it with bounded rich text, sending ordinary signed chat through Minecraft's local player connection, or reading the local user's original outgoing text. Incoming filtering/replacement need their separate grants; outgoing observation also requires `events.subscribe`, and veto requires `events.cancel`. |
| `client.ui.notify` | Showing bounded vanilla toast notifications on the physical client. |
| `client.audio.play` | Playing a registered sound locally through Minecraft's sound manager and stopping a tracked one-shot owned by the module runtime. |
| `client.audio.loop` | Starting/stopping bounded module-owned local loops; cleanup stops only that module's sounds. |
| `client.audio.music` | Subscribing on a physical client to bounded situational music-selection callbacks; replacement tracks must be registered by that exact module. |
| `client.render` | Subscribing to HUD, screen, or world-render callbacks. |
| `client.render.gui_layer.suppress` | Suppressing an exact vanilla GUI-layer ID on the physical client; requires a matching active client module, and does not provide arbitrary screen or render-state access. |
| `client.render.opengl` | Using typed OpenGL resources and GUI textures. Most drawing also needs `client.render`. |

Add only the capabilities that the code uses. The [SDK guide](SDK_GUIDE.md),
[rendering reference](RENDERING.md), [networking reference](NETWORKING.md),
[storage reference](STORAGE.md), and [security boundaries](SECURITY.md)
describe each API's bounds and additional requirements.

## Package payloads

The CLI packages declared WASM modules, explicit `[[files]]` paths, and regular
files below `assets/`, `data/`, and `config/`. Client resources live under
`assets/<package-id>/`; server data resources live under
`data/<package-id>/`. Rust source, Cargo files, and `sjml-project.toml` are not
included in the `.sjml` archive. The package command validates paths and WASM,
then writes sorted checksums into the archive's copy of `sjml.toml`.

Files under the project `config/` directory are copied as package entries, but
the current host does not load that directory as administrator configuration
or expose a general guest configuration API. In particular, capability grants
belong in the Minecraft instance's `config/sjml-permissions.toml`, not in the
project's package payload.
