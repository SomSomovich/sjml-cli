# `.sjml` Package Format

An `.sjml` package is a ZIP-compatible archive containing a UTF-8 TOML manifest
at `sjml.toml`, one or more declared WebAssembly modules, and optional asset,
data, or configuration files. The archive is validated in memory; paths are
never extracted to the filesystem.

## Manifest

```toml
conflicts = ["incompatible"]

[mod]
id = "example"
name = "Example Mod"
version = "1.0.0"
authors = ["Developer"]
description = "An example SJML package"
license = "MIT"
distribution = "server_only"

[runtime]
abi = "^1.0"
minecraft = "~1.21.1"
sjml = "^0.42.0"

[[modules]]
id = "core"
path = "modules/core.wasm"
side = "both" # client, server, or both; default: both
permissions = ["content.register"]
provides = ["example.service"]
depends = []

[[modules.exports]]
id = "example.service"
version = "1.0.0"
visibility = "public"

[[modules.commands]]
name = "chat.echo" # Matches #[command(name = "chat.echo")].
schema_version = 1 # Must match #[sjml(version = 1)] on the Rust input record.
permission_level = 2 # Brigadier permission level required to execute.

[[modules.commands.arguments]]
name = "message"
type = "string"

[[modules.commands.arguments]]
name = "repeat"
type = "integer"
min = 1
max = 5

[[modules.commands.arguments]]
name = "loud"
type = "boolean"

[[dependencies]]
id = "base"
version = "^1.2.0"
optional = false
side = "both"

[[files]]
path = "modules/core.wasm"
sha256 = "0000000000000000000000000000000000000000000000000000000000000000" # Replace with the entry's real SHA-256.
```

The parser rejects unknown keys, malformed or duplicate IDs, unsupported side
values, invalid SemVer constraints, invalid module paths, duplicate hash
paths, and non-canonical checksum declarations. Runtime compatibility ranges,
dependency graphs, and requested permissions are consumed by later bootstrap
stages; archive validation alone does not resolve or grant them. The portable
core's `ModuleServiceRouter` consumes service export declarations when
resolving calls. The current NeoForge adapter routes server-side service
exports and calls through its async module runtime; client-side service
providers and calls remain unavailable.

### Package distribution

`[mod].distribution` controls whether the same SJML package must be present at
the other end of a multiplayer connection. It is independent from each
`[[modules]].side`: `side` selects where a module executes, while
`distribution` controls peer installation requirements. The distribution
value does not suppress a module whose execution side includes the current
runtime.

| Value | Peer requirement |
| --- | --- |
| `optional` | No matching package is required. This is the default for manifests written before the field existed. |
| `server_only` | A server deployment does not normally require clients to install SJML or this package. If a server-executed module declares network channels or requests `content.register`, the server handshake automatically requires this package and its exact version because those features can need matching client behavior or synchronized registry definitions. |
| `client_only` | The client package never makes the server install SJML or the package. |
| `both_required` | The remote peer must install this package at the exact same version. |

The `server_only` auto-require is deliberately conservative: `content.register`
is a broad capability that can add NeoForge registry entries, and merely
requesting it can trigger the rule even if the module does not use it.
Declared channels advertise peer-facing behavior. Pure server modules without
either declaration or a required peer dependency remain joinable by clients
without SJML. Existing manifests that omit `distribution` retain the old
optional peer behavior. Packages that register client-synchronized content
should declare an appropriate distribution explicitly.

### Guest command declarations

Commands are declared in the owning module's manifest. Each `[[modules.commands]]`
entry names the callback identifier, its typed-record schema version, and the
minimum Brigadier permission level. Its nested
`[[modules.commands.arguments]]` tables are ordered: their order and types
must match the fields in the Rust callback's `StableDecode` record. The
example above corresponds to a record with fields `message: String`,
`repeat: i32`, and `loud: bool`, in that order.

The currently supported argument types are:

| `type` | Brigadier parser | Rust field | Extra keys |
| --- | --- | --- | --- |
| `word` | One unquoted word | `String` | None |
| `string` | One word, or one quoted string | `String` | None |
| `greedy_string` | The remaining input, including spaces | `String` | None; must be the last argument |
| `integer` | Signed 32-bit integer | `i32` | Optional signed 32-bit `min` and `max` |
| `boolean` | `true` or `false` | `bool` | None |

All arguments are required and have no manifest defaults. Integer bounds are
inclusive; an omitted bound leaves that side at the full `i32` limit. A
`greedy_string` must be the final argument because Brigadier consumes the
rest of the command line for it. `word`, `string`, and `greedy_string` values
are limited to 65,536 UTF-8 bytes each; the encoded callback input is limited
to 1 MiB total.

Each module may declare at most 64 commands, and each command may declare at
most 16 arguments; the host stages at most 8,192 commands across all packages.
Command names use the callback identifier grammar: 1–128
lowercase ASCII letters, digits, `.`, `_`, or `-`, starting with a letter or
digit. Argument names are unique within a command and use the manifest
identifier grammar. Command declarations in client-only modules are rejected.
The module must request `content.register` and receive the administrator's
grant for that capability before its commands are made available.

