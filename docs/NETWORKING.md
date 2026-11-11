# Networking

SJML channels are declared by their owning module in `sjml.toml`. A runtime subscription enables a callback; it does not create a channel or change its negotiated schema. A channel identity has three separate parts: package ID, module ID, and channel name.

```toml
[[modules]]
id = "server"
path = "server.wasm"
side = "server"
permissions = ["network.mod"]

[[modules.channels]]
name = "chat"
schema_version = 1
directions = ["client_to_server", "server_to_client"]
```

The containing package supplies the package ID. The schema version is a positive `u32`. Directions must form a nonempty set of `client_to_server` and/or `server_to_client`. The host validates declarations and granted permissions. Access to a different module's channels additionally requires `network.external`.

## Compatibility

The handshake includes ABI and protocol versions, installed and required peer mods, supported and required capabilities, and declared channels. A shared channel's schema version and direction set must match exactly. A mismatch rejects the connection. A peer missing a channel alone remains compatible. `distribution = "both_required"` advertises an exact-version self requirement, so the absence of that package on either peer rejects the SJML handshake. Server-only packages with server-executed network channels or the `content.register` permission are also automatically made peer-required, since remote channel handling or synchronized registry definitions can depend on a client copy. A plain server-only package with no peer dependencies can run when clients have neither the package nor SJML. Client-only packages never require a server copy. Manifests with no distribution field keep the legacy optional package behavior. No handshake or payload transfers or installs WASM implicitly.

The loader's own `sjml:network_v1` transport is an optional NeoForge payload.
On the target NeoForge 21.1.265 runtime, `NetworkComponentNegotiator` removes
optional components missing on the opposite side and only rejects missing
non-optional components. FML 4.0.45 does not expose the older Forge
`IExtensionPoint.DisplayTest` API or a `displayTest` mod metadata field, so
SJML does not add one to `neoforge.mods.toml`. The adapter checks the negotiated
channel before every send. If local peer requirements exist but the optional
channel is absent, SJML disconnects with a compatibility reason instead of
silently ignoring the required handshake. With no peer requirements it skips
the handshake and allows the connection.

## Guest subscriptions

```rust,ignore
use sjml::network::{self, ChannelRef, NetworkContext};
use sjml::{SjmlResult, StableDecode};

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ChatMessage {
    text: String,
}

fn install_handler() -> SjmlResult<()> {
    let subscription = network::subscribe(
        ChannelRef::local("chat")?,
        |context: NetworkContext, message: ChatMessage| {
            // Apply application validation before using this peer-supplied message.
            let _ = (context, message);
            Ok(())
        },
    )?;
    // Keep the copyable handle to remove the handler when appropriate.
    network::unsubscribe(subscription)?;
    Ok(())
}
```

`ChannelRef::new("other_package", "other_module", "chat")?` references an explicit owner. Constructors reject malformed identifiers before calling the host. They do not resolve declarations or grant permissions.

A subscription owns a `'static` `FnMut(NetworkContext, T) -> SjmlResult<()>` closure, with `T: StableDecode`. The callback receives a detached decoded value. A malformed or trailing payload fails before callback execution. Callbacks can capture guest state and can subscribe or unsubscribe while running. Handles belong to one module instance; a copied stale handle cannot reactivate its callback. Host unload must invalidate subscriptions and pending deliveries before destroying the instance.

Callbacks are local to one guest and do not require Rust `Send`. The SDK bounds registrations to 1,024 live callbacks per instance and retains monotonically increasing callback IDs to prevent stale queued messages from addressing a replacement callback. Host quotas may be stricter.

## Context, players, and replies

Each callback receives `NetworkContext` with read-only field access:

- `channel: ChannelRef`: the host-resolved channel identity.
- `direction: NetworkDirection`: `ClientToServer` or `ServerToClient`.
- `side: LogicalSide`: `Client` or `Server`, including logical-server delivery within an integrated client.
- `player_uuid: Option<Uuid>`: the authenticated connection identity; client deliveries may omit it.

These fields live in an immutable snapshot behind read-only `Deref`; no mutable access is exposed. `is_server()` and `is_client()` inspect the receiving side. `Uuid` stores 16 bytes and parses/formats canonical `8-4-4-4-12` text without an external dependency.

`context.player()?` resolves the UUID through the existing player API, which requires `entity.read`. It calls the permission-gated API even when the UUID is absent, then returns `None` for an absent UUID or a session that has ended. Permission and other host failures remain typed errors. The returned `Player` wraps a private `PlayerHandle` and provides `snapshot()`. `Player::from_handle` wraps a typed handle obtained from `player::online()`; host validation still occurs on each use.

