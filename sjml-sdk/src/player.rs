//! Scoped access to online player sessions on the owning logical server. Snapshots, vitals, and game-mode reads require
//! requested and granted `entity.read`; effect application also requires `entity.effect.apply`, teleportation requires
//! `player.teleport`, game-mode changes require `player.game_mode.write`, vitals changes require
//! `player.vitals.write`, velocity changes require `player.motion.write`, and flight changes require
//! `player.abilities.write`; action-bar messages require `player.ui.actionbar`. Player handles
//! survive respawn and dimension changes, and are invalidated when logout begins or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::entity::status_result;
use crate::entity::{validate_effect_id, MAX_EFFECT_DURATION_TICKS};
#[cfg(target_arch = "wasm32")]
use crate::MAX_STRING_BYTES;
#[cfg(target_arch = "wasm32")]
use crate::{to_bytes, Decoder};
use crate::{
    ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode, StableEncode, WorldHandle,
};

/// Current identity, dimension, and coordinates of one online player.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerSnapshot {
    /// Canonical lowercase UUID string.
    pub uuid: String,
    /// Current profile name.
    pub profile_name: String,
    /// Canonical namespace:path dimension ID.
    pub dimension_id: String,
    /// Current X coordinate.
    pub x: f64,
    /// Current Y coordinate.
    pub y: f64,
    /// Current Z coordinate.
    pub z: f64,
}

/// Detached health, hunger, and experience data for one live player session.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerVitals {
    /// Current health, between zero and the host's supported ceiling.
    pub health: f32,
    /// Maximum health, greater than zero and no more than the host's supported ceiling.
    pub max_health: f32,
    /// Current food level, from zero through twenty.
    pub food_level: u32,
    /// Current food saturation, from zero through twenty.
    pub saturation: f32,
    /// Current experience level, from zero through one million.
    pub experience_level: u32,
    /// Progress toward the next experience level, from zero through one.
    pub experience_progress: f32,
}

/// Maximum number of bytes in a canonical item registry ID returned for equipment.
pub const MAX_PLAYER_EQUIPMENT_ITEM_ID_BYTES: usize = 256;
/// Maximum stack size accepted in a detached equipment snapshot.
pub const MAX_PLAYER_EQUIPMENT_STACK_SIZE: u32 = 4096;
/// Maximum durability value accepted in a detached equipment snapshot.
pub const MAX_PLAYER_EQUIPMENT_DAMAGE: u32 = 1_000_000;
/// Maximum encoded size of all six equipment slots in schema v1.
pub const MAX_PLAYER_EQUIPMENT_BYTES: u32 = 1690;

/// Detached stack identity and durability data for an equipped item.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerEquipmentStack {
    /// Canonical lowercase namespace:path item registry ID.
    pub item_id: String,
    /// Current stack count.
    pub count: u32,
    /// Item stack's maximum size.
    pub max_stack_size: u32,
    /// Current durability damage; zero for items without durability.
    pub damage: u32,
    /// Maximum durability damage; zero for items without durability.
    pub max_damage: u32,
}

impl PlayerEquipmentStack {
    /// Builds a detached equipment value after validating its registry ID, stack size, and durability.
    pub fn new(
        item_id: impl Into<String>,
        count: u32,
        max_stack_size: u32,
        damage: u32,
        max_damage: u32,
    ) -> SjmlResult<Self> {
        let stack = Self {
            item_id: item_id.into(),
            count,
            max_stack_size,
            damage,
            max_damage,
        };
        stack.validate()?;
        Ok(stack)
    }