NeoForge exposes a guest command at
`/sjml guest <package-id> <module-id> <command-name> ...`. For example, the
declaration above is invoked as
`/sjml guest example core chat.echo "hello there" 3 true`. The declared
`permission_level` is checked when the command is parsed and when it executes;
it does not grant the module any additional host capability.
If command execution returns a guest error, traps, or produces an invalid
callback response, the host disables only that module and reports a generic
command failure to the caller.

Optional publisher metadata has this shape:

```toml
[signature]
algorithm = "ed25519"
key_id = "publisher-key-1"
public_key = "<canonical-base64-32-byte-key>"
value = "<canonical-base64-64-byte-signature>"
```

The current manifest parser validates the algorithm, key identifier, and
canonical Base64 lengths. It does not yet cryptographically verify the
signature; a package carrying this table must not be described as
publisher-authenticated.

## Archive rules and current limits

- `sjml.toml` is required and must be a regular file.
- Every declared `.wasm` module path must refer to an archive entry.
- Declared file SHA-256 values are compared to the exact retained entry bytes.
- Absolute, traversal, backslash, non-NFC, duplicate-normalized, control-
  character, and otherwise invalid paths are rejected.
- ZIP CRCs and declared entry sizes are checked while reading.
- The reader does not extract entries, and fails on unknown or inconsistent
  sizes, excessive compression ratios, malformed ZIP data, and invalid UTF-8.

Default per-archive bounds:

| Bound | Default |
| --- | ---: |
| Compressed archive | 256 MiB |
| ZIP entries | 4096 |
| One uncompressed entry | 64 MiB |
| Total uncompressed archive | 128 MiB |
| Manifest | 1 MiB |
| Compression ratio | 200:1 |

The NeoForge startup scanner additionally caps the scan at 256 direct-child
archives, 1 GiB of compressed package input, and 512 MiB of retained
uncompressed data. The current `/sjml status` reports archive/manifest
validation only; it does not claim that any module has been compiled or run.

### Client resource assets

Client assets belong under `assets/<package-id>/` in the `.sjml` archive. The
host mounts every validated regular file in that package namespace from a
generated, always-enabled client resource pack; archive files are never
extracted into the game's resource directories. Other namespaces are ignored.
This includes `sounds.json` and referenced OGG audio, particle atlas PNGs, and
entity or block-entity model textures. For a registered sound such as
`example:content/engine`, provide `assets/example/sounds.json` with an
`"content/engine"` event key and the referenced `.ogg` file below
`assets/example/sounds/`; also stage the sound event with
`registry::register_sound`.

Entity and block-entity model textures use the same mounted assets: for
`example:textures/entity/clockwork_mite.png`, include the file at
`assets/example/textures/entity/clockwork_mite.png`. Model texture IDs must stay
inside the package namespace and point to `.png` files below `textures/`.
Particle sprite IDs such as `example:particle/ruby_spark` resolve to
`assets/example/textures/particle/ruby_spark.png`.

### Server registry tags

Packages may define registry tags in their own namespace at
`data/<mod.id>/tags/<registry-path>/<tag-path>.json`. For example,
`data/example/tags/item/ruby_gems.json` defines the item tag `example:ruby_gems`,
which a recipe can reference with `RecipeIngredient::tag("example:ruby_gems")`.
Block tags use paths such as `data/example/tags/block/mineable/pickaxe.json`;
world-generation tags use nested registry paths such as
`data/example/tags/worldgen/biome/is_overworld.json`. The supported vanilla
JSON form is a `values` array with an optional boolean `replace` field; values
may be resource-ID strings or `{ "id": "...", "required": false }` entries.
Tag data is copied into SJML's generated server data pack without extraction.
Only files in a package's own `tags/` namespace are mounted. Resource IDs in
tag entries are limited to 256 characters. Tag files are limited to 4,096
values each.

### Server package data

