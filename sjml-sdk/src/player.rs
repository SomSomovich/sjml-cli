//! Scoped access to online player sessions on the owning logical server. Snapshots, vitals, movement-state, active-effect, team, and game-mode reads require
//! requested and granted `entity.read`; effect application also requires `entity.effect.apply`, teleportation requires
//! `player.teleport`, game-mode changes require `player.game_mode.write`, vitals changes require
//! `player.vitals.write`, velocity changes require `player.motion.write`, and ability changes require
//! `player.abilities.write`; item cooldown changes require `player.cooldown.write`; action-bar messages require `player.ui.actionbar`, system chat messages require `player.chat.system_message`, and title packets
//! require `player.ui.title`; module-owned tab-list header/footer changes require
//! `player.ui.tab_list`; advancement changes require `player.advancement.write`; vanilla custom-stat
//! changes require `player.statistics.write`; resetting a player's fall distance requires
//! [`FALL_DISTANCE_WRITE_CAPABILITY`]; air-supply writes require [`AIR_SUPPLY_WRITE_CAPABILITY`]. Player handles
//! survive respawn and dimension changes, and are invalidated when logout begins or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::entity::status_result;
pub use crate::entity::ActiveEffect;
#[cfg(target_arch = "wasm32")]
use crate::entity::ActiveEffectsResponse;
use crate::entity::{validate_effect_id, MAX_EFFECT_DURATION_TICKS};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
#[cfg(any(target_arch = "wasm32", test))]
use crate::Decoder;
#[cfg(target_arch = "wasm32")]
use crate::MAX_STRING_BYTES;
use crate::{
    Encoder, ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode, StableEncode,
    WorldHandle,
};

/// Maximum number of detached status effects returned for one live player session.
pub const MAX_PLAYER_ACTIVE_EFFECTS: usize = crate::entity::MAX_ACTIVE_ENTITY_EFFECTS;
/// Maximum response bytes reserved for one player's detached active-effect snapshot.
pub const MAX_PLAYER_ACTIVE_EFFECTS_RESPONSE_BYTES: usize =
    crate::entity::MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES;
/// Maximum UTF-8 bytes accepted for an existing vanilla scoreboard team name.
pub const MAX_PLAYER_TEAM_NAME_BYTES: usize = 64;
/// Maximum schema-v1 response size for a current player scoreboard-team assignment.
pub const MAX_PLAYER_TEAM_RESPONSE_BYTES: usize = 4 + 1 + 4 + MAX_PLAYER_TEAM_NAME_BYTES;
/// Capability required with `entity.read` to change an online player's current air supply.
pub const AIR_SUPPLY_WRITE_CAPABILITY: &str = "player.air.write";
/// Capability required with `entity.read` to reset an online player's fall distance.
pub const FALL_DISTANCE_WRITE_CAPABILITY: &str = "player.fall_distance.write";

#[cfg(any(target_arch = "wasm32", test))]
const PLAYER_TEAM_RESPONSE_SCHEMA: u32 = 1;

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, StableEncode)]
#[sjml(version = 1)]
struct PlayerTeamResponse {
    team_name: Option<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for PlayerTeamResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != PLAYER_TEAM_RESPONSE_SCHEMA {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported player scoreboard-team response schema",
            ));
        }
        let team_name = match decoder.read_u8()? {
            0 => None,
            1 => {
                let length = decoder.read_length(MAX_PLAYER_TEAM_NAME_BYTES)?;
                if length == 0 {
                    return Err(SjmlError::new(
                        ErrorCode::InvalidArgument,
                        "player scoreboard-team name cannot be empty",
                    ));
                }
                let bytes = decoder.read_raw(length)?;
                let value = core::str::from_utf8(bytes).map_err(|_| {
                    SjmlError::new(
                        ErrorCode::InvalidUtf8,
                        "player scoreboard-team name is not valid UTF-8",
                    )
                })?;
                validate_player_team_name(value)?;
                Some(value.to_owned())
            }
            value => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    format!("player scoreboard-team option tag must be 0 or 1, found {value}"),
                ));
            }
        };
        if decoder
            .read_raw(decoder.remaining())?
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player scoreboard-team response has nonzero trailing data",
            ));
        }
        Ok(Self { team_name })
    }
}

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

/// Detached vanilla respawn destination for one online player.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerRespawnPoint {
    /// Block X coordinate used by vanilla when the player respawns.
    pub x: i32,
    /// Block Y coordinate used by vanilla when the player respawns.
    pub y: i32,
    /// Block Z coordinate used by vanilla when the player respawns.
    pub z: i32,
    /// Canonical namespace:path ID of an already-loaded server dimension.
    pub dimension_id: String,
    /// Vanilla respawn yaw in degrees.
    pub angle: f32,
    /// Whether vanilla should force the configured position when its spawn block is unsuitable.
    pub forced: bool,
}

impl PlayerRespawnPoint {
    /// Creates a respawn destination after validating its fixed coordinate, dimension, and angle bounds.
    pub fn new(
        x: i32,
        y: i32,
        z: i32,
        dimension_id: impl Into<String>,
        angle: f32,
        forced: bool,
    ) -> SjmlResult<Self> {
        let point = Self {
            x,
            y,
            z,
            dimension_id: dimension_id.into(),
            angle,
            forced,
        };
        point.validate()?;
        Ok(point)
    }

    fn validate(&self) -> SjmlResult<()> {
        if self.x.unsigned_abs() > MAX_PLAYER_RESPAWN_COORDINATE as u32
            || self.y.unsigned_abs() > MAX_PLAYER_RESPAWN_COORDINATE as u32
            || self.z.unsigned_abs() > MAX_PLAYER_RESPAWN_COORDINATE as u32
            || !self.angle.is_finite()
            || self.angle.abs() > MAX_PLAYER_RESPAWN_ANGLE
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player respawn coordinates or angle are outside the supported range",
            ));
        }
        validate_player_respawn_dimension(&self.dimension_id)
    }
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

/// Detached current and maximum air supply for an online player.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAirSupply {
    /// Current air ticks; vanilla may report a value down to -20 during drowning damage.
    pub current: i32,
    /// The current entity-specific maximum air supply.
    pub max: u32,
}

impl PlayerAirSupply {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<()> {
        if self.current < -20 || self.current as i64 > i64::from(self.max) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned air supply outside the supported current/maximum bounds",
            ));
        }
        Ok(())
    }
}

/// Detached movement flags and fall distance for an online player.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerMovementState {
    /// Whether vanilla currently considers the player grounded.
    pub on_ground: bool,
    /// Whether the player is sprinting.
    pub sprinting: bool,
    /// Whether the player is crouching.
    pub crouching: bool,
    /// Whether the player is swimming.
    pub swimming: bool,
    /// Current fall distance in blocks, bounded to `0..=MAX_PLAYER_FALL_DISTANCE`.
    pub fall_distance: f32,
}

impl PlayerMovementState {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<()> {
        if !self.fall_distance.is_finite()
            || !(0.0..=MAX_PLAYER_FALL_DISTANCE).contains(&self.fall_distance)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a non-finite or out-of-range player fall distance",
            ));
        }
        Ok(())
    }
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
/// Maximum fall distance in blocks accepted from the player movement-state snapshot.
pub const MAX_PLAYER_FALL_DISTANCE: f32 = 1_000_000.0;
/// Maximum absolute yaw accepted by [`set_rotation`].
pub const MAX_PLAYER_ROTATION_YAW: f32 = 36_000.0;
/// Maximum duration accepted for a player item cooldown, in server ticks.
pub const MAX_PLAYER_ITEM_COOLDOWN_TICKS: u32 = 72_000;
/// Maximum UTF-8 size accepted for a namespaced item ID used with player cooldown APIs.
pub const MAX_PLAYER_COOLDOWN_ITEM_ID_BYTES: usize = 256;
/// Maximum UTF-8 size accepted for a registered advancement ID.
pub const MAX_PLAYER_ADVANCEMENT_ID_BYTES: usize = 256;
/// Maximum UTF-8 dimension ID accepted for a player respawn point.
pub const MAX_PLAYER_RESPAWN_DIMENSION_ID_BYTES: usize = 256;
/// Maximum absolute block coordinate accepted before target dimension build-height and border checks.
pub const MAX_PLAYER_RESPAWN_COORDINATE: i32 = 30_000_000;
/// Maximum absolute yaw accepted for a player respawn point.
pub const MAX_PLAYER_RESPAWN_ANGLE: f32 = 36_000.0;
/// Maximum schema-v1 encoded size of one respawn point.
pub const MAX_PLAYER_RESPAWN_POINT_BYTES: u32 = 281;
/// Maximum schema-v1 encoded size of an optional respawn-point snapshot.
pub const MAX_PLAYER_RESPAWN_SNAPSHOT_BYTES: u32 = MAX_PLAYER_RESPAWN_POINT_BYTES + 1;
/// Maximum amount one call may add to a vanilla custom-stat counter.
pub const MAX_PLAYER_STATISTIC_INCREMENT: u32 = 1_000_000;
/// Maximum exact value accepted by vanilla's signed 32-bit statistic counters.
pub const MAX_PLAYER_STATISTIC_VALUE: u32 = i32::MAX as u32;

/// One of Minecraft's built-in player game modes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum PlayerGameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

/// A vanilla 1.21.1 custom player statistic supported by the SJML host.
///
/// Registry-backed item, block, and entity statistic types are intentionally not represented here.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerCustomStat {
    LeaveGame,
    PlayTime,
    TotalWorldTime,
    TimeSinceDeath,
    TimeSinceRest,
    SneakTime,
    WalkOneCm,
    CrouchOneCm,
    SprintOneCm,
    WalkOnWaterOneCm,
    FallOneCm,
    ClimbOneCm,
    FlyOneCm,
    WalkUnderWaterOneCm,
    MinecartOneCm,
    BoatOneCm,
    PigOneCm,
    HorseOneCm,
    AviateOneCm,
    SwimOneCm,
    StriderOneCm,
    Jump,
    Drop,
    DamageDealt,
    DamageDealtAbsorbed,
    DamageDealtResisted,
    DamageTaken,
    DamageBlockedByShield,
    DamageAbsorbed,
    DamageResisted,
    Deaths,
    MobKills,
    AnimalsBred,
    PlayerKills,
    FishCaught,
    TalkedToVillager,
    TradedWithVillager,
    EatCakeSlice,
    FillCauldron,
    UseCauldron,
    CleanArmor,
    CleanBanner,
    CleanShulkerBox,
    InteractWithBrewingStand,
    InteractWithBeacon,
    InspectDropper,
    InspectHopper,
    InspectDispenser,
    PlayNoteBlock,
    TuneNoteBlock,
    PotFlower,
    TriggerTrappedChest,
    OpenEnderChest,
    EnchantItem,
    PlayRecord,
    InteractWithFurnace,
    InteractWithCraftingTable,
    OpenChest,
    SleepInBed,
    OpenShulkerBox,
    OpenBarrel,
    InteractWithBlastFurnace,
    InteractWithSmoker,
    InteractWithLectern,
    InteractWithCampfire,
    InteractWithCartographyTable,
    InteractWithLoom,
    InteractWithStonecutter,
    BellRing,
    RaidTrigger,
    RaidWin,
    InteractWithAnvil,
    InteractWithGrindstone,
    TargetHit,
    InteractWithSmithingTable,
}

