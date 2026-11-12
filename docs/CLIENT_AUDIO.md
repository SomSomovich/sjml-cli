# Client-local sound playback

Client-side Rust modules can play a sound resource at the local listener through
`sjml::client_audio::play`. Playback uses Minecraft's `SoundManager`; the host validates that the
sound is present in the currently loaded sound resources before queuing it. Use `play_tracked`
when that particular one-shot must be stopped early.

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

## Tracked one-shot playback

`play_tracked` returns an opaque `TrackedSoundHandle`; `stop_tracked(handle)` stops only that exact
playback instance:

```rust
use sjml::client_audio::{play_tracked, stop_tracked, SoundCategory};

let sound = play_tracked(
    "minecraft:entity.player.levelup",
    SoundCategory::Players,
    0.8,
    1.0,
)?;
// Stop this instance before its natural end if needed.
stop_tracked(sound)?;
```

Tracked one-shots can also fade linearly from their current volume to a new level. Start at zero
for a fade-in and target zero for a fade-out:

```rust
use sjml::client_audio::{fade_tracked, play_tracked, SoundCategory};

let sound = play_tracked("example:music.intro", SoundCategory::Music, 0.0, 1.0)?;
fade_tracked(sound, 0.8, 40)?; // Fade in over two seconds.
// Later, while the same one-shot is still active:
fade_tracked(sound, 0.0, 20)?; // Fade out over one second.
```

`fade_tracked(handle, target_volume, duration_ticks)` changes only the active sound instance held
by that exact runtime-owned handle. The target is finite `0..=4`, and duration is `1..=1200`
client ticks. A later fade request starts from the instance's current volume and replaces its
previous ramp. Fades advance on Minecraft's client sound tick; calls require the physical client
main thread and use `client.audio.play`. The instance remains owned until explicitly stopped or
runtime cleanup. Disconnect and runtime teardown stop all owned instances, including a fade in
progress. If a short one-shot ends naturally before its requested duration, it cannot be faded
after it has ended. Keep the handle until the ramp has finished before calling `stop_tracked`, or
let runtime cleanup release it.

Tracked playback uses the existing `client.audio.play` capability and shared five-starts-per-two-
seconds limit. A module runtime can hold at most sixteen tracked handles, while the physical-client
host allows up to 512 across all runtimes. A naturally completed sound retains its handle until the
module calls `stop_tracked` or its runtime closes. Runtime cleanup stops every remaining tracked
instance automatically.

The host records the exact `SoundInstance` object passed to `SoundManager.play`, then stops that
object by identity. If NeoForge's `PlaySoundEvent` cancels or replaces the instance, no handle is
returned and the replacement is left untouched. Unit tests cover per-runtime ownership, handle
bounds, rejected starts, and automatic cleanup. Actual playback and interruption still need a
display-backed client with audio enabled.

The ABI imports are `sjml_client_audio_v1.play_tracked(i32, i32, i32, i32, i32) -> i64`,
`sjml_client_audio_v1.stop_tracked(i32) -> i32`, and
`sjml_client_audio_v1.fade_tracked(handle, target_volume_bits, duration_ticks) -> i32`. As with
looping audio, the upper half of the packed start result contains the status code and the lower half
contains a nonzero module-scoped handle.

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
