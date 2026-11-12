# Scoreboard API

SJML exposes bounded reads and writes for entries in vanilla scoreboard
objectives that already exist on the logical server. The API never creates an
objective, team, display slot, or Minecraft score-holder object.

## Permissions and bounds

- `scoreboard.read` grants `sjml::scoreboard::score(objective, entry)`.
- `scoreboard.write` grants `sjml::scoreboard::set_score(objective, entry, value)`.
- Objective names contain 1–16 ASCII letters, digits, `.`, `_`, `+`, or `-`.
- Entry names contain 1–40 UTF-8 bytes and characters, have no control
  characters, and cannot start with `#` (reserved for hidden scoreboard rows).
- Scores are limited to −1,000,000,000 through 1,000,000,000.
- Only the active logical-server scoreboard can be accessed, and calls must run
  on its owning server thread. Client-side modules cannot use this API.
- Read-only objectives cannot be changed. `score()` returns `None` if the
  objective exists but the entry has no score; a missing objective is an
  invalid argument.

`set_score` creates or updates only the named entry within an existing,
writable objective. It does not create objectives or change objective
criteria, display names, display slots, teams, or other entries.

## Example

```rust
use sjml::scoreboard;

fn award_kill() -> sjml::SjmlResult<()> {
    let old = scoreboard::score("kills", "Alice")?.unwrap_or(0);
    scoreboard::set_score("kills", "Alice", old.saturating_add(1))
}
```

Declare only the permissions the module uses in `sjml.toml`:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "server"
permissions = ["scoreboard.read", "scoreboard.write"]
```

The server administrator must grant those permissions in
`config/sjml-permissions.toml` as usual. The guest API uses the stable
`sjml_scoreboard_v1` import module and a fixed 12-byte optional-score response.
