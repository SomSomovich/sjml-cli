//! Reads of loaded non-player entities and capability-gated actions on the owning logical server.
//! This global API takes no world handle. Calls are synchronous on the server
//! thread. Handles are module/context and live-entity-instance scoped and expire when their entity
//! leaves its level, its world unloads, or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::BlockPos;
use crate::world::RaycastPoint;
#[cfg(target_arch = "wasm32")]
use crate::{to_bytes, MAX_STRING_BYTES};
use crate::{
    Decoder, EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode,
    WorldHandle,
};

/// Maximum number of active status effects returned for one entity.
pub const MAX_ACTIVE_ENTITY_EFFECTS: usize = 64;
/// Maximum UTF-8 byte length of an active effect's canonical ID.
pub const MAX_ENTITY_EFFECT_ID_BYTES: usize = 256;
/// Maximum response bytes for the bounded versioned active-effect snapshot.
pub const MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES: usize =
    8 + MAX_ACTIVE_ENTITY_EFFECTS * (4 + 4 + MAX_ENTITY_EFFECT_ID_BYTES + 4 + 4);
const MAX_EFFECT_ID_BYTES: usize = MAX_ENTITY_EFFECT_ID_BYTES;
pub(crate) const MAX_EFFECT_DURATION_TICKS: u32 = 72_000;
/// Maximum absolute entity velocity component in blocks per tick.
pub const MAX_ENTITY_VELOCITY_COMPONENT: f64 = 64.0;
/// Maximum absolute yaw accepted for a loaded non-player entity, in degrees.
pub const MAX_ENTITY_YAW_DEGREES: f32 = 36_000.0;
/// Maximum absolute pitch accepted for a loaded non-player entity, in degrees.
pub const MAX_ENTITY_PITCH_DEGREES: f32 = 90.0;
/// Capability required with `entity.read` to change a loaded non-player entity's rotation.
pub const ROTATION_WRITE_CAPABILITY: &str = "entity.rotation.write";
/// Maximum damage amount accepted by one [`damage`] call.
pub const MAX_ENTITY_DAMAGE: f32 = 64.0;
/// Maximum positive heal amount accepted by one [`heal`] call.
pub const MAX_ENTITY_HEAL_AMOUNT: f32 = 64.0;
/// Current health, maximum health, or absorption value returned in one bounded snapshot.
pub const MAX_ENTITY_VITALS_VALUE: f32 = 1_000_000.0;
/// Lower bound for current air supply during vanilla's drowning damage window.
pub const MIN_ENTITY_AIR_SUPPLY: i32 = -20;
/// Maximum current or maximum air supply accepted by the detached entity API.
pub const MAX_ENTITY_AIR_SUPPLY: u32 = 1_000_000;
/// Exact schema-v1 response size for current and maximum entity air supply.
pub const MAX_ENTITY_AIR_SUPPLY_RESPONSE_BYTES: u32 = 12;
/// Exact schema-v1 response size for seven detached entity state flags.
pub const MAX_ENTITY_STATE_RESPONSE_BYTES: u32 = 11;
/// Exact schema-v1 response size for six detached entity bounding-box coordinates.
pub const MAX_ENTITY_BOUNDS_RESPONSE_BYTES: u32 = 52;
/// Maximum absolute coordinate accepted in detached world-space entity bounds.
pub const MAX_ENTITY_BOUNDS_COORDINATE: f64 = 30_000_064.0;
/// Capability required with `entity.read` to request a bounded heal for a live entity.
pub const HEALTH_WRITE_CAPABILITY: &str = "entity.health.write";
/// Maximum UTF-8 byte length of a canonical item registry ID in an entity equipment snapshot.
pub const MAX_ENTITY_EQUIPMENT_ITEM_ID_BYTES: usize = 256;
/// Maximum item count accepted in one detached entity equipment slot.
pub const MAX_ENTITY_EQUIPMENT_STACK_COUNT: u32 = 4096;
/// Maximum damage or maximum-damage value in a detached entity equipment slot.
pub const MAX_ENTITY_EQUIPMENT_DAMAGE: u32 = 1_000_000;
/// Maximum response bytes reserved for the six fixed slots in schema v1.
pub const MAX_ENTITY_EQUIPMENT_RESPONSE_BYTES: u32 = 1666;
/// Capability required with `entity.read` to add a default item to an empty entity equipment slot.
pub const EQUIPMENT_WRITE_CAPABILITY: &str = "entity.equipment.write";
/// Capability required with `entity.read` to change a loaded entity's glowing flag.
pub const GLOWING_WRITE_CAPABILITY: &str = "entity.glowing.write";
/// Capability required with `entity.read` to change a loaded entity's silent flag.
pub const STATE_WRITE_CAPABILITY: &str = "entity.state.write";
/// Capability required with `entity.read` to toggle a loaded entity's invulnerability flag.
pub const INVULNERABILITY_WRITE_CAPABILITY: &str = "entity.invulnerability.write";
/// Capability required with `entity.read` to change a loaded living entity's current air supply.
pub const AIR_SUPPLY_WRITE_CAPABILITY: &str = "entity.air.write";
/// Maximum active fire duration that an entity fire-tick API call accepts or returns.
pub const MAX_ENTITY_FIRE_TICKS: u32 = 72_000;
/// Capability required, together with `entity.read`, to read or control a loaded Mob's AI state.
pub const AI_CONTROL_CAPABILITY: &str = "entity.ai.control";
/// Capability required with `entity.read` to control a loaded Mob's short-range path navigation.
pub const NAVIGATION_WRITE_CAPABILITY: &str = "entity.navigation.write";
/// Maximum target distance accepted by [`navigate_to`], in blocks.
pub const MAX_ENTITY_NAVIGATION_RANGE: f64 = 32.0;
/// Maximum pathfinding speed multiplier accepted by [`navigate_to`].
pub const MAX_ENTITY_NAVIGATION_SPEED: f32 = 4.0;
/// Maximum UTF-8 byte length for a plain entity custom name.
pub const MAX_ENTITY_CUSTOM_NAME_BYTES: usize = 1024;
/// Maximum Unicode scalar count for a plain entity custom name.
pub const MAX_ENTITY_CUSTOM_NAME_CODE_POINTS: usize = 256;
/// Maximum encoded response size for an optional entity custom name.
pub const MAX_ENTITY_CUSTOM_NAME_RESPONSE_BYTES: usize = 4 + 1 + 4 + MAX_ENTITY_CUSTOM_NAME_BYTES;
/// Maximum number of loaded entities reported in one vehicle passenger snapshot.
pub const MAX_ENTITY_PASSENGERS: usize = 16;
/// Maximum UTF-8 byte length of an entity relationship type ID.
pub const MAX_ENTITY_RIDING_TYPE_ID_BYTES: usize = 256;
const MAX_ENTITY_RIDING_RELATIONSHIP_BYTES: usize =
    4 + 4 + 36 + 4 + MAX_ENTITY_RIDING_TYPE_ID_BYTES + 1;
/// Maximum response bytes for one bounded entity riding snapshot.
pub const MAX_ENTITY_RIDING_SNAPSHOT_RESPONSE_BYTES: usize = 4
    + 1
    + MAX_ENTITY_RIDING_RELATIONSHIP_BYTES
    + 4
    + MAX_ENTITY_PASSENGERS * MAX_ENTITY_RIDING_RELATIONSHIP_BYTES;
/// Capability required with `entity.read` to start or stop a non-player entity riding.
pub const MOUNT_WRITE_CAPABILITY: &str = "entity.mount.write";
/// Capability required with `entity.read` to set or clear a loaded Mob's target.
pub const TARGET_WRITE_CAPABILITY: &str = "entity.target.write";
/// Capability required with `entity.read` to teleport a loaded, unmounted non-player entity.
pub const TELEPORT_CAPABILITY: &str = "entity.teleport";
/// Maximum absolute coordinate accepted for an entity teleport request.
pub const MAX_ENTITY_TELEPORT_COORDINATE: f64 = 30_000_000.0;
/// Maximum response bytes for the versioned optional loaded-Mob target handle.
pub const MAX_ENTITY_TARGET_RESPONSE_BYTES: usize = 4 + 1 + 24;
/// Maximum absolute distance accepted when assigning a loaded Mob target, in blocks.
pub const MAX_ENTITY_TARGET_RANGE: f64 = 128.0;
/// Maximum UTF-8 byte length of a canonical UUID in a detached target snapshot.
pub const MAX_ENTITY_TARGET_UUID_BYTES: usize = 36;
/// Maximum UTF-8 byte length of a canonical target or dimension ID.
pub const MAX_ENTITY_TARGET_ID_BYTES: usize = 256;
/// Maximum absolute coordinate in a detached entity target snapshot.
pub const MAX_ENTITY_TARGET_COORDINATE: f64 = 30_000_000.0;
/// Maximum output bytes for one optional detached loaded-Mob target snapshot.
pub const MAX_ENTITY_TARGET_SNAPSHOT_RESPONSE_BYTES: usize = 4
    + 1
    + 4
    + (4 + MAX_ENTITY_TARGET_UUID_BYTES)
    + 2 * (4 + MAX_ENTITY_TARGET_ID_BYTES)
    + 3 * core::mem::size_of::<f64>();

/// Current identity, type, dimension, and coordinates of one loaded non-player entity.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntitySnapshot {
    /// Canonical lowercase UUID string.
    pub uuid: String,
    /// Canonical namespace:path registry ID for the entity type.
    pub entity_type_id: String,
    /// Canonical namespace:path dimension ID.
    pub dimension_id: String,
    /// Current X coordinate.
    pub x: f64,
    /// Current Y coordinate.
    pub y: f64,
    /// Current Z coordinate.
    pub z: f64,
}

/// Detached world-space axis-aligned bounds for one live loaded non-player entity.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode)]
#[sjml(version = 1)]
pub struct EntityBounds {
    /// Minimum X coordinate of the entity's bounding box.
    pub min_x: f64,
    /// Minimum Y coordinate of the entity's bounding box.
    pub min_y: f64,
    /// Minimum Z coordinate of the entity's bounding box.
    pub min_z: f64,
    /// Maximum X coordinate of the entity's bounding box.
    pub max_x: f64,
    /// Maximum Y coordinate of the entity's bounding box.
    pub max_y: f64,
    /// Maximum Z coordinate of the entity's bounding box.
    pub max_z: f64,
}

impl EntityBounds {
    /// Creates bounds after checking finite world-space coordinates and axis ordering.
    pub fn new(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> SjmlResult<Self> {
        let bounds = Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        };
        bounds.validate()?;
        Ok(bounds)
    }

    /// Validates finite bounded coordinates and `min <= max` on every axis.
    pub fn validate(self) -> SjmlResult<()> {
        if !bounded_bounds_coordinate(self.min_x)
            || !bounded_bounds_coordinate(self.min_y)
            || !bounded_bounds_coordinate(self.min_z)
            || !bounded_bounds_coordinate(self.max_x)
            || !bounded_bounds_coordinate(self.max_y)
            || !bounded_bounds_coordinate(self.max_z)
            || self.min_x > self.max_x
            || self.min_y > self.max_y
            || self.min_z > self.max_z
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity bounds must be finite, within ±30,000,064 blocks, and ordered min-to-max",
            ));
        }
        Ok(())
    }
}

impl StableDecode for EntityBounds {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported entity bounds schema",
            ));
        }
        Self::new(
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
            decoder.read_f64()?,
        )
    }
}

fn bounded_bounds_coordinate(value: f64) -> bool {
    value.is_finite() && value.abs() <= MAX_ENTITY_BOUNDS_COORDINATE
}

/// Detached bounded identity and position of a loaded non-player living Mob target.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityTargetSnapshot {
    /// Canonical lowercase UUID string.
    pub uuid: String,
    /// Canonical namespace:path entity type ID.
    pub entity_type_id: String,
    /// Canonical namespace:path dimension ID.
    pub dimension_id: String,
    /// Current X coordinate in blocks.
    pub x: f64,
    /// Current Y coordinate in blocks.
    pub y: f64,
    /// Current Z coordinate in blocks.
    pub z: f64,
}

