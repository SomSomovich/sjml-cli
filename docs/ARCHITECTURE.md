# SJML architecture

SJML keeps portable module rules in `sjml-core` and puts Minecraft-specific
integration in the root NeoForge mod. Rust guests use the `sjml-sdk` crate and
are built for `wasm32-unknown-unknown`; the `sjml` CLI builds their Cargo
packages and produces validated `.sjml` archives.

```text
Rust guest crates ── sjml CLI ──> .sjml archive
                                      │
                                      ▼
NeoForge discovery -> archive/manifest validation -> dependency and side selection
                                      │
                         mod_register (before static registry freeze)
                                      │
                         NeoForge content registration
                                      │
                 runtime compile -> mod_init -> callbacks/imports
                                      │
        server APIs and storage       │       client GUI/world/OpenGL APIs
```

The loader validates archive paths, manifest data, declared WASM modules,
permissions, side selection, and package dependencies before runtime
activation. It calls the guest's static registration export before NeoForge
freezes item/block registries, then activates logical-side instances after
content registration. Client-only classes are reached through the client
entrypoint; common and dedicated-server bootstrap code uses client-free host
interfaces.

`SjmlWasmRuntime` parses modules with Endive, enforces memory and table limits,
compiles every function to JVM bytecode, and disables interpreter fallback.
The host binds only numeric WebAssembly functions explicitly present in a
module's allowlist. Optional Minecraft APIs bind only when the module requests
the capability and the administrator grants it. Guest memory crosses the
boundary only through bounded, checked copies; the ABI does not expose JVM or
Minecraft object references.

Module callbacks use stable little-endian values and scoped opaque handles.
World, player, entity, inventory, block-entity, storage, event, network,
render, and OpenGL imports validate the owning module and execution context;
thread-affine APIs validate their server or client-render thread before using
Minecraft state. Closing a runtime invalidates its handles and subscriptions
and schedules cleanup for resources that need a live render context.

The Rust SDK owns typed serialization, callback closures, and local async
futures. The host decides when a future is polled and on which configured lane.
Cross-module services copy bounded request and response bytes. Persistent
records live in the current save under a per-mod namespace. Networking uses
declared channels, versioned payloads, and a login handshake before routing
messages.

The portable core and NeoForge adapter do not have identical coverage. See
[Compatibility](COMPATIBILITY.md), [the ABI](ABI.md), [the async model](ASYNC_MODEL.md),
[networking](NETWORKING.md), [storage](STORAGE.md), and [client rendering](RENDERING.md)
for implemented surfaces and explicit verification limits.
