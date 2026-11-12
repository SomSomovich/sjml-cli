# Client localization lookups

A physical-client module can resolve a translation key through Minecraft's
currently selected language and loaded resource packs:

```toml
[[modules]]
id = "client"
side = "client"
permissions = ["client.localization.read"]
```

The server administrator must also grant `client.localization.read` to the
module in `config/sjml-permissions.toml`:

```toml
[[grant]]
module = "example/client"
capabilities = ["client.localization.read"]
```

```rust
use sjml::{client_localization, SjmlResult};

fn menu_title() -> SjmlResult<String> {
    client_localization::translate("screen.example.menu.title")
}
```

The lookup reads the language data active at the moment of the call. Package
language assets follow Minecraft's normal layout, such as
`assets/example/lang/en_us.json`; they become visible after Minecraft loads or
reloads those resources. A missing key follows Minecraft's fallback and
returns the key itself. The result is plain detached text, so translation
format placeholders are not expanded by this API.

Keys are limited to 256 UTF-8 bytes and results to 4,096 UTF-8 bytes. The call
must run on the physical client main thread. It never retains a language,
resource-manager, or component object and cannot be called during static
registration or from a dedicated-server module. If the client language data is
not ready, the host returns `SERVICE_BUSY`.

Host tests exercise key validation, permission denial, thread checks, missing
resource readiness, and bounded result copying. A display-backed test is still
needed to check a package language file after a live resource reload.
