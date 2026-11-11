# Client toast notifications

Client-side Rust modules can show bounded vanilla-style toast notifications through the public
`sjml::client_notifications::notify` API. The host creates the toast on the Minecraft client main
thread and never exposes Minecraft objects to WASM.

```rust
use sjml::client_notifications::{notify, NotificationKind};

notify(
    "Download complete",
    "The resource pack is ready to use.",
    NotificationKind::Success,
    100, // ticks, or five seconds
)?;
```

The module must declare and the administrator must grant `client.ui.notify`:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "client"
permissions = ["client.ui.notify"]
```

```toml
[[grant]]
module = "example/main"
capabilities = ["client.ui.notify"]
```

`title` is limited to 128 UTF-8 bytes and `body` to 128 bytes. Both must be nonblank and contain
no control characters. Literal text is wrapped by Minecraft's font; it is not parsed as a
translation key or rich-text markup. Duration is 20–600 ticks (1–30 seconds), measured from the
toast’s first render; the Minecraft notification-speed preference does not change it. `Info`,
`Success`, `Warning`, and `Error` styles change the title color while retaining the vanilla toast
background.

Each runtime module owns one toast slot. A new notification updates that module's queued or
visible toast, preventing guest code from growing the vanilla toast queue. Calls are rate-limited
to six per module per 30 seconds; additional calls return `ServiceBusy`. Toasts are client-only:
server modules and static-registration callbacks receive a denied import. The API requires the
physical client and client main thread and cannot modify server state.

The ABI import is `sjml_client_ui_v1.notify(i32, i32, i32, i32, i32, i32) -> i32`, carrying the
title and body pointers/lengths, the notification kind (`0` through `3`), and duration ticks.
The host validates UTF-8 and all bounds before copying guest strings.

Unit/ABI tests cover permissions, static registration, physical-client and thread guards, current
grant revocation, invalid UTF-8/text bounds, duration/type codes, and rate limiting. Visible toast
layout and timing still need display-backed verification in a Minecraft client.
