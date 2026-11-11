# `sjml-sdk`

`sjml-sdk` is the Rust guest API for Somuch Java Mod Loader (SJML). Its Rust
crate name is `sjml`; guest projects normally depend on it like this:

```toml
[dependencies]
sjml-sdk = "0.41.0"
```

SJML guests target `wasm32-unknown-unknown`. The SDK also builds natively so
guest logic and data types can be tested, but native builds do not provide the
host imports or WebAssembly callback exports.

SDK 0.41.0 includes the client-only `sjml::client_chat` API for native chat
layout control. It requires `client.chat.modify`; see the [SDK guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SDK_GUIDE.md)
and [chat API contract](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CHAT.md).

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    host::log(LogLevel::Info, "guest initialized")?;
    Ok(())
}
```

See the [SDK guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SDK_GUIDE.md)
for callback macros, API modules, capability requirements, bounds, and examples;
the [docs.rs API reference](https://docs.rs/sjml-sdk/0.41.0/sjml/) lists the
public Rust types and functions. The [ABI v1 specification](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ABI.md)
describes the raw WebAssembly interface. For `sjml.toml`, project layout, and
administrator grants, see the [project configuration guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PROJECT_CONFIGURATION.md)
and [manifest schema](https://github.com/SomSomovich/sjml-cli/blob/master/docs/MANIFEST_SCHEMA.md).

The full workspace and command-line tool are in the
[sjml-cli repository](https://github.com/SomSomovich/sjml-cli).
