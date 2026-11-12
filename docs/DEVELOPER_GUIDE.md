# SJML command-line tools

The `sjml` Rust executable builds SDK guest projects and creates `.sjml` archives
accepted by the package format in [FORMAT.md](FORMAT.md). It does not run
Minecraft, grant permissions, or install/reload mods in a running server.

## Install and create a project

From this checkout:

```sh
cargo install --path sjml-cli --locked --offline
rustup target add wasm32-unknown-unknown
sjml-cli init ./hello --id hello --sdk-path ./sjml-cli/sjml-sdk
sjml-cli check --project ./hello --offline
sjml-cli test --project ./hello --offline
sjml-cli package --project ./hello --offline
sjml-cli inspect ./hello/dist/hello-0.1.0.sjml
```

`--offline` requires cached Rust dependencies; `rustup target add` requires the
standard library target to be installed separately. Without `--sdk-path`, the
generated Cargo manifest declares `sjml-sdk = "0.65.0"` and Cargo must resolve
that dependency through the user's configured registry. Use the local path
shown above when developing against this checkout. `--sdk-path` identifies the
`sjml-sdk` package directory, not the SDK workspace directory.

`init` refuses an existing destination and validates the ID and SDK path before
creating the project. It generates `Cargo.toml`, `sjml.toml`,
`sjml-project.toml`, `src/lib.rs`, and `.gitignore`. The initial server module
requires no host permissions. Add behavior using [SDK_GUIDE.md](SDK_GUIDE.md),
then declare the corresponding capabilities in `sjml.toml` and configure the
server's administrator grants separately.

## Commands

| Command | Behavior |
| --- | --- |
| `init <directory> [--id <id>] [--sdk-path <path>]` | Create a new Rust guest project. |
| `build --project <directory>` | Incrementally compile mapped libraries for `wasm32-unknown-unknown`, validate their WASM, and copy them to the declared module paths. |
| `check --project <directory>` | Validate the strict package manifest and Cargo/module mappings, then run Cargo's WASM type checker. It does not require prebuilt module files. |
| `test --project <directory>` | Run `cargo test --workspace` using the user's native Cargo test configuration. Host imports remain unsupported in native SDK tests. |
| `package --project <directory>` | Build the modules, collect payloads, regenerate sorted SHA-256 declarations, and atomically publish an archive. |
| `package --no-build --project <directory>` | Package existing WASM files at the manifest's declared paths. No Cargo workspace is required. |
| `dev --project <directory>` | Build/package once, then poll project changes and rebuild/package again. A rebuild error is printed while the watcher remains active. Stop with Ctrl-C. |
| `dev --once --project <directory>` | Build/package once and exit, for scripts and editor tasks. |
| `inspect <archive.sjml>` | Validate bounded ZIP data, paths, manifest, module WASM and declared hashes; print package identity, modules, and sorted file hashes/sizes. No extraction occurs. |

Build/check/test/package/dev accept `--release`, `--offline`, and `--locked`.
Without `--release`, Cargo's debug profile is used. `--locked` requires a
compatible existing lockfile. `package` and `dev` accept `--output <file.sjml>`;
the default is `<project>/dist/<id>-<version>.sjml`. Relative output paths are
relative to the invoking working directory. Outputs must end in `.sjml` and
must not overwrite project inputs or reside inside `assets`, `data`, `config`,
or `src`, including through a parent-directory symlink.

`dev` polls every 500 ms by default; `--interval-ms` accepts 50–60000 ms. It
ignores top-level `target`, `.git`, `dist`, `.sjml`, compiled module outputs, symlinks,
and its selected output archive. Watch scans stop at 100,000 filesystem
entries. Source watching uses modification times and sizes; changes outside
the project directory require a manual rebuild. Restart Minecraft after
changing a loaded package; this command does not imply runtime hot reload.
With `dev --no-build`, declared WASM files are watched as package inputs.

## Multiple crates

The package manifest remains the host-compatible `sjml.toml`; build mapping
belongs in the separate `sjml-project.toml`:

```toml
[[crates]]
module = "content"
package = "my-content"

[[crates]]
module = "logic"
package = "my-logic"
```

Both packages must be members of the Cargo workspace rooted at the project's
`Cargo.toml`. Each selected package must expose exactly one `cdylib` target.
The `module` value names its existing `[[modules]].id`; its compiled artifact
is copied to that module's existing `path`. Multiple modules can deliberately
use one crate. Different crates cannot overwrite a shared module path with
different bytes.

