# Module-owned scoreboard objectives

Server-side modules can create, update, score, display, and remove their own
vanilla dummy objectives through `sjml::scoreboard`. The host derives a
16-character vanilla objective name from the package, module, and local ID, so
the Rust API never needs the generated name.

## Permissions and bounds

- `create_objective`, `set_objective_score`,
  `set_objective_display_slot`, and `remove_objective` require
  `scoreboard.write`.
- `objective_score` requires `scoreboard.read`.
- Local IDs use 1–64 lowercase ASCII letters, digits, `_`, or `-`, beginning
  with a letter.
- Display names are literal text, limited to 128 UTF-8 bytes and 64 Unicode
  characters, without control characters.
- Objectives use the writable vanilla `dummy` criterion and integer render
  type. Arbitrary criteria and custom number formats are not exposed.

`create_objective(id, display_name)` creates the objective if missing or
updates its display name if this exact package/module/local ID already owns it.
`objective_score` returns `None` until an entry has a score. `remove_objective`
deletes only the recorded owned objective; vanilla clears its score entries and
any display slots pointing to it.

```rust
use sjml::scoreboard::{self, DisplaySlot};

fn initialize_scoreboard() -> sjml::SjmlResult<()> {
    scoreboard::create_objective("round_points", "Round points")?;
    scoreboard::set_objective_score("round_points", "team_red", 12)?;
    scoreboard::set_objective_display_slot("round_points", DisplaySlot::Sidebar)
}

fn score_for_red_team() -> sjml::SjmlResult<Option<i32>> {
    scoreboard::objective_score("round_points", "team_red")
}

fn shutdown_scoreboard() -> sjml::SjmlResult<()> {
    scoreboard::remove_objective("round_points")
}
```

The owner tuple is stored in the world's SJML scoreboard SavedData and checked
on every operation. Existing vanilla or foreign objectives are never adopted.
The SavedData migration preserves existing team ownership records; corrupted
ownership data disables module-owned scoreboard writes rather than weakening
the check. Each module may own at most 128 objectives, and each save stores at
most 16,384 objective owner records.

The existing `score`, `set_score`, and `set_display_slot` operations continue
to address existing objectives directly. They do not create or delete them.