`context.reply(&message)?` replies to the currently executing callback's authenticated peer. The host must allow the channel's reverse direction and validate the live connection, module, capability grants, negotiated channel, and payload bound. No connection ID is exposed. Reply authority expires when the callback returns, including failure returns. A retained clone or deserialized context cannot extend or recreate that authority; expired replies return `ErrorCode::StaleHandle`. Asynchronous replies retained beyond the callback are deliberately unsupported. The host must return a typed stale-handle error if the connection has already closed.

The SDK never derives `player_uuid` or routing authority from application payload bytes. The host must construct context from its authenticated connection metadata, never from peer-controlled context fields.

## Explicit sends

- `send_to_server(channel, &message)`: client → server.
- `send_to(channel, player, &message)`: server → one player's connected client; accepts either an existing `PlayerHandle` or the `Player` returned by `context.player()`.
- `broadcast(channel, &message)`: server → connected clients with the negotiated channel.

Messages implement `StableEncode`. Success means host acceptance for queuing; it is not a peer acknowledgment or a guarantee of delivery. The host rejects calls from the wrong logical side, undeclared channels/directions, missing permissions, stale target sessions, unavailable transport, and incompatible peers. A client lacking the channel is not a broadcast recipient. There is no ambiguous generic `send` function.

`MAX_MESSAGE_BYTES` is 1,048,262 encoded bytes: the 1 MiB ABI limit minus 314 bytes reserved for the largest delivery envelope, including two maximum-length owner IDs, a maximum-length channel name, typed context, and optional UUID. Both sends and incoming delivery decoding enforce this bound. Stable encoding retains its independent 64 KiB string and 65,536-item collection bounds. Binary application schemas can implement `StableEncode`/`StableDecode` directly when a `Vec<u8>` collection bound would be too restrictive.

## Guest ABI and host integration

The SDK imports `sjml_network_v1` functions. All pointers/lengths are wasm32 `i32`; statuses are zero for success or an existing typed ABI error code for failure.

| Import | Parameters |
| --- | --- |
| `subscribe` | channel pointer, channel length, callback ID `i64` (unsigned u64 bits), 24-byte handle output pointer |
| `unsubscribe` | 24-byte handle pointer, handle length |
| `send_to_server` | channel pointer, channel length, message pointer, message length |
| `send_to` | channel pointer, channel length, player-handle pointer, player-handle length, message pointer, message length |
| `broadcast` | channel pointer, channel length, message pointer, message length |
| `reply` | channel pointer, channel length, message pointer, message length |

`ChannelRef` encoding is `u32 schema=1`, `u8 scope` (`0` local, `1` explicit owner), optional package/module stable strings for scope 1, then the channel stable string. Local identity resolves against the calling module. The host enforces `network.mod` and, for other owners, `network.external`; the guest has no authority to grant capabilities to itself.

The host dispatches `sjml_v1_network_dispatch(pointer, length) -> i64` with the standard packed response span. The input layout is `u32 schema=1`, `u64 callback_id`, `NetworkContext`, then the message's remaining stable bytes with no collection wrapper. Context layout is `u32 schema=1`, `ChannelRef`, `u32 direction` (1 C2S, 2 S2C), `u32 side` (1 Client, 2 Server), and `Option<Uuid>` (u8 tag 0/1; sixteen raw bytes when present). Direction and receiving side must agree. The opaque reply scope is guest-local and absent from this encoding.

The host allocates/initializes input via `sjml_v1_alloc`, retains the allocation during dispatch, then releases it through `sjml_v1_dealloc_input`. It reads the standard status-framed response and releases it via `sjml_v1_dealloc`. The entire delivery input is bounded to 1 MiB. The host must enqueue delivery until the current guest invocation completes, preserve the module's owning lane, and never call the guest reentrantly from `subscribe`, sends, or `reply`. The SDK additionally rejects nested network dispatch.

To support `reply`, the host establishes an authenticated current-delivery scope around that export and removes it immediately afterward. It validates the requested channel against the original active delivery and routes only to that delivery's live peer. A guest-created context encoding cannot supply routing authority. Queued deliveries must be canceled or fail closed after unsubscribe, disconnect, or unload. The host must invalidate every scoped handle and destroy the guest instance on unload. Handler-returned errors are isolated from other callbacks; a WASM trap requires host containment and appropriate instance teardown.

The NeoForge adapter registers the payload, exchanges authenticated handshake metadata, tracks connection-scoped negotiated sessions, binds the guest imports, and queues deliveries on the owning runtime lane. Adapter tests exercise the payload codec, session negotiation, and subscription flow in memory. A live Minecraft client/server session has not been run, so end-to-end handshake negotiation and payload routing remain unverified. Native SDK operations report unavailable host imports; they do not simulate transport.
