# Package-owned client asset discovery

A physical-client module can list path names for assets included in its own `.sjml` archive. The
API indexes only `assets/<package-id>/` entries from the calling package. Paths are returned
relative to that directory; the call does not inspect Minecraft's global resource manager or
read/return file contents.

```rust
use sjml::client_assets;

let page = client_assets::list("textures/gui/", 32)?;
for path in page.paths {
    host::log(LogLevel::Info, &path)?;
}
if page.truncated {
    // Narrow the prefix to inspect one subtree at a time.
}
```

The prefix may be empty to list from the package asset root. Otherwise it must be a canonical
lowercase relative path prefix such as `textures/gui/`; absolute paths, empty segments, `.` and
`..` segments, backslashes, and control characters are rejected. Results are sorted and capped at
60 paths per call. Each path is at most 1,024 UTF-8 bytes; the request prefix is at most 256 bytes.
The result's `truncated` flag tells whether the limit omitted additional matching paths; narrow the
prefix to inspect a large asset tree in smaller groups.

The import is available only in a physical-client context on the client main thread and requires
the module to request `client.assets.read`, which the administrator must grant. Server-only modules
do not receive the import. A `both` module can use the API from its client instance; calling it from
its server instance returns `PERMISSION_DENIED`.

```toml
[[modules]]
id = "client"
path = "modules/client.wasm"
side = "client"
permissions = ["client.assets.read"]
```

The list reports archive paths, not resource availability after a client resource reload. Package
authors still include assets in the ordinary layout, for example
`assets/example/textures/gui/status.png`.