impl PlayerCustomStat {
    /// Returns the canonical vanilla resource ID for this custom statistic.
    pub const fn id(self) -> &'static str {
        match self {
            Self::LeaveGame => "minecraft:leave_game",
            Self::PlayTime => "minecraft:play_time",
            Self::TotalWorldTime => "minecraft:total_world_time",
            Self::TimeSinceDeath => "minecraft:time_since_death",
            Self::TimeSinceRest => "minecraft:time_since_rest",
            Self::SneakTime => "minecraft:sneak_time",
            Self::WalkOneCm => "minecraft:walk_one_cm",
            Self::CrouchOneCm => "minecraft:crouch_one_cm",
            Self::SprintOneCm => "minecraft:sprint_one_cm",
            Self::WalkOnWaterOneCm => "minecraft:walk_on_water_one_cm",
            Self::FallOneCm => "minecraft:fall_one_cm",
            Self::ClimbOneCm => "minecraft:climb_one_cm",
            Self::FlyOneCm => "minecraft:fly_one_cm",
            Self::WalkUnderWaterOneCm => "minecraft:walk_under_water_one_cm",
            Self::MinecartOneCm => "minecraft:minecart_one_cm",
            Self::BoatOneCm => "minecraft:boat_one_cm",
            Self::PigOneCm => "minecraft:pig_one_cm",
            Self::HorseOneCm => "minecraft:horse_one_cm",
            Self::AviateOneCm => "minecraft:aviate_one_cm",
            Self::SwimOneCm => "minecraft:swim_one_cm",
            Self::StriderOneCm => "minecraft:strider_one_cm",
            Self::Jump => "minecraft:jump",
            Self::Drop => "minecraft:drop",
            Self::DamageDealt => "minecraft:damage_dealt",
            Self::DamageDealtAbsorbed => "minecraft:damage_dealt_absorbed",
            Self::DamageDealtResisted => "minecraft:damage_dealt_resisted",
            Self::DamageTaken => "minecraft:damage_taken",
            Self::DamageBlockedByShield => "minecraft:damage_blocked_by_shield",
            Self::DamageAbsorbed => "minecraft:damage_absorbed",
            Self::DamageResisted => "minecraft:damage_resisted",
            Self::Deaths => "minecraft:deaths",
            Self::MobKills => "minecraft:mob_kills",
            Self::AnimalsBred => "minecraft:animals_bred",
            Self::PlayerKills => "minecraft:player_kills",
            Self::FishCaught => "minecraft:fish_caught",
            Self::TalkedToVillager => "minecraft:talked_to_villager",
            Self::TradedWithVillager => "minecraft:traded_with_villager",
            Self::EatCakeSlice => "minecraft:eat_cake_slice",
            Self::FillCauldron => "minecraft:fill_cauldron",
            Self::UseCauldron => "minecraft:use_cauldron",
            Self::CleanArmor => "minecraft:clean_armor",
            Self::CleanBanner => "minecraft:clean_banner",
            Self::CleanShulkerBox => "minecraft:clean_shulker_box",
            Self::InteractWithBrewingStand => "minecraft:interact_with_brewingstand",
            Self::InteractWithBeacon => "minecraft:interact_with_beacon",
            Self::InspectDropper => "minecraft:inspect_dropper",
            Self::InspectHopper => "minecraft:inspect_hopper",
            Self::InspectDispenser => "minecraft:inspect_dispenser",
            Self::PlayNoteBlock => "minecraft:play_noteblock",
            Self::TuneNoteBlock => "minecraft:tune_noteblock",
            Self::PotFlower => "minecraft:pot_flower",
            Self::TriggerTrappedChest => "minecraft:trigger_trapped_chest",
            Self::OpenEnderChest => "minecraft:open_enderchest",
            Self::EnchantItem => "minecraft:enchant_item",
            Self::PlayRecord => "minecraft:play_record",
            Self::InteractWithFurnace => "minecraft:interact_with_furnace",
            Self::InteractWithCraftingTable => "minecraft:interact_with_crafting_table",
            Self::OpenChest => "minecraft:open_chest",
            Self::SleepInBed => "minecraft:sleep_in_bed",
            Self::OpenShulkerBox => "minecraft:open_shulker_box",
            Self::OpenBarrel => "minecraft:open_barrel",
            Self::InteractWithBlastFurnace => "minecraft:interact_with_blast_furnace",
            Self::InteractWithSmoker => "minecraft:interact_with_smoker",
            Self::InteractWithLectern => "minecraft:interact_with_lectern",
            Self::InteractWithCampfire => "minecraft:interact_with_campfire",
            Self::InteractWithCartographyTable => "minecraft:interact_with_cartography_table",
            Self::InteractWithLoom => "minecraft:interact_with_loom",
            Self::InteractWithStonecutter => "minecraft:interact_with_stonecutter",
            Self::BellRing => "minecraft:bell_ring",
            Self::RaidTrigger => "minecraft:raid_trigger",
            Self::RaidWin => "minecraft:raid_win",
            Self::InteractWithAnvil => "minecraft:interact_with_anvil",
            Self::InteractWithGrindstone => "minecraft:interact_with_grindstone",
            Self::TargetHit => "minecraft:target_hit",
            Self::InteractWithSmithingTable => "minecraft:interact_with_smithing_table",
        }
    }
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

impl PlayerMotion {
    /// Returns the normalized Minecraft look direction as `[x, y, z]`.
    ///
    /// Yaw and pitch are interpreted in degrees using Minecraft's convention:
    /// yaw 0 and pitch 0 point toward positive Z, positive yaw turns toward
    /// negative X, and positive pitch points downward.
    pub fn look_direction(&self) -> [f64; 3] {
        let yaw = f64::from(self.yaw).to_radians();
        let pitch = f64::from(self.pitch).to_radians();
        let cos_pitch = pitch.cos();
        [-yaw.sin() * cos_pitch, -pitch.sin(), yaw.cos() * cos_pitch]
    }
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
/// Minimum flight and walking speed accepted by the player ability write API.
pub const MIN_PLAYER_ABILITY_SPEED: f32 = 0.0;
/// Maximum flight and walking speed accepted by the player ability write API.
pub const MAX_PLAYER_ABILITY_SPEED: f32 = 0.5;
/// Maximum UTF-8 size accepted for one literal player action-bar message.
pub const MAX_ACTION_BAR_TEXT_BYTES: usize = 1024;
/// Maximum Unicode scalar values accepted for one literal player action-bar message.
pub const MAX_ACTION_BAR_TEXT_CODE_POINTS: usize = 256;
/// Maximum UTF-8 size accepted for one literal player system chat message.
pub const MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES: usize = 1024;
/// Maximum Unicode scalar values accepted for one literal player system chat message.
pub const MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS: usize = 256;
/// Maximum encoded payload for one typed rich system-chat component tree.
pub const MAX_PLAYER_RICH_SYSTEM_MESSAGE_PAYLOAD_BYTES: usize = 8_192;
/// Maximum UTF-8 bytes in one literal, hover, or click-action string.
pub const MAX_PLAYER_RICH_SYSTEM_MESSAGE_STRING_BYTES: usize = 1_024;
/// Maximum aggregate UTF-8 bytes across all rich message strings.
pub const MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES: usize = 4_096;
/// Maximum number of nodes, including hover text nodes.
pub const MAX_PLAYER_RICH_SYSTEM_MESSAGE_NODES: usize = 64;
/// Maximum tree nesting depth, counting a root node as depth one.
pub const MAX_PLAYER_RICH_SYSTEM_MESSAGE_DEPTH: usize = 8;

/// Supported named Minecraft chat colors plus explicit 24-bit RGB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RichTextColor {
    Rgb(u32),
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl RichTextColor {
    pub fn rgb(value: u32) -> SjmlResult<Self> {
        if value > 0xFF_FFFF {
            return Err(rich_text_invalid("RGB color must fit in 24 bits"));
        }
        Ok(Self::Rgb(value))
    }
}

/// Safe click behavior; executing a command is deliberately unavailable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RichTextClickAction {
    OpenUrl(String),
    SuggestCommand(String),
    CopyToClipboard(String),
}

impl RichTextClickAction {
    pub fn open_url(value: impl Into<String>) -> SjmlResult<Self> {
        let value = value.into();
        validate_rich_text_string(&value, "open_url value")?;
        if !valid_rich_http_url(&value) {
            return Err(rich_text_invalid(
                "open_url requires a valid HTTP(S) URI without user info",
            ));
        }
        Ok(Self::OpenUrl(value))
    }
    pub fn suggest_command(value: impl Into<String>) -> SjmlResult<Self> {
        let value = value.into();
        validate_rich_text_string(&value, "suggest_command value")?;
        Ok(Self::SuggestCommand(value))
    }
    pub fn copy_to_clipboard(value: impl Into<String>) -> SjmlResult<Self> {
        let value = value.into();
        validate_rich_text_string(&value, "copy_to_clipboard value")?;
        Ok(Self::CopyToClipboard(value))
    }
}

/// One literal text component with bounded nested children and optional show-text hover content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RichTextNode {
    text: String,
    color: Option<RichTextColor>,
    flags: u8,
    click: Option<RichTextClickAction>,
    hover: Option<RichText>,
    children: Vec<Self>,
}

