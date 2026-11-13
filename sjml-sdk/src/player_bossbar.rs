//! Bounded vanilla boss bars scoped to one live server-player session.
//!
//! Every operation requires `entity.read` and `player.bossbar.write` in a server-side module.
//! Bars are private to the selected player, belong to the creating module, and are removed when
//! that player logs out or the module stops.

use crate::{ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Maximum UTF-8 title size accepted by the host.
pub const MAX_PLAYER_BOSS_BAR_TITLE_BYTES: usize = 256;
/// Maximum Unicode scalar values accepted for a title.
pub const MAX_PLAYER_BOSS_BAR_TITLE_CHARS: usize = 128;
/// Maximum number of module-owned bars across all online players.
pub const MAX_PLAYER_BOSS_BARS_PER_MODULE: usize = 32;

/// Vanilla boss-bar color, restricted to the seven values supported by Minecraft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BossBarColor {
    Pink = 0,
    Blue = 1,
    Red = 2,
    Green = 3,
    Yellow = 4,
    Purple = 5,
    White = 6,
}

/// Vanilla boss-bar progress texture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BossBarOverlay {
    Progress = 0,
    Notched6 = 1,
    Notched10 = 2,
    Notched12 = 3,
    Notched20 = 4,
}

/// Detached, validated values for a literal vanilla boss-bar presentation.
#[derive(Clone, Debug, PartialEq)]
pub struct BossBarDefinition {
    /// Literal text shown in the bar. Formatting codes and control characters are not interpreted.
    pub title: String,
    /// Normalized progress in the inclusive range `0.0..=1.0`.
    pub progress: f32,
    /// Vanilla bar color.
    pub color: BossBarColor,
    /// Vanilla progress overlay.
    pub overlay: BossBarOverlay,
    /// Whether Minecraft currently displays this bar to its one target player.
    pub visible: bool,
}

impl BossBarDefinition {
    /// Creates a bounded boss-bar definition.
    pub fn new(
        title: impl Into<String>,
        progress: f32,
        color: BossBarColor,
        overlay: BossBarOverlay,
        visible: bool,
    ) -> SjmlResult<Self> {
        let value = Self {
            title: title.into(),
            progress,
            color,
            overlay,
            visible,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> SjmlResult<()> {
        if self.title.trim().is_empty()
            || self.title.len() > MAX_PLAYER_BOSS_BAR_TITLE_BYTES
            || self.title.chars().count() > MAX_PLAYER_BOSS_BAR_TITLE_CHARS
            || self.title.chars().any(char::is_control)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "boss-bar title must be nonblank, control-free, and at most 256 UTF-8 bytes or 128 characters",
            ));
        }
        if !self.progress.is_finite() || !(0.0..=1.0).contains(&self.progress) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "boss-bar progress must be finite and normalized to 0..=1",
            ));
        }
        Ok(())
    }

    #[cfg(any(target_arch = "wasm32", test))]
    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(15 + self.title.len());
        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend_from_slice(&(self.title.len() as u32).to_le_bytes());
        bytes.extend_from_slice(self.title.as_bytes());
        bytes.extend_from_slice(&self.progress.to_le_bytes());
        bytes.push(self.color as u8);
        bytes.push(self.overlay as u8);
        bytes.push(u8::from(self.visible));
        bytes
    }
}

/// Module-local identifier for one bar attached to a live player handle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlayerBossBarId(u32);

impl PlayerBossBarId {
    /// Returns the wire-safe numeric identifier.
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_player_bossbar_v1")]
extern "C" {
    #[link_name = "create"]
    fn imported_create(
        player: i32,
        player_length: i32,
        definition: i32,
        definition_length: i32,
        output_id: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "update"]
    fn imported_update(
        player: i32,
        player_length: i32,
        id: i32,
        definition: i32,
        definition_length: i32,
    ) -> i32;
    #[link_name = "remove"]
    fn imported_remove(player: i32, player_length: i32, id: i32) -> i32;
}

/// Creates a module-owned bar visible only to `player` and returns its module-local identifier.
///
/// The live `PlayerHandle` must come from [`crate::player::online`]. The bar is removed on logout
/// and module shutdown. Minecraft retains its normal boss-bar packet behavior and visible toggle.
pub fn create(player: PlayerHandle, definition: &BossBarDefinition) -> SjmlResult<PlayerBossBarId> {
    definition.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_player = crate::to_bytes(&player)?;
        let encoded_definition = definition.encode();
        let mut id = 0_u32;
        // SAFETY: all borrowed buffers and the output word remain live through the synchronous host call.
        let status = unsafe {
            imported_create(
                encoded_player.as_ptr() as usize as u32 as i32,
                encoded_player.len() as u32 as i32,
                encoded_definition.as_ptr() as usize as u32 as i32,
                encoded_definition.len() as u32 as i32,
                &mut id as *mut u32 as usize as u32 as i32,
                core::mem::size_of::<u32>() as i32,
            )
        };
        check_status(status, "host rejected player boss-bar creation")?;
        if id == 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned the reserved zero boss-bar ID",
            ));
        }
        Ok(PlayerBossBarId(id))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = player;
        Err(unsupported_target())
    }
}