The generated server data pack also mounts package-owned JSON under
`data/<mod.id>/recipe/`, `loot_table/`, `loot_modifiers/`, `worldgen/`,
`neoforge/biome_modifier/`, and `neoforge/structure_modifier/`. Global loot
modifier files are indexed into a generated
`data/neoforge/loot_modifiers/global_loot_modifiers.json`; its `replace: false`
entry list lets NeoForge layer the SJML entries with other data packs. It accepts
dynamic registry JSON under
`banner_pattern/`, `cat_variant/`, `chat_type/`, `damage_type/`, `dimension/`,
`dimension_type/`, `enchantment/`, `enchantment_provider/`, `instrument/`,
`jukebox_song/`, `painting_variant/`, `trim_material/`, `trim_pattern/`, and
`wolf_variant/`. It also accepts data-pack resources under `advancement/`,
`item_modifier/`, and `predicate/`. This includes Minecraft 1.21 loot tables
(for example, `data/example/loot_table/chests/ruby_cache.json`),
world-generation registries (for example,
`data/example/worldgen/configured_feature/ruby_ore.json`), NeoForge biome and
structure modifiers, global loot modifiers, and the listed dynamic registries
and data-pack resources. Recipe
files can refer to a guest callback serializer registered by the module; their
`type` is `<mod.id>:<module>/<serializer-path>`, and they may include a bounded
UTF-8 `payload` string. NeoForge data maps can also be contributed at
`data/neoforge/data_maps/<registry-path>/<map-id>.json`, for example
`data/neoforge/data_maps/item/furnace_fuels.json`. This is the only supported
foreign data namespace because NeoForge resolves built-in data map IDs in its
own `neoforge` namespace. If multiple packages provide the same map path, the
package whose mod ID sorts first lexicographically wins and later copies are
skipped with a startup warning. Other JSON files must remain under the
package's own namespace; unsupported `data/` paths are ignored.

Binary structure templates are accepted as compressed NBT under
`data/<mod.id>/structure/<path>.nbt`, matching Minecraft 1.21.1's
structure-template resource layout.

The host validates strict UTF-8 JSON with an object root, a maximum nesting
depth of 64, and at most 65,536 values and object keys per file. Minecraft and
NeoForge still validate each resource's type-specific schema during data-pack
reload; syntactically valid but schema-invalid JSON can make that reload fail.
Across all mounted package data (recipes, registry tags, loot tables, global
loot modifiers, NeoForge data maps, world-generation JSON, biome and structure
modifiers, dynamic registry entries, and structure templates), the loaded
packages may supply at most 512 files, 1 MiB per compressed file, and 8 MiB
total. Each compressed structure template is decoded with a 4 MiB NBT budget;
Minecraft validates template-specific fields during data-pack loading. Invalid
or over-limit files are skipped with a startup warning and do not disable other
generated recipes or valid package data.


## Read-only world permission

A server module can request `permissions = ["world.read"]` and an administrator
can grant it with `capabilities = ["world.read"]` in that module's existing
`[[grant]]` record. This exposes all loaded dimensions on the owning logical
server, including dimensions supplied by other mods. Permissions are per module;
there is no dimension allowlist. Lookup and enumeration never load dimensions.
World handles are scoped to the active module/context, expire on world unload
or runtime close, and are limited to 1,024 live handles per module. Enumeration
returns the complete sorted list or `HANDLE_TABLE_FULL`, never a truncated list.
See `SDK_GUIDE.md` and `ABI.md` for the four read-only world functions.

## Entity permissions

A module can request `entity.read` to enumerate loaded non-player entities
across the logical server's loaded dimensions and read their schema-v1
snapshots. This permission is also used by the player snapshot API. Modules
with `entity.data.read` or `entity.data.write` can enumerate opaque handles for
their own entity attachments without receiving snapshot access. Entity handles
are scoped to a module and live entity instance, expire when the entity leaves
its level or the module closes, and are limited to 1,024 live handles per
module. Enumeration returns all handles sorted by canonical UUID or an error;
it never loads chunks. See `SDK_GUIDE.md` and `ABI.md` for the record fields and
import encoding.

Applying a status effect also requires `entity.effect.apply` in the module
manifest and administrator grant, together with `entity.read`. The host accepts
only a loaded living non-player handle and a registered `namespace:path` effect
ID, with a duration of 1–72,000 ticks and amplifier of 0–255. Calls run on the
owning server thread and do not load chunks.

Entity attachments are raw bytes saved in the entity's own NeoForge persistent
NBT under SJML's fixed `sjml:entity_attachments` key. Values are partitioned by
package ID, module ID, and a module-local key; no Java object or foreign mod
namespace crosses the ABI. `entity.data.read` permits reads and
`entity.data.write` permits writes and removals. Keys are lowercase paths up to
128 UTF-8 bytes; values are at most 16 KiB. Per module, at most 128 values and
64 KiB of accounted bytes may be stored on an entity; all SJML buckets together
are bounded to 1,024 values and 256 KiB. Empty data and missing keys remain
distinct. Entity data follows entity save/load and dimension transfer. It is
separate from per-save entity-scope storage, which is a mod-wide UUID-keyed
database not stored in entity NBT.

Entity creation requires `world.read` plus the separate `entity.spawn` grant.
Only registered non-player types can be spawned into loaded chunks, and each
module is limited to eight spawns per server tick.

## Read-only block inventory permission

A module can request `world.read` and `inventory.read` to query a loaded block's
item-handler capability. An administrator must grant both capabilities in the
same `[[grant]]` record; inventory access is enabled only by their intersection.
The API accepts an existing module-owned world handle and block position,
supports an optional side, returns detached slot values, and never loads a
chunk. Results are limited to 4,096 slots and 1 MiB. See `SDK_GUIDE.md` and
`ABI.md` for its exact behavior and stable encoding.