impl RichTextNode {
    pub fn new(text: impl Into<String>) -> SjmlResult<Self> {
        let text = text.into();
        validate_rich_text_string(&text, "rich message text")?;
        Ok(Self {
            text,
            color: None,
            flags: 0,
            click: None,
            hover: None,
            children: Vec::new(),
        })
    }
    pub fn color(mut self, color: RichTextColor) -> SjmlResult<Self> {
        if let RichTextColor::Rgb(value) = color {
            if value > 0xFF_FFFF {
                return Err(rich_text_invalid("RGB color must fit in 24 bits"));
            }
        }
        self.color = Some(color);
        Ok(self)
    }
    pub fn bold(mut self, enabled: bool) -> Self {
        self.set_flag(1, enabled);
        self
    }
    pub fn italic(mut self, enabled: bool) -> Self {
        self.set_flag(1 << 1, enabled);
        self
    }
    pub fn underline(mut self, enabled: bool) -> Self {
        self.set_flag(1 << 2, enabled);
        self
    }
    pub fn strikethrough(mut self, enabled: bool) -> Self {
        self.set_flag(1 << 3, enabled);
        self
    }
    pub fn click(mut self, action: RichTextClickAction) -> Self {
        self.click = Some(action);
        self
    }
    pub fn hover_text(mut self, value: RichText) -> SjmlResult<Self> {
        validate_rich_text(&value, false)?;
        self.hover = Some(value);
        Ok(self)
    }
    pub fn child(mut self, child: RichTextNode) -> SjmlResult<Self> {
        self.children.push(child);
        validate_rich_node(&self, 1, &mut RichTextBudget::default(), true)?;
        Ok(self)
    }
    fn set_flag(&mut self, flag: u8, enabled: bool) {
        if enabled {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

/// Bounded tree of literal Minecraft text components.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RichText {
    nodes: Vec<RichTextNode>,
}

impl RichText {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn plain(text: impl Into<String>) -> SjmlResult<Self> {
        let mut result = Self::new();
        result.push(RichTextNode::new(text)?)?;
        Ok(result)
    }
    pub fn push(&mut self, node: RichTextNode) -> SjmlResult<()> {
        self.nodes.push(node);
        if let Err(error) = validate_rich_text(self, true) {
            self.nodes.pop();
            return Err(error);
        }
        Ok(())
    }
    pub fn nodes(&self) -> &[RichTextNode] {
        &self.nodes
    }
    fn write_nodes(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_length(self.nodes.len())?;
        for node in &self.nodes {
            write_rich_node(node, encoder)?;
        }
        Ok(())
    }
}

impl StableEncode for RichText {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_rich_text(self, false)?;
        encoder.write_u32(1)?;
        self.write_nodes(encoder)
    }
}
/// Maximum UTF-8 size accepted for a literal player title or subtitle.
pub const MAX_PLAYER_TITLE_TEXT_BYTES: usize = 1024;
/// Maximum Unicode scalar values accepted for a literal player title or subtitle.
pub const MAX_PLAYER_TITLE_TEXT_CODE_POINTS: usize = 256;
/// Maximum UTF-8 size accepted for a literal player-list header or footer contribution.
pub const MAX_PLAYER_TAB_LIST_TEXT_BYTES: usize = 1024;
/// Maximum Unicode scalar values accepted for a player-list header or footer contribution.
pub const MAX_PLAYER_TAB_LIST_TEXT_CODE_POINTS: usize = 256;
/// Maximum fade or stay duration, in ticks, for a player title.
pub const MAX_PLAYER_TITLE_TIMING_TICKS: u32 = 6_000;
/// Largest valid selected hotbar index; vanilla players have slots `0..=8`.
pub const MAX_PLAYER_HOTBAR_SLOT_INDEX: u8 = 8;

/// Vanilla title animation timings in server ticks.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PlayerTitleTimings {
    /// Fade-in duration.
    pub fade_in: u32,
    /// Visible duration before fade-out.
    pub stay: u32,
    /// Fade-out duration.
    pub fade_out: u32,
}