impl EntityTargetSnapshot {
    /// Creates a detached target snapshot after validating its bounded identity and coordinates.
    pub fn new(
        uuid: impl Into<String>,
        entity_type_id: impl Into<String>,
        dimension_id: impl Into<String>,
        x: f64,
        y: f64,
        z: f64,
    ) -> SjmlResult<Self> {
        let snapshot = Self {
            uuid: uuid.into(),
            entity_type_id: entity_type_id.into(),
            dimension_id: dimension_id.into(),
            x,
            y,
            z,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn validate(&self) -> SjmlResult<()> {
        if !canonical_entity_uuid(&self.uuid)
            || self.uuid.len() > MAX_ENTITY_TARGET_UUID_BYTES
            || self.entity_type_id.len() > MAX_ENTITY_TARGET_ID_BYTES
            || self.dimension_id.len() > MAX_ENTITY_TARGET_ID_BYTES
            || validate_entity_type_id(&self.entity_type_id).is_err()
            || validate_entity_type_id(&self.dimension_id).is_err()
            || !bounded_target_coordinate(self.x)
            || !bounded_target_coordinate(self.y)
            || !bounded_target_coordinate(self.z)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity target snapshot contains invalid or unbounded identity or coordinates",
            ));
        }
        Ok(())
    }
}

/// Detached velocity of one loaded living non-player entity, in blocks per tick.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityVelocity {
    /// X velocity component.
    pub x: f64,
    /// Y velocity component.
    pub y: f64,
    /// Z velocity component.
    pub z: f64,
}

/// Detached yaw and pitch of one loaded non-player entity, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityRotation {
    /// Yaw around the vertical axis, bounded to ±36,000 degrees.
    pub yaw: f32,
    /// Pitch around the horizontal axis, bounded to -90..=90 degrees.
    pub pitch: f32,
}

/// Detached health, maximum health, and absorption values for one living non-player entity.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityVitals {
    /// Current health, between zero and `max_health`.
    pub health: f32,
    /// Current effective maximum health, finite and within the host response bound.
    pub max_health: f32,
    /// Current absorption health, independently bounded by the host response limit.
    pub absorption: f32,
}

/// Detached current and maximum air supply for one loaded living non-player entity.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityAirSupply {
    /// Current air ticks. Vanilla may report a value down to -20 during drowning damage.
    pub current: i32,
    /// Current entity-specific maximum air supply.
    pub max: u32,
}

/// Detached movement and visibility flags for one loaded non-player entity.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityStateSnapshot {
    /// Whether Minecraft currently considers the entity grounded.
    pub on_ground: bool,
    /// Whether the entity is currently in water.
    pub in_water: bool,
    /// Whether the entity is currently burning.
    pub on_fire: bool,
    /// Whether the entity is sprinting.
    pub sprinting: bool,
    /// Whether the entity is crouching.
    pub crouching: bool,
    /// Whether the entity is invisible.
    pub invisible: bool,
    /// Whether the entity is currently glowing.
    pub glowing: bool,
}

/// Detached item identity, count, and durability for one living entity equipment slot.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityEquipmentStack {
    /// Canonical lowercase namespace:path item registry ID.
    pub item_id: String,
    /// Current count in this equipment slot.
    pub count: u32,
    /// Current durability damage; zero for items without durability.
    pub damage: u32,
    /// Maximum durability damage; zero for items without durability.
    pub max_damage: u32,
}

impl EntityEquipmentStack {
    /// Constructs a detached equipment value after validating its ID, count, and durability.
    pub fn new(
        item_id: impl Into<String>,
        count: u32,
        damage: u32,
        max_damage: u32,
    ) -> SjmlResult<Self> {
        let stack = Self {
            item_id: item_id.into(),
            count,
            damage,
            max_damage,
        };
        stack.validate()?;
        Ok(stack)
    }

    fn validate(&self) -> SjmlResult<()> {
        let Some((namespace, path)) = self.item_id.split_once(':') else {
            return Err(invalid_entity_equipment(
                "item ID must be a canonical namespace:path value",
            ));
        };
        if self.item_id.len() > MAX_ENTITY_EQUIPMENT_ITEM_ID_BYTES
            || namespace.is_empty()
            || path.is_empty()
            || !namespace.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
            })
            || !path.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
            })
        {
            return Err(invalid_entity_equipment(
                "item ID must be a canonical namespace:path value",
            ));
        }
        if self.count == 0
            || self.count > MAX_ENTITY_EQUIPMENT_STACK_COUNT
            || self.damage > MAX_ENTITY_EQUIPMENT_DAMAGE
            || self.max_damage > MAX_ENTITY_EQUIPMENT_DAMAGE
            || self.damage > self.max_damage
        {
            return Err(invalid_entity_equipment(
                "equipment count or durability is outside the supported range",
            ));
        }
        Ok(())
    }
}

/// Detached six-slot equipment snapshot for one loaded living non-player entity.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityEquipment {
    /// Main-hand item.
    pub main_hand: Option<EntityEquipmentStack>,
    /// Offhand item.
    pub off_hand: Option<EntityEquipmentStack>,
    /// Head armor slot.
    pub head: Option<EntityEquipmentStack>,
    /// Chest armor slot.
    pub chest: Option<EntityEquipmentStack>,
    /// Leg armor slot.
    pub legs: Option<EntityEquipmentStack>,
    /// Feet armor slot.
    pub feet: Option<EntityEquipmentStack>,
}

/// One fixed vanilla equipment slot accepted by [`try_equip_default`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EntityEquipmentSlot {
    /// Main-hand item.
    MainHand,
    /// Offhand item.
    OffHand,
    /// Head armor slot.
    Head,
    /// Chest armor slot.
    Chest,
    /// Leg armor slot.
    Legs,
    /// Feet armor slot.
    Feet,
}

impl EntityEquipmentSlot {
    #[cfg(target_arch = "wasm32")]
    const fn code(self) -> u32 {
        match self {
            Self::MainHand => 1,
            Self::OffHand => 2,
            Self::Head => 3,
            Self::Chest => 4,
            Self::Legs => 5,
            Self::Feet => 6,
        }
    }
}

#[derive(StableDecode)]
#[sjml(version = 1)]
#[cfg(target_arch = "wasm32")]
struct EntityEquipmentWriteResult {
    equipped: bool,
}

impl EntityEquipment {
    #[cfg(target_arch = "wasm32")]
    fn validate(&self) -> SjmlResult<()> {
        for stack in [
            &self.main_hand,
            &self.off_hand,
            &self.head,
            &self.chest,
            &self.legs,
            &self.feet,
        ]
        .into_iter()
        .flatten()
        {
            stack.validate()?;
        }
        Ok(())
    }
}

fn invalid_entity_equipment(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

impl EntityVitals {
    #[cfg(target_arch = "wasm32")]
    fn validate(self) -> SjmlResult<()> {
        if !self.health.is_finite()
            || !self.max_health.is_finite()
            || !self.absorption.is_finite()
            || self.max_health <= 0.0
            || self.max_health > MAX_ENTITY_VITALS_VALUE
            || self.health < 0.0
            || self.health > self.max_health
            || self.absorption < 0.0
            || self.absorption > MAX_ENTITY_VITALS_VALUE
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned non-finite or out-of-range entity health values",
            ));
        }
        Ok(())
    }
}

impl EntityAirSupply {
    /// Constructs a detached air-supply value after validating its bounded vanilla range.
    pub fn new(current: i32, max: u32) -> SjmlResult<Self> {
        let air_supply = Self { current, max };
        air_supply.validate()?;
        Ok(air_supply)
    }

    fn validate(self) -> SjmlResult<()> {
        if self.current < MIN_ENTITY_AIR_SUPPLY
            || i64::from(self.current) > i64::from(self.max)
            || self.max > MAX_ENTITY_AIR_SUPPLY
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned current air outside -20..=maximum or a maximum above 1,000,000",
            ));
        }
        Ok(())
    }
}

impl EntityRotation {
    /// Constructs an entity rotation after checking finite angles and supported bounds.
    pub fn new(yaw: f32, pitch: f32) -> SjmlResult<Self> {
        let rotation = Self { yaw, pitch };
        rotation.validate()?;
        Ok(rotation)
    }

    fn validate(self) -> SjmlResult<()> {
        if !self.yaw.is_finite()
            || self.yaw.abs() > MAX_ENTITY_YAW_DEGREES
            || !self.pitch.is_finite()
            || !(-MAX_ENTITY_PITCH_DEGREES..=MAX_ENTITY_PITCH_DEGREES).contains(&self.pitch)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity yaw must be finite within ±36,000 degrees and pitch within -90..=90 degrees",
            ));
        }
        Ok(())
    }
}

/// Detached active status effect on a loaded living non-player entity.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ActiveEffect {
    /// Canonical lowercase namespace:path effect registry ID.
    pub effect_id: String,
    /// Zero-based vanilla effect amplifier.
    pub amplifier: u32,
    /// Remaining ticks, or `-1` for an infinite-duration effect.
    pub remaining_duration_ticks: i32,
}

/// Detached identity for one vehicle or passenger relationship.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityRidingRelationship {
    /// Canonical lowercase UUID string.
    pub uuid: String,
    /// Canonical namespace:path entity type ID.
    pub entity_type_id: String,
    /// True when the related entity is a player. Player state and handles are not exposed here.
    pub is_player: bool,
}

/// Detached mount relationships for one loaded non-player entity.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct EntityRidingSnapshot {
    /// Current vehicle, including its identity when it is a player.
    pub vehicle: Option<EntityRidingRelationship>,
    /// Current passenger identities, sorted by UUID and bounded to 16 entries.
    pub passengers: Vec<EntityRidingRelationship>,
}

impl EntityRidingSnapshot {
    /// Checks the passenger count, relationship identifiers, and stable passenger ordering.
    pub fn validate(&self) -> SjmlResult<()> {
        if self.passengers.len() > MAX_ENTITY_PASSENGERS {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "entity passenger snapshot exceeds the 16-passenger limit",
            ));
        }
        let mut previous_uuid: Option<&str> = None;
        for relationship in self.vehicle.iter().chain(self.passengers.iter()) {
            validate_riding_relationship(relationship)?;
        }
        for passenger in &self.passengers {
            if let Some(previous) = previous_uuid {
                if previous >= passenger.uuid.as_str() {
                    return Err(SjmlError::new(
                        ErrorCode::InvalidArgument,
                        "entity passengers must have unique UUIDs sorted in ascending order",
                    ));
                }
            }
            previous_uuid = Some(&passenger.uuid);
        }
        Ok(())
    }
}

fn validate_riding_relationship(relationship: &EntityRidingRelationship) -> SjmlResult<()> {
    if relationship.uuid.len() != 36
        || !relationship
            .uuid
            .bytes()
            .enumerate()
            .all(|(index, byte)| match index {
                8 | 13 | 18 | 23 => byte == b'-',
                _ => byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte),
            })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity relationship UUID is not canonical lowercase UUID text",
        ));
    }
    let id = relationship.entity_type_id.as_bytes();
    if id.is_empty()
        || id.len() > MAX_ENTITY_RIDING_TYPE_ID_BYTES
        || !id.iter().filter(|byte| **byte == b':').count().eq(&1)
        || !id.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-/:".contains(byte)
        })
    {
        return Err(SjmlError::new(
            if id.len() > MAX_ENTITY_RIDING_TYPE_ID_BYTES {
                ErrorCode::PayloadTooLarge
            } else {
                ErrorCode::InvalidArgument
            },
            "entity relationship type ID must be a canonical namespace:path value of at most 256 bytes",
        ));
    }
    let mut split = relationship.entity_type_id.split(':');
    let namespace = split.next().unwrap_or_default();
    let path = split.next().unwrap_or_default();
    if namespace.is_empty()
        || path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains("//")
        || path.split('/').any(|part| part == "." || part == "..")
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity relationship type ID must be a canonical namespace:path value",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, StableEncode)]
#[sjml(version = 1)]
pub(crate) struct ActiveEffectsResponse {
    pub(crate) effects: Vec<ActiveEffect>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for ActiveEffectsResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported active entity-effect response schema",
            ));
        }
        let count = decoder.read_length(MAX_ACTIVE_ENTITY_EFFECTS)?;
        let mut effects = Vec::new();
        effects.try_reserve_exact(count).map_err(|_| {
            SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not reserve memory for active entity effects",
            )
        })?;
        for _ in 0..count {
            effects.push(ActiveEffect::decode(decoder)?);
        }
        validate_active_effects(&effects)?;
        Ok(Self { effects })
    }
}

