# Build and troubleshooting

## First build

Install Java 21, Rust 1.85 or newer, and Rust's WebAssembly target, then run
the repository checks from the root:

```sh
rustup target add wasm32-unknown-unknown
./gradlew :sjml-core:test
./gradlew --configuration-cache test build runGameTestServer
cargo fmt --manifest-path sjml-cli/Cargo.toml --all -- --check
cargo test --manifest-path sjml-cli/Cargo.toml --workspace --locked
cargo build --manifest-path sjml-cli/Cargo.toml -p sjml-sdk --example guest \
  --target wasm32-unknown-unknown --locked
```

The cross-platform CI workflow also runs strict Clippy checks. See
[Compatibility](COMPATIBILITY.md) for the versions and coverage this checkout
supports. The root `test`, `build`, and GameTest server tasks have been verified
with Gradle's configuration cache enabled.

In the managed Linux workspace, run `. scripts/cloud-env.sh` first. It selects
the task-owned Rust and Gradle caches and configures Java's proxy from the
workspace proxy URL without copying any proxy credentials into JVM options.

## Create and package a guest

Use `sjml-cli init <directory> --id <lowercase-id> --sdk-path sjml-cli/sjml-sdk` to create a
project that uses this checkout's SDK. Build, test, and package it with the
CLI, for example:

```sh
cargo run --manifest-path sjml-cli/Cargo.toml -p sjml-cli -- init ./my-mod \
  --id my_mod --sdk-path ./sjml-cli/sjml-sdk
cargo run --manifest-path sjml-cli/Cargo.toml -p sjml-cli -- check --project ./my-mod
cargo run --manifest-path sjml-cli/Cargo.toml -p sjml-cli -- test --project ./my-mod
cargo run --manifest-path sjml-cli/Cargo.toml -p sjml-cli -- package --project ./my-mod
```

`sjml-cli check` validates manifest/module correspondence, `sjml-cli build` compiles
every declared Cargo module to WebAssembly, and `sjml-cli package` writes the
reproducible archive under the project's `dist/` directory unless an output is
provided. `sjml-cli inspect <archive.sjml>` reports validated package contents.
The CLI does not install or hot-reload a package in Minecraft.

## Common failures

- **`wasm32-unknown-unknown` target missing:** run `rustup target add wasm32-unknown-unknown` for the toolchain selected by `rustup show`.
- **Module requests an unbound import:** verify the module side, required API version, declared capability, and host permission grant. Static `mod_register` calls intentionally deny runtime-only APIs.
- **`PERMISSION_DENIED`:** add the capability to the module's `permissions` list and grant the exact `<package>/<module>` pair in `config/sjml-permissions.toml`; both are required.
- **Stale or owner-mismatched handle:** discard the handle and reacquire it from the owning module after a world/session/runtime lifecycle change. Handles cannot be transferred between module instances.
- **No server result during early initialization:** dedicated `mod_init` may run before a server is bound. Subscribe to `server.started` for operations that need loaded worlds or players.
- **Rendering imports absent on a dedicated server:** client render and OpenGL code is only bound to client module instances and requires the corresponding manifest requests and administrator grants.
- **Network handshake mismatch:** check the declared channel schema/direction on both packages and their negotiated runtime compatibility; live end-to-end behavior still needs a connected client and server.
- **Archive rejected:** use `sjml-cli inspect` and preserve its specific diagnostic; the loader rejects traversal paths, malformed manifests, missing modules, and checksum mismatches before activation.

Runtime startup diagnostics are available through `/sjml status` and the
server log. Keep package IDs, module IDs, ABI errors, and sanitized loader
messages when reporting a failure; do not include private permission files or
credentials.