impl PlayerTitleTimings {
    /// Creates bounded title animation timings. Each duration is at most five minutes.
    pub fn new(fade_in: u32, stay: u32, fade_out: u32) -> SjmlResult<Self> {
        let timings = Self {
            fade_in,
            stay,
            fade_out,
        };
        if [fade_in, stay, fade_out]
            .into_iter()
            .any(|ticks| ticks > MAX_PLAYER_TITLE_TIMING_TICKS)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player title timings must each be between zero and 6000 ticks",
            ));
        }
        Ok(timings)
    }
}

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
    #[link_name = "set_may_fly"]
    fn imported_set_may_fly(handle: i32, length: i32, enabled: i32) -> i32;
    #[link_name = "set_instabuild"]
    fn imported_set_instabuild(handle: i32, length: i32, enabled: i32) -> i32;
    #[link_name = "set_invulnerable"]
    fn imported_set_invulnerable(handle: i32, length: i32, enabled: i32) -> i32;
    #[link_name = "set_may_build"]
    fn imported_set_may_build(handle: i32, length: i32, enabled: i32) -> i32;
    #[link_name = "set_flying_speed"]
    fn imported_set_flying_speed(handle: i32, length: i32, speed_bits: i32) -> i32;
    #[link_name = "set_walking_speed"]
    fn imported_set_walking_speed(handle: i32, length: i32, speed_bits: i32) -> i32;
    #[link_name = "vitals"]
    fn imported_vitals(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "air_supply"]
    fn imported_air_supply(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "movement_state"]
    fn imported_movement_state(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "game_mode"]
    fn imported_game_mode(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "selected_hotbar_slot"]
    fn imported_selected_hotbar_slot(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_selected_hotbar_slot"]
    fn imported_set_selected_hotbar_slot(handle: i32, length: i32, slot: i32) -> i32;
    #[link_name = "set_game_mode"]
    fn imported_set_game_mode(handle: i32, length: i32, game_mode: i32) -> i32;
    #[link_name = "respawn_point"]
    fn imported_respawn_point(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_respawn_point"]
    fn imported_set_respawn_point(
        handle: i32,
        handle_length: i32,
        point: i32,
        point_length: i32,
    ) -> i32;
    #[link_name = "clear_respawn_point"]
    fn imported_clear_respawn_point(handle: i32, handle_length: i32) -> i32;
    #[link_name = "set_health"]
    fn imported_set_health(handle: i32, length: i32, health_bits: i32) -> i32;
    #[link_name = "set_food"]
    fn imported_set_food(handle: i32, length: i32, food_level: i32, saturation_bits: i32) -> i32;
    #[link_name = "set_experience"]
    fn imported_set_experience(handle: i32, length: i32, level: i32, progress_bits: i32) -> i32;
    #[link_name = "set_air_supply"]
    fn imported_set_air_supply(handle: i32, length: i32, air_supply: i32) -> i32;
    #[link_name = "reset_fall_distance"]
    fn imported_reset_fall_distance(handle: i32, length: i32) -> i32;
    #[link_name = "has_item_cooldown"]
    fn imported_has_item_cooldown(
        handle: i32,
        handle_length: i32,
        item_id: i32,
        item_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "set_item_cooldown"]
    fn imported_set_item_cooldown(
        handle: i32,
        handle_length: i32,
        item_id: i32,
        item_id_length: i32,
        ticks: i32,
    ) -> i32;
    #[link_name = "clear_item_cooldown"]
    fn imported_clear_item_cooldown(
        handle: i32,
        handle_length: i32,
        item_id: i32,
        item_id_length: i32,
    ) -> i32;
    #[link_name = "custom_stat"]
    fn imported_custom_stat(
        handle: i32,
        handle_length: i32,
        statistic_id: i32,
        statistic_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "increment_custom_stat"]
    fn imported_increment_custom_stat(
        handle: i32,
        handle_length: i32,
        statistic_id: i32,
        statistic_id_length: i32,
        amount: i32,
    ) -> i32;
    #[link_name = "set_custom_stat"]
    fn imported_set_custom_stat(
        handle: i32,
        handle_length: i32,
        statistic_id: i32,
        statistic_id_length: i32,
        value: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "grant_advancement"]
    fn imported_grant_advancement(
        handle: i32,
        handle_length: i32,
        advancement_id: i32,
        advancement_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "revoke_advancement"]
    fn imported_revoke_advancement(
        handle: i32,
        handle_length: i32,
        advancement_id: i32,
        advancement_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "apply_effect"]
    fn imported_apply_effect(
        handle: i32,
        handle_length: i32,
        effect_id: i32,
        effect_id_length: i32,
        duration_ticks: i32,
        amplifier: i32,
    ) -> i32;
    #[link_name = "active_effects"]
    fn imported_active_effects(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "team"]
    fn imported_team(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_team"]
    fn imported_set_team(
        handle: i32,
        handle_length: i32,
        team_name: i32,
        team_name_length: i32,
    ) -> i32;
    #[link_name = "clear_team"]
    fn imported_clear_team(handle: i32, handle_length: i32) -> i32;
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
    #[link_name = "set_rotation"]
    fn imported_set_rotation(
        handle: i32,
        handle_length: i32,
        yaw_bits: i32,
        pitch_bits: i32,
    ) -> i32;
    #[link_name = "action_bar"]
    fn imported_action_bar(handle: i32, handle_length: i32, text: i32, text_length: i32) -> i32;
    #[link_name = "system_message"]
    fn imported_system_message(handle: i32, handle_length: i32, text: i32, text_length: i32)
        -> i32;
    #[link_name = "rich_system_message"]
    fn imported_rich_system_message(
        handle: i32,
        handle_length: i32,
        payload: i32,
        payload_length: i32,
    ) -> i32;
    #[link_name = "title"]
    fn imported_title(
        handle: i32,
        handle_length: i32,
        title: i32,
        title_length: i32,
        subtitle: i32,
        subtitle_length: i32,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    ) -> i32;
    #[link_name = "set_tab_list"]
    fn imported_set_tab_list(
        handle: i32,
        handle_length: i32,
        header: i32,
        header_length: i32,
        footer: i32,
        footer_length: i32,
    ) -> i32;
    #[link_name = "clear_tab_list"]
    fn imported_clear_tab_list(handle: i32, handle_length: i32) -> i32;
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

/// Reads current and maximum air supply for an online server player.
///
/// Requires `entity.read` and a live player handle on the owning logical-server
/// thread. `current` may be negative (down to -20) during vanilla's drowning
/// damage phase; `max` is read from the current player entity. The returned
/// value is detached and includes the maximum needed to choose a valid write.
pub fn air_supply(handle: PlayerHandle) -> SjmlResult<PlayerAirSupply> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let air: PlayerAirSupply = read_output(12, |output, capacity| {
            // SAFETY: handle and output stay valid for this synchronous host read.
            unsafe {
                imported_air_supply(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        air.validate()?;
        Ok(air)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads detached movement flags and fall distance for an online server player.
///
/// Requires `entity.read` and a live player handle on the owning logical-server
/// thread. Grounded, sprinting, crouching, and swimming values reflect the
/// current vanilla entity state. Fall distance must be finite and in
/// `0..=MAX_PLAYER_FALL_DISTANCE`. This API does not change player state.
pub fn movement_state(handle: PlayerHandle) -> SjmlResult<PlayerMovementState> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let state: PlayerMovementState = read_output(12, |output, capacity| {
            // SAFETY: the encoded handle and output remain valid through the synchronous host read.
            unsafe {
                imported_movement_state(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        state.validate()?;
        Ok(state)
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

/// Reads the selected hotbar index of an online player on the logical server.
///
/// The returned slot is always in `0..=8`. Requires the declared and granted
/// `entity.read` capability. Client modules, static registration, and stale
/// player handles cannot read the selection.
pub fn selected_hotbar_slot(handle: PlayerHandle) -> SjmlResult<u8> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let slot = read_output::<u8>(1, |output, capacity| {
            // SAFETY: the encoded handle and output buffer remain live through this synchronous import.
            unsafe {
                imported_selected_hotbar_slot(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if slot > MAX_PLAYER_HOTBAR_SLOT_INDEX {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a selected hotbar slot outside 0..=8",
            ));
        }
        Ok(slot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Changes an online player's selected hotbar index without changing item stacks.
///
/// The slot must be in `0..=8`. Requires `entity.read` and `inventory.write`
/// in a logical-server module. The host validates the live session and server
/// thread, then sends Minecraft's carried-slot packet to synchronize the
/// selection to that player's client.
pub fn set_selected_hotbar_slot(handle: PlayerHandle, slot: u8) -> SjmlResult<()> {
    if slot > MAX_PLAYER_HOTBAR_SLOT_INDEX {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "selected hotbar slot must be between 0 and 8",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and bounded slot scalar remain live for the synchronous host call.
        let status = unsafe {
            imported_set_selected_hotbar_slot(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                slot as i32,
            )
        };
        status_result(status, "host rejected the selected hotbar slot change")
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

/// Reads an online player's configured vanilla respawn point.
///
/// Returns `Ok(None)` when the player has no bed, anchor, or other vanilla respawn point. A
/// present snapshot contains its block position, canonical dimension ID, yaw angle, and forced
/// flag. This reads only the player's saved respawn metadata; it does not load a dimension or
/// inspect a chunk. Requires `entity.read` and a live handle in a logical-server module.
pub fn respawn_point(handle: PlayerHandle) -> SjmlResult<Option<PlayerRespawnPoint>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let point = read_output::<Option<PlayerRespawnPoint>>(
            MAX_PLAYER_RESPAWN_SNAPSHOT_BYTES,
            |output, capacity| {
                // SAFETY: handle and output storage remain live through the synchronous call.
                unsafe {
                    imported_respawn_point(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if let Some(point) = &point {
            point.validate()?;
        }
        Ok(point)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets the player's vanilla respawn destination.
///
/// Requires `entity.read` and `player.respawn.write` in a logical-server module. The target
/// dimension must already be loaded by this server; the host checks its build height and world
/// border, then stores the bounded position, yaw, and forced flag through Minecraft's vanilla
/// `ServerPlayer` respawn setter. The operation does not inspect, load, or request the target
/// chunk. Vanilla may load that chunk later when it actually respawns the player. A NeoForge
/// spawn hook may reject the update.
pub fn set_respawn_point(handle: PlayerHandle, point: PlayerRespawnPoint) -> SjmlResult<()> {
    point.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_point = to_bytes(&point)?;
        // SAFETY: both stable byte buffers remain live through the synchronous host import.
        let status = unsafe {
            imported_set_respawn_point(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                encoded_point.as_ptr() as usize as u32 as i32,
                encoded_point.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the player respawn-point change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Clears the player's configured vanilla respawn point.
///
/// Requires `entity.read` and `player.respawn.write` in a logical-server module. After clearing,
/// Minecraft chooses its normal default spawn when this player next respawns.
pub fn clear_respawn_point(handle: PlayerHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle remains live during the synchronous host call.
        let status = unsafe {
            imported_clear_respawn_point(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected clearing the player respawn point")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
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

/// Sends bounded literal text to one online player as an unsigned vanilla system-chat message.
///
/// Minecraft delivers this on the normal chat channel. The text is not interpreted as JSON, a
/// translation key, a command, or a signed message from the player. It must be nonblank, contain
/// no control characters, and fit within 1,024 UTF-8 bytes and 256 Unicode scalar values. Requires
/// declared and administrator-granted `entity.read` and `player.chat.system_message`; the host
/// revalidates the live player session and owning logical-server thread on every call. Client
/// modules and static registration cannot send messages. Each module may send at most ten system
/// messages in any rolling five-second window.
pub fn send_system_message(handle: PlayerHandle, text: &str) -> SjmlResult<()> {
    validate_system_message_text(text)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and borrowed UTF-8 text remain live during this import.
        let status = unsafe {
            imported_system_message(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                text.as_ptr() as usize as u32 as i32,
                text.len() as i32,
            )
        };
        status_result(status, "host rejected the player system chat message")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sends a bounded typed text-component tree as an unsigned system-chat message.
///
/// Nodes contain literal text, optional RGB or named color, basic style flags, nested children,
/// an optional `show_text` hover tree, and safe `open_url`, `suggest_command`, or
/// `copy_to_clipboard` click actions. Values are never interpreted as raw JSON. `open_url` only
/// accepts HTTP(S) URIs without user info; command execution is not exposed. The tree is limited
/// to 8 levels, 64 total nodes, 1,024 UTF-8 bytes per string, 4,096 aggregate string bytes, and an
/// 8,192-byte encoded payload. Requires `entity.read` and `player.chat.system_message`; server
/// thread, live handle, rate limit, and permissions are checked by the host.
pub fn send_rich_system_message(handle: PlayerHandle, message: &RichText) -> SjmlResult<()> {
    validate_rich_text(message, false)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let payload = to_bytes(message)?;
        if payload.len() > MAX_PLAYER_RICH_SYSTEM_MESSAGE_PAYLOAD_BYTES {
            return Err(rich_text_too_large(
                "rich system message payload exceeds 8192 bytes",
            ));
        }
        // SAFETY: both encoded slices remain live for the complete host import call.
        let status = unsafe {
            imported_rich_system_message(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as i32,
            )
        };
        status_result(status, "host rejected the rich player system chat message")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, message);
        Err(unsupported_target())
    }
}

fn write_rich_node(node: &RichTextNode, encoder: &mut Encoder) -> SjmlResult<()> {
    write_rich_string(encoder, &node.text)?;
    encoder.write_u8(node.flags)?;
    match node.color {
        None => encoder.write_u8(0)?,
        Some(RichTextColor::Rgb(rgb)) => {
            encoder.write_u8(1)?;
            encoder.write_u32(rgb)?;
        }
        Some(color) => {
            encoder.write_u8(2)?;
            encoder.write_u8(named_rich_color_wire(color)?)?;
        }
    }
    match &node.click {
        None => encoder.write_u8(0)?,
        Some(RichTextClickAction::OpenUrl(value)) => {
            encoder.write_u8(1)?;
            write_rich_string(encoder, value)?;
        }
        Some(RichTextClickAction::SuggestCommand(value)) => {
            encoder.write_u8(2)?;
            write_rich_string(encoder, value)?;
        }
        Some(RichTextClickAction::CopyToClipboard(value)) => {
            encoder.write_u8(3)?;
            write_rich_string(encoder, value)?;
        }
    }
    match &node.hover {
        None => encoder.write_u8(0)?,
        Some(hover) => {
            encoder.write_u8(1)?;
            hover.write_nodes(encoder)?;
        }
    }
    encoder.write_length(node.children.len())?;
    for child in &node.children {
        write_rich_node(child, encoder)?;
    }
    Ok(())
}

fn write_rich_string(encoder: &mut Encoder, value: &str) -> SjmlResult<()> {
    encoder.write_length(value.len())?;
    encoder.write_raw(value.as_bytes())
}

fn named_rich_color_wire(color: RichTextColor) -> SjmlResult<u8> {
    let value = match color {
        RichTextColor::Black => 0,
        RichTextColor::DarkBlue => 1,
        RichTextColor::DarkGreen => 2,
        RichTextColor::DarkAqua => 3,
        RichTextColor::DarkRed => 4,
        RichTextColor::DarkPurple => 5,
        RichTextColor::Gold => 6,
        RichTextColor::Gray => 7,
        RichTextColor::DarkGray => 8,
        RichTextColor::Blue => 9,
        RichTextColor::Green => 10,
        RichTextColor::Aqua => 11,
        RichTextColor::Red => 12,
        RichTextColor::LightPurple => 13,
        RichTextColor::Yellow => 14,
        RichTextColor::White => 15,
        RichTextColor::Rgb(_) => {
            return Err(rich_text_invalid("RGB color has no named-color code"));
        }
    };
    Ok(value)
}

#[derive(Default)]
struct RichTextBudget {
    nodes: usize,
    bytes: usize,
    visible: String,
}

fn validate_rich_text(value: &RichText, allow_empty: bool) -> SjmlResult<()> {
    if value.nodes.is_empty() {
        if allow_empty {
            return Ok(());
        }
        return Err(rich_text_invalid(
            "rich system message must contain at least one node",
        ));
    }
    let mut budget = RichTextBudget::default();
    for node in &value.nodes {
        validate_rich_node(node, 1, &mut budget, true)?;
    }
    if budget.visible.trim().is_empty() {
        return Err(rich_text_invalid(
            "rich system message must contain visible non-whitespace text",
        ));
    }
    if budget.bytes > MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES {
        return Err(rich_text_too_large(
            "rich system message strings exceed 4096 UTF-8 bytes total",
        ));
    }
    Ok(())
}

fn validate_rich_node(
    node: &RichTextNode,
    depth: usize,
    budget: &mut RichTextBudget,
    visible: bool,
) -> SjmlResult<()> {
    if depth > MAX_PLAYER_RICH_SYSTEM_MESSAGE_DEPTH {
        return Err(rich_text_too_large(
            "rich system message nesting exceeds depth 8",
        ));
    }
    budget.nodes += 1;
    if budget.nodes > MAX_PLAYER_RICH_SYSTEM_MESSAGE_NODES {
        return Err(rich_text_too_large(
            "rich system message contains more than 64 nodes",
        ));
    }
    validate_rich_text_string(&node.text, "rich message text")?;
    budget.bytes = budget
        .bytes
        .checked_add(node.text.len())
        .ok_or_else(|| rich_text_too_large("rich message string size overflowed"))?;
    if visible {
        budget.visible.push_str(&node.text);
    }
    if let Some(action) = &node.click {
        let value = match action {
            RichTextClickAction::OpenUrl(value) => {
                if !valid_rich_http_url(value) {
                    return Err(rich_text_invalid(
                        "open_url requires a valid HTTP(S) URI without user info",
                    ));
                }
                value
            }
            RichTextClickAction::SuggestCommand(value)
            | RichTextClickAction::CopyToClipboard(value) => value,
        };
        validate_rich_text_string(value, "click action value")?;
        budget.bytes = budget
            .bytes
            .checked_add(value.len())
            .ok_or_else(|| rich_text_too_large("rich message string size overflowed"))?;
    }
    if let Some(RichTextColor::Rgb(value)) = node.color {
        if value > 0xFF_FFFF {
            return Err(rich_text_invalid("RGB color must fit in 24 bits"));
        }
    }
    if let Some(hover) = &node.hover {
        if hover.nodes.is_empty() {
            return Err(rich_text_invalid("show_text hover tree cannot be empty"));
        }
        for child in &hover.nodes {
            validate_rich_node(child, depth + 1, budget, false)?;
        }
    }
    for child in &node.children {
        validate_rich_node(child, depth + 1, budget, visible)?;
    }
    if budget.bytes > MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES {
        return Err(rich_text_too_large(
            "rich system message strings exceed 4096 UTF-8 bytes total",
        ));
    }
    Ok(())
}

fn validate_rich_text_string(value: &str, label: &str) -> SjmlResult<()> {
    if value.len() > MAX_PLAYER_RICH_SYSTEM_MESSAGE_STRING_BYTES {
        return Err(rich_text_too_large(format!(
            "{label} exceeds 1024 UTF-8 bytes"
        )));
    }
    if value.trim().is_empty() {
        return Err(rich_text_invalid(format!("{label} must not be blank")));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err(rich_text_invalid(format!(
            "{label} contains a control character"
        )));
    }
    Ok(())
}

fn valid_rich_http_url(value: &str) -> bool {
    let uri_bytes = value.as_bytes();
    let mut index = 0;
    while index < uri_bytes.len() {
        let byte = uri_bytes[index];
        if byte == b'%' {
            if index + 2 >= uri_bytes.len()
                || !uri_bytes[index + 1].is_ascii_hexdigit()
                || !uri_bytes[index + 2].is_ascii_hexdigit()
            {
                return false;
            }
            index += 3;
            continue;
        }
        if !(byte.is_ascii_alphanumeric() || b"-._~:/?#[]@!$&'()*+,;=".contains(&byte)) {
            return false;
        }
        index += 1;
    }
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")) {
        return false;
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() || authority.contains('@') {
        return false;
    }
    let (host, port) = if let Some(bracketed) = authority.strip_prefix('[') {
        let Some(close) = bracketed.find(']') else {
            return false;
        };
        if bracketed[..close].parse::<std::net::Ipv6Addr>().is_err() {
            return false;
        }
        let suffix = &bracketed[close + 1..];
        if suffix.is_empty() {
            (&bracketed[..close], None)
        } else if let Some(port) = suffix.strip_prefix(':') {
            (&bracketed[..close], Some(port))
        } else {
            return false;
        }
    } else {
        let mut parts = authority.split(':');
        let host = parts.next().unwrap_or_default();
        let port = parts.next();
        if parts.next().is_some() {
            return false;
        }
        (host, port)
    };
    let valid_host = !host.is_empty()
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !host.starts_with('-')
        && !host.ends_with('-')
        && !host.contains("..")
        && host
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | ':'));
    let valid_port = port.is_none_or(|port| {
        !port.is_empty()
            && port.bytes().all(|byte| byte.is_ascii_digit())
            && port.parse::<u16>().is_ok_and(|number| number != 0)
    });
    valid_host && valid_port
}

fn rich_text_invalid(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
fn rich_text_too_large(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::PayloadTooLarge, message)
}

fn validate_system_message_text(text: &str) -> SjmlResult<()> {
    if text.trim().is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player system message must not be blank",
        ));
    }
    if text.len() > MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES
        || text.chars().count() > MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player system message exceeds 1024 UTF-8 bytes or 256 code points",
        ));
    }
    if text.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player system message must not contain control characters",
        ));
    }
    Ok(())
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

/// Sends literal title and optional subtitle text to an online player.
///
/// The host sends Minecraft's normal title packets and applies the supplied timings. Text is not
/// interpreted as JSON, a translation key, or a command. The title must be nonblank; the optional
/// subtitle may be empty. Each string is limited to 1,024 UTF-8 bytes and 256 Unicode scalar
/// values, and control characters are rejected. Timings are bounded to 6,000 ticks each.
///
/// Requires declared and administrator-granted `entity.read` and `player.ui.title`. The operation
/// is available only to logical-server modules and is rate limited to ten sends per five seconds.
pub fn show_title(
    handle: PlayerHandle,
    title: &str,
    subtitle: Option<&str>,
    timings: PlayerTitleTimings,
) -> SjmlResult<()> {
    validate_player_title_text(title, false)?;
    let subtitle = subtitle.unwrap_or_default();
    validate_player_title_text(subtitle, true)?;
    PlayerTitleTimings::new(timings.fade_in, timings.stay, timings.fade_out)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: handle and both borrowed UTF-8 strings remain live during the synchronous call.
        let status = unsafe {
            imported_title(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                title.as_ptr() as usize as u32 as i32,
                title.len() as i32,
                subtitle.as_ptr() as usize as u32 as i32,
                subtitle.len() as i32,
                timings.fade_in as i32,
                timings.stay as i32,
                timings.fade_out as i32,
            )
        };
        status_result(status, "host rejected the player title")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, title, subtitle, timings);
        Err(unsupported_target())
    }
}

fn validate_player_title_text(text: &str, allow_blank: bool) -> SjmlResult<()> {
    if (!allow_blank && text.trim().is_empty())
        || text.len() > MAX_PLAYER_TITLE_TEXT_BYTES
        || text.chars().count() > MAX_PLAYER_TITLE_TEXT_CODE_POINTS
    {
        return Err(SjmlError::new(
            if text.len() > MAX_PLAYER_TITLE_TEXT_BYTES
                || text.chars().count() > MAX_PLAYER_TITLE_TEXT_CODE_POINTS
            {
                ErrorCode::PayloadTooLarge
            } else {
                ErrorCode::InvalidArgument
            },
            "player title text is blank or exceeds its bounded text size",
        ));
    }
    if text.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player title text must not contain control characters",
        ));
    }
    Ok(())
}

/// Sets this module's literal header and footer contribution in an online player's tab list.
///
/// Either field may be empty, but both may not be blank; call [`clear_tab_list`] to remove this
/// module's contribution. Text is not parsed as JSON or a translation key and is limited to
/// 1,024 UTF-8 bytes and 256 Unicode scalar values per field, without control characters. Active
/// SJML modules' contributions are composed in deterministic text order, so clearing or closing
/// one module leaves the other modules' text intact. A single module can own one contribution per
/// player. The host caps contributions at 32 modules per player and the composed field at 32 KiB.
///
/// Requires declared and administrator-granted `entity.read` and `player.ui.tab_list`. Only
/// logical-server modules may use this operation. Each module is limited to ten updates per
/// five-second window.
pub fn set_tab_list(handle: PlayerHandle, header: &str, footer: &str) -> SjmlResult<()> {
    validate_player_tab_list_pair(header, footer)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and both UTF-8 slices remain live during the synchronous host call.
        let status = unsafe {
            imported_set_tab_list(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                header.as_ptr() as usize as u32 as i32,
                header.len() as i32,
                footer.as_ptr() as usize as u32 as i32,
                footer.len() as i32,
            )
        };
        status_result(status, "host rejected the player tab-list text")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Removes only this module's header/footer contribution from an online player's tab list.
///
/// Other active SJML modules' contributions are recomposed and remain visible. This operation
/// requires declared and administrator-granted `entity.read` and `player.ui.tab_list` and is
/// available only to logical-server modules.
pub fn clear_tab_list(handle: PlayerHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle remains live through the synchronous import.
        let status = unsafe {
            imported_clear_tab_list(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
            )
        };
        status_result(status, "host rejected clearing the player tab-list text")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

fn validate_player_tab_list_pair(header: &str, footer: &str) -> SjmlResult<()> {
    validate_player_tab_list_text(header)?;
    validate_player_tab_list_text(footer)?;
    if header.trim().is_empty() && footer.trim().is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "set_tab_list requires a nonblank header or footer; use clear_tab_list to remove the contribution",
        ));
    }
    Ok(())
}

fn validate_player_tab_list_text(text: &str) -> SjmlResult<()> {
    if text.len() > MAX_PLAYER_TAB_LIST_TEXT_BYTES
        || text.chars().count() > MAX_PLAYER_TAB_LIST_TEXT_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player tab-list text exceeds 1024 UTF-8 bytes or 256 code points",
        ));
    }
    if text.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player tab-list text must not contain control characters",
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

/// Sets the current air supply for an online player without exceeding its maximum.
///
/// Requires both `entity.read` and `player.air.write`. The requested value must
/// be nonnegative and no greater than [`PlayerAirSupply::max`] from the latest
/// `air_supply` snapshot. The host rechecks the current entity maximum on every
/// call, validates the live login session, and requires the owning logical-server
/// thread. Client modules, stale handles, and static registration are denied.
pub fn set_air_supply(handle: PlayerHandle, air_supply: u32) -> SjmlResult<()> {
    if air_supply > i32::MAX as u32 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player air supply exceeds the supported integer range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and scalar remain valid throughout this synchronous host call.
        let status = unsafe {
            imported_set_air_supply(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                air_supply as i32,
            )
        };
        status_result(status, "host rejected the player air-supply change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Resets fall-distance tracking for an online player to vanilla's zero state.
///
/// Requires `entity.read` and `player.fall_distance.write`, a live player handle,
/// and the owning logical-server thread. The host revalidates the current login
/// session for every call. This operation exposes no arbitrary fall-distance
/// value setter and delegates to Minecraft's `Entity::resetFallDistance`.
pub fn reset_fall_distance(handle: PlayerHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle remains live for the complete synchronous host call.
        let status = unsafe {
            imported_reset_fall_distance(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
            )
        };
        status_result(status, "host rejected the player fall-distance reset")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Returns whether an online player has an active vanilla cooldown for a registered item.
///
/// Requires `entity.read` and a logical-server player handle. Minecraft exposes the active state
/// through its public cooldown API; it does not expose the exact remaining tick count. Item IDs
/// must be canonical non-air `namespace:path` values. The host checks the current login session
/// and owning server thread for every call.
pub fn has_item_cooldown(handle: PlayerHandle, item_id: &str) -> SjmlResult<bool> {
    validate_cooldown_item_id(item_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let mut active = 0u8;
        // SAFETY: handle, item ID, and the one-byte output remain live for the synchronous host call.
        let status = unsafe {
            imported_has_item_cooldown(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                item_id.as_ptr() as usize as u32 as i32,
                item_id.len() as i32,
                &mut active as *mut u8 as usize as u32 as i32,
                1,
            )
        };
        status_result(status, "host rejected the player item-cooldown read")?;
        match active {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid player cooldown flag",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Starts a vanilla cooldown for a registered item on the online player.
///
/// Requires both `entity.read` and `player.cooldown.write`, declared by the module and granted by
/// the server administrator. The duration is limited to 1–72,000 ticks. Client modules, stale
/// sessions, static registration, and calls off the owning server thread are denied.
pub fn set_item_cooldown(handle: PlayerHandle, item_id: &str, ticks: u32) -> SjmlResult<()> {
    validate_cooldown_item_id(item_id)?;
    if !(1..=MAX_PLAYER_ITEM_COOLDOWN_TICKS).contains(&ticks) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player item cooldown must be between 1 and 72000 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: handle and item ID remain live throughout the synchronous bounded import.
        let status = unsafe {
            imported_set_item_cooldown(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                item_id.as_ptr() as usize as u32 as i32,
                item_id.len() as i32,
                ticks as i32,
            )
        };
        status_result(status, "host rejected the player item-cooldown change")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Clears a vanilla cooldown for a registered item on the online player.
///
/// Requires `entity.read` and `player.cooldown.write`. The host applies the change only to the
/// current live server-side login session and synchronizes it using Minecraft's cooldown path.
pub fn clear_item_cooldown(handle: PlayerHandle, item_id: &str) -> SjmlResult<()> {
    validate_cooldown_item_id(item_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: handle and item ID remain live throughout the synchronous bounded import.
        let status = unsafe {
            imported_clear_item_cooldown(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                item_id.as_ptr() as usize as u32 as i32,
                item_id.len() as i32,
            )
        };
        status_result(status, "host rejected the player item-cooldown clear")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads one vanilla 1.21.1 custom statistic from an online server player.
///
/// `statistic` is a typed member of vanilla `Stats.CUSTOM`; registry-backed item, block, entity,
/// or mod-defined statistic types are not accepted. Requires `entity.read` and a live logical-server
/// player session. The result is the stored nonnegative counter value.
pub fn custom_stat(handle: PlayerHandle, statistic: PlayerCustomStat) -> SjmlResult<u32> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let statistic_id = statistic.id();
        let mut value = [0u8; core::mem::size_of::<u32>()];
        // SAFETY: the handle, static statistic ID, and four-byte output all remain valid for this synchronous import.
        let status = unsafe {
            imported_custom_stat(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                statistic_id.as_ptr() as usize as u32 as i32,
                statistic_id.len() as i32,
                value.as_mut_ptr() as usize as u32 as i32,
                value.len() as i32,
            )
        };
        status_result(status, "host rejected the player custom-stat read")?;
        let value = u32::from_le_bytes(value);
        if value > i32::MAX as u32 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a player custom-stat value outside vanilla's integer range",
            ));
        }
        Ok(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, statistic);
        Err(unsupported_target())
    }
}

/// Adds a bounded positive amount to one vanilla 1.21.1 custom statistic.
///
/// Requires both `entity.read` and `player.statistics.write`, declared by the module and granted by
/// the server administrator. The amount must be in `1..=MAX_PLAYER_STATISTIC_INCREMENT`. The host
/// rejects an addition that would exceed Minecraft's signed 32-bit stat counter and validates the
/// registered `Stats.CUSTOM` entry, active login session, and owning server thread.
pub fn increment_custom_stat(
    handle: PlayerHandle,
    statistic: PlayerCustomStat,
    amount: u32,
) -> SjmlResult<()> {
    if !(1..=MAX_PLAYER_STATISTIC_INCREMENT).contains(&amount) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player statistic increment must be between 1 and 1000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let statistic_id = statistic.id();
        // SAFETY: the handle and static statistic ID remain valid throughout the synchronous import.
        let status = unsafe {
            imported_increment_custom_stat(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                statistic_id.as_ptr() as usize as u32 as i32,
                statistic_id.len() as i32,
                amount as i32,
            )
        };
        status_result(status, "host rejected the player custom-stat increment")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, statistic);
        Err(unsupported_target())
    }
}

/// Sets one registered vanilla custom statistic to an exact nonnegative value.
///
/// Requires both `entity.read` and `player.statistics.write`, declared by the module and granted
/// by the server administrator. `value` must fit Minecraft's signed 32-bit counter. The host
/// validates the registered `Stats.CUSTOM` entry, active login session, and owning server thread.
/// Returns the effective value after NeoForge statistic-award listeners run.
pub fn set_custom_stat(
    handle: PlayerHandle,
    statistic: PlayerCustomStat,
    value: u32,
) -> SjmlResult<u32> {
    if value > MAX_PLAYER_STATISTIC_VALUE {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player statistic value must be between 0 and 2147483647",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let statistic_id = statistic.id();
        let applied: u32 = read_output(core::mem::size_of::<u32>() as u32, |output, capacity| {
            // SAFETY: handle, static statistic ID, and output span remain valid for this sync import.
            unsafe {
                imported_set_custom_stat(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    statistic_id.as_ptr() as usize as u32 as i32,
                    statistic_id.len() as i32,
                    value as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if applied > MAX_PLAYER_STATISTIC_VALUE {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a player custom-stat value outside vanilla's integer range",
            ));
        }
        Ok(applied)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, statistic);
        Err(unsupported_target())
    }
}

fn validate_cooldown_item_id(item_id: &str) -> SjmlResult<()> {
    let valid_resource_id = item_id.split_once(':').is_some_and(|(namespace, path)| {
        !namespace.is_empty()
            && !path.is_empty()
            && namespace.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
            })
            && path.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
            })
    });
    if item_id.len() > MAX_PLAYER_COOLDOWN_ITEM_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player cooldown item ID exceeds 256 UTF-8 bytes",
        ));
    }
    if !valid_resource_id || item_id == "minecraft:air" {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player cooldown item ID must be a canonical non-air namespace:path value",
        ));
    }
    Ok(())
}

/// Grants every remaining criterion for a registered Minecraft advancement to an online player.
///
/// Returns `true` when the player's progress changed and `false` when it was already complete.
/// Requires `entity.read` and `player.advancement.write`. Advancement IDs must be canonical,
/// loaded `namespace:path` IDs. An operation is limited to 256 criteria, and granting can invoke
/// vanilla advancement rewards. Only logical-server modules can call this API.
pub fn grant_advancement(handle: PlayerHandle, advancement_id: &str) -> SjmlResult<bool> {
    validate_advancement_id(advancement_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let mut changed = 0u8;
        // SAFETY: input strings and the one-byte output remain live through the synchronous host call.
        let status = unsafe {
            imported_grant_advancement(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                advancement_id.as_ptr() as usize as u32 as i32,
                advancement_id.len() as i32,
                &mut changed as *mut u8 as usize as u32 as i32,
                1,
            )
        };
        status_result(status, "host rejected the player advancement grant")?;
        match changed {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid player advancement result",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Revokes every completed criterion for a registered Minecraft advancement from an online player.
///
/// Returns `true` when progress changed and `false` when it was already incomplete. Requires
/// `entity.read` and `player.advancement.write`; the advancement must be present in the active
/// registry and contain at most 256 criteria. Only logical-server modules can call this API.
pub fn revoke_advancement(handle: PlayerHandle, advancement_id: &str) -> SjmlResult<bool> {
    validate_advancement_id(advancement_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let mut changed = 0u8;
        // SAFETY: input strings and the one-byte output remain live through the synchronous host call.
        let status = unsafe {
            imported_revoke_advancement(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                advancement_id.as_ptr() as usize as u32 as i32,
                advancement_id.len() as i32,
                &mut changed as *mut u8 as usize as u32 as i32,
                1,
            )
        };
        status_result(status, "host rejected the player advancement revocation")?;
        match changed {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid player advancement result",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

fn validate_advancement_id(advancement_id: &str) -> SjmlResult<()> {
    let valid_resource_id = advancement_id
        .split_once(':')
        .is_some_and(|(namespace, path)| {
            !namespace.is_empty()
                && !path.is_empty()
                && namespace.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
                })
                && path.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
                })
        });
    if advancement_id.len() > MAX_PLAYER_ADVANCEMENT_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player advancement ID exceeds 256 UTF-8 bytes",
        ));
    }
    if !valid_resource_id {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player advancement ID must be a canonical namespace:path value",
        ));
    }
    Ok(())
}