impl EntityVelocity {
    /// Constructs a velocity after checking that every component is finite and bounded.
    pub fn new(x: f64, y: f64, z: f64) -> SjmlResult<Self> {
        let velocity = Self { x, y, z };
        velocity.validate()?;
        Ok(velocity)
    }

    fn validate(self) -> SjmlResult<()> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.z.is_finite()
            || self.x.abs() > MAX_ENTITY_VELOCITY_COMPONENT
            || self.y.abs() > MAX_ENTITY_VELOCITY_COMPONENT
            || self.z.abs() > MAX_ENTITY_VELOCITY_COMPONENT
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity velocity components must be finite and within ±64 blocks per tick",
            ));
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_entity_v1")]
extern "C" {
    #[link_name = "loaded"]
    fn imported_loaded(output: i32, capacity: i32) -> i32;
    #[link_name = "within"]
    fn imported_within(request: i32, request_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "raycast"]
    fn imported_raycast(request: i32, request_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "snapshot"]
    fn imported_snapshot(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "state"]
    fn imported_state(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "bounds"]
    fn imported_bounds(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "equipment"]
    fn imported_equipment(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_equipment"]
    fn imported_set_equipment(request: i32, request_length: i32, output: i32, capacity: i32)
        -> i32;
    #[link_name = "riding_snapshot"]
    fn imported_riding_snapshot(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "target"]
    fn imported_target(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "target_snapshot"]
    fn imported_target_snapshot(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_target"]
    fn imported_set_target(mob: i32, mob_length: i32, target: i32, target_length: i32) -> i32;
    #[link_name = "clear_target"]
    fn imported_clear_target(handle: i32, length: i32) -> i32;
    #[link_name = "start_riding"]
    fn imported_start_riding(
        passenger: i32,
        passenger_length: i32,
        vehicle: i32,
        vehicle_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "stop_riding"]
    fn imported_stop_riding(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "active_effects"]
    fn imported_active_effects(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "apply_effect"]
    fn imported_apply_effect(
        handle: i32,
        handle_length: i32,
        effect_id: i32,
        effect_id_length: i32,
        duration_ticks: i32,
        amplifier: i32,
    ) -> i32;
    #[link_name = "remove_effect"]
    fn imported_remove_effect(
        handle: i32,
        handle_length: i32,
        effect_id: i32,
        effect_id_length: i32,
    ) -> i32;
    #[link_name = "discard"]
    fn imported_discard(handle: i32, handle_length: i32) -> i32;
    #[link_name = "velocity"]
    fn imported_velocity(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "fire_ticks"]
    fn imported_fire_ticks(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_fire_ticks"]
    fn imported_set_fire_ticks(handle: i32, handle_length: i32, ticks: i32) -> i32;
    #[link_name = "is_ai_enabled"]
    fn imported_is_ai_enabled(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_ai_enabled"]
    fn imported_set_ai_enabled(handle: i32, handle_length: i32, enabled: i32) -> i32;
    #[link_name = "navigate_to"]
    fn imported_navigate_to(request: i32, request_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "is_glowing"]
    fn imported_is_glowing(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_glowing"]
    fn imported_set_glowing(handle: i32, handle_length: i32, glowing: i32) -> i32;
    #[link_name = "is_silent"]
    fn imported_is_silent(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_silent"]
    fn imported_set_silent(handle: i32, handle_length: i32, silent: i32) -> i32;
    #[link_name = "set_no_gravity"]
    fn imported_set_no_gravity(handle: i32, handle_length: i32, no_gravity: i32) -> i32;
    #[link_name = "set_invulnerable"]
    fn imported_set_invulnerable(handle: i32, handle_length: i32, invulnerable: i32) -> i32;
    #[link_name = "custom_name"]
    fn imported_custom_name(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_custom_name"]
    fn imported_set_custom_name(
        handle: i32,
        handle_length: i32,
        name: i32,
        name_length: i32,
    ) -> i32;
    #[link_name = "clear_custom_name"]
    fn imported_clear_custom_name(handle: i32, handle_length: i32) -> i32;
    #[link_name = "is_custom_name_visible"]
    fn imported_is_custom_name_visible(
        handle: i32,
        handle_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "set_custom_name_visible"]
    fn imported_set_custom_name_visible(handle: i32, handle_length: i32, visible: i32) -> i32;
    #[link_name = "set_velocity"]
    fn imported_set_velocity(
        handle: i32,
        handle_length: i32,
        velocity: i32,
        velocity_length: i32,
    ) -> i32;
    #[link_name = "rotation"]
    fn imported_rotation(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "vitals"]
    fn imported_vitals(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "air_supply"]
    fn imported_air_supply(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_air_supply"]
    fn imported_set_air_supply(handle: i32, handle_length: i32, ticks: i32) -> i32;
    #[link_name = "set_rotation"]
    fn imported_set_rotation(
        handle: i32,
        handle_length: i32,
        rotation: i32,
        rotation_length: i32,
    ) -> i32;
    #[link_name = "teleport"]
    fn imported_teleport(request: i32, request_length: i32) -> i32;
    #[link_name = "heal"]
    fn imported_heal(handle: i32, handle_length: i32, amount_bits: i32) -> i32;
    #[link_name = "damage"]
    fn imported_damage(handle: i32, handle_length: i32, amount_bits: i32) -> i32;
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct SpatialQuery {
    world: WorldHandle,
    center: BlockPos,
    horizontal_radius: f64,
    vertical_radius: f64,
    entity_type_id: Option<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct SpatialResults {
    handles: Vec<EntityHandle>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct EntityRaycastQuery {
    world: WorldHandle,
    start: RaycastPoint,
    end: RaycastPoint,
    entity_type_id: Option<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct EntityRaycastResult {
    handle: Option<EntityHandle>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct EntityTargetResult {
    target: Option<EntityHandle>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct EntityTargetSnapshotResult {
    target: Option<EntityTargetSnapshot>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode, StableDecode)]
#[sjml(version = 1)]
struct EntityTeleportRequest {
    entity: EntityHandle,
    x: f64,
    y: f64,
    z: f64,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
struct EntityCustomNameResponse {
    name: Option<String>,
}

/// Maximum number of loaded non-player entities returned by [`within`].
pub const MAX_SPATIAL_QUERY_RESULTS: usize = 256;
/// Maximum horizontal or vertical extent for a loaded-entity spatial query.
pub const MAX_SPATIAL_QUERY_RADIUS: f64 = 64.0;
/// Maximum distance covered by [`raycast`].
pub const MAX_ENTITY_RAYCAST_DISTANCE: f64 = 128.0;

/// Returns loaded non-player entity handles inside a horizontal cylinder and vertical range.
///
/// Requires both `entity.read` and `world.read`; the world handle must belong to this module and
/// active logical server. Horizontal distance is measured in the X/Z plane from `center`, and the
/// vertical distance is an absolute Y difference. Each radius must be finite and in `0..=64`.
/// `entity_type_id` optionally filters by canonical entity registry ID, for example
/// `Some("minecraft:zombie")`. The host examines only entities already tracked in loaded chunks,
/// never requests chunk loads, returns results sorted by UUID, and fails the whole query if more
/// than 256 entities match. Returned handles retain the usual module, server, generation, and
/// live-instance checks used by the other `entity` APIs.
pub fn within(
    world: WorldHandle,
    center: BlockPos,
    horizontal_radius: f64,
    vertical_radius: f64,
    entity_type_id: Option<&str>,
) -> SjmlResult<Vec<EntityHandle>> {
    validate_spatial_query(center, horizontal_radius, vertical_radius, entity_type_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = SpatialQuery {
            world,
            center,
            horizontal_radius,
            vertical_radius,
            entity_type_id: entity_type_id.map(str::to_owned),
        };
        let request = to_bytes(&request)?;
        let mut results: SpatialResults = read_output(
            (8 + MAX_SPATIAL_QUERY_RESULTS * 24) as u32,
            |output, capacity| {
                // SAFETY: the serialized request remains live for the synchronous import and
                // read_output owns the exact destination allocation through decoding.
                unsafe {
                    imported_within(
                        request.as_ptr() as usize as u32 as i32,
                        request.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if results.handles.len() > MAX_SPATIAL_QUERY_RESULTS {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "host returned more than 256 spatial query results",
            ));
        }
        Ok(std::mem::take(&mut results.handles))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Returns the nearest loaded non-player entity whose collision box intersects a segment.
///
/// Requires `entity.read` and `world.read`. The world handle must belong to this module and its
/// active logical server. Both endpoints must be finite and within 30,000,000 blocks; the segment
/// must be nonzero and at most 128 blocks long. The optional filter accepts one canonical
/// namespace:path entity type ID. The host checks only loaded entities in the selected dimension,
/// never loads chunks, returns `None` when there is no hit, and uses the UUID as a deterministic
/// tie-break when entity boxes intersect the ray at the same distance.
pub fn raycast(
    world: WorldHandle,
    start: RaycastPoint,
    end: RaycastPoint,
    entity_type_id: Option<&str>,
) -> SjmlResult<Option<EntityHandle>> {
    validate_entity_raycast(start, end, entity_type_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = EntityRaycastQuery {
            world,
            start,
            end,
            entity_type_id: entity_type_id.map(str::to_owned),
        };
        let request = to_bytes(&request)?;
        let result: EntityRaycastResult = read_output(4 + 1 + 24, |output, capacity| {
            // SAFETY: the encoded request stays live through the synchronous import and the
            // output helper owns its entire destination allocation until stable decoding ends.
            unsafe {
                imported_raycast(
                    request.as_ptr() as usize as u32 as i32,
                    request.len() as u32 as i32,
                    output,
                    capacity,
                )
            }
        })?;
        Ok(result.handle)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

fn validate_entity_raycast(
    start: RaycastPoint,
    end: RaycastPoint,
    entity_type_id: Option<&str>,
) -> SjmlResult<()> {
    let valid_point = |point: RaycastPoint| {
        point.x.is_finite()
            && point.y.is_finite()
            && point.z.is_finite()
            && point.x.abs() <= 30_000_000.0
            && point.y.abs() <= 30_000_000.0
            && point.z.abs() <= 30_000_000.0
    };
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dz = end.z - start.z;
    let length_squared = dx * dx + dy * dy + dz * dz;
    if !valid_point(start)
        || !valid_point(end)
        || !length_squared.is_finite()
        || length_squared <= 0.0
        || length_squared > MAX_ENTITY_RAYCAST_DISTANCE * MAX_ENTITY_RAYCAST_DISTANCE
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity ray endpoints must be finite and the segment length must be greater than zero and at most 128 blocks",
        ));
    }
    if let Some(entity_type_id) = entity_type_id {
        validate_entity_type_id(entity_type_id)?;
    }
    Ok(())
}

fn validate_entity_type_id(entity_type_id: &str) -> SjmlResult<()> {
    let Some((namespace, path)) = entity_type_id.split_once(':') else {
        return Err(invalid_entity_type_id());
    };
    if entity_type_id.len() > 256
        || namespace.is_empty()
        || path.is_empty()
        || path.contains(':')
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'-' | b'/')
        })
    {
        return Err(invalid_entity_type_id());
    }
    Ok(())
}

fn validate_spatial_query(
    center: BlockPos,
    horizontal_radius: f64,
    vertical_radius: f64,
    entity_type_id: Option<&str>,
) -> SjmlResult<()> {
    if center.x < -30_000_000
        || center.x > 30_000_000
        || center.z < -30_000_000
        || center.z > 30_000_000
        || !horizontal_radius.is_finite()
        || !(0.0..=MAX_SPATIAL_QUERY_RADIUS).contains(&horizontal_radius)
        || !vertical_radius.is_finite()
        || !(0.0..=MAX_SPATIAL_QUERY_RADIUS).contains(&vertical_radius)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity query center or radii are outside their supported bounds",
        ));
    }
    if let Some(entity_type_id) = entity_type_id {
        let Some((namespace, path)) = entity_type_id.split_once(':') else {
            return Err(invalid_entity_type_id());
        };
        if entity_type_id.len() > 256
            || namespace.is_empty()
            || path.is_empty()
            || path.contains(':')
            || !namespace.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'.' | b'-')
            })
            || !path.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'.' | b'-' | b'/')
            })
        {
            return Err(invalid_entity_type_id());
        }
    }
    Ok(())
}

fn invalid_entity_type_id() -> SjmlError {
    SjmlError::new(
        ErrorCode::InvalidArgument,
        "entity type ID must be a lowercase namespace:path value of at most 256 bytes",
    )
}

fn canonical_entity_uuid(value: &str) -> bool {
    if value.len() != MAX_ENTITY_TARGET_UUID_BYTES {
        return false;
    }
    value.bytes().enumerate().all(|(index, byte)| match index {
        8 | 13 | 18 | 23 => byte == b'-',
        _ => byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte),
    })
}

fn bounded_target_coordinate(value: f64) -> bool {
    value.is_finite() && value.abs() <= MAX_ENTITY_TARGET_COORDINATE
}

/// Returns all loaded non-player entities across the server's loaded dimensions, including modded
/// dimensions, sorted by canonical UUID. Enumeration does not load chunks and returns the complete
/// list or an error at the per-module handle quota; it never truncates. Before server binding and
/// in client modules this returns an empty vector.
pub fn loaded() -> SjmlResult<Vec<EntityHandle>> {
    #[cfg(target_arch = "wasm32")]
    {
        read_output(4 + 24 * 1024, |output, capacity| {
            // SAFETY: read_output owns the destination allocation for this synchronous import.
            unsafe { imported_loaded(output, capacity) }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Reads a detached snapshot after the host validates the handle's owner, context, kind,
/// generation, capability, and live-entity instance.
pub fn snapshot(handle: EntityHandle) -> SjmlResult<EntitySnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let capacity = (4 + 3 * (4 + MAX_STRING_BYTES) + 3 * core::mem::size_of::<f64>()) as u32;
        read_output(capacity, |output, capacity| {
            // SAFETY: the encoded handle and destination remain live through the synchronous call.
            unsafe {
                imported_snapshot(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
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

/// Reads seven detached movement/render-state flags for one loaded non-player entity.
///
/// The module must request and receive `entity.read`. The read is synchronous on the owning
/// logical-server thread; the host validates the live, module-owned handle and does not load
/// chunks. The flags report Minecraft's current grounded, water, fire, sprint, crouch, invisibility,
/// and glowing state. No Minecraft object or mutable state crosses the ABI.
pub fn state_snapshot(handle: EntityHandle) -> SjmlResult<EntityStateSnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output::<EntityStateSnapshot>(MAX_ENTITY_STATE_RESPONSE_BYTES, |output, capacity| {
            // SAFETY: the encoded handle and destination stay live through this synchronous import.
            unsafe {
                imported_state(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
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

/// Reads the finite world-space AABB of one live, already-loaded non-player entity.
///
/// The module must request and receive `entity.read`. The host validates handle owner, generation,
/// logical-server side, and server thread; it reads the entity only when its chunk is already
/// loaded. This returns detached coordinates and never creates another entity handle.
pub fn bounds(handle: EntityHandle) -> SjmlResult<EntityBounds> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output::<EntityBounds>(MAX_ENTITY_BOUNDS_RESPONSE_BYTES, |output, capacity| {
            // SAFETY: the encoded handle and destination stay live through this synchronous import.
            unsafe {
                imported_bounds(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
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

/// Reads the current vehicle and up to 16 passenger identities for a loaded non-player entity.
///
/// The module must request and receive `entity.read`. The host runs on the owning logical server
/// thread and does not load chunks. Related player entities are identified by UUID and type only;
/// this API does not create player handles or expose player state.
pub fn riding_snapshot(handle: EntityHandle) -> SjmlResult<EntityRidingSnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let snapshot = read_output::<EntityRidingSnapshot>(
            MAX_ENTITY_RIDING_SNAPSHOT_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: handle and bounded output allocation live for this synchronous server import.
                unsafe {
                    imported_riding_snapshot(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        snapshot.validate()?;
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads a detached bounded snapshot of a loaded Mob's current live non-player target.
///
/// Requires `entity.read` and runs synchronously on the owning logical-server thread.
/// Player targets and Mobs without a target are reported as `None`. The returned snapshot contains
/// only UUID, entity type, dimension, and finite bounded coordinates; it exposes no entity handle.
pub fn target_snapshot(mob: EntityHandle) -> SjmlResult<Option<EntityTargetSnapshot>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&mob)?;
        let response = read_output::<EntityTargetSnapshotResult>(
            MAX_ENTITY_TARGET_SNAPSHOT_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: handle bytes and the bounded output allocation live through this import.
                unsafe {
                    imported_target_snapshot(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if let Some(snapshot) = &response.target {
            snapshot.validate()?;
        }
        Ok(response.target)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mob;
        Err(unsupported_target())
    }
}

/// Reads a loaded Mob's current loaded non-player target handle, if it has one.
///
/// This preserves the original SDK and `sjml_entity_v1.target` import contract. Player targets and
/// Mobs without a target are reported as `None`; returned handles remain scoped to this module.
/// Use [`target_snapshot`] when detached identity and position data are enough.
pub fn target(mob: EntityHandle) -> SjmlResult<Option<EntityHandle>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&mob)?;
        let response = read_output::<EntityTargetResult>(
            MAX_ENTITY_TARGET_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: handle bytes and the bounded output allocation live through this import.
                unsafe {
                    imported_target(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        Ok(response.target)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mob;
        Err(unsupported_target())
    }
}

/// Alias for [`target`] that makes the returned module-scoped handle explicit at the call site.
pub fn target_handle(mob: EntityHandle) -> SjmlResult<Option<EntityHandle>> {
    target(mob)
}

/// Sets one loaded non-player living entity as the current target of a loaded Mob.
///
/// Requires `entity.read` and [`TARGET_WRITE_CAPABILITY`]. Both handles must belong
/// to this module and identify live entities on the same logical server and in the
/// same dimension; the target cannot be a player or the Mob itself. The host calls
/// Minecraft synchronously on the owning server thread and does not load chunks.
pub fn set_target(mob: EntityHandle, target: EntityHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let mob = to_bytes(&mob)?;
        let target = to_bytes(&target)?;
        // SAFETY: both fixed-width handles remain live during the synchronous import.
        let status = unsafe {
            imported_set_target(
                mob.as_ptr() as usize as u32 as i32,
                mob.len() as i32,
                target.as_ptr() as usize as u32 as i32,
                target.len() as i32,
            )
        };
        status_result(status, "host rejected the entity target update")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (mob, target);
        Err(unsupported_target())
    }
}

/// Clears a loaded Mob's current target.
///
/// Requires `entity.read` and [`TARGET_WRITE_CAPABILITY`]. The Mob handle must be
/// live and owned by this module; the operation runs on the owning server thread
/// and never loads chunks.
pub fn clear_target(mob: EntityHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&mob)?;
        // SAFETY: the fixed-width handle remains live during the synchronous import.
        let status = unsafe {
            imported_clear_target(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
            )
        };
        status_result(status, "host rejected the entity target clear request")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = mob;
        Err(unsupported_target())
    }
}

/// Attempts ordinary Minecraft mounting of one existing loaded non-player entity onto another.
///
/// Both entities must have live handles in the same loaded world and the passenger must not
/// already be riding. The operation never forces a mount and does not load chunks. It requires
/// both `entity.read` and `entity.mount.write`. Returns `false` when vanilla rejects the mount.
pub fn start_riding(passenger: EntityHandle, vehicle: EntityHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let passenger = to_bytes(&passenger)?;
        let vehicle = to_bytes(&vehicle)?;
        let result = read_output::<u32>(4, |output, capacity| {
            // SAFETY: handle bytes and output allocation remain live for this synchronous call.
            unsafe {
                imported_start_riding(
                    passenger.as_ptr() as usize as u32 as i32,
                    passenger.len() as i32,
                    vehicle.as_ptr() as usize as u32 as i32,
                    vehicle.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        decode_mount_result(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (passenger, vehicle);
        Err(unsupported_target())
    }
}

/// Stops an existing mount for one loaded non-player entity; returns whether it dismounted.
///
/// It requires `entity.read` and `entity.mount.write`, runs on the owning logical server thread,
/// and changes no other entity's mount. Returns `false` if the entity had no vehicle.
pub fn stop_riding(passenger: EntityHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&passenger)?;
        let result = read_output::<u32>(4, |output, capacity| {
            // SAFETY: handle bytes and output allocation remain live for this synchronous call.
            unsafe {
                imported_stop_riding(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        decode_mount_result(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = passenger;
        Err(unsupported_target())
    }
}

#[cfg(target_arch = "wasm32")]
fn decode_mount_result(value: u32) -> SjmlResult<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned an invalid entity mount result",
        )),
    }
}

/// Reads a sorted detached list of active effects from a loaded living non-player entity.
///
/// Requires `entity.read` and runs on the owning logical server. The response contains at most
/// 64 effects, sorted by canonical registry ID. IDs are limited to 256 UTF-8 bytes; amplifier
/// values are `0..=255`. `remaining_duration_ticks` is the current remaining duration, or `-1`
/// for a vanilla infinite-duration effect. The host verifies the handle and live entity on every
/// call and does not load chunks.
pub fn active_effects(handle: EntityHandle) -> SjmlResult<Vec<ActiveEffect>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let response: ActiveEffectsResponse = read_output(
            MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the encoded handle and bounded output allocation remain live while the
                // synchronous server import checks the handle and writes the detached snapshot.
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

/// Applies a registered status effect to a loaded living entity identified by a current handle.
///
/// The module must request and receive both `entity.read` and
/// `entity.effect.apply`. Duration is limited to 1–72,000 ticks and amplifier
/// to 0–255. The target is revalidated on the owning server thread for every
/// call; this does not spawn entities or apply effects to players.
pub fn apply_effect(
    handle: EntityHandle,
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
        status_result(status, "host rejected the entity effect application")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, amplifier);
        Err(unsupported_target())
    }
}

/// Removes one registered status effect from a loaded living non-player entity.
///
/// The module must request and receive both `entity.read` and
/// `entity.effect.remove`. The target is revalidated on the owning server
/// thread for every call. An effect that is not currently active is a
/// successful no-op; an unknown effect ID is rejected. Players, nonliving
/// entities, unloaded targets, client contexts, and static registration calls
/// are rejected. The guest receives no Minecraft objects.
pub fn remove_effect(handle: EntityHandle, effect_id: &str) -> SjmlResult<()> {
    validate_effect_id(effect_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the host copies this bounded ID and handle before returning.
        let status = unsafe {
            imported_remove_effect(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                effect_id.as_ptr() as usize as u32 as i32,
                effect_id.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity effect removal")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Discards one loaded non-player entity.
///
/// The module must request and receive both `entity.read` and `entity.remove`.
/// The host revalidates this handle on the owning server thread and revokes it
/// immediately after the entity is discarded. Players, unloaded entities,
/// client contexts, and static registration calls are rejected. The operation
/// does not expose a Minecraft entity object to the guest.
pub fn discard(handle: EntityHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the host decodes the fixed-width handle during this synchronous call.
        let status = unsafe {
            imported_discard(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity removal")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads the current velocity of one loaded living non-player entity.
///
/// The module must request and receive `entity.read`. Values are returned as
/// schema-v1 doubles in blocks per tick; each component must be finite and no
/// greater than 64 in absolute value. Player and non-living entity handles are
/// rejected by the host.
pub fn velocity(handle: EntityHandle) -> SjmlResult<EntityVelocity> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let capacity = (4 + 3 * core::mem::size_of::<f64>()) as u32;
        read_output(capacity, |output, capacity| {
            // SAFETY: read_output owns the destination allocation for this synchronous import.
            unsafe {
                imported_velocity(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
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

/// Reads the current yaw and pitch of one loaded non-player entity.
///
/// The module must request `entity.read`. Angles are returned in degrees and
/// checked for finite values, yaw within ±36,000, and pitch within -90..=90.
/// The host revalidates the live entity on the owning server thread and only
/// reads entities already present in a loaded chunk.
pub fn rotation(handle: EntityHandle) -> SjmlResult<EntityRotation> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let rotation = read_output::<EntityRotation>(12, |output, capacity| {
            // SAFETY: read_output owns the destination allocation for this synchronous import.
            unsafe {
                imported_rotation(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        rotation.validate()?;
        Ok(rotation)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads current health, effective maximum health, and absorption for one loaded living entity.
///
/// The module must request `entity.read`. The host returns exactly one schema-v1 snapshot and
/// rejects non-finite values, health outside `0..=max_health`, maximum health outside
/// `0..=1,000,000`, or absorption outside `0..=1,000,000`. The target must be a live, loaded,
/// non-player living entity on the owning server thread; this call never loads a chunk.
pub fn vitals(handle: EntityHandle) -> SjmlResult<EntityVitals> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let vitals = read_output::<EntityVitals>(16, |output, capacity| {
            // SAFETY: read_output owns the destination allocation for this synchronous import.
            unsafe {
                imported_vitals(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        vitals.validate()?;
        Ok(vitals)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads current and maximum air supply for one loaded living non-player entity.
///
/// The module must request `entity.read`. The current value may be as low as `-20` during
/// vanilla's drowning damage window; the entity-specific maximum is bounded to `0..=1,000,000`.
/// The host checks the live entity on the owning server thread and never loads chunks.
pub fn air_supply(handle: EntityHandle) -> SjmlResult<EntityAirSupply> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let air_supply = read_output::<EntityAirSupply>(
            MAX_ENTITY_AIR_SUPPLY_RESPONSE_BYTES,
            |output, capacity| {
                // SAFETY: read_output owns the destination allocation for this synchronous import.
                unsafe {
                    imported_air_supply(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        air_supply.validate()?;
        Ok(air_supply)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets current air supply for one loaded living entity without changing its maximum.
///
/// Requires `entity.read` and [`AIR_SUPPLY_WRITE_CAPABILITY`]. The host rejects values above the
/// entity's live maximum. `ticks` is nonnegative and bounded to `0..=1,000,000`; the target must be
/// a live loaded non-player living entity on the owning server thread.
pub fn set_air_supply(handle: EntityHandle, ticks: u32) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        if ticks > MAX_ENTITY_AIR_SUPPLY {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity air-supply ticks must be within 0..=1,000,000",
            ));
        }
        let encoded = to_bytes(&handle)?;
        // SAFETY: encoded is a live ABI handle for the synchronous imported call.
        status_result(
            unsafe {
                imported_set_air_supply(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    ticks as i32,
                )
            },
            "host rejected the entity air-supply change",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, ticks);
        Err(unsupported_target())
    }
}

/// Reads the six vanilla equipment slots for one loaded living non-player entity.
///
/// Requires `entity.read` and a live entity handle on the logical-server thread. Slots are
/// returned in main hand, offhand, head, chest, legs, feet order. Each occupied slot contains
/// only its canonical item ID, count, current damage, and maximum damage. Empty slots are `None`.
/// This read does not load chunks and exposes no item NBT or data components.
pub fn equipment(handle: EntityHandle) -> SjmlResult<EntityEquipment> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let equipment: EntityEquipment =
            read_output(MAX_ENTITY_EQUIPMENT_RESPONSE_BYTES, |output, capacity| {
                // SAFETY: the encoded handle and destination remain live through this synchronous host import.
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

/// Adds one registered item's default stack to an empty slot of a loaded living non-player entity.
///
/// The module must request and receive `entity.read` and `entity.equipment.write`. Only the six
/// vanilla slots in [`EntityEquipmentSlot`] are supported. `item_id` must be a canonical ID for an
/// item already present in Minecraft's item registry. The host creates exactly one default stack,
/// with no guest-supplied NBT or data components. Returns `true` after inserting it, or `false`
/// when the slot is occupied; occupied stacks are never replaced or discarded. The operation runs
/// on the owning logical-server thread and never loads chunks.
pub fn try_equip_default(
    handle: EntityHandle,
    slot: EntityEquipmentSlot,
    item_id: &str,
) -> SjmlResult<bool> {
    // Reuse detached stack validation to reject malformed IDs before any host call.
    EntityEquipmentStack::new(item_id, 1, 0, 0)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let item_id_bytes = item_id.as_bytes();
        let mut request = Vec::with_capacity(4 + encoded_handle.len() + 8 + item_id_bytes.len());
        request.extend_from_slice(&1_u32.to_le_bytes());
        request.extend_from_slice(&encoded_handle);
        request.extend_from_slice(&slot.code().to_le_bytes());
        request.extend_from_slice(&(item_id_bytes.len() as u32).to_le_bytes());
        request.extend_from_slice(item_id_bytes);
        let result: EntityEquipmentWriteResult = read_output(5, |output, capacity| {
            // SAFETY: the versioned request and destination remain live for the synchronous import.
            unsafe {
                imported_set_equipment(
                    request.as_ptr() as usize as u32 as i32,
                    request.len() as u32 as i32,
                    output,
                    capacity,
                )
            }
        })?;
        Ok(result.equipped)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, slot);
        Err(unsupported_target())
    }
}

/// Sets yaw and pitch on one loaded non-player entity.
///
/// The module must request and receive both `entity.read` and
/// `entity.rotation.write`. Yaw is finite within ±36,000 degrees and pitch is
/// finite within -90..=90 degrees. The host revalidates the live entity and
/// owning server thread before changing the vanilla entity rotation.
pub fn set_rotation(handle: EntityHandle, rotation: EntityRotation) -> SjmlResult<()> {
    rotation.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_rotation = to_bytes(&rotation)?;
        // SAFETY: both bounded stable values remain live during the synchronous host call.
        let status = unsafe {
            imported_set_rotation(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                encoded_rotation.as_ptr() as usize as u32 as i32,
                encoded_rotation.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity rotation mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Teleports one loaded, unmounted non-player entity within its current dimension.
///
/// The module must request and receive both `entity.read` and
/// [`TELEPORT_CAPABILITY`]. Coordinates must be finite and within ±30,000,000
/// blocks. The host also checks the entity's full destination bounding box
/// against build height and the world border, and requires every destination
/// chunk touched by that box to already be loaded. Calls run synchronously on
/// the owning logical-server thread, never load chunks, and do not transfer
/// dimensions. Entities that are riding or have passengers are rejected.
pub fn teleport(handle: EntityHandle, x: f64, y: f64, z: f64) -> SjmlResult<()> {
    validate_teleport_target(x, y, z)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = to_bytes(&EntityTeleportRequest {
            entity: handle,
            x,
            y,
            z,
        })?;
        // SAFETY: the fixed-schema request remains live during the synchronous host call.
        let status = unsafe {
            imported_teleport(
                request.as_ptr() as usize as u32 as i32,
                request.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity teleport request")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

fn validate_teleport_target(x: f64, y: f64, z: f64) -> SjmlResult<()> {
    if !x.is_finite()
        || !y.is_finite()
        || !z.is_finite()
        || x.abs() > MAX_ENTITY_TELEPORT_COORDINATE
        || y.abs() > MAX_ENTITY_TELEPORT_COORDINATE
        || z.abs() > MAX_ENTITY_TELEPORT_COORDINATE
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity teleport coordinates must be finite and within ±30,000,000 blocks",
        ));
    }
    Ok(())
}

/// Sets all velocity components for one loaded living non-player entity.
///
/// The module must request and receive both `entity.read` and
/// `entity.velocity.write`. Components are bounded to ±64 blocks per tick;
/// the host additionally revalidates the live entity, server context, and
/// server thread before mutation and marks motion for normal client tracking.
pub fn set_velocity(handle: EntityHandle, velocity: EntityVelocity) -> SjmlResult<()> {
    velocity.validate()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_velocity = to_bytes(&velocity)?;
        // SAFETY: both bounded stable values remain live during the synchronous host call.
        let status = unsafe {
            imported_set_velocity(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                encoded_velocity.as_ptr() as usize as u32 as i32,
                encoded_velocity.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity velocity mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads effective remaining fire duration for one loaded non-player entity.
///
/// The module must request `entity.read`. Vanilla's negative sentinel values
/// for entities that are not burning are normalized to zero. Values above
/// [`MAX_ENTITY_FIRE_TICKS`] are rejected by the host rather than truncated.
pub fn fire_ticks(handle: EntityHandle) -> SjmlResult<u32> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output::<u32>(core::mem::size_of::<u32>() as u32, |output, capacity| {
            // SAFETY: read_output owns the destination allocation for this synchronous import.
            unsafe {
                imported_fire_ticks(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })
        .and_then(|ticks| {
            if ticks <= MAX_ENTITY_FIRE_TICKS {
                Ok(ticks)
            } else {
                Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "host returned entity fire ticks above the supported bound",
                ))
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets remaining active fire duration on one loaded non-player entity.
///
/// The module must request and receive both `entity.read` and
/// `entity.fire.write`. The tick count must be in `0..=72_000`; zero clears
/// the active fire duration. Mutation happens synchronously on the owning
/// logical-server thread after the host validates the module-scoped live handle.
pub fn set_fire_ticks(handle: EntityHandle, ticks: u32) -> SjmlResult<()> {
    if ticks > MAX_ENTITY_FIRE_TICKS {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity fire ticks must be within 0..=72,000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and bounded tick count remain live during the synchronous call.
        let status = unsafe {
            imported_set_fire_ticks(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                ticks as i32,
            )
        };
        status_result(status, "host rejected the entity fire-tick mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads whether AI is enabled for a loaded non-player Mob.
///
/// Requires both `entity.read` and [`AI_CONTROL_CAPABILITY`]. The operation
/// runs synchronously on the owning logical-server thread and rejects stale
/// handles and entities that are not Mobs. This flag covers vanilla Mob AI;
/// it does not expose arbitrary goals, targets, or pathfinding.
pub fn is_ai_enabled(handle: EntityHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output::<u32>(4, |output, capacity| {
            // SAFETY: the encoded handle and output allocation remain valid for this synchronous call.
            unsafe {
                imported_is_ai_enabled(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })
        .and_then(|enabled| match enabled {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid entity AI enabled flag",
            )),
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Enables or disables vanilla AI for one loaded non-player Mob.
///
/// Requires both `entity.read` and [`AI_CONTROL_CAPABILITY`]. Calls are
/// synchronous on the owning logical-server thread. The host changes only
/// Minecraft's no-AI flag; it does not read, stop, or create a navigation path.
/// Re-enabling AI resumes normal Mob AI on later ticks. The host resolves only
/// already-loaded entity handles and never loads chunks.
pub fn set_ai_enabled(handle: EntityHandle, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle remains valid during the synchronous import.
        let status = unsafe {
            imported_set_ai_enabled(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                i32::from(enabled),
            )
        };
        status_result(status, "host rejected the entity AI control request")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, enabled);
        Err(unsupported_target())
    }
}

/// Starts a bounded vanilla navigation path for one loaded non-player Mob.
///
/// Requires `entity.read` and [`NAVIGATION_WRITE_CAPABILITY`]. The target must
/// be within 32 blocks in the entity's current dimension and every chunk in a
/// bounded path-search corridor must already be loaded. SJML does not load
/// terrain. The speed multiplier must be finite and within `0.01..=4.0`.
/// Returns `true` if Minecraft accepted a path and `false` if no path was
/// accepted (including when the Mob has AI disabled). Calls are synchronous on
/// the logical-server thread.
pub fn navigate_to(handle: EntityHandle, target: BlockPos, speed: f32) -> SjmlResult<bool> {
    validate_navigation_target(target, speed)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = to_bytes(&EntityNavigationRequest {
            entity: handle,
            x: target.x,
            y: target.y,
            z: target.z,
            speed,
        })?;
        read_output::<EntityNavigationResult>(5, |output, capacity| {
            // SAFETY: the stable request and output allocation remain valid through the synchronous import.
            unsafe {
                imported_navigate_to(
                    request.as_ptr() as usize as u32 as i32,
                    request.len() as i32,
                    output,
                    capacity,
                )
            }
        })
        .map(|result| result.accepted)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct EntityNavigationRequest {
    entity: EntityHandle,
    x: i32,
    y: i32,
    z: i32,
    speed: f32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityNavigationResult {
    accepted: bool,
}

fn validate_navigation_target(target: BlockPos, speed: f32) -> SjmlResult<()> {
    if i64::from(target.x).abs() > 30_000_000 || i64::from(target.z).abs() > 30_000_000 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity navigation X/Z coordinates must be within ±30000000",
        ));
    }
    if !speed.is_finite() || !(0.01..=MAX_ENTITY_NAVIGATION_SPEED).contains(&speed) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity navigation speed must be finite and within 0.01..=4",
        ));
    }
    Ok(())
}

/// Reads whether a loaded non-player entity is currently glowing.
///
/// The module must request and receive `entity.read`. The value reflects
/// Minecraft's entity-level glowing flag. This reads the entity flag only;
/// it does not expose client rendering state or force a client outline.
pub fn is_glowing(handle: EntityHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let value = read_output::<u32>(4, |output, capacity| {
            // SAFETY: the handle and output allocation remain valid for this synchronous host call.
            unsafe {
                imported_is_glowing(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid entity glowing flag",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets the entity-level glowing flag on a loaded non-player entity.
///
/// The module must request and receive both `entity.read` and
/// [`GLOWING_WRITE_CAPABILITY`]. Normal Minecraft entity data synchronization
/// still controls whether connected clients see the outline.
pub fn set_glowing(handle: EntityHandle, glowing: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean value remain live during the synchronous host call.
        let status = unsafe {
            imported_set_glowing(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(glowing),
            )
        };
        status_result(status, "host rejected the entity glowing flag mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, glowing);
        Err(unsupported_target())
    }
}

/// Reads whether a loaded non-player entity suppresses its ordinary sounds.
///
/// Requires `entity.read` and is available only on the owning logical server.
pub fn is_silent(handle: EntityHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let value = read_output::<u32>(4, |output, capacity| {
            // SAFETY: the handle and output allocation remain valid for this synchronous host call.
            unsafe {
                imported_is_silent(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid entity silent flag",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets whether a loaded non-player entity suppresses its ordinary sounds.
///
/// The module must request and receive both `entity.read` and
/// [`STATE_WRITE_CAPABILITY`]. Minecraft's sound engine and entity tracking
/// remain responsible for client-side behavior.
pub fn set_silent(handle: EntityHandle, silent: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean value remain live during the synchronous host call.
        let status = unsafe {
            imported_set_silent(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(silent),
            )
        };
        status_result(status, "host rejected the entity silent flag mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, silent);
        Err(unsupported_target())
    }
}

/// Sets whether a loaded non-player entity ignores gravity.
///
/// The module must request and receive both `entity.read` and
/// [`STATE_WRITE_CAPABILITY`]. Calls are synchronous on the owning logical-server thread and
/// revalidate the module-owned live entity handle. This changes only Minecraft's no-gravity flag.
pub fn set_no_gravity(handle: EntityHandle, no_gravity: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean value remain live during the synchronous host call.
        let status = unsafe {
            imported_set_no_gravity(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(no_gravity),
            )
        };
        status_result(status, "host rejected the entity no-gravity flag mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, no_gravity);
        Err(unsupported_target())
    }
}

/// Sets whether a loaded non-player entity is invulnerable to ordinary Minecraft damage.
///
/// The module must request and receive both `entity.read` and
/// [`INVULNERABILITY_WRITE_CAPABILITY`]. Calls are synchronous on the owning logical-server
/// thread and revalidate the module-owned live entity handle. Minecraft's ordinary damage
/// pipeline still decides which damage sources bypass invulnerability.
pub fn set_invulnerable(handle: EntityHandle, invulnerable: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean value remain live during the synchronous host call.
        let status = unsafe {
            imported_set_invulnerable(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(invulnerable),
            )
        };
        status_result(status, "host rejected the entity invulnerability mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, invulnerable);
        Err(unsupported_target())
    }
}

/// Reads a loaded non-player entity's custom name as plain text, if one exists.
///
/// Requires `entity.read`. Rich styling on an existing vanilla name is
/// flattened to display text. The returned value is bounded to 1,024 UTF-8
/// bytes and 256 Unicode scalar values.
pub fn custom_name(handle: EntityHandle) -> SjmlResult<Option<String>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let response: EntityCustomNameResponse = read_output(
            MAX_ENTITY_CUSTOM_NAME_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the handle and output allocation live for this synchronous host call.
                unsafe {
                    imported_custom_name(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if let Some(name) = response.name.as_ref() {
            validate_custom_name(name, true)?;
        }
        Ok(response.name)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets a bounded plain-text custom name on a loaded non-player entity.
///
/// Requires both `entity.read` and `entity.custom_name.write`. The host uses
/// Minecraft's `Component.literal`, so formatting-like characters remain
/// literal text and cannot create rich commands. Names must be nonempty,
/// contain no control characters, and fit the 1,024-byte / 256-character
/// limits. Use [`clear_custom_name`] to remove a name.
pub fn set_custom_name(handle: EntityHandle, name: &str) -> SjmlResult<()> {
    validate_custom_name(name, false)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: both byte slices remain live and immutable during the synchronous call.
        let status = unsafe {
            imported_set_custom_name(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                name.as_ptr() as usize as u32 as i32,
                name.len() as i32,
            )
        };
        status_result(status, "host rejected the entity custom-name mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Clears a loaded non-player entity's custom name.
///
/// Requires both `entity.read` and `entity.custom_name.write`. Clearing is a
/// separate operation so an empty displayed name cannot be confused with an
/// absent custom name.
pub fn clear_custom_name(handle: EntityHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle remains valid through this synchronous host call.
        let status = unsafe {
            imported_clear_custom_name(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
            )
        };
        status_result(status, "host rejected the entity custom-name clear request")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads the vanilla custom-name visibility flag on a loaded non-player entity.
///
/// Requires `entity.read` and runs on the owning logical-server thread. This
/// reads only whether Minecraft displays the custom name; it does not read or
/// modify the name itself.
pub fn is_custom_name_visible(handle: EntityHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let value = read_output::<u32>(4, |output, capacity| {
            // SAFETY: the handle and output allocation remain valid for the synchronous host call.
            unsafe {
                imported_is_custom_name_visible(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid custom-name visibility flag",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets whether Minecraft displays a loaded non-player entity's custom name.
///
/// This changes only the vanilla visibility flag; the custom name and other
/// entity state are unchanged. Requires both `entity.read` and
/// `entity.custom_name.write`, and runs on the owning logical-server thread.
pub fn set_custom_name_visible(handle: EntityHandle, visible: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean value remain live during the synchronous host call.
        let status = unsafe {
            imported_set_custom_name_visible(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(visible),
            )
        };
        status_result(
            status,
            "host rejected the entity custom-name visibility update",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, visible);
        Err(unsupported_target())
    }
}

fn validate_custom_name(name: &str, allow_empty: bool) -> SjmlResult<()> {
    if !allow_empty && name.is_empty() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity custom name must not be empty; use clear_custom_name",
        ));
    }
    if name.len() > MAX_ENTITY_CUSTOM_NAME_BYTES
        || name.chars().count() > MAX_ENTITY_CUSTOM_NAME_CODE_POINTS
    {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity custom name exceeds the 1,024-byte or 256-character limit",
        ));
    }
    if !allow_empty && name.chars().any(char::is_control) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity custom name cannot contain control characters",
        ));
    }
    Ok(())
}

/// Requests one bounded generic damage hit against a loaded living non-player entity.
///
/// The module must request and receive both `entity.read` and `entity.damage`. The host uses
/// Minecraft's fixed generic damage source and ordinary `LivingEntity::hurt` path, so vanilla
/// invulnerability, mitigation, damage hooks, and death behavior remain authoritative. A
/// successful result means the host performed the request; Minecraft may ignore or cancel it.
/// The guest cannot choose an attacker, damage type, or Java object.
pub fn damage(handle: EntityHandle, amount: f32) -> SjmlResult<()> {
    if !amount.is_finite() || amount <= 0.0 || amount > MAX_ENTITY_DAMAGE {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity damage must be finite and within (0, 64]",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and f32 bits remain valid for this synchronous import.
        let status = unsafe {
            imported_damage(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                amount.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the entity damage request")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Requests one bounded heal for a loaded living non-player entity.
///
/// The module must request and receive both `entity.read` and `entity.health.write`; amounts must
/// be finite and within `(0, 64]`. Minecraft's ordinary `LivingEntity::heal` path handles the
/// request, so vanilla behavior and existing heal-attempt callbacks can cancel or ignore it. A
/// successful result means Minecraft received the heal request, not that health necessarily
/// changed. The host checks the live handle and owning server thread and does not load chunks.
pub fn heal(handle: EntityHandle, amount: f32) -> SjmlResult<()> {
    if !amount.is_finite() || amount <= 0.0 || amount > MAX_ENTITY_HEAL_AMOUNT {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity heal amount must be finite and within (0, 64]",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and f32 bits remain valid during this synchronous import.
        let status = unsafe {
            imported_heal(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                amount.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the entity heal request")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

pub(crate) fn validate_effect_id(effect_id: &str) -> SjmlResult<()> {
    let invalid = || {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "mob effect ID must be a lowercase namespace:path identifier of at most 256 bytes",
        )
    };
    if effect_id.len() < 3 || effect_id.len() > MAX_EFFECT_ID_BYTES {
        return Err(invalid());
    }
    let Some((namespace, path)) = effect_id.split_once(':') else {
        return Err(invalid());
    };
    if namespace.is_empty()
        || path.is_empty()
        || path.contains(':')
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'-' | b'/')
        })
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_active_effects(effects: &[ActiveEffect]) -> SjmlResult<()> {
    if effects.len() > MAX_ACTIVE_ENTITY_EFFECTS {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "host returned more than 64 active entity effects",
        ));
    }
    let mut previous: Option<&str> = None;
    for effect in effects {
        validate_effect_id(&effect.effect_id)?;
        if effect.amplifier > 255 || effect.remaining_duration_ticks < -1 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid active effect amplifier or duration",
            ));
        }
        if previous.is_some_and(|previous| previous >= effect.effect_id.as_str()) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host active effects are not strictly sorted by canonical ID",
            ));
        }
        previous = Some(&effect.effect_id);
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn status_result(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
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
            "could not allocate entity output buffer",
        ));
    }
    // SAFETY: initialize the complete allocation so output padding can be read safely.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: the initialized allocation stays live until stable decoding is complete.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        // Stable values delimit themselves; the remaining bytes are destination-buffer padding.
        Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the entity read",
        ))
    };
    // SAFETY: release this exact allocation once after host access and decoding finish.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "entity operations require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        active_effects, air_supply, apply_effect, clear_custom_name, clear_target, custom_name,
        damage, fire_ticks, navigate_to, raycast, remove_effect, set_air_supply, set_custom_name,
        set_fire_ticks, set_target, set_velocity, target, target_snapshot, teleport,
        validate_active_effects, validate_custom_name, validate_entity_raycast,
        validate_navigation_target, validate_spatial_query, validate_teleport_target, within,
        ActiveEffect, ActiveEffectsResponse, EntityAirSupply, EntityBounds,
        EntityCustomNameResponse, EntityNavigationRequest, EntityNavigationResult,
        EntityRaycastQuery, EntityRaycastResult, EntityStateSnapshot, EntityTargetResult,
        EntityTargetSnapshot, EntityTargetSnapshotResult, EntityTeleportRequest, EntityVelocity,
        SpatialQuery, SpatialResults, MAX_ACTIVE_ENTITY_EFFECTS,
        MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES, MAX_ENTITY_AIR_SUPPLY,
        MAX_ENTITY_AIR_SUPPLY_RESPONSE_BYTES, MAX_ENTITY_BOUNDS_COORDINATE,
        MAX_ENTITY_BOUNDS_RESPONSE_BYTES, MAX_ENTITY_CUSTOM_NAME_BYTES,
        MAX_ENTITY_CUSTOM_NAME_CODE_POINTS, MAX_ENTITY_CUSTOM_NAME_RESPONSE_BYTES,
        MAX_ENTITY_DAMAGE, MAX_ENTITY_FIRE_TICKS, MAX_ENTITY_NAVIGATION_RANGE,
        MAX_ENTITY_NAVIGATION_SPEED, MAX_ENTITY_RAYCAST_DISTANCE, MAX_ENTITY_STATE_RESPONSE_BYTES,
        MAX_ENTITY_TARGET_COORDINATE, MAX_ENTITY_TARGET_ID_BYTES, MAX_ENTITY_TARGET_RANGE,
        MAX_ENTITY_TARGET_RESPONSE_BYTES, MAX_ENTITY_TARGET_SNAPSHOT_RESPONSE_BYTES,
        MAX_ENTITY_TARGET_UUID_BYTES, MAX_ENTITY_TELEPORT_COORDINATE,
        MAX_ENTITY_VELOCITY_COMPONENT, MAX_SPATIAL_QUERY_RADIUS, MAX_SPATIAL_QUERY_RESULTS,
        NAVIGATION_WRITE_CAPABILITY,
    };
    use crate::{
        inventory::BlockPos, to_bytes, world::RaycastPoint, EntityHandle, ErrorCode, WorldHandle,
    };

    #[test]
    fn air_supply_response_matches_schema_and_rejects_out_of_range_values() {
        let air_supply = EntityAirSupply::new(-20, 320).unwrap();
        let encoded = to_bytes(&air_supply).unwrap();
        assert_eq!(
            encoded,
            [
                1_u32.to_le_bytes().as_slice(),
                &(-20_i32).to_le_bytes(),
                &320_u32.to_le_bytes(),
            ]
            .concat()
        );
        assert_eq!(encoded.len(), MAX_ENTITY_AIR_SUPPLY_RESPONSE_BYTES as usize);
        assert_eq!(
            crate::from_bytes::<EntityAirSupply>(&encoded).unwrap(),
            air_supply
        );
        assert!(air_supply.validate().is_ok());

        assert_eq!(
            EntityAirSupply::new(-21, 320).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EntityAirSupply::new(321, 320).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EntityAirSupply::new(0, MAX_ENTITY_AIR_SUPPLY + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut wrong_schema = encoded;
        wrong_schema[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            crate::from_bytes::<EntityAirSupply>(&wrong_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn air_supply_host_calls_are_unavailable_on_native_targets() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            air_supply(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_air_supply(handle, 1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn bounds_wire_schema_matches_host_field_order_and_validates_decoded_values() {
        let bounds = EntityBounds::new(-1.25, 63.0, 2.5, 0.75, 65.0, 3.5).unwrap();
        let encoded = to_bytes(&bounds).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for coordinate in [-1.25_f64, 63.0, 2.5, 0.75, 65.0, 3.5] {
            expected.extend_from_slice(&coordinate.to_le_bytes());
        }
        assert_eq!(encoded, expected);
        assert_eq!(encoded.len(), MAX_ENTITY_BOUNDS_RESPONSE_BYTES as usize);
        assert_eq!(crate::from_bytes::<EntityBounds>(&encoded).unwrap(), bounds);

        let invalid = EntityBounds {
            min_x: 2.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 1.0,
            max_y: 1.0,
            max_z: 1.0,
        };
        assert_eq!(
            invalid.validate().unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            EntityBounds::new(0.0, 0.0, 0.0, MAX_ENTITY_BOUNDS_COORDINATE + 1.0, 1.0, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut wrong_schema = encoded.clone();
        wrong_schema[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            crate::from_bytes::<EntityBounds>(&wrong_schema)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
        let mut non_finite = encoded;
        non_finite[4 + 3 * core::mem::size_of::<f64>()..4 + 4 * core::mem::size_of::<f64>()]
            .copy_from_slice(&f64::INFINITY.to_le_bytes());
        assert_eq!(
            crate::from_bytes::<EntityBounds>(&non_finite)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn state_snapshot_wire_schema_matches_host_field_order() {
        let state = EntityStateSnapshot {
            on_ground: true,
            in_water: false,
            on_fire: true,
            sprinting: false,
            crouching: true,
            invisible: false,
            glowing: true,
        };
        let expected = [1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1];
        let encoded = crate::to_bytes(&state).unwrap();
        assert_eq!(encoded, expected);
        assert_eq!(MAX_ENTITY_STATE_RESPONSE_BYTES as usize, encoded.len());
        assert_eq!(
            crate::from_bytes::<EntityStateSnapshot>(&encoded).unwrap(),
            state
        );
    }

    #[test]
    fn active_effect_response_is_versioned_bounded_and_validated() {
        assert_eq!(MAX_ACTIVE_ENTITY_EFFECTS, 64);
        assert_eq!(MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES, 17_416);
        let response = ActiveEffectsResponse {
            effects: vec![
                ActiveEffect {
                    effect_id: "minecraft:haste".to_owned(),
                    amplifier: 1,
                    remaining_duration_ticks: 200,
                },
                ActiveEffect {
                    effect_id: "minecraft:speed".to_owned(),
                    amplifier: 3,
                    remaining_duration_ticks: -1,
                },
            ],
        };
        let encoded = to_bytes(&response).unwrap();
        assert_eq!(
            encoded.len(),
            8 + (4 + 4 + "minecraft:haste".len() + 8) + (4 + 4 + "minecraft:speed".len() + 8)
        );
        assert_eq!(
            crate::from_bytes::<ActiveEffectsResponse>(&encoded)
                .unwrap()
                .effects,
            response.effects
        );

        let oversized_count = [1_u32.to_le_bytes(), 65_u32.to_le_bytes()].concat();
        assert_eq!(
            crate::from_bytes::<ActiveEffectsResponse>(&oversized_count)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_active_effects(&[response.effects[1].clone(), response.effects[0].clone(),])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_active_effects(&[ActiveEffect {
                effect_id: "minecraft:speed".to_owned(),
                amplifier: 256,
                remaining_duration_ticks: 10,
            }])
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            active_effects(EntityHandle::new(1, 2, 3, 4).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn spatial_query_bounds_are_checked_before_host_access() {
        let center = BlockPos {
            x: 10,
            y: 64,
            z: -5,
        };
        assert_eq!(MAX_SPATIAL_QUERY_RESULTS, 256);
        assert_eq!(MAX_SPATIAL_QUERY_RADIUS, 64.0);
        assert!(validate_spatial_query(center, 64.0, 0.0, Some("minecraft:zombie")).is_ok());
        assert_eq!(
            validate_spatial_query(center, 64.01, 1.0, None)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_spatial_query(center, f64::INFINITY, 1.0, None)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_spatial_query(center, 1.0, 1.0, Some("Minecraft:Zombie"))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_spatial_query(
                BlockPos {
                    x: 30_000_001,
                    y: 0,
                    z: 0
                },
                1.0,
                1.0,
                None
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            within(
                WorldHandle::new(1, 2, 3, 4).unwrap(),
                center,
                1.0,
                1.0,
                None
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn spatial_query_and_results_match_the_versioned_host_layout() {
        let query = SpatialQuery {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            center: BlockPos {
                x: -12,
                y: 64,
                z: 33,
            },
            horizontal_radius: 8.5,
            vertical_radius: 4.0,
            entity_type_id: Some("minecraft:zombie".to_owned()),
        };
        let encoded = crate::to_bytes(&query).unwrap();
        assert_eq!(encoded.len(), 61 + 4 + "minecraft:zombie".len());
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[28..32].try_into().unwrap()), 1);
        assert_eq!(crate::from_bytes::<SpatialQuery>(&encoded).unwrap(), query);

        let results = SpatialResults {
            handles: vec![EntityHandle::new(1, 2, 3, 4).unwrap()],
        };
        let encoded_results = crate::to_bytes(&results).unwrap();
        assert_eq!(encoded_results.len(), 8 + 24);
        assert_eq!(
            u32::from_le_bytes(encoded_results[4..8].try_into().unwrap()),
            1
        );
        assert_eq!(
            crate::from_bytes::<SpatialResults>(&encoded_results).unwrap(),
            results
        );
    }

    #[test]
    fn entity_raycast_bounds_and_permissions_fail_closed_before_host_access() {
        let start = RaycastPoint::new(0.0, 64.0, 0.0);
        let end = RaycastPoint::new(10.0, 64.0, 0.0);
        assert_eq!(MAX_ENTITY_RAYCAST_DISTANCE, 128.0);
        assert!(validate_entity_raycast(start, end, Some("minecraft:zombie")).is_ok());
        for (from, to, filter) in [
            (start, start, None),
            (start, RaycastPoint::new(128.01, 64.0, 0.0), None),
            (RaycastPoint::new(f64::NAN, 64.0, 0.0), end, None),
            (start, end, Some("Minecraft:Zombie")),
        ] {
            assert_eq!(
                validate_entity_raycast(from, to, filter)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            raycast(WorldHandle::new(1, 2, 3, 4).unwrap(), start, end, None,)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn entity_raycast_codec_matches_host_schema_and_optional_handle_response() {
        let query = EntityRaycastQuery {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            start: RaycastPoint::new(0.5, 64.0, -3.0),
            end: RaycastPoint::new(12.0, 66.0, -3.0),
            entity_type_id: Some("minecraft:zombie".to_owned()),
        };
        let encoded = crate::to_bytes(&query).unwrap();
        assert_eq!(encoded.len(), 85 + 4 + "minecraft:zombie".len());
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[28..32].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[56..60].try_into().unwrap()), 1);
        assert_eq!(
            crate::from_bytes::<EntityRaycastQuery>(&encoded).unwrap(),
            query
        );

        let hit = EntityRaycastResult {
            handle: Some(EntityHandle::new(1, 2, 3, 4).unwrap()),
        };
        let encoded_hit = crate::to_bytes(&hit).unwrap();
        assert_eq!(encoded_hit.len(), 4 + 1 + 24);
        assert_eq!(
            crate::from_bytes::<EntityRaycastResult>(&encoded_hit).unwrap(),
            hit
        );
        let miss = EntityRaycastResult { handle: None };
        let encoded_miss = crate::to_bytes(&miss).unwrap();
        assert_eq!(encoded_miss.len(), 4 + 1);
        assert_eq!(
            crate::from_bytes::<EntityRaycastResult>(&encoded_miss).unwrap(),
            miss
        );
    }

    #[test]
    fn entity_target_response_matches_the_host_optional_handle_schema() {
        assert_eq!(MAX_ENTITY_TARGET_RESPONSE_BYTES, 4 + 1 + 24);
        let target = EntityTargetResult {
            target: Some(EntityHandle::new(1, 2, 3, 4).unwrap()),
        };
        let encoded_target = to_bytes(&target).unwrap();
        assert_eq!(encoded_target.len(), MAX_ENTITY_TARGET_RESPONSE_BYTES);
        assert_eq!(
            u32::from_le_bytes(encoded_target[0..4].try_into().unwrap()),
            1
        );
        assert_eq!(encoded_target[4], 1);
        assert_eq!(
            &encoded_target[5..],
            &to_bytes(&target.target.unwrap()).unwrap()[..]
        );
        assert_eq!(
            crate::from_bytes::<EntityTargetResult>(&encoded_target).unwrap(),
            target
        );

        let absent = EntityTargetResult { target: None };
        let encoded_absent = to_bytes(&absent).unwrap();
        assert_eq!(
            encoded_absent,
            [1_u32.to_le_bytes().as_slice(), &[0]].concat()
        );
        assert_eq!(
            crate::from_bytes::<EntityTargetResult>(&encoded_absent).unwrap(),
            absent
        );
    }

    #[test]
    fn entity_teleport_request_matches_the_fixed_host_schema_and_bounds_coordinates() {
        assert_eq!(MAX_ENTITY_TELEPORT_COORDINATE, 30_000_000.0);
        assert!(validate_teleport_target(0.0, -64.0, MAX_ENTITY_TELEPORT_COORDINATE).is_ok());
        for (x, y, z) in [
            (f64::NAN, 0.0, 0.0),
            (0.0, f64::INFINITY, 0.0),
            (0.0, 0.0, f64::NEG_INFINITY),
            (MAX_ENTITY_TELEPORT_COORDINATE + 1.0, 0.0, 0.0),
            (0.0, -MAX_ENTITY_TELEPORT_COORDINATE - 1.0, 0.0),
        ] {
            assert_eq!(
                validate_teleport_target(x, y, z).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }

        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        let request = EntityTeleportRequest {
            entity: handle,
            x: 1.25,
            y: 64.0,
            z: -32.5,
        };
        let encoded = to_bytes(&request).unwrap();
        assert_eq!(encoded.len(), 4 + 24 + 3 * 8);
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(&encoded[4..28], &to_bytes(&handle).unwrap());
        assert_eq!(
            f64::from_le_bytes(encoded[28..36].try_into().unwrap()),
            request.x
        );
        assert_eq!(
            f64::from_le_bytes(encoded[36..44].try_into().unwrap()),
            request.y
        );
        assert_eq!(
            f64::from_le_bytes(encoded[44..52].try_into().unwrap()),
            request.z
        );
        assert_eq!(
            crate::from_bytes::<EntityTeleportRequest>(&encoded)
                .unwrap()
                .x,
            request.x
        );

        assert_eq!(
            teleport(handle, f64::NAN, 0.0, 0.0).unwrap_err().code(),
            ErrorCode::InvalidArgument,
            "invalid target coordinates must fail before the host import"
        );
    }

    #[test]
    fn target_accessors_fail_closed_without_a_wasm_host() {
        let mob = EntityHandle::new(1, 2, 3, 4).unwrap();
        let target_handle = EntityHandle::new(1, 2, 4, 4).unwrap();
        assert_eq!(target(mob).unwrap_err().code(), ErrorCode::UnknownErrorCode);
        assert_eq!(
            target_snapshot(mob).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_target(mob, target_handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            super::target_handle(mob).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            clear_target(mob).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn target_snapshot_contract_is_detached_bounded_and_versioned() {
        assert_eq!(MAX_ENTITY_TARGET_RANGE, 128.0);
        assert_eq!(MAX_ENTITY_TARGET_UUID_BYTES, 36);
        assert_eq!(MAX_ENTITY_TARGET_ID_BYTES, 256);
        assert_eq!(MAX_ENTITY_TARGET_COORDINATE, 30_000_000.0);
        assert_eq!(MAX_ENTITY_TARGET_SNAPSHOT_RESPONSE_BYTES, 593);

        let target = EntityTargetSnapshot::new(
            "00000000-0000-0000-0000-000000000001",
            "minecraft:pig",
            "minecraft:overworld",
            1.25,
            64.0,
            -2.5,
        )
        .unwrap();
        let response = EntityTargetSnapshotResult {
            target: Some(target.clone()),
        };
        let encoded = crate::to_bytes(&response).unwrap();
        assert_eq!(
            encoded.len(),
            4 + 1
                + 4
                + (4 + target.uuid.len())
                + (4 + target.entity_type_id.len())
                + (4 + target.dimension_id.len())
                + 24
        );
        assert_eq!(
            crate::from_bytes::<EntityTargetSnapshotResult>(&encoded).unwrap(),
            response
        );

        let absent = crate::to_bytes(&EntityTargetSnapshotResult { target: None }).unwrap();
        assert_eq!(absent, [1_u32.to_le_bytes().as_slice(), &[0]].concat());
        assert_eq!(
            crate::from_bytes::<EntityTargetSnapshotResult>(&absent)
                .unwrap()
                .target,
            None
        );
    }

    #[test]
    fn target_snapshot_constructor_rejects_noncanonical_or_unbounded_fields() {
        for (uuid, entity_type_id, dimension_id, x, y, z) in [
            (
                "00000000-0000-0000-0000-00000000000A",
                "minecraft:pig",
                "minecraft:overworld",
                0.0,
                0.0,
                0.0,
            ),
            (
                "00000000-0000-0000-0000-000000000001",
                "Minecraft:pig",
                "minecraft:overworld",
                0.0,
                0.0,
                0.0,
            ),
            (
                "00000000-0000-0000-0000-000000000001",
                "minecraft:pig",
                "bad dimension",
                0.0,
                0.0,
                0.0,
            ),
            (
                "00000000-0000-0000-0000-000000000001",
                "minecraft:pig",
                "minecraft:overworld",
                f64::NAN,
                0.0,
                0.0,
            ),
            (
                "00000000-0000-0000-0000-000000000001",
                "minecraft:pig",
                "minecraft:overworld",
                0.0,
                0.0,
                30_000_001.0,
            ),
        ] {
            assert_eq!(
                EntityTargetSnapshot::new(uuid, entity_type_id, dimension_id, x, y, z)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn target_snapshot_response_rejects_noncanonical_host_fields() {
        let invalid = EntityTargetSnapshot {
            uuid: "00000000-0000-0000-0000-00000000000A".to_owned(),
            entity_type_id: "minecraft:pig".to_owned(),
            dimension_id: "minecraft:overworld".to_owned(),
            x: 0.0,
            y: 64.0,
            z: 0.0,
        };
        let encoded = crate::to_bytes(&EntityTargetSnapshotResult {
            target: Some(invalid),
        })
        .unwrap();
        let decoded = crate::from_bytes::<EntityTargetSnapshotResult>(&encoded).unwrap();
        assert_eq!(
            decoded.target.unwrap().validate().unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn effect_application_validates_the_identifier_and_duration_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            apply_effect(handle, "RubyPack:power", 200, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            apply_effect(handle, "ruby_pack:content:power", 200, 0)
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
    fn effect_removal_validates_the_identifier_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            remove_effect(handle, "RubyPack:power").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            remove_effect(handle, "ruby_pack:content:power")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            remove_effect(handle, "ruby_pack:content/power")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn entity_velocity_rejects_non_finite_and_out_of_range_components() {
        let invalid = [
            (f64::NAN, 0.0, 0.0),
            (0.0, f64::INFINITY, 0.0),
            (0.0, 0.0, f64::NEG_INFINITY),
            (MAX_ENTITY_VELOCITY_COMPONENT + 0.000_001, 0.0, 0.0),
            (0.0, -MAX_ENTITY_VELOCITY_COMPONENT - 1.0, 0.0),
        ];
        for (x, y, z) in invalid {
            assert_eq!(
                EntityVelocity::new(x, y, z).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }

        assert_eq!(
            EntityVelocity::new(64.0, -64.0, 0.0).unwrap(),
            EntityVelocity {
                x: 64.0,
                y: -64.0,
                z: 0.0,
            }
        );
        let invalid = EntityVelocity {
            x: 0.0,
            y: f64::NAN,
            z: 0.0,
        };
        assert_eq!(
            set_velocity(EntityHandle::new(1, 2, 3, 4).unwrap(), invalid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "invalid values must fail before any host import is attempted"
        );
    }

    #[test]
    fn navigation_bounds_and_wire_request_match_the_host_schema() {
        assert_eq!(MAX_ENTITY_NAVIGATION_RANGE, 32.0);
        assert_eq!(MAX_ENTITY_NAVIGATION_SPEED, 4.0);
        assert_eq!(NAVIGATION_WRITE_CAPABILITY, "entity.navigation.write");
        let target = BlockPos {
            x: -30_000_000,
            y: -64,
            z: 30_000_000,
        };
        assert!(validate_navigation_target(target, 0.01).is_ok());
        assert!(validate_navigation_target(target, MAX_ENTITY_NAVIGATION_SPEED).is_ok());

        for invalid in [
            (
                BlockPos {
                    x: 30_000_001,
                    y: 0,
                    z: 0,
                },
                1.0,
            ),
            (
                BlockPos {
                    x: 0,
                    y: 0,
                    z: -30_000_001,
                },
                1.0,
            ),
            (BlockPos { x: 0, y: 0, z: 0 }, 0.0),
            (BlockPos { x: 0, y: 0, z: 0 }, 4.01),
            (BlockPos { x: 0, y: 0, z: 0 }, f32::NAN),
            (BlockPos { x: 0, y: 0, z: 0 }, f32::INFINITY),
        ] {
            assert_eq!(
                validate_navigation_target(invalid.0, invalid.1)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }

        let request = EntityNavigationRequest {
            entity: EntityHandle::new(1, 2, 3, 4).unwrap(),
            x: 8,
            y: 64,
            z: -3,
            speed: 1.25,
        };
        let encoded = to_bytes(&request).unwrap();
        assert_eq!(encoded.len(), 44);
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(i32::from_le_bytes(encoded[28..32].try_into().unwrap()), 8);
        assert_eq!(i32::from_le_bytes(encoded[32..36].try_into().unwrap()), 64);
        assert_eq!(i32::from_le_bytes(encoded[36..40].try_into().unwrap()), -3);
        assert_eq!(
            f32::from_le_bytes(encoded[40..44].try_into().unwrap()),
            1.25
        );

        let result_bytes = [1_u32.to_le_bytes().as_slice(), &[1]].concat();
        assert!(
            crate::from_bytes::<EntityNavigationResult>(&result_bytes)
                .unwrap()
                .accepted
        );
        let rejected_bytes = [1_u32.to_le_bytes().as_slice(), &[0]].concat();
        assert!(
            !crate::from_bytes::<EntityNavigationResult>(&rejected_bytes)
                .unwrap()
                .accepted
        );
    }

    #[test]
    fn navigation_rejects_invalid_values_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            navigate_to(handle, BlockPos { x: 0, y: 0, z: 0 }, f32::NAN)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            navigate_to(handle, BlockPos { x: 0, y: 0, z: 0 }, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn generic_damage_is_finite_positive_and_bounded_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(MAX_ENTITY_DAMAGE, 64.0);
        assert_eq!(
            damage(handle, 0.0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            damage(handle, -0.5).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            damage(handle, f32::NAN).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            damage(handle, f32::INFINITY).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            damage(handle, MAX_ENTITY_DAMAGE + 0.01).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            damage(handle, 1.0).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn entity_fire_tick_setter_enforces_the_supported_range_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(MAX_ENTITY_FIRE_TICKS, 72_000);
        assert_eq!(
            set_fire_ticks(handle, MAX_ENTITY_FIRE_TICKS + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_fire_ticks(handle, MAX_ENTITY_FIRE_TICKS)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "the inclusive upper bound should reach the WASM-only import on native targets"
        );
        assert_eq!(
            fire_ticks(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn custom_name_bounds_and_wire_response_are_stable() {
        let name = "😀".repeat(MAX_ENTITY_CUSTOM_NAME_CODE_POINTS);
        assert_eq!(name.len(), MAX_ENTITY_CUSTOM_NAME_BYTES);
        assert_eq!(MAX_ENTITY_CUSTOM_NAME_RESPONSE_BYTES, 1033);
        assert!(validate_custom_name(&name, false).is_ok());
        assert_eq!(
            validate_custom_name(&"😀".repeat(MAX_ENTITY_CUSTOM_NAME_CODE_POINTS + 1), false)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            validate_custom_name("", false).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_custom_name("bad\nname", false).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );

        let response = EntityCustomNameResponse {
            name: Some("name".to_owned()),
        };
        let encoded = crate::to_bytes(&response).unwrap();
        assert_eq!(&encoded[..4], &1_u32.to_le_bytes());
        assert_eq!(
            encoded[4], 1,
            "the optional string uses the stable option tag"
        );
        assert_eq!(
            crate::from_bytes::<EntityCustomNameResponse>(&encoded).unwrap(),
            response
        );
    }

    #[test]
    fn custom_name_requires_literal_safe_bounded_input_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            set_custom_name(handle, "plain § text").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_custom_name(handle, &"x".repeat(MAX_ENTITY_CUSTOM_NAME_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            custom_name(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            clear_custom_name(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