    fn validate(&self) -> SjmlResult<()> {
        let Some((namespace, path)) = self.item_id.split_once(':') else {
            return Err(invalid_equipment(
                "item ID must be a canonical namespace:path value",
            ));
        };
        if self.item_id.len() > MAX_PLAYER_EQUIPMENT_ITEM_ID_BYTES
            || namespace.is_empty()
            || path.is_empty()
            || !namespace.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
            })
            || !path.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
            })
        {
            return Err(invalid_equipment(
                "item ID must be a canonical namespace:path value",
            ));
        }
        if self.count == 0
            || self.count > MAX_PLAYER_EQUIPMENT_STACK_SIZE
            || self.max_stack_size == 0
            || self.max_stack_size > MAX_PLAYER_EQUIPMENT_STACK_SIZE
            || self.count > self.max_stack_size
            || self.max_damage > MAX_PLAYER_EQUIPMENT_DAMAGE
            || self.damage > self.max_damage
        {
            return Err(invalid_equipment(
                "equipment stack size or durability is outside the supported range",
            ));
        }
        Ok(())
    }
}

/// Detached main-hand, offhand, and armor slots for one online player session.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerEquipment {
    /// Item held in the player's selected main-hand slot.
    pub main_hand: Option<PlayerEquipmentStack>,
    /// Item in the offhand slot.
    pub off_hand: Option<PlayerEquipmentStack>,
    /// Item in the feet armor slot.
    pub feet: Option<PlayerEquipmentStack>,
    /// Item in the legs armor slot.
    pub legs: Option<PlayerEquipmentStack>,
    /// Item in the chest armor slot.
    pub chest: Option<PlayerEquipmentStack>,
    /// Item in the head armor slot.
    pub head: Option<PlayerEquipmentStack>,
}

impl PlayerEquipment {
    #[cfg(target_arch = "wasm32")]
    fn validate(&self) -> SjmlResult<()> {
        for stack in [
            &self.main_hand,
            &self.off_hand,
            &self.feet,
            &self.legs,
            &self.chest,
            &self.head,
        ]
        .into_iter()
        .flatten()
        {
            stack.validate()?;
        }
        Ok(())
    }
}

fn invalid_equipment(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

/// Detached, read-only snapshot of the four built-in player abilities exposed by the host.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAbilities {
    /// Whether the current game mode or server grants flight capability.
    pub may_fly: bool,
    /// Whether the player is currently flying.
    pub flying: bool,
    /// Whether the player is invulnerable.
    pub invulnerable: bool,
    /// Whether the player may build and interact with blocks.
    pub may_build: bool,
}

impl PlayerAbilities {
    #[cfg(target_arch = "wasm32")]
    fn validate(self) -> SjmlResult<()> {
        if self.flying && !self.may_fly {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned flying=true while may_fly=false",
            ));
        }
        Ok(())
    }
}

/// Maximum health value accepted by [`set_health`]. The host also limits it to the player's current maximum health.
pub const MAX_PLAYER_HEALTH: f32 = 1_000_000.0;
/// Maximum food level accepted by [`set_food`].
pub const MAX_PLAYER_FOOD_LEVEL: u32 = 20;
/// Maximum experience level accepted by [`set_experience`].
pub const MAX_PLAYER_EXPERIENCE_LEVEL: u32 = 20_000;

/// One of Minecraft's built-in player game modes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum PlayerGameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl TryFrom<u32> for PlayerGameMode {
    type Error = SjmlError;

    fn try_from(code: u32) -> Result<Self, Self::Error> {
        match code {
            0 => Ok(Self::Survival),
            1 => Ok(Self::Creative),
            2 => Ok(Self::Adventure),
            3 => Ok(Self::Spectator),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an unsupported player game mode",
            )),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(StableEncode, StableDecode)]
#[sjml(version = 1)]
struct PlayerGameModeSnapshot {
    mode: u32,
}

impl PlayerVitals {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<()> {
        if !self.health.is_finite()
            || !(0.0..=1_000_000.0).contains(&self.health)
            || !self.max_health.is_finite()
            || !(f32::MIN_POSITIVE..=1_000_000.0).contains(&self.max_health)
            || self.food_level > 20
            || !self.saturation.is_finite()
            || !(0.0..=20.0).contains(&self.saturation)
            || self.experience_level > 1_000_000
            || !self.experience_progress.is_finite()
            || !(0.0..=1.0).contains(&self.experience_progress)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned non-finite or out-of-range player vitals",
            ));
        }
        Ok(())
    }
}

