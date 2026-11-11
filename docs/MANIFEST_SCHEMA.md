# `sjml.toml` manifest schema

This is the complete TOML key reference for a package manifest. Keys not listed
here are rejected by the host parser. For the files placed in an archive and
their limits, see the [`.sjml` package format](FORMAT.md). For the files made by
`sjml-cli init` and Cargo workspace mapping, see
[project configuration](PROJECT_CONFIGURATION.md).

## Top-level tables and arrays

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `[mod]` | table | yes | Package identity, metadata, and peer installation policy. |
| `[runtime]` | table | yes | ABI and optional host-version compatibility ranges. |
| `[[modules]]` | array of tables | yes | One or more WebAssembly guest modules. |
| `[[dependencies]]` | array of tables | no | Other SJML packages required or optionally used by this package. |
| `conflicts` | array of strings | no | Package IDs that cannot be installed alongside this package. |
| `[[files]]` | array of tables | no | Optional path and SHA-256 declarations for archive entries. `sjml-cli package` generates these for its output. |
| `[signature]` | table | no | Publisher signature metadata. The current host checks its structure but does not cryptographically authenticate it; the CLI cannot repackage a manifest that already contains this table. |

## `[mod]`

| Key | Type | Required | Default or allowed values |
| --- | --- | --- | --- |
| `id` | string | yes | Lowercase package ID, 1–64 characters, starts with a letter; remaining characters are lowercase letters, digits, `_`, or `-`. |
| `name` | string | yes | Human-readable, non-empty name. |
| `version` | string | yes | Semantic Versioning version, such as `0.1.0`. |
| `authors` | array of strings | no | Empty when omitted. |
| `description` | string | no | Empty when omitted. |
| `license` | string | no | Empty when omitted. |
| `metadata` | table | no | Application-defined TOML metadata; bounded to depth 8, 4,096 values, 256 entries per table/array, and 4,096 characters per string. |
| `distribution` | string | no | `optional` (default), `server_only`, `client_only`, or `both_required`. See [package distribution](FORMAT.md#package-distribution). |

## `[runtime]`

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `abi` | string | yes | ABI compatibility range. The host currently implements ABI `1.32.0`. |
| `minecraft` | string | no | Minecraft compatibility range. |
| `sjml` | string | no | SJML loader compatibility range. |

Ranges use the loader's SemVer constraint syntax: exact versions, `*`, and
`^` or `~` ranges. The range is checked by the host at validation or
activation. It does not select or download a host version.

## `[[modules]]`

| Key | Type | Required | Default or allowed values |
| --- | --- | --- | --- |
| `id` | string | yes | Unique module ID using the package ID character rules. |
| `path` | string | yes | Relative archive path ending in `.wasm`; normally `modules/<id>.wasm`. |
| `side` | string | no | `client`, `server`, or `both`; default `both`. Selects the logical runtime where the module is instantiated. |
| `depends` | array of strings | no | Module IDs in this package that must be available first. Every ID must name a declared module. |
| `permissions` | array of strings | no | Capability requests; each must also be granted by the server administrator before protected operations are enabled. |
| `provides` | array of strings | no | Service IDs this module implements. Each must also have a matching entry in `exports`. |
| `[[modules.exports]]` | array of tables | no | Versioned service exports and their visibility. |
| `[[modules.commands]]` | array of tables | no | Guest commands declared for this module; client-only modules cannot declare commands. |
| `[[modules.channels]]` | array of tables | no | Versioned network channels and allowed directions. |

`side` is an execution setting; `[mod].distribution` is a separate peer
installation setting. For example, `side = "server"` does not by itself mean
that clients must or must not install the package. See the
[side and distribution guide](PROJECT_CONFIGURATION.md#sjmltoml).

### `[[modules.exports]]`

| Key | Type | Required | Default or allowed values |
| --- | --- | --- | --- |
| `id` | string | yes | Service ID; must match an ID in the same module's `provides` list. |
| `version` | string | yes | Semantic Versioning service version. |
| `visibility` | string | no | `private` (default) or `public`. Private exports can be resolved only within the same package; public exports may be resolved by other packages. |

Server-side callers also declare a package dependency when they consume another
package's service. Callers need the `service.call` capability. Client service
calls and client service providers are currently unavailable. See
[async services](ASYNC_MODEL.md#cross-module-services).

### `[[modules.commands]]` and `[[modules.commands.arguments]]`

| Command key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `name` | string | yes | Callback name matching the Rust `#[command(name = "...")]` attribute. |
| `schema_version` | integer | yes | Positive `u32`, matching `#[sjml(version = N)]` on the Rust input record. |
| `permission_level` | integer | yes | Brigadier permission level from 0 through 4. |
| `arguments` | array of tables | no | Ordered command arguments; their order and types must match the Rust record fields. |

Each argument accepts `name` (unique identifier) and `type` (`word`, `string`,
`greedy_string`, `integer`, or `boolean`). Integer arguments may also set
inclusive signed-32-bit `min` and `max`. A `greedy_string` must be the final
argument. Command names are limited to 128 characters, 64 commands per module,
and 16 arguments per command. Commands require the `content.register`
capability. See [guest command declarations](FORMAT.md#guest-command-declarations)
and the [SDK command guide](SDK_GUIDE.md#manifest-declared-commands).

### `[[modules.channels]]`

| Key | Type | Required | Meaning |
| --- | --- | --- | --- |
| `name` | string | yes | Dot-separated lowercase channel name, at most 128 characters. |
| `schema_version` | integer | yes | Positive `u32`; peers must declare matching versions for a shared channel. |
| `directions` | array of strings | yes | Non-empty set containing `client_to_server`, `server_to_client`, or both. |

Channel use requires `network.mod`; using a channel declared by another module
also requires `network.external`. Declaring channels can make a
`server_only` package peer-required. See [networking](NETWORKING.md).

## `[[dependencies]]`, `conflicts`, and `[[files]]`

Each dependency table accepts:

| Key | Type | Required | Default or allowed values |
| --- | --- | --- | --- |
| `id` | string | yes | Other package ID; unique within this package. |
| `version` | string | yes | SemVer compatibility range for that package. |
| `optional` | boolean | no | `false`; if true, absence of that dependency does not by itself prevent activation. |
| `side` | string | no | `both`; `client`, `server`, or `both` selects the runtime side where this dependency is required. |

`conflicts` is a unique string array of package IDs. A package cannot conflict
with itself. Each `[[files]]` table accepts a safe relative `path` and a
64-character hexadecimal `sha256` digest. Archive validation compares declared
hashes with the exact stored bytes.

## `[signature]`

The table accepts `algorithm = "ed25519"`, a non-empty `key_id`, canonical
Base64 `public_key` (32 decoded bytes), and canonical Base64 `value` (64
decoded bytes). These checks validate shape only. A valid-looking table does
not prove publisher identity or package authenticity.

## Shared validation limits

The host accepts at most 1 MiB of UTF-8 manifest text, nesting of 32 TOML
levels, 256 entries in a manifest array, and strings up to 4,096 characters.
There can be at most 256 modules, 64 commands per module, 16 arguments per
command, and 1,024 channels total. Metadata has the tighter bounds listed
above. Archive path rules and ZIP limits are listed in the
[package format](FORMAT.md#archive-rules-and-current-limits).

`sjml-cli check` verifies the project manifest, Cargo module mapping, and Rust
target build. `sjml-cli package` validates and writes the archive. Neither
command launches Minecraft or proves that the target host accepts the package's
runtime requirements, capability grants, data-pack JSON, or multiplayer
distribution; those are checked by the installed host at runtime.
