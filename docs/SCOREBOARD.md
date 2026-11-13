# Scoreboard API

SJML exposes bounded reads and writes for entries in existing vanilla
scoreboard objectives, guarded display-slot assignment for existing objectives,
and a separate API for teams owned by one SJML module. Modules cannot create or
change objectives or vanilla and other-module teams.

## Permissions and bounds

- `scoreboard.read` grants `sjml::scoreboard::score(objective, entry)`.
- `scoreboard.write` grants `sjml::scoreboard::set_score(objective, entry, value)`.
- `scoreboard.write` also grants `set_display_slot(objective, slot)` and
  `clear_display_slot(slot)`.
- `scoreboard.read` also grants `sjml::scoreboard::team(id)`.
- `scoreboard.write` grants module-owned team creation, removal, options, and
  player membership operations.
- Objective names contain 1–16 ASCII letters, digits, `.`, `_`, `+`, or `-`.
- Entry names contain 1–40 UTF-8 bytes and characters, have no control
  characters, and cannot start with `#` (reserved for hidden scoreboard rows).
- Scores are limited to −1,000,000,000 through 1,000,000,000.
- Only the active logical-server scoreboard can be accessed, and calls must run
  on its owning server thread. Client-side modules cannot use this API.
- Scores in read-only objectives cannot be changed. `score()` returns `None` if the
  objective exists but the entry has no score; a missing objective is an
  invalid argument.

`set_score` creates or updates only the named entry within an existing,
writable objective. It does not create objectives or change objective
criteria, display names, display slots, teams, or other entries.

`set_display_slot` assigns an existing objective to one fixed vanilla display
slot, replacing the objective currently shown there. `clear_display_slot`
removes the current objective from that slot and succeeds if it is already
empty. These calls can select or clear any of the 19 fixed vanilla slots; they
do not create objectives or alter the objective itself. Slot codes are stable:
`List` 0, `Sidebar` 1, `BelowName` 2, then `TeamBlack`, `TeamDarkBlue`,
`TeamDarkGreen`, `TeamDarkAqua`, `TeamDarkRed`, `TeamDarkPurple`, `TeamGold`,
`TeamGray`, `TeamDarkGray`, `TeamBlue`, `TeamGreen`, `TeamAqua`, `TeamRed`,
`TeamLightPurple`, `TeamYellow`, and `TeamWhite` (3–18). Display-slot calls use
the same server-thread and `scoreboard.write` guard as score mutations.

## Module-owned teams

Team IDs are local to one package/module. Use 1–64 lowercase ASCII letters,
digits, `_`, or `-`, starting with a letter. The host maps the owner identity
and local ID to a bounded opaque vanilla team name, so modules never choose or
address vanilla team names directly. An unknown existing team name is treated
as foreign. A module can only read, change, or delete a team with an exact
persisted owner record for its own package/module ID and local ID.

`create_team(id)` creates an empty team or succeeds if that exact team already
exists. `remove_team(id)` deletes only that owned team and its memberships.
The owner map is stored as strings in the current world's SavedData and is
reopened for each call; no server, scoreboard, or team object is retained
across a world reload. If the map is corrupt or a generated name has no exact
owner record, team operations fail closed.

`TeamOptions` sets literal display name, player prefix and suffix, color,
name-tag visibility, death-message visibility, entity collision, friendly
fire, and visibility of friendly invisible players. Display name is nonempty;
prefix and suffix may be empty. Each text value is limited to 128 UTF-8 bytes
and 64 Unicode characters, with control characters rejected. Text is passed
as a literal component; it is not parsed as formatting markup.

The enum codes are stable: colors are `Reset`, `Black`, `DarkBlue`,
`DarkGreen`, `DarkAqua`, `DarkRed`, `DarkPurple`, `Gold`, `Gray`, `DarkGray`,
`Blue`, `Green`, `Aqua`, `Red`, `LightPurple`, `Yellow`, `White` (codes 0–16
in that order). Visibility is `Always`, `Never`, `HideForOtherTeams`,
`HideForOwnTeam` (0–3). Collision is `Always`, `Never`, `PushOtherTeams`,
`PushOwnTeam` (0–3).

Player membership operations take a canonical lowercase UUID and require the
player to be online. A player already assigned to a vanilla or another
module's team cannot be moved. Removal affects the player only if they belong
to the requested owned team. Team and membership calls run on the logical
server thread; client-only modules cannot use them.

```rust
use sjml::scoreboard::{self, TeamColor, TeamCollisionRule, TeamOptions, TeamVisibility};

fn configure_team(player_uuid: &str) -> sjml::SjmlResult<()> {
    scoreboard::create_team("moderators")?;
    scoreboard::set_team_options("moderators", &TeamOptions {
        display_name: "Moderators".into(),
        prefix: "[Mod] ".into(),
        suffix: String::new(),
        color: TeamColor::Aqua,
        name_tag_visibility: TeamVisibility::Always,
        death_message_visibility: TeamVisibility::Always,
        collision_rule: TeamCollisionRule::Always,
        friendly_fire: false,
        see_friendly_invisibles: true,
    })?;
    scoreboard::add_player_to_team("moderators", player_uuid)
}
```

The host imports use `sjml_scoreboard_v1`: `team` reads a bounded detached
snapshot; `create_team`, `remove_team`, and `set_team_options` control owned
teams; `add_player_to_team` and `remove_player_from_team` control membership.
All reads require `scoreboard.read`; all mutations require
`scoreboard.write`. Both grants must also be declared in the module manifest
and explicitly granted by the server administrator as needed.

## Example

```rust
use sjml::scoreboard;
use sjml::DisplaySlot;

fn award_kill() -> sjml::SjmlResult<()> {
    let old = scoreboard::score("kills", "Alice")?.unwrap_or(0);
    scoreboard::set_score("kills", "Alice", old.saturating_add(1))
}

fn show_kills_sidebar() -> sjml::SjmlResult<()> {
    scoreboard::set_display_slot("kills", DisplaySlot::Sidebar)
}

fn hide_kills_sidebar() -> sjml::SjmlResult<()> {
    scoreboard::clear_display_slot(DisplaySlot::Sidebar)
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