fn validate_player_respawn_dimension(dimension_id: &str) -> SjmlResult<()> {
    if dimension_id.len() > MAX_PLAYER_RESPAWN_DIMENSION_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player respawn dimension ID exceeds 256 UTF-8 bytes",
        ));
    }
    let valid_resource_id = dimension_id
        .split_once(':')
        .is_some_and(|(namespace, path)| {
            !namespace.is_empty()
                && !path.is_empty()
                && namespace.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
                })
                && path.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
                })
        });
    if !valid_resource_id {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player respawn dimension must be a canonical namespace:path value",
        ));
    }
    Ok(())
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

/// Reads at most 64 active status effects from the current live server-player session.
///
/// Requires the declared and granted `entity.read` capability and a logical-server context.
/// Every call resolves the player handle against the active login session, so logout and runtime
/// shutdown invalidate access while respawn and dimension changes keep the session handle valid.
/// The detached effects are sorted by canonical registry ID and include the current remaining
/// duration (`-1` for a vanilla infinite-duration effect). Client modules and static registration
/// cannot read live player effects; no player or Minecraft objects cross the ABI.
pub fn active_effects(handle: PlayerHandle) -> SjmlResult<Vec<ActiveEffect>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let response: ActiveEffectsResponse = read_output(
            MAX_PLAYER_ACTIVE_EFFECTS_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the handle and bounded response allocation remain live for the synchronous host read.
                unsafe {
                    imported_active_effects(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        Ok(response.effects)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads the current scoreboard-team name for the live player, if the player is assigned to a team.
///
/// Requires the declared and granted `entity.read` capability and a logical-server context. The
/// detached name is limited to 64 ASCII bytes. The lookup is revalidated against the player's
/// current login session and current server-level scoreboard; no scoreboard or Minecraft object
/// references cross the ABI.
pub fn team(handle: PlayerHandle) -> SjmlResult<Option<String>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let response: PlayerTeamResponse =
            read_output(MAX_PLAYER_TEAM_RESPONSE_BYTES as u32, |output, capacity| {
                // SAFETY: the encoded handle and bounded response allocation remain live for this import.
                unsafe {
                    imported_team(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            })?;
        Ok(response.team_name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Assigns the live player to an existing scoreboard team.
///
/// Requires both `entity.read` and `player.team.write`. The name must identify a team that already
/// exists on the current logical-server scoreboard; this API never creates teams. An existing
/// membership can be replaced, so `player.team.write` grants authority over any team's membership
/// for this live player. Names are limited to 64 ASCII letters, digits, '.', '_', '+', or '-'.
pub fn set_team(handle: PlayerHandle, team_name: &str) -> SjmlResult<()> {
    validate_player_team_name(team_name)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the handle and bounded team-name slices remain valid for this synchronous host call.
        let status = unsafe {
            imported_set_team(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                team_name.as_ptr() as usize as u32 as i32,
                team_name.len() as u32 as i32,
            )
        };
        status_result(
            status,
            "host rejected the player scoreboard-team assignment",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Removes the live player from its current scoreboard team, if assigned.
///
/// Requires both `entity.read` and `player.team.write`. This can remove membership from any
/// existing team, including a vanilla team or a team owned by another module.
pub fn clear_team(handle: PlayerHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the stable handle remains valid for this synchronous host call.
        let status = unsafe {
            imported_clear_team(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
            )
        };
        status_result(
            status,
            "host rejected clearing the player scoreboard-team assignment",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

fn validate_player_team_name(team_name: &str) -> SjmlResult<()> {
    if team_name.is_empty()
        || team_name.len() > MAX_PLAYER_TEAM_NAME_BYTES
        || !team_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'+' | b'-'))
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard team names must be 1–64 ASCII letters, digits, '.', '_', '+', or '-'",
        ));
    }
    Ok(())
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
/// The snapshot itself is read-only. Separate `set_*` functions below change individual fields
/// under `player.abilities.write`. The host validates the active session and logical-server thread
/// for each call; client and static-registration calls fail closed.
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

/// Enables or disables flight permission for a live server player.
///
/// Requires `entity.read` and `player.abilities.write`. Disabling `may_fly` also clears current
/// flight in the same server-thread update. Use [`set_flying`] to change only current flight.
pub fn set_may_fly(handle: PlayerHandle, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        set_player_ability_flag(handle, enabled, imported_set_may_fly)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, enabled);
        Err(unsupported_target())
    }
}

/// Enables or disables instant-build ability for a live server player.
///
/// Requires `entity.read` and `player.abilities.write`.
pub fn set_instabuild(handle: PlayerHandle, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        set_player_ability_flag(handle, enabled, imported_set_instabuild)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, enabled);
        Err(unsupported_target())
    }
}

/// Enables or disables invulnerability for a live server player.
///
/// Requires `entity.read` and `player.abilities.write`.
pub fn set_invulnerable(handle: PlayerHandle, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        set_player_ability_flag(handle, enabled, imported_set_invulnerable)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, enabled);
        Err(unsupported_target())
    }
}

