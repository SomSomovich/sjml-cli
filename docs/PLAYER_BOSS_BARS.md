# Player boss bars

Server-side Rust modules can create, update, and remove one vanilla boss bar for a live player
session with `sjml::player_bossbar`. Every call requires both `entity.read` and
`player.bossbar.write`:

```toml
permissions = ["entity.read", "player.bossbar.write"]
```

Use `player::online()` to obtain the player handle. A bar is visible only to that player. The
module receives a nonzero module-local `PlayerBossBarId` from `create`, then supplies the same
player handle and ID to `update` or `remove`. Each module can own one bar per player session and at
most 32 bars total. The logical server allows at most 1,024 SJML bars total.

Titles are literal, nonblank text with no control characters, limited to 256 UTF-8 bytes and 128
Unicode scalar values. Progress must be finite and within `0.0..=1.0`. Colors are `Pink`, `Blue`,
`Red`, `Green`, `Yellow`, `Purple`, and `White`; overlays are `Progress`, `Notched6`, `Notched10`,
`Notched12`, and `Notched20`. The host exposes no arbitrary components or custom boss-event types.

ABI v1 exposes `create`, `update`, and `remove` in `sjml_player_bossbar_v1`. Each returns an `i32`
status. Create receives a 24-byte player handle and writes its nonzero module-local ID as a
little-endian `u32`. Update and remove receive that same handle and ID. Create and update take this
little-endian definition record: `u32(1)` schema, `u32` title byte length, UTF-8 title bytes,
`f32` progress, `u8` color code, `u8` overlay code, and `u8` visibility (`0` or `1`). IDs start at
one and are not reused during that module activation.

```rust
use sjml::player_bossbar::{self, BossBarColor, BossBarDefinition, BossBarOverlay};

let player = sjml::player::online()?.into_iter().next().ok_or_else(|| {
    sjml::SjmlError::new(sjml::ErrorCode::InvalidHandle, "no online player")
})?;
let bar = BossBarDefinition::new(
    "Server objective",
    0.4,
    BossBarColor::Blue,
    BossBarOverlay::Notched10,
    true,
)?;
let id = player_bossbar::create(player, &bar)?;
player_bossbar::update(
    player,
    id,
    &BossBarDefinition::new(
        "Server objective",
        0.8,
        BossBarColor::Green,
        BossBarOverlay::Progress,
        true,
    )?,
)?;
player_bossbar::remove(player, id)?;
```

Calls run on the owning server thread and require an active `ServerPlayer` login session. A bar is
removed when that player logs out, its module deactivates, or the server stops. Server stop cleanup
runs through the existing module registry shutdown and player-provider unbind lifecycle.
