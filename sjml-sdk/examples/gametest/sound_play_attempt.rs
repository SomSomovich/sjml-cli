use sjml::{
    event,
    events::{self, EventPriority, SoundPlayAttempt},
    SjmlResult,
};

pub(super) fn initialize() -> SjmlResult<()> {
    events::subscribe("sound.play_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "sound.play_attempt")]
fn veto_marked_level_sound(event: SoundPlayAttempt) -> SjmlResult<bool> {
    Ok(event.sound_id == "minecraft:block.note_block.pling"
        && event.dimension_id == "minecraft:overworld"
        && event.source == "blocks"
        && event.volume == 1.0
        && event.pitch == 1.0
        && event.position.is_some()
        && event.entity.is_none())
}
