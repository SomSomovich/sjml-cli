# Client-local sound playback

Client-side Rust modules can play a sound resource at the local listener through
`sjml::client_audio::play`. Playback uses Minecraft's `SoundManager`; the host validates that the
sound is present in the currently loaded sound resources before queuing it.

```rust
use sjml::client_audio::{play, SoundCategory};

play(
    "minecraft:entity.player.levelup",
    SoundCategory::Players,
    0.8, // volume
    1.0, // pitch
)?;
```

The module must declare and the administrator must grant `client.audio.play`:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "client"
permissions = ["client.audio.play"]
```

```toml
[[grant]]
module = "example/main"
capabilities = ["client.audio.play"]
```

Sound IDs use canonical lowercase `namespace:path` syntax and are limited to 256 UTF-8 bytes.
The ID must resolve in the active sound resources, so both vanilla sounds and registered mod
sounds are accepted. Categories are `Master`, `Music`, `Records`, `Weather`, `Blocks`, `Hostile`,
`Neutral`, `Players`, `Ambient`, and `Voice`, matching Minecraft's volume controls. Volume is
finite in `0..=4`; pitch is finite in `0.5..=2`.

Each runtime module can submit at most five playback requests in any two-second window. A sixth
request returns `ServiceBusy`. Playback is client-local: this API sends no packets and cannot
trigger sounds for other players. It requires a physical client and the client main thread;
server modules and static-registration callbacks receive no usable import.

The guest ABI is
`sjml_client_audio_v1.play(i32, i32, i32, i32, i32) -> i32`, carrying a resource-ID pointer and
length, category code (`0` through `9`), and the raw `f32` bits for volume and pitch. The host
copies and validates the ID before calling Minecraft.

Unit/ABI tests cover the permission and side guards, client thread, grant revocation, malformed
IDs and UTF-8, category codes, finite level bounds, and the per-module rate limit. Acoustic output
still needs verification in a display-backed client with audio enabled.

## Module-owned looping sounds

Modules that need sustained local audio can use the separate `client.audio.loop` capability. It
does not imply `client.audio.play`; request both capabilities only if the module needs both APIs.

```rust
use sjml::client_audio::{start_loop, stop_loop, SoundCategory};

let ambience = start_loop("example:ambience.wind", SoundCategory::Ambient, 0.6, 1.0)?;
// Keep `ambience` in this module runtime while the sound should continue.
stop_loop(ambience)?;
```

Declare and grant the loop capability separately:

```toml
permissions = ["client.audio.loop"]
```

```toml
[[grant]]
module = "example/main"
capabilities = ["client.audio.loop"]
```

`start_loop` returns an opaque `LoopingSoundHandle`. A handle belongs to the exact package/module
runtime that created it; another module cannot use it. A module may own at most eight active loops,
and the physical-client sink caps all modules together at 256. One-shot plays and loop starts share
the existing five-starts-per-two-seconds rate limit. Stops are bounded to one owned handle at a
time. Closing or deactivating a module stops every remaining loop that module owns.

Cleanup keeps the exact `SoundInstance` object passed to `SoundManager.play` and calls
`SoundManager.stop(instance)` for that object only. It never stops by sound ID, category, or global
sound-manager state, so a module cannot stop a matching sound from another mod. The sound manager
can be intercepted by NeoForge's `PlaySoundEvent`; if a listener cancels or replaces SJML's exact
instance, the host returns `ServiceBusy` rather than issuing an unowned handle. A replacement made
by that listener is not stopped by SJML.

The loop imports are
`sjml_client_audio_v1.start_loop(i32, i32, i32, i32, i32) -> i64` and
`sjml_client_audio_v1.stop_loop(i32) -> i32`. The 64-bit start result packs the ABI status in its
upper 32 bits and the nonzero module-scoped handle in its lower 32 bits. Both operations enforce
the client context, requested/granted `client.audio.loop`, and the physical-client main thread.