/// Enables or disables the player's permission to build and interact with blocks.
///
/// Requires `entity.read` and `player.abilities.write`.
pub fn set_may_build(handle: PlayerHandle, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        set_player_ability_flag(handle, enabled, imported_set_may_build)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, enabled);
        Err(unsupported_target())
    }
}

/// Sets flight speed to a finite value in [`MIN_PLAYER_ABILITY_SPEED`]..=[`MAX_PLAYER_ABILITY_SPEED`].
///
/// Minecraft's ordinary default is approximately `0.05`. Requires `entity.read` and
/// `player.abilities.write`.
pub fn set_flying_speed(handle: PlayerHandle, speed: f32) -> SjmlResult<()> {
    validate_player_ability_speed(speed)?;
    #[cfg(target_arch = "wasm32")]
    {
        set_player_ability_speed(handle, speed, imported_set_flying_speed)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, speed);
        Err(unsupported_target())
    }
}

/// Sets walking speed to a finite value in [`MIN_PLAYER_ABILITY_SPEED`]..=[`MAX_PLAYER_ABILITY_SPEED`].
///
/// Minecraft's ordinary default is approximately `0.1`. Requires `entity.read` and
/// `player.abilities.write`.
pub fn set_walking_speed(handle: PlayerHandle, speed: f32) -> SjmlResult<()> {
    validate_player_ability_speed(speed)?;
    #[cfg(target_arch = "wasm32")]
    {
        set_player_ability_speed(handle, speed, imported_set_walking_speed)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, speed);
        Err(unsupported_target())
    }
}

