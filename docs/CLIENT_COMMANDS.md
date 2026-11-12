# Client commands

Modules can register bounded commands in the local Minecraft command dispatcher. These commands run only on the physical client, on its main thread. They do not send a request to the server or grant authority over server state.

Declare client commands with `[[modules.client_commands]]` and set the module side to `"client"` or `"both"`:

```toml
[[modules]]
id = "client"
path = "modules/client.wasm"
side = "client"
permissions = ["client.command"]

[[modules.client_commands]]
name = "tools.say"
schema_version = 1
permission_level = 0

[[modules.client_commands.arguments]]
name = "message"
type = "greedy_string"
```

The module must request `client.command` in `permissions` and the local Minecraft instance must grant it in `config/sjml-permissions.toml`:

```toml
[[grant]]
module = "my-mod/client"
capabilities = ["client.command"]
```

The Rust `#[client_command]` macro exports the typed callback as `sjml_v1_client_command_<name>`. The callback input is a `StableDecode` type whose fields match the declared arguments in order, including the schema version. Its output must be `SjmlResult<String>`:

```rust
use sjml::prelude::*;

#[derive(StableDecode)]
#[sjml(version = 1)]
struct SayArguments {
    message: String,
}

#[client_command(name = "tools.say")]
fn say(arguments: SayArguments) -> SjmlResult<String> {
    Ok(format!("Local message: {}", arguments.message))
}
```

Run it as `/sjmlclient my-mod client tools.say hello`. The root is namespaced to keep these local commands separate from server commands. `permission_level` is checked against the local command source; use `0` for commands available to ordinary local users. It is not a server authorization mechanism.

Supported argument types are `word`, `string`, `greedy_string`, `integer`, and `boolean`. Integer arguments may specify inclusive `min` and `max` bounds. A greedy string must be the final argument. A module may declare at most 128 server and client commands combined across both command arrays; names must be unique across them. Each command may have up to 16 arguments, each string is limited to 64 KiB of UTF-8, and the complete callback payload is limited to 1 MiB. Callback feedback is one UTF-8 string limited to 64 KiB. Empty feedback emits no success message.

The host validates the callback export before loading the package, checks the client side and capability again at registration and execution, and resolves the active client module for each call. If a callback traps, returns an error, or produces an invalid response, the host reports a generic local failure and disables that module instance; the detailed cause is written to the log. Client commands do not support guest-provided suggestions or server-tree changes.