/// Updates a bar owned by this module for the same live player handle.
pub fn update(
    player: PlayerHandle,
    id: PlayerBossBarId,
    definition: &BossBarDefinition,
) -> SjmlResult<()> {
    definition.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_player = crate::to_bytes(&player)?;
        let encoded_definition = definition.encode();
        // SAFETY: both borrowed buffers remain live through the synchronous host call.
        let status = unsafe {
            imported_update(
                encoded_player.as_ptr() as usize as u32 as i32,
                encoded_player.len() as u32 as i32,
                id.0 as i32,
                encoded_definition.as_ptr() as usize as u32 as i32,
                encoded_definition.len() as u32 as i32,
            )
        };
        check_status(status, "host rejected player boss-bar update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, id);
        Err(unsupported_target())
    }
}

/// Removes a bar owned by this module for the same live player handle.
pub fn remove(player: PlayerHandle, id: PlayerBossBarId) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_player = crate::to_bytes(&player)?;
        // SAFETY: the encoded handle remains live through the synchronous host call.
        let status = unsafe {
            imported_remove(
                encoded_player.as_ptr() as usize as u32 as i32,
                encoded_player.len() as u32 as i32,
                id.0 as i32,
            )
        };
        check_status(status, "host rejected player boss-bar removal")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (player, id);
        Err(unsupported_target())
    }
}

#[cfg(target_arch = "wasm32")]
fn check_status(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "player boss bars require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_definition_boundaries() {
        assert!(BossBarDefinition::new(
            "Done",
            0.0,
            BossBarColor::Pink,
            BossBarOverlay::Progress,
            true
        )
        .is_ok());
        assert!(BossBarDefinition::new(
            "Done",
            1.0,
            BossBarColor::White,
            BossBarOverlay::Notched20,
            false
        )
        .is_ok());
        for progress in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
            assert_eq!(
                BossBarDefinition::new(
                    "bad",
                    progress,
                    BossBarColor::Blue,
                    BossBarOverlay::Progress,
                    true
                )
                .unwrap_err()
                .code(),
                ErrorCode::InvalidArgument,
            );
        }
        for title in ["", " \t ", "line\nbreak", &"x".repeat(257)] {
            assert_eq!(
                BossBarDefinition::new(
                    title,
                    0.5,
                    BossBarColor::Red,
                    BossBarOverlay::Progress,
                    true
                )
                .unwrap_err()
                .code(),
                ErrorCode::InvalidArgument,
            );
        }
    }

    #[test]
    fn definition_codec_has_exact_little_endian_v1_layout() {
        let definition = BossBarDefinition::new(
            "HP",
            0.5,
            BossBarColor::Purple,
            BossBarOverlay::Notched12,
            false,
        )
        .unwrap();
        assert_eq!(
            definition.encode(),
            [
                1, 0, 0, 0, // schema
                2, 0, 0, 0, // UTF-8 title length
                b'H', b'P', 0, 0, 0x00, 0x3f, // title and 0.5f
                5, 3, 0, // color, overlay, visibility
            ]
        );
    }

    #[test]
    fn unicode_title_is_bounded_by_encoded_bytes_and_scalar_count() {
        let value = "é".repeat(128);
        assert!(BossBarDefinition::new(
            value,
            0.5,
            BossBarColor::Green,
            BossBarOverlay::Progress,
            true
        )
        .is_ok());
        let too_many_scalars = "a".repeat(129);
        assert!(BossBarDefinition::new(
            too_many_scalars,
            0.5,
            BossBarColor::Green,
            BossBarOverlay::Progress,
            true
        )
        .is_err());
    }
}
