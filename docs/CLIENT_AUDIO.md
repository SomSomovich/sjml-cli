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
