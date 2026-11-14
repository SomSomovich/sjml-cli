# Observing started client sounds

`client.sound_started` reports bounded metadata after NeoForge creates an audio source for a
non-streaming or streaming sound. It observes playback and cannot cancel, replace, stop, or change
the sound. The payload contains a canonical sound ID, vanilla category, finite volume and pitch,
and source position; it contains no `SoundInstance`, audio channel, player, or sound-manager handle.

NeoForge 21.1.265 for Minecraft 1.21.1 exposes `PlaySoundSourceEvent` for non-streaming sounds and
`PlayStreamingSourceEvent` for streaming sounds. Both are client-only `SoundSourceEvent` subclasses
fired on the main NeoForge event bus after a sound starts on an audio channel. The adapter copies
the same fields from either event and immediately drops the event reference. It does not subscribe
to the earlier mutable `PlaySoundEvent` hook.

## Permission and callback

Declare `events.subscribe` and `client.audio.observe` on a client module, then ask the administrator
to grant both:

```toml
[[modules]]
id = "main"
path = "modules/main.wasm"
side = "client"
permissions = ["events.subscribe", "client.audio.observe"]
```

```toml
[[grant]]
module = "example/main"
capabilities = ["events.subscribe", "client.audio.observe"]
```

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("client.sound_started", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "client.sound_started")]
fn sound_started(event: ClientSoundStarted) -> SjmlResult<()> {
    host::log(
        LogLevel::Info,
        &format!("{} ({:?}) at {}, {}, {}", event.sound_id,
            event.category, event.x, event.y, event.z),
    )?;
    Ok(())
}
```

The SDK category enum is `ClientSoundCategory`: `Master`, `Music`, `Records`, `Weather`, `Blocks`,
`Hostile`, `Neutral`, `Players`, `Ambient`, and `Voice`. Volume is finite in `0..=64`, pitch is
finite in `0..=16`, position coordinates are finite and within ±30,000,000, and the sound ID is at
most 256 UTF-8 bytes. The total payload is at most 300 bytes.

Callbacks execute on the physical client's main thread and are capped at 64 total callbacks per
client tick across all sound-start events. Keep handlers short because they run inline on the
client thread. A sound started reentrantly by a callback is not dispatched recursively. Server
contexts cannot subscribe. These notifications do not require or imply
`client.audio.play`; the existing playback API and its rate limits are unchanged.

The host accepts the vanilla category wire codes `0` through `9` in the order listed above. Core
codec tests cover valid values and malformed IDs, category codes, numeric bounds, and payloads.
The event source itself is verified against the NeoForge 21.1.265 source JAR; interactive playback
observation still needs a client with audio enabled.