/// Absolute same-dimension destination for a server-authoritative player teleport.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerTeleport {
    /// Destination X coordinate.
    pub x: f64,
    /// Destination Y coordinate; the host checks the current dimension's build height.
    pub y: f64,
    /// Destination Z coordinate.
    pub z: f64,
    /// Absolute yaw in degrees, limited to ±36,000.
    pub yaw: f32,
    /// Absolute pitch in degrees, limited to -90..=90.
    pub pitch: f32,
}

/// Current velocity and yaw/pitch rotation of an online player.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerMotion {
    /// X velocity in blocks per tick.
    pub velocity_x: f64,
    /// Y velocity in blocks per tick.
    pub velocity_y: f64,
    /// Z velocity in blocks per tick.
    pub velocity_z: f64,
    /// Current yaw in degrees.
    pub yaw: f32,
    /// Current pitch in degrees.
    pub pitch: f32,
}

/// Bounded velocity to apply to an online player, in blocks per tick.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerVelocity {
    /// X velocity component.
    pub x: f64,
    /// Y velocity component.
    pub y: f64,
    /// Z velocity component.
    pub z: f64,
}

impl PlayerVelocity {
    /// Creates a velocity after checking that every component is finite and within the host limit.
    pub fn new(x: f64, y: f64, z: f64) -> SjmlResult<Self> {
        let velocity = Self { x, y, z };
        velocity.validate()?;
        Ok(velocity)
    }

    fn validate(self) -> SjmlResult<()> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.z.is_finite()
            || self.x.abs() > MAX_PLAYER_VELOCITY_COMPONENT
            || self.y.abs() > MAX_PLAYER_VELOCITY_COMPONENT
            || self.z.abs() > MAX_PLAYER_VELOCITY_COMPONENT
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player velocity components must be finite and within ±64 blocks per tick",
            ));
        }
        Ok(())
    }
}

impl PlayerTeleport {
    /// Creates a bounded destination. The host additionally checks build height, world border,
    /// and that the destination chunk is already loaded.
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> SjmlResult<Self> {
        let destination = Self {
            x,
            y,
            z,
            yaw,
            pitch,
        };
        destination.validate()?;
        Ok(destination)
    }

    fn validate(self) -> SjmlResult<()> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.z.is_finite()
            || self.x.abs() > MAX_TELEPORT_COORDINATE
            || self.y.abs() > MAX_TELEPORT_COORDINATE
            || self.z.abs() > MAX_TELEPORT_COORDINATE
            || !self.yaw.is_finite()
            || self.yaw.abs() > MAX_TELEPORT_YAW
            || !self.pitch.is_finite()
            || !(-90.0..=90.0).contains(&self.pitch)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player teleport coordinates or rotation are outside the supported range",
            ));
        }
        Ok(())
    }
}

