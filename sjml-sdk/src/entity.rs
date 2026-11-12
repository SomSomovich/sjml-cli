//! Reads of loaded non-player entities and capability-gated actions on the owning logical server.
//! This global API takes no world handle. Calls are synchronous on the server
//! thread. Handles are module/context and live-entity-instance scoped and expire when their entity
//! leaves its level, its world unloads, or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::BlockPos;
use crate::world::RaycastPoint;
#[cfg(target_arch = "wasm32")]
use crate::{to_bytes, Decoder, MAX_STRING_BYTES};
use crate::{
    EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode, WorldHandle,
};

const MAX_EFFECT_ID_BYTES: usize = 256;
pub(crate) const MAX_EFFECT_DURATION_TICKS: u32 = 72_000;
/// Maximum absolute entity velocity component in blocks per tick.
pub const MAX_ENTITY_VELOCITY_COMPONENT: f64 = 64.0;
/// Maximum damage amount accepted by one [`damage`] call.
pub const MAX_ENTITY_DAMAGE: f32 = 64.0;
/// Maximum active fire duration that an entity fire-tick API call accepts or returns.
pub const MAX_ENTITY_FIRE_TICKS: u32 = 72_000;

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
    #[link_name = "is_glowing"]
    fn imported_is_glowing(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_glowing"]
    fn imported_set_glowing(handle: i32, handle_length: i32, glowing: i32) -> i32;
    #[link_name = "is_silent"]
    fn imported_is_silent(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_silent"]
    fn imported_set_silent(handle: i32, handle_length: i32, silent: i32) -> i32;
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
        apply_effect, damage, fire_ticks, raycast, remove_effect, set_fire_ticks, set_velocity,
        validate_entity_raycast, validate_spatial_query, within, EntityRaycastQuery,
        EntityRaycastResult, EntityVelocity, SpatialQuery, SpatialResults, MAX_ENTITY_DAMAGE,
        MAX_ENTITY_FIRE_TICKS, MAX_ENTITY_RAYCAST_DISTANCE, MAX_ENTITY_VELOCITY_COMPONENT,
        MAX_SPATIAL_QUERY_RADIUS, MAX_SPATIAL_QUERY_RESULTS,
    };
    use crate::{inventory::BlockPos, world::RaycastPoint, EntityHandle, ErrorCode, WorldHandle};

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
}