A one-module, one-package project needs no mapping file. Otherwise an unmapped
module is matched to a workspace package with the same name, treating `-` and
`_` alike; explicit mappings remove ambiguity. Unknown or duplicate mappings
fail before compilation. Cargo receives one invocation containing all selected
packages, retaining its existing target directory and incremental build cache.
The CLI does not clean build outputs or spawn one redundant build per module.
Cargo selects dependencies, features, and compiler settings from the existing
Cargo manifests/configuration; there is no second Rust dependency system.

Manifest `[[modules.channels]]` declarations are preserved and validated using
the host contract: lowercase dot-qualified names up to 128 bytes, positive u32
`schema_version`, and one or two distinct `client_to_server`/`server_to_client`
directions. Channel names are unique within a module and at most 1024 channels
are allowed across the package. This validates declarations; it does not test
an actual client/server handshake or grant networking permissions.

## Package inputs and deterministic output

Payloads are the declared module paths, any paths already listed in `[[files]]`,
and regular files recursively beneath `assets/`, `data/`, and `config/`.
Other files, including Rust source, Cargo metadata, and `sjml-project.toml`, are
excluded. Project payload symlinks, special files, traversal, absolute paths,
backslashes, non-NFC names, and control characters fail validation.

The source `sjml.toml` is preserved. Its packaged copy is serialized with fresh
SHA-256 checksums for every payload; old source checksum declarations are
regenerated, not used as an assertion about newly compiled files. Entries and
checksum records are sorted. ZIP timestamps and permissions are fixed, and
entries use Stored compression so content cannot violate the reader's maximum
compression ratio. Identical payload bytes and manifest values produce identical
archives on the same toolchain. Compiled WASM reproducibility itself depends
on the Rust compiler, build settings, paths, and build scripts.

Creation and inspection enforce the format's 256 MiB archive, 64 MiB entry,
128 MiB total, 1 MiB manifest, and 4096-entry limits. Inspection also checks the
200:1 compression ratio, CRCs, central/local header agreement, UTF-8 names,
duplicate paths, entry sizes, and missing modules. Source traversal includes
empty directories in its shared 4096-node bound. The strict manifest permits
at most 256 `[[files]]` records, so creation, which hashes every payload, permits
at most 256 payload files. Inspection accepts existing packages with more
entries when their declarations and other format limits permit them.

The CLI reads standard single-disk ZIP archives using Stored or Deflate
compression. Encrypted, split, ZIP64, prefixed, overlapping, or trailing-data
archives are rejected with an actionable error; recreate such packages with
`sjml-cli package`. Nothing is extracted during inspection. Output is written to a
temporary file beside the destination, validated again, and renamed over the
destination only after validation succeeds. Failed validation preserves a
previous archive.

Publisher signature metadata is structurally validated during inspection but
is not cryptographically authenticated. Packaging refuses existing signature
metadata because regenerating the manifest/hashes could invalidate it. Neither
successful inspection nor successful compilation proves runtime compatibility,
permission grants, dependency availability, or successful Minecraft activation.

## Errors and verification

Failures exit with status 1 and a stable diagnostic prefix:
`E_INIT`, `E_PROJECT`, `E_MANIFEST`, `E_CARGO`, `E_PATH`, `E_LIMIT`,
`E_ARCHIVE`, `E_CHECKSUM`, `E_WASM`, or `E_IO`. Argument parsing/help follows
Clap's standard exit behavior (invalid arguments: status 2). Cargo diagnostics
are retained alongside the CLI's failure summary.

Run the Rust workspace tests and CLI checks from the repository root:

```sh
CARGO_TARGET_DIR=build/tmp/task-15-cli-target cargo test --manifest-path sjml-cli/Cargo.toml --workspace --locked --offline
CARGO_TARGET_DIR=build/tmp/task-15-cli-target cargo clippy --manifest-path sjml-cli/Cargo.toml -p sjml-cli --all-targets --locked --offline -- -D warnings
```

The integration tests generate real guest projects, run Cargo offline, build
WASM for the installed target, test native code, package/inspect generated
archives, and exercise workspace mappings and adversarial input handling.
Their temporary project directories stay under `build/tmp/task-15-cli-tests`.
