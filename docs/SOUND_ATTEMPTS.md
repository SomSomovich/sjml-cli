# Level sound attempts

Server modules can veto a sound before a logical-server level broadcasts it:

```rust
use sjml::{event, events::{self, EventPriority, SoundPlayAttempt}, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    events::subscribe("sound.play_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "sound.play_attempt")]
fn mute_note_blocks(event: SoundPlayAttempt) -> SjmlResult<bool> {
    Ok(event.sound_id == "minecraft:block.note_block.pling")
}
```

The event runs synchronously on the owning logical-server thread through
NeoForge's cancellable `PlayLevelSoundEvent`, covering base, at-position, and
at-entity sound dispatch. Returning `Ok(true)` suppresses playback completely;
returning `Ok(false)` leaves the vanilla sound unchanged. The callback cannot
change the sound, category, volume, or pitch — those values are observational.

The detached schema-v1 payload contains the canonical sound ID, the lowercase
vanilla sound category (`master`, `music`, `record`, `weather`, `blocks`,
`hostile`, `neutral`, `players`, `ambient`, `voice`, or `ui`), the dimension,
the effective volume and pitch (finite, bounded to ±1,000,000), an optional
world-space position for at-position sounds, and an optional entity UUID/type
for at-entity sounds. Each string is bounded to 256 UTF-8 bytes and the payload
to 2 KiB. Up to 64 subscribed handlers run per attempt; all subscriptions
require both `events.subscribe` and `events.cancel`, and `#[event_async]` is
rejected because the veto must apply before playback returns.

The host checks the active module subscriptions before building a snapshot, so
unsubscribed servers pay only a small lookup cost per sound.

GameTests post real at-position `PlayLevelSoundEvent` instances for a marked
note-block sound and an unmarked pickup sound, verifying that only the marked
one is canceled. Unit tests cover both payload variants, bounds, the cancellable
NeoForge contract, and detached snapshot values.
