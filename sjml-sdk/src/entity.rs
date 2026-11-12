//! Reads of loaded non-player entities and capability-gated actions on the owning logical server.
//! This global API takes no world handle. Calls are synchronous on the server
//! thread. Handles are module/context and live-entity-instance scoped and expire when their entity
//! leaves its level, its world unloads, or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::BlockPos;
use crate::world::RaycastPoint;
#[cfg(any(target_arch = "wasm32", test))]
use crate::Decoder;
#[cfg(target_arch = "wasm32")]
use crate::{to_bytes, MAX_STRING_BYTES};
use crate::{
    EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode, WorldHandle,
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
/// Maximum damage amount accepted by one [`damage`] call.
pub const MAX_ENTITY_DAMAGE: f32 = 64.0;
/// Maximum active fire duration that an entity fire-tick API call accepts or returns.
pub const MAX_ENTITY_FIRE_TICKS: u32 = 72_000;
/// Capability required, together with `entity.read`, to read or control a loaded Mob's AI state.
pub const AI_CONTROL_CAPABILITY: &str = "entity.ai.control";
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
struct ActiveEffectsResponse {
    effects: Vec<ActiveEffect>,
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
    #[link_name = "riding_snapshot"]
    fn imported_riding_snapshot(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
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
    #[link_name = "is_glowing"]
    fn imported_is_glowing(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_glowing"]
    fn imported_set_glowing(handle: i32, handle_length: i32, glowing: i32) -> i32;
    #[link_name = "is_silent"]
    fn imported_is_silent(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_silent"]
    fn imported_set_silent(handle: i32, handle_length: i32, silent: i32) -> i32;
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
    #[link_name = "set_velocity"]
    fn imported_set_velocity(
        handle: i32,
        handle_length: i32,
        velocity: i32,
        velocity_length: i32,
    ) -> i32;
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
/// `entity.state.write`. Normal Minecraft entity data synchronization still
/// controls whether connected clients see the outline.
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
/// `entity.state.write`. Minecraft's sound engine and entity tracking remain
/// responsible for client-side behavior.
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
        active_effects, apply_effect, clear_custom_name, custom_name, damage, fire_ticks, raycast,
        remove_effect, set_custom_name, set_fire_ticks, set_velocity, validate_active_effects,
        validate_custom_name, validate_entity_raycast, validate_spatial_query, within,
        ActiveEffect, ActiveEffectsResponse, EntityCustomNameResponse, EntityRaycastQuery,
        EntityRaycastResult, EntityVelocity, SpatialQuery, SpatialResults,
        MAX_ACTIVE_ENTITY_EFFECTS, MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES,
        MAX_ENTITY_CUSTOM_NAME_BYTES, MAX_ENTITY_CUSTOM_NAME_CODE_POINTS,
        MAX_ENTITY_CUSTOM_NAME_RESPONSE_BYTES, MAX_ENTITY_DAMAGE, MAX_ENTITY_FIRE_TICKS,
        MAX_ENTITY_RAYCAST_DISTANCE, MAX_ENTITY_VELOCITY_COMPONENT, MAX_SPATIAL_QUERY_RADIUS,
        MAX_SPATIAL_QUERY_RESULTS,
    };
    use crate::{
        inventory::BlockPos, to_bytes, world::RaycastPoint, EntityHandle, ErrorCode, WorldHandle,
    };

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
