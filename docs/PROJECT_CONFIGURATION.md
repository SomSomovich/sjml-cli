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
"0.40.0"`; `--sdk-path` instead writes a local path dependency. Keep the
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
abi = "^1.32.0"
minecraft = "~1.21.1"
sjml = "^0.42.0"

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
| `client` | Physical client only. Client-only modules cannot declare server commands. |
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
`^1.32.0` and `~1.21.1` are checked by the host.

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
by the server administrator. Neither side alone is enough. The administrator
file is `config/sjml-permissions.toml` in the Minecraft instance, not inside
the mod project:

```toml
[[grant]]
module = "mymod/main"
capabilities = ["events.subscribe", "world.read"]
```

`module` is the combined `<package-id>/<module-id>` identity. A missing file
grants no optional capabilities. The host grants only the intersection of the
module's manifest request and the administrator's list.

| Capability | Enables |
| --- | --- |
| `content.register` | Static registry declarations and guest commands. |
| `events.subscribe` | Subscribing to supported host events. |
| `events.cancel` | Vetoing cancellable `block.break` events; also requires `events.subscribe`. |
| `world.read`, `world.write` | Reading loaded server-world state; writes require both. |
| `entity.read`, `entity.spawn`, `entity.effect.apply` | Reading player/entity snapshots, spawning registered entities, or applying registered effects. |
| `entity.data.read`, `entity.data.write` | Reading or changing module-owned data attached to non-player entities. |
| `inventory.read`, `inventory.write` | Reading loaded block inventories or changing registered string components. Some operations require both `world.read` and an inventory capability. |
| `block_entity.nbt.read` | Reading bounded block-entity snapshots and NBT. |
| `network.mod`, `network.external` | Declaring/using SJML network channels; cross-module channels need `network.external`. |
| `storage.read`, `storage.write` | Reading or changing persistent typed records. |
| `service.call` | Calling versioned server-side guest services; client service calls are unavailable. |
| `task.compute` | Running bounded server-side worker computations. |
| `menu.open` | Opening a bounded read-only server menu; action callbacks also need `entity.read` and `events.subscribe`. |
| `client.input` | Receiving bounded client screen/global input events. |
| `client.render` | Subscribing to HUD, screen, or world-render callbacks. |
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