fn validate_player_ability_speed(speed: f32) -> SjmlResult<()> {
    if !speed.is_finite() || !(MIN_PLAYER_ABILITY_SPEED..=MAX_PLAYER_ABILITY_SPEED).contains(&speed)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player ability speed must be finite and between 0 and 0.5",
        ));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn set_player_ability_flag(
    handle: PlayerHandle,
    enabled: bool,
    imported_setter: unsafe extern "C" fn(i32, i32, i32) -> i32,
) -> SjmlResult<()> {
    let encoded = to_bytes(&handle)?;
    // SAFETY: the encoded handle and boolean scalar stay valid through this synchronous host import.
    let status = unsafe {
        imported_setter(
            encoded.as_ptr() as usize as u32 as i32,
            encoded.len() as i32,
            i32::from(enabled),
        )
    };
    status_result(status, "host rejected the player ability change")
}

#[cfg(target_arch = "wasm32")]
fn set_player_ability_speed(
    handle: PlayerHandle,
    speed: f32,
    imported_setter: unsafe extern "C" fn(i32, i32, i32) -> i32,
) -> SjmlResult<()> {
    let encoded = to_bytes(&handle)?;
    // SAFETY: the encoded handle and float bits stay valid through this synchronous host import.
    let status = unsafe {
        imported_setter(
            encoded.as_ptr() as usize as u32 as i32,
            encoded.len() as i32,
            speed.to_bits() as i32,
        )
    };
    status_result(status, "host rejected the player ability speed change")
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

/// Sets a live server player's yaw and pitch without changing position or velocity.
///
/// Requires `entity.read` and `player.rotation.write`. Yaw must be finite and within
/// ±36,000 degrees; pitch must be finite and within -90..=90 degrees. The server
/// synchronizes the authoritative rotation to the player and tracking clients.
pub fn set_rotation(handle: PlayerHandle, yaw: f32, pitch: f32) -> SjmlResult<()> {
    if !yaw.is_finite() || yaw.abs() > MAX_PLAYER_ROTATION_YAW {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player yaw must be finite and within ±36,000 degrees",
        ));
    }
    if !pitch.is_finite() || !(-90.0..=90.0).contains(&pitch) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player pitch must be finite and within -90..=90 degrees",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        // SAFETY: the handle is a bounded stable value and both angles are validated above.
        let status = unsafe {
            imported_set_rotation(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                yaw.to_bits() as i32,
                pitch.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the player rotation mutation")
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
        active_effects, air_supply, apply_effect, clear_team, movement_state, reset_fall_distance,
        send_system_message, set_air_supply, set_rotation, set_team, team, teleport,
        teleport_to_world, validate_player_ability_speed, validate_player_tab_list_pair,
        validate_player_tab_list_text, validate_player_team_name, validate_player_title_text,
        validate_system_message_text, vitals, PlayerAirSupply, PlayerMotion, PlayerMovementState,
        PlayerTeamResponse, PlayerTeleport, PlayerTitleTimings, PlayerVitals,
        MAX_PLAYER_ABILITY_SPEED, MAX_PLAYER_ACTIVE_EFFECTS,
        MAX_PLAYER_ACTIVE_EFFECTS_RESPONSE_BYTES, MAX_PLAYER_FALL_DISTANCE,
        MAX_PLAYER_ROTATION_YAW, MAX_PLAYER_STATISTIC_VALUE, MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES,
        MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS, MAX_PLAYER_TAB_LIST_TEXT_BYTES,
        MAX_PLAYER_TAB_LIST_TEXT_CODE_POINTS, MAX_PLAYER_TEAM_NAME_BYTES,
        MAX_PLAYER_TEAM_RESPONSE_BYTES, MAX_PLAYER_TITLE_TEXT_BYTES, MAX_TELEPORT_COORDINATE,
        MIN_PLAYER_ABILITY_SPEED,
    };
    use crate::{ActiveEffect, ErrorCode, PlayerHandle, StableDecode, WorldHandle};

    #[test]
    fn player_ability_speeds_are_finite_and_bounded() {
        assert_eq!(MIN_PLAYER_ABILITY_SPEED, 0.0);
        assert_eq!(MAX_PLAYER_ABILITY_SPEED, 0.5);
        assert!(validate_player_ability_speed(MIN_PLAYER_ABILITY_SPEED).is_ok());
        assert!(validate_player_ability_speed(0.05).is_ok());
        assert!(validate_player_ability_speed(MAX_PLAYER_ABILITY_SPEED).is_ok());
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.001, 0.501] {
            assert_eq!(
                validate_player_ability_speed(invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument,
            );
        }
    }

    #[test]
    fn player_active_effects_reuse_the_bounded_entity_effect_schema() {
        assert_eq!(MAX_PLAYER_ACTIVE_EFFECTS, 64);
        assert_eq!(MAX_PLAYER_ACTIVE_EFFECTS_RESPONSE_BYTES, 17_416);
        let effects = vec![
            ActiveEffect {
                effect_id: "minecraft:speed".to_owned(),
                amplifier: 2,
                remaining_duration_ticks: 1_234,
            },
            ActiveEffect {
                effect_id: "minecraft:strength".to_owned(),
                amplifier: 0,
                remaining_duration_ticks: -1,
            },
        ];
        let response = crate::entity::ActiveEffectsResponse {
            effects: effects.clone(),
        };
        let encoded = crate::to_bytes(&response).unwrap();
        assert_eq!(
            crate::from_bytes::<crate::entity::ActiveEffectsResponse>(&encoded)
                .unwrap()
                .effects,
            effects,
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            active_effects(PlayerHandle::new(1, 2, 3, 4).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn player_team_membership_uses_a_bounded_optional_schema_and_name_contract() {
        assert_eq!(MAX_PLAYER_TEAM_NAME_BYTES, 64);
        assert_eq!(MAX_PLAYER_TEAM_RESPONSE_BYTES, 73);

        let no_team = crate::to_bytes(&PlayerTeamResponse { team_name: None }).unwrap();
        assert_eq!(no_team, [1, 0, 0, 0, 0]);
        assert_eq!(
            crate::from_bytes::<PlayerTeamResponse>(&no_team)
                .unwrap()
                .team_name,
            None
        );

        let some_team = crate::to_bytes(&PlayerTeamResponse {
            team_name: Some("red-team".to_owned()),
        })
        .unwrap();
        assert_eq!(
            &some_team,
            &[1, 0, 0, 0, 1, 8, 0, 0, 0, b'r', b'e', b'd', b'-', b't', b'e', b'a', b'm']
        );
        assert_eq!(
            crate::from_bytes::<PlayerTeamResponse>(&some_team)
                .unwrap()
                .team_name
                .as_deref(),
            Some("red-team")
        );

        let mut padded = some_team.clone();
        padded.resize(MAX_PLAYER_TEAM_RESPONSE_BYTES, 0);
        assert_eq!(
            PlayerTeamResponse::decode(&mut crate::Decoder::new(&padded).unwrap())
                .unwrap()
                .team_name
                .as_deref(),
            Some("red-team")
        );
        let mut malformed = padded;
        *malformed.last_mut().unwrap() = 1;
        assert_eq!(
            PlayerTeamResponse::decode(&mut crate::Decoder::new(&malformed).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        for invalid in ["", "has space", "../team", "red\nteammates", "команда"] {
            assert_eq!(
                validate_player_team_name(invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "invalid team name {invalid:?} should be rejected"
            );
        }
        validate_player_team_name("sJML-team.1+ok").unwrap();
        assert_eq!(
            validate_player_team_name(&"a".repeat(MAX_PLAYER_TEAM_NAME_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        {
            assert_eq!(
                team(handle).unwrap_err().code(),
                ErrorCode::UnknownErrorCode
            );
            assert_eq!(
                set_team(handle, "red-team").unwrap_err().code(),
                ErrorCode::UnknownErrorCode
            );
            assert_eq!(
                clear_team(handle).unwrap_err().code(),
                ErrorCode::UnknownErrorCode
            );
        }
    }

    fn motion_with_rotation(yaw: f32, pitch: f32) -> PlayerMotion {
        PlayerMotion {
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
            yaw,
            pitch,
        }
    }

    fn assert_direction_close(actual: [f64; 3], expected: [f64; 3]) {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 1e-12,
                "direction component {actual} did not match expected {expected}"
            );
        }
    }

    #[test]
    fn player_motion_look_direction_uses_minecraft_cardinal_yaw() {
        for (yaw, expected) in [
            (0.0, [0.0, 0.0, 1.0]),
            (90.0, [-1.0, 0.0, 0.0]),
            (180.0, [0.0, 0.0, -1.0]),
            (-90.0, [1.0, 0.0, 0.0]),
        ] {
            assert_direction_close(motion_with_rotation(yaw, 0.0).look_direction(), expected);
        }
    }

    #[test]
    fn player_rotation_setter_validates_yaw_and_pitch_before_host_access() {
        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(MAX_PLAYER_ROTATION_YAW, 36_000.0);
        for (yaw, pitch) in [
            (f32::NAN, 0.0),
            (f32::INFINITY, 0.0),
            (MAX_PLAYER_ROTATION_YAW + 0.1, 0.0),
            (0.0, f32::NEG_INFINITY),
            (0.0, -90.1),
            (0.0, 90.1),
        ] {
            assert_eq!(
                set_rotation(handle, yaw, pitch).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "yaw={yaw}, pitch={pitch} should be rejected"
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            set_rotation(handle, 270.0, -90.0).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid angles should reach the non-WASM unsupported-target path"
        );
    }

    #[test]
    fn player_motion_look_direction_handles_vertical_pitch_and_is_normalized() {
        assert_direction_close(
            motion_with_rotation(37.0, 90.0).look_direction(),
            [0.0, -1.0, 0.0],
        );
        assert_direction_close(
            motion_with_rotation(-121.0, -90.0).look_direction(),
            [0.0, 1.0, 0.0],
        );

        let diagonal = motion_with_rotation(45.0, 30.0).look_direction();
        assert_direction_close(
            diagonal,
            [
                -3.0_f64.sqrt() / 2.0_f64.sqrt() / 2.0,
                -0.5,
                3.0_f64.sqrt() / 2.0_f64.sqrt() / 2.0,
            ],
        );
        let length = diagonal
            .iter()
            .map(|component| component * component)
            .sum::<f64>()
            .sqrt();
        assert!(
            (length - 1.0).abs() < 1e-12,
            "look direction length was {length}"
        );
    }

    #[test]
    fn player_title_timings_and_literal_text_are_bounded() {
        assert_eq!(PlayerTitleTimings::new(10, 70, 20).unwrap().stay, 70);
        assert_eq!(
            PlayerTitleTimings::new(0, 6_001, 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_player_title_text("", false).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert!(validate_player_title_text("", true).is_ok());
        assert_eq!(
            validate_player_title_text("line\nfeed", true)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_player_title_text(&"x".repeat(MAX_PLAYER_TITLE_TEXT_BYTES + 1), false)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn system_chat_text_is_literal_control_free_and_unicode_bounded() {
        assert_eq!(MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES, 1_024);
        assert_eq!(MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS, 256);
        assert!(validate_system_message_text("Привет ✨ {\"text\":\"literal\"}").is_ok());
        assert!(validate_system_message_text(&"😀".repeat(256)).is_ok());
        for invalid in ["", " \t", "line\nfeed", "zero\0width"] {
            assert_eq!(
                validate_system_message_text(invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            validate_system_message_text(&"x".repeat(MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_system_message_text(
                &"😀".repeat(MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS + 1)
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );

        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            send_system_message(handle, "line\nfeed")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid text is rejected before a host call"
        );
        assert_eq!(
            send_system_message(handle, "valid").unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid text reaches the target-specific host import"
        );
    }

    #[test]
    fn rich_system_message_builder_encodes_typed_nodes_and_enforces_bounds() {
        use super::{
            RichText, RichTextClickAction, RichTextColor, RichTextNode,
            MAX_PLAYER_RICH_SYSTEM_MESSAGE_DEPTH, MAX_PLAYER_RICH_SYSTEM_MESSAGE_NODES,
            MAX_PLAYER_RICH_SYSTEM_MESSAGE_PAYLOAD_BYTES,
            MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES,
        };

        let hover = RichText::plain("Tooltip").unwrap();
        let child = RichTextNode::new(" child")
            .unwrap()
            .color(RichTextColor::rgb(0xC0D0E0).unwrap())
            .unwrap();
        let root = RichTextNode::new("Open")
            .unwrap()
            .color(RichTextColor::Gold)
            .unwrap()
            .bold(true)
            .click(RichTextClickAction::open_url("https://example.com/path").unwrap())
            .hover_text(hover)
            .unwrap()
            .child(child)
            .unwrap();
        let mut message = RichText::new();
        message.push(root).unwrap();
        let encoded = crate::to_bytes(&message).unwrap();
        assert!(encoded.len() <= MAX_PLAYER_RICH_SYSTEM_MESSAGE_PAYLOAD_BYTES);
        assert_eq!(
            &encoded[..8],
            &[1, 0, 0, 0, 1, 0, 0, 0],
            "schema v1 is followed by one top-level node"
        );
        assert!(encoded
            .windows(b"https://example.com/path".len())
            .any(|window| window == b"https://example.com/path"));

        for invalid in [
            "javascript:alert(1)",
            "https://user@example.com",
            "https://example.com\n",
        ] {
            assert_eq!(
                RichTextClickAction::open_url(invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            RichTextClickAction::suggest_command("\n")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            RichTextClickAction::copy_to_clipboard("")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let invalid_click =
            RichTextNode::new("unsafe")
                .unwrap()
                .click(RichTextClickAction::OpenUrl(
                    "javascript:alert(1)".to_owned(),
                ));
        let mut invalid_message = RichText::new();
        assert_eq!(
            invalid_message.push(invalid_click).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            RichTextNode::new("x".repeat(MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge,
            "individual strings have an independent 1,024-byte cap",
        );
        let mut aggregate = RichText::new();
        for _ in 0..4 {
            aggregate
                .push(RichTextNode::new("x".repeat(1_024)).unwrap())
                .unwrap();
        }
        assert_eq!(
            aggregate
                .push(RichTextNode::new("overflow").unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge,
            "all literal, hover, and click strings share a 4,096-byte budget",
        );

        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            super::send_rich_system_message(handle, &RichText::new())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
        );
        assert_eq!(
            super::send_rich_system_message(handle, &message)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "a valid typed tree reaches the target-specific host import",
        );

        let mut deep = RichTextNode::new("leaf").unwrap();
        for _ in 1..MAX_PLAYER_RICH_SYSTEM_MESSAGE_DEPTH {
            deep = RichTextNode::new("n").unwrap().child(deep).unwrap();
        }
        assert!(RichTextNode::new("too deep").unwrap().child(deep).is_err());

        let mut many = RichText::new();
        for _ in 0..MAX_PLAYER_RICH_SYSTEM_MESSAGE_NODES {
            many.push(RichTextNode::new("x").unwrap()).unwrap();
        }
        assert_eq!(
            many.push(RichTextNode::new("overflow").unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge,
        );
    }

    #[test]
    fn custom_stat_setter_rejects_values_outside_vanillas_signed_counter_range() {
        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(MAX_PLAYER_STATISTIC_VALUE, i32::MAX as u32);
        assert_eq!(
            super::set_custom_stat(
                handle,
                super::PlayerCustomStat::Deaths,
                MAX_PLAYER_STATISTIC_VALUE + 1
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            super::set_custom_stat(handle, super::PlayerCustomStat::Deaths, 0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn player_tab_list_text_is_literal_bounded_and_can_clear_one_side() {
        assert!(validate_player_tab_list_pair("Header", "").is_ok());
        assert!(validate_player_tab_list_pair("", "Footer").is_ok());
        assert!(validate_player_tab_list_text("").is_ok());
        assert_eq!(
            validate_player_tab_list_pair("", "").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_player_tab_list_text("header\nfooter")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_player_tab_list_text(&"x".repeat(MAX_PLAYER_TAB_LIST_TEXT_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_player_tab_list_text(&"😀".repeat(MAX_PLAYER_TAB_LIST_TEXT_CODE_POINTS + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert!(
            validate_player_tab_list_text(&"😀".repeat(MAX_PLAYER_TAB_LIST_TEXT_CODE_POINTS))
                .is_ok()
        );
    }

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
    fn air_supply_snapshot_is_detached_and_setter_rejects_unrepresentable_values() {
        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        let air = PlayerAirSupply {
            current: 270,
            max: 300,
        };
        air.validate().unwrap();
        assert_eq!(
            crate::to_bytes(&air).unwrap(),
            [1, 0, 0, 0, 14, 1, 0, 0, 44, 1, 0, 0]
        );
        assert!(PlayerAirSupply {
            current: -20,
            max: 300
        }
        .validate()
        .is_ok());
        assert_eq!(
            PlayerAirSupply {
                current: -21,
                max: 300
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            PlayerAirSupply {
                current: 301,
                max: 300
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_air_supply(handle, i32::MAX as u32 + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_air_supply(handle, 300).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid values reach the wasm-only host import"
        );
        assert_eq!(
            air_supply(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "live player reads require the wasm-only host import"
        );
    }

    #[test]
    fn movement_state_schema_and_boundaries_are_stable_and_read_only() {
        let state = PlayerMovementState {
            on_ground: true,
            sprinting: false,
            crouching: true,
            swimming: false,
            fall_distance: 1.5,
        };
        state.validate().unwrap();
        assert_eq!(
            crate::to_bytes(&state).unwrap(),
            [1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 192, 63]
        );
        assert_eq!(MAX_PLAYER_FALL_DISTANCE, 1_000_000.0);
        for invalid in [
            f32::NAN,
            f32::INFINITY,
            -0.01,
            MAX_PLAYER_FALL_DISTANCE + 1.0,
        ] {
            assert_eq!(
                PlayerMovementState {
                    fall_distance: invalid,
                    ..state
                }
                .validate()
                .unwrap_err()
                .code(),
                ErrorCode::InvalidArgument
            );
        }

        let malformed = [1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(
            super::Decoder::new(&malformed)
                .and_then(|mut decoder| PlayerMovementState::decode(&mut decoder))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean,
            "schema booleans accept only canonical zero and one bytes"
        );

        let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            movement_state(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "native targets fail closed when a live player import is unavailable"
        );
    }

    #[test]
    fn fall_distance_reset_fails_closed_without_the_server_host() {
        assert_eq!(
            reset_fall_distance(PlayerHandle::new(1, 2, 3, 4).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
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