/// Maximum absolute X, Y, or Z coordinate accepted by the teleport API.
pub const MAX_TELEPORT_COORDINATE: f64 = 29_999_984.0;
/// Maximum absolute yaw accepted by the teleport API.
pub const MAX_TELEPORT_YAW: f32 = 36_000.0;
/// Maximum absolute velocity component accepted by the player motion API, in blocks per tick.
pub const MAX_PLAYER_VELOCITY_COMPONENT: f64 = 64.0;
/// Maximum UTF-8 size accepted for one literal player action-bar message.
pub const MAX_ACTION_BAR_TEXT_BYTES: usize = 1024;
/// Maximum Unicode scalar values accepted for one literal player action-bar message.
pub const MAX_ACTION_BAR_TEXT_CODE_POINTS: usize = 256;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_player_v1")]
extern "C" {
    #[link_name = "online"]
    fn imported_online(output: i32, capacity: i32) -> i32;
    #[link_name = "snapshot"]
    fn imported_snapshot(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "equipment"]
    fn imported_equipment(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "abilities"]
    fn imported_abilities(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_flying"]
    fn imported_set_flying(handle: i32, length: i32, flying: i32) -> i32;
    #[link_name = "vitals"]
    fn imported_vitals(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "game_mode"]
    fn imported_game_mode(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_game_mode"]
    fn imported_set_game_mode(handle: i32, length: i32, game_mode: i32) -> i32;
    #[link_name = "set_health"]
    fn imported_set_health(handle: i32, length: i32, health_bits: i32) -> i32;
    #[link_name = "set_food"]
    fn imported_set_food(handle: i32, length: i32, food_level: i32, saturation_bits: i32) -> i32;
    #[link_name = "set_experience"]
    fn imported_set_experience(handle: i32, length: i32, level: i32, progress_bits: i32) -> i32;
    #[link_name = "apply_effect"]
    fn imported_apply_effect(
        handle: i32,
        handle_length: i32,
        effect_id: i32,
        effect_id_length: i32,
        duration_ticks: i32,
        amplifier: i32,
    ) -> i32;
    #[link_name = "teleport"]
    fn imported_teleport(
        handle: i32,
        handle_length: i32,
        destination: i32,
        destination_length: i32,
    ) -> i32;
    #[link_name = "teleport_to_world"]
    fn imported_teleport_to_world(
        player: i32,
        player_length: i32,
        world: i32,
        world_length: i32,
        destination: i32,
        destination_length: i32,
    ) -> i32;
    #[link_name = "motion"]
    fn imported_motion(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_velocity"]
    fn imported_set_velocity(
        handle: i32,
        handle_length: i32,
        velocity: i32,
        velocity_length: i32,
    ) -> i32;
    #[link_name = "action_bar"]
    fn imported_action_bar(handle: i32, handle_length: i32, text: i32, text_length: i32) -> i32;
}

/// Returns every online player, sorted by canonical UUID. Before server binding and in client
/// modules this returns an empty vector. Enumeration is complete or returns an error at the 1,024
/// handle quota; it never truncates the list.
pub fn online() -> SjmlResult<Vec<PlayerHandle>> {
    #[cfg(target_arch = "wasm32")]
    {
        read_output(4 + 24 * 1024, |output, capacity| {
            // SAFETY: read_output owns the output allocation for the complete synchronous import.
            unsafe { imported_online(output, capacity) }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Reads a player snapshot after validating the handle owner, context, kind, session, and capability.
pub fn snapshot(handle: PlayerHandle) -> SjmlResult<PlayerSnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let capacity = (4 + 3 * (4 + MAX_STRING_BYTES) + 3 * core::mem::size_of::<f64>()) as u32;
        read_output(capacity, |output, capacity| {
            // SAFETY: the encoded handle and output buffer remain live through the synchronous call.
            unsafe {
                imported_snapshot(
                    encoded.as_ptr() as usize as u32 as i32,
                    24,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads health, food level, saturation, and experience for an online player.
///
/// Requires the declared and granted `entity.read` capability, a live player handle,
/// and a logical-server context. The host resolves the current player object on
/// every call, so a handle remains valid across respawn while logout and runtime
/// shutdown invalidate it. The returned value is detached; it cannot mutate the
/// player and exposes no Minecraft object. Values outside the bounded schema are
/// reported as `InvalidArgument`.
pub fn vitals(handle: PlayerHandle) -> SjmlResult<PlayerVitals> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let encoded_vitals = read_output::<PlayerVitals>(28, |output, capacity| {
            // SAFETY: the encoded handle and output buffer remain live through the synchronous import.
            unsafe {
                imported_vitals(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                    output,
                    capacity,
                )
            }
        })?;
        encoded_vitals.validate()?;
        Ok(encoded_vitals)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads the current server-authoritative game mode of an online player session.
///
/// Requires the declared and granted `entity.read` capability and a live player
/// handle. Client modules and static registration never receive live player state.
pub fn game_mode(handle: PlayerHandle) -> SjmlResult<PlayerGameMode> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let snapshot = read_output::<PlayerGameModeSnapshot>(8, |output, capacity| {
            // SAFETY: the encoded handle and output buffer remain live through the synchronous import.
            unsafe {
                imported_game_mode(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                    output,
                    capacity,
                )
            }
        })?;
        PlayerGameMode::try_from(snapshot.mode)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Changes the game mode of an online player on the owning logical server.
///
/// Requires both `entity.read` and `player.game_mode.write` to be declared in
/// the module and granted by the host. The host checks the owning server thread
/// and re-resolves the current object for the same login session on every call.
/// Client contexts and static registration cannot change a player's mode.
pub fn set_game_mode(handle: PlayerHandle, game_mode: PlayerGameMode) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle buffer stays live during this bounded synchronous host import.
        let status = unsafe {
            imported_set_game_mode(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                game_mode as u32 as i32,
            )
        };
        status_result(status, "host rejected the player game-mode change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, game_mode);
        Err(unsupported_target())
    }
}

/// Sends bounded literal text to an online player as a vanilla action-bar message.
///
/// The text is not interpreted as JSON, a translation key, or a command. It must be nonblank,
/// contain no control characters, and fit within 1,024 UTF-8 bytes and 256 Unicode scalar values.
/// The host revalidates the module-owned player session and owning server thread for every call.
///
/// Requires the declared and administrator-granted entity.read and player.ui.actionbar
/// capabilities. It is available only to logical-server modules; client modules and static
/// registration cannot send messages. Each module is limited to ten messages per five-second
/// window.
pub fn send_action_bar(handle: PlayerHandle, text: &str) -> SjmlResult<()> {
    validate_action_bar_text(text)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and borrowed UTF-8 text remain live throughout the synchronous import.
        let status = unsafe {
            imported_action_bar(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                text.as_ptr() as usize as u32 as i32,
                text.len() as i32,
            )
        };
        status_result(status, "host rejected the player action-bar message")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

fn validate_action_bar_text(text: &str) -> SjmlResult<()> {
    if text.trim().is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "action-bar text must not be blank",
        ));
    }
    if text.len() > MAX_ACTION_BAR_TEXT_BYTES
        || text.chars().count() > MAX_ACTION_BAR_TEXT_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "action-bar text exceeds 1024 UTF-8 bytes or 256 code points",
        ));
    }
    if text.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "action-bar text must not contain control characters",
        ));
    }
    Ok(())
}

/// Sets the current health of an online player on the owning logical server.
///
/// Requires both `entity.read` and `player.vitals.write` to be declared and
/// granted. Health must be finite, no greater than the current maximum health,
/// and within `0..=MAX_PLAYER_HEALTH`; zero health can cause normal Minecraft
/// death processing. Client contexts, static registration, stale sessions, and
/// calls off the owning server thread are denied by the host.
pub fn set_health(handle: PlayerHandle, health: f32) -> SjmlResult<()> {
    if !health.is_finite() || !(0.0..=MAX_PLAYER_HEALTH).contains(&health) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player health must be finite and between zero and 1,000,000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and scalar remain valid throughout this synchronous bounded call.
        let status = unsafe {
            imported_set_health(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                health.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the player health change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets food level and saturation together for an online player.
///
/// Requires `entity.read` and `player.vitals.write`. Food is limited to 0–20;
/// saturation must be finite and between zero and the requested food level.
/// The host applies both fields only after validating the complete pair and the
/// current live session on its owning logical-server thread.
pub fn set_food(handle: PlayerHandle, food_level: u32, saturation: f32) -> SjmlResult<()> {
    if food_level > MAX_PLAYER_FOOD_LEVEL
        || !saturation.is_finite()
        || !(0.0..=food_level as f32).contains(&saturation)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "food level must be 0..20 and saturation must be finite and between zero and food level",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and scalar values remain valid throughout this synchronous bounded call.
        let status = unsafe {
            imported_set_food(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                food_level as i32,
                saturation.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the player food change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets an online player's experience level and normalized bar progress.
///
/// Requires `entity.read` and `player.vitals.write`. Levels are limited to
/// `0..=MAX_PLAYER_EXPERIENCE_LEVEL`; progress must be finite and in `[0, 1)`.
/// Minecraft stores experience in whole points, so the host rounds progress to
/// the nearest point for the chosen level. The host validates everything before
/// changing state and requires the owning logical-server thread.
pub fn set_experience(handle: PlayerHandle, level: u32, progress: f32) -> SjmlResult<()> {
    if level > MAX_PLAYER_EXPERIENCE_LEVEL
        || !progress.is_finite()
        || !(0.0..1.0).contains(&progress)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "experience level must be 0..20,000 and progress must be finite in [0, 1)",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and scalar values remain valid throughout this synchronous bounded call.
        let status = unsafe {
            imported_set_experience(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                level as i32,
                progress.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the player experience change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Applies a registered status effect to the current player for this login session.
///
/// The module must request and receive both `entity.read` and
/// `entity.effect.apply`. Duration is limited to 1–72,000 ticks and amplifier
/// to 0–255. The player handle is revalidated on the owning server thread for
/// every call, including after respawn. Client and native targets cannot apply effects.
pub fn apply_effect(
    handle: PlayerHandle,
    effect_id: &str,
    duration_ticks: u32,
    amplifier: u8,
) -> SjmlResult<()> {
    validate_effect_id(effect_id)?;
    if !(1..=MAX_EFFECT_DURATION_TICKS).contains(&duration_ticks) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "mob effect duration must be between 1 and 72000 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the host copies this bounded ID and handle and decodes fixed-width scalar values.
        let status = unsafe {
            imported_apply_effect(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                effect_id.as_ptr() as usize as u32 as i32,
                effect_id.len() as u32 as i32,
                duration_ticks as i32,
                amplifier as i32,
            )
        };
        status_result(status, "host rejected the player effect application")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, amplifier);
        Err(unsupported_target())
    }
}

/// Teleports an online player to an absolute position in their current dimension.
///
/// The module must request and receive both `entity.read` and `player.teleport`.
/// The destination coordinates and rotation are bounded before host access. The host then checks
/// the current dimension's build height and world border and rejects destinations in unloaded
/// chunks; it never loads a chunk or transfers the player to another dimension.
pub fn teleport(handle: PlayerHandle, destination: PlayerTeleport) -> SjmlResult<()> {
    destination.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_destination = to_bytes(&destination)?;
        // SAFETY: both stable buffers remain live during this synchronous bounded host import.
        let status = unsafe {
            imported_teleport(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                encoded_destination.as_ptr() as usize as u32 as i32,
                encoded_destination.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the player teleport")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Teleports an online player into a selected, already-loaded server world.
///
/// The module must request and receive `entity.read`, `player.teleport`, and `world.read`.
/// The target world handle must belong to this module and active logical server. Destination
/// coordinates must pass the same bounds as [`teleport`]; the host additionally checks the target
/// world's build height and border and requires the destination chunk to already be loaded. This
/// operation never requests a chunk load. Server dimension-travel hooks may cancel the move.
pub fn teleport_to_world(
    handle: PlayerHandle,
    target_world: WorldHandle,
    destination: PlayerTeleport,
) -> SjmlResult<()> {
    destination.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_player = to_bytes(&handle)?;
        let encoded_world = to_bytes(&target_world)?;
        let encoded_destination = to_bytes(&destination)?;
        // SAFETY: all three stable buffers remain live through this synchronous host import.
        let status = unsafe {
            imported_teleport_to_world(
                encoded_player.as_ptr() as usize as u32 as i32,
                encoded_player.len() as u32 as i32,
                encoded_world.as_ptr() as usize as u32 as i32,
                encoded_world.len() as u32 as i32,
                encoded_destination.as_ptr() as usize as u32 as i32,
                encoded_destination.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the cross-dimension player teleport")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, target_world);
        Err(unsupported_target())
    }
}

/// Reads the velocity and rotation of the live server player identified by this login-session handle.
///
/// Requires `entity.read`. Client instances, static registration, and stale login sessions are denied.
pub fn motion(handle: PlayerHandle) -> SjmlResult<PlayerMotion> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output(
            (4 + 3 * core::mem::size_of::<f64>() + 2 * core::mem::size_of::<f32>()) as u32,
            |output, capacity| {
                // SAFETY: the encoded handle and output stay live during this synchronous import.
                unsafe {
                    imported_motion(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads a detached snapshot of the main hand, offhand, and four armor slots.
///
/// Requires `entity.read` and a live player handle in a logical-server context.
/// The six positions are fixed and never include the player's inventory contents.
/// Each item is a detached ID/count/stack-limit/durability value; there is no
/// item handle, mutable component access, or ability to copy stacks into the
/// game. Static registration, client contexts, and stale sessions are denied.
pub fn equipment(handle: PlayerHandle) -> SjmlResult<PlayerEquipment> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let equipment: PlayerEquipment =
            read_output(MAX_PLAYER_EQUIPMENT_BYTES, |output, capacity| {
                // SAFETY: the encoded handle and output buffer remain valid during this synchronous host import.
                unsafe {
                    imported_equipment(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            })?;
        equipment.validate()?;
        Ok(equipment)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads the four built-in abilities of the current live server-player session.
///
/// Requires `entity.read`. The snapshot is read-only: game-mode/admin-owned
/// fields such as `may_fly`, `invulnerable`, and `may_build` are never writable
/// through this API. The host validates the active session and logical-server
/// thread for each call; client and static-registration calls fail closed.
pub fn abilities(handle: PlayerHandle) -> SjmlResult<PlayerAbilities> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let abilities: PlayerAbilities = read_output(8, |output, capacity| {
            // SAFETY: the encoded handle and output buffer stay valid through this synchronous import.
            unsafe {
                imported_abilities(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        abilities.validate()?;
        Ok(abilities)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Enables or disables flight for a live server player without changing `may_fly`.
///
/// Requires both `entity.read` and `player.abilities.write`. Enabling flight is
/// accepted only while the current player already has `may_fly`; disabling
/// flight is always allowed. The host synchronizes an actual state change to
/// clients and leaves invulnerability, build permission, and game mode intact.
pub fn set_flying(handle: PlayerHandle, flying: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean scalar stay valid through this synchronous host import.
        let status = unsafe {
            imported_set_flying(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(flying),
            )
        };
        status_result(status, "host rejected the player flight change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, flying);
        Err(unsupported_target())
    }
}

/// Replaces all velocity components for a live server player.
///
/// Requires `entity.read` and `player.motion.write`. Every component must be finite and within
/// ±64 blocks per tick. The host revalidates the live session and synchronizes motion to clients.
pub fn set_velocity(handle: PlayerHandle, velocity: PlayerVelocity) -> SjmlResult<()> {
    velocity.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_velocity = to_bytes(&velocity)?;
        // SAFETY: both bounded stable values remain live during this synchronous host call.
        let status = unsafe {
            imported_set_velocity(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                encoded_velocity.as_ptr() as usize as u32 as i32,
                encoded_velocity.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the player velocity mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

#[cfg(target_arch = "wasm32")]
fn read_output<T: StableDecode>(
    capacity: u32,
    invoke: impl FnOnce(i32, i32) -> i32,
) -> SjmlResult<T> {
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate player output buffer",
        ));
    }
    // SAFETY: initialize the complete allocation so decoding can safely inspect output padding.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: the initialized allocation stays live through stable decoding.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the player read",
        ))
    };
    // SAFETY: free the exact allocation after import and decode complete.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "player operations require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        apply_effect, teleport, teleport_to_world, vitals, PlayerTeleport, PlayerVitals,
        MAX_TELEPORT_COORDINATE,
    };
    use crate::{ErrorCode, PlayerHandle, WorldHandle};

    #[test]
    fn player_effect_application_validates_before_host_access() {
        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            apply_effect(handle, "RubyPack:power", 200, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            apply_effect(handle, "ruby_pack:content/power", 0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            apply_effect(handle, "ruby_pack:content/power", 72_001, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            apply_effect(handle, "ruby_pack:content/power", 200, 0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn teleport_destination_rejects_non_finite_and_out_of_range_values() {
        for destination in [
            PlayerTeleport {
                x: f64::NAN,
                y: 64.0,
                z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
            },
            PlayerTeleport {
                x: MAX_TELEPORT_COORDINATE + 1.0,
                y: 64.0,
                z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
            },
            PlayerTeleport {
                x: 0.0,
                y: 64.0,
                z: 0.0,
                yaw: 0.0,
                pitch: 91.0,
            },
            PlayerTeleport {
                x: 0.0,
                y: 64.0,
                z: 0.0,
                yaw: f32::INFINITY,
                pitch: 0.0,
            },
        ] {
            assert_eq!(
                teleport(PlayerHandle::new(1, 2, 3, 4).unwrap(), destination)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn teleport_requires_wasm_host_after_local_validation() {
        assert_eq!(
            teleport(
                PlayerHandle::new(1, 2, 3, 4).unwrap(),
                PlayerTeleport::new(1.0, 64.0, -2.0, 180.0, 20.0).unwrap(),
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn cross_dimension_teleport_validates_before_host_access() {
        let player = PlayerHandle::new(1, 2, 3, 4).unwrap();
        let world = WorldHandle::new(5, 6, 7, 8).unwrap();
        assert_eq!(
            teleport_to_world(
                player,
                world,
                PlayerTeleport {
                    x: f64::INFINITY,
                    y: 64.0,
                    z: 0.0,
                    yaw: 0.0,
                    pitch: 0.0,
                }
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            teleport_to_world(
                player,
                world,
                PlayerTeleport::new(1.0, 64.0, -2.0, 180.0, 20.0).unwrap(),
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn vitals_rejects_non_finite_or_out_of_range_host_values() {
        for vitals in [
            PlayerVitals {
                health: f32::NAN,
                max_health: 20.0,
                food_level: 20,
                saturation: 5.0,
                experience_level: 3,
                experience_progress: 0.5,
            },
            PlayerVitals {
                health: 10.0,
                max_health: 0.0,
                food_level: 20,
                saturation: 5.0,
                experience_level: 3,
                experience_progress: 0.5,
            },
            PlayerVitals {
                health: 10.0,
                max_health: 20.0,
                food_level: 21,
                saturation: 5.0,
                experience_level: 3,
                experience_progress: 0.5,
            },
            PlayerVitals {
                health: 10.0,
                max_health: 20.0,
                food_level: 20,
                saturation: f32::INFINITY,
                experience_level: 3,
                experience_progress: 0.5,
            },
            PlayerVitals {
                health: 10.0,
                max_health: 20.0,
                food_level: 20,
                saturation: 5.0,
                experience_level: 3,
                experience_progress: f32::NAN,
            },
        ] {
            assert_eq!(
                vitals.validate().unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn vitals_requires_the_wasm_host() {
        assert_eq!(
            vitals(PlayerHandle::new(1, 2, 3, 4).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
