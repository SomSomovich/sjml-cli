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
sjml-cli init mymod --sdk-path /path/to/somuch-jml/sdk/sjml
```

Without `--sdk-path`, the generated Cargo manifest depends on `sjml-sdk =
"0.40.0"`; that crate must be available from a configured Cargo registry.
Until the SDK crates are published, use `--sdk-path` when you need to compile
the generated project.

## Documentation

- [Rust SDK guide](docs/SDK_GUIDE.md) and [SDK API reference on docs.rs](https://docs.rs/sjml-sdk/0.41.0/sjml/)
- [WebAssembly ABI v1](docs/ABI.md)
- [Project configuration and capability grants](docs/PROJECT_CONFIGURATION.md)
- [Complete manifest schema](docs/MANIFEST_SCHEMA.md) and [`.sjml` archive format](docs/FORMAT.md)
- [Compatibility and host support](docs/COMPATIBILITY.md)
- [Networking](docs/NETWORKING.md), [storage](docs/STORAGE.md), [async services](docs/ASYNC_MODEL.md), and [client rendering/OpenGL](docs/RENDERING.md)
- [Security boundaries](docs/SECURITY.md), [module lifecycle](docs/MODULE_LIFECYCLE.md), [block entities](docs/BLOCK_ENTITIES.md), [particles](docs/PARTICLES.md), and [worker compute](docs/WORKER_COMPUTE.md)

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
