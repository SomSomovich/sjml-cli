//! Worlds on this module's owning logical server. Reads require requested and granted `world.read`;
//! block mutations and particle spawning additionally require `world.write`. No call loads a
//! dimension or chunk. Client instances expose no server worlds. Handles are module-scoped and
//! limited to 1,024 live worlds.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::inventory::{BlockPos, Direction};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
#[cfg(target_arch = "wasm32")]
use crate::Decoder;
use crate::{
    ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode, WorldHandle, MAX_STRING_BYTES,
};

const MAX_RESOURCE_ID_BYTES: usize = 256;

/// Maximum number of positions accepted by one batched block-state query.
pub const MAX_WORLD_BLOCK_STATES: usize = 1024;
/// Maximum length of one raycast segment in blocks.
pub const MAX_WORLD_RAY_DISTANCE: f64 = 128.0;
/// Maximum absolute coordinate accepted for a raycast endpoint or hit location.
pub const MAX_WORLD_RAY_COORDINATE: f64 = 30_000_000.0;
/// Maximum encoded optional raycast hit, including the stable snapshot headers.
pub const MAX_WORLD_RAYCAST_RESPONSE_BYTES: usize = 65_857;
/// Maximum number of block positions visited by one collision query.
pub const MAX_WORLD_COLLISION_BLOCKS: usize = 4096;
/// Maximum bytes in one canonical entity registry ID accepted by [`spawn_entity`].
pub const MAX_WORLD_ENTITY_TYPE_ID_BYTES: usize = 256;

/// Immutable, detached block identity and state value returned by [`block_states`].
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockStateSnapshot {
    pub block_id: String,
    pub state: String,
}

/// Finite world-space point used by [`raycast`].
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct RaycastPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl RaycastPoint {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Minecraft block collision shape used by a raycast.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum RaycastBlockMode {
    Collider,
    Outline,
    Visual,
}

/// Fluid collision behavior used by a raycast.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum RaycastFluidMode {
    None,
    SourceOnly,
    Any,
}

/// Detached hit data returned by [`raycast`].
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct RaycastHit {
    pub position: BlockPos,
    pub face: Direction,
    pub location: RaycastPoint,
    pub block: BlockStateSnapshot,
}

/// Axis-aligned world-space box used by [`collides`].
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    pub const fn new(min: RaycastPoint, max: RaycastPoint) -> Self {
        Self {
            min_x: min.x,
            min_y: min.y,
            min_z: min.z,
            max_x: max.x,
            max_y: max.y,
            max_z: max.z,
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct BlockStateQuery {
    world: WorldHandle,
    positions: Vec<BlockPos>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockStateBatch {
    states: Vec<Option<BlockStateSnapshot>>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldRaycastQuery {
    world: WorldHandle,
    start_x: f64,
    start_y: f64,
    start_z: f64,
    end_x: f64,
    end_y: f64,
    end_z: f64,
    block_mode: u8,
    fluid_mode: u8,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldCollisionQuery {
    world: WorldHandle,
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldEntitySpawnRequest {
    world: WorldHandle,
    entity_type_id: String,
    x: f64,
    y: f64,
    z: f64,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_world_v1")]
extern "C" {
    #[link_name = "get"]
    fn imported_get(id: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "loaded"]
    fn imported_loaded(output: i32, capacity: i32) -> i32;
    #[link_name = "dimension_id"]
    fn imported_dimension_id(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "game_time"]
    fn imported_game_time(handle: i32, length: i32, output: i32) -> i32;
    #[link_name = "block_states"]
    fn imported_block_states(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "raycast"]
    fn imported_raycast(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "collides"]
    fn imported_collides(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "spawn_entity"]
    fn imported_spawn_entity(query: i32, query_length: i32) -> i32;
    #[link_name = "set_block"]
    fn imported_set_block(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        block_id: i32,
        block_id_length: i32,
    ) -> i32;
    #[link_name = "spawn_particle"]
    fn imported_spawn_particle(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        particle_id: i32,
        particle_id_length: i32,
    ) -> i32;
    #[link_name = "play_sound"]
    fn imported_play_sound(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        sound_id: i32,
        sound_id_length: i32,
        volume_bits: i32,
        pitch_bits: i32,
    ) -> i32;
}

/// Finds an already loaded dimension by canonical `namespace:path` ID. Unknown/unloaded IDs return
/// `None`. Before a dedicated server is bound (including early `mod_init`), returns `None`.
pub fn get(dimension_id: &str) -> SjmlResult<Option<WorldHandle>> {
    validate_dimension_id(dimension_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        read_output(25, |output, capacity| {
            // SAFETY: the borrowed UTF-8 ID and allocated output stay live for this synchronous call.
            unsafe {
                imported_get(
                    dimension_id.as_ptr() as usize as u32 as i32,
                    dimension_id.len() as i32,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Returns all loaded dimensions, including modded ones, sorted by canonical dimension ID.
/// Shares handles with `get`. Returns `HandleTableFull` instead of a partial list at the quota.
/// Returns an empty list before server binding and in client instances.
pub fn loaded() -> SjmlResult<Vec<WorldHandle>> {
    #[cfg(target_arch = "wasm32")]
    {
        read_output(4 + 24 * 1024, |output, capacity| {
            // SAFETY: read_output owns the live destination allocation throughout this call.
            unsafe { imported_loaded(output, capacity) }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Reads the canonical dimension ID after validating the handle's owner, context and capability.
pub fn dimension_id(handle: WorldHandle) -> SjmlResult<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output((4 + MAX_STRING_BYTES) as u32, |output, capacity| {
            // SAFETY: the stable handle encoding and output remain live for the synchronous call.
            unsafe {
                imported_dimension_id(
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

/// Reads this dimension's current signed 64-bit game time on the owning server thread.
pub fn game_time(handle: WorldHandle) -> SjmlResult<i64> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output(8, |output, _| {
            // SAFETY: the stable handle encoding and 8-byte output remain live for this call.
            unsafe { imported_game_time(encoded.as_ptr() as usize as u32 as i32, 24, output) }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads up to 1,024 block positions in one `world.read` host call. Unloaded chunks and positions
/// outside the world's build height are returned as `None`; the host never loads a chunk. Results
/// preserve input order, including duplicate positions.
pub fn block_states(
    world: WorldHandle,
    positions: &[BlockPos],
) -> SjmlResult<Vec<Option<BlockStateSnapshot>>> {
    if positions.len() > MAX_WORLD_BLOCK_STATES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "world block query exceeds the 1024-position limit",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&BlockStateQuery {
            world,
            positions: positions.to_vec(),
        })?;
        // Reserve the largest valid response for this count, capped by the ABI batch limit. This
        // keeps a small read from zeroing a full MiB while preserving enough room for any reply.
        let maximum_entry_bytes = 1 + 4 + 4 + MAX_STRING_BYTES + 4 + MAX_STRING_BYTES;
        let capacity =
            (2 * 4 + positions.len() * maximum_entry_bytes).min(crate::MAX_PAYLOAD_BYTES) as u32;
        let response: BlockStateBatch = read_output(capacity, |output, capacity| {
            // SAFETY: the encoded request and output allocation remain live for this synchronous import.
            unsafe {
                imported_block_states(
                    query.as_ptr() as usize as u32 as i32,
                    query.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if response.states.len() != positions.len() {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a different number of block states than requested",
            ));
        }
        Ok(response.states)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, positions);
        Err(unsupported_target())
    }
}

/// Clips a bounded ray against one loaded dimension. The module must request and receive
/// `world.read`. Endpoints must be finite and within ±30,000,000, and their distance cannot
/// exceed 128 blocks. The host checks every intersected chunk first and returns `Ok(None)` if any
/// part of the ray crosses an unloaded chunk; it never requests or generates chunks.
pub fn raycast(
    world: WorldHandle,
    start: RaycastPoint,
    end: RaycastPoint,
    block_mode: RaycastBlockMode,
    fluid_mode: RaycastFluidMode,
) -> SjmlResult<Option<RaycastHit>> {
    validate_raycast(start, end)?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldRaycastQuery {
            world,
            start_x: start.x,
            start_y: start.y,
            start_z: start.z,
            end_x: end.x,
            end_y: end.y,
            end_z: end.z,
            block_mode: block_mode as u8,
            fluid_mode: fluid_mode as u8,
        })?;
        let result: Option<RaycastHit> = read_output(
            MAX_WORLD_RAYCAST_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the versioned query and bounded response allocation stay live for this call.
                unsafe {
                    imported_raycast(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if result.as_ref().is_some_and(|hit| {
            !valid_raycast_point(hit.location)
                || hit.block.block_id.is_empty()
                || hit.block.block_id.len() > 256
                || hit.block.state.is_empty()
                || hit.block.state.len() > MAX_STRING_BYTES
        }) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid world raycast hit",
            ));
        }
        Ok(result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, block_mode, fluid_mode);
        Err(unsupported_target())
    }
}

/// Tests whether an axis-aligned box overlaps any block collision shape in the loaded dimension.
/// Returns `None` when the box intersects an unloaded chunk, leaves build height, or a block shape
/// needs data from an unloaded neighbor. The host visits at most 4,096 block positions and never
/// loads chunks. Bounds must be finite, ordered, and within ±30,000,000 blocks. Requires
/// `world.read`.
pub fn collides(world: WorldHandle, bounds: Aabb) -> SjmlResult<Option<bool>> {
    validate_aabb(bounds)?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldCollisionQuery {
            world,
            min_x: bounds.min_x,
            min_y: bounds.min_y,
            min_z: bounds.min_z,
            max_x: bounds.max_x,
            max_y: bounds.max_y,
            max_z: bounds.max_z,
        })?;
        read_output(2, |output, capacity| {
            // SAFETY: the encoded request and two-byte optional-bool output remain live for the call.
            unsafe {
                imported_collides(
                    query.as_ptr() as usize as u32 as i32,
                    query.len() as i32,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Spawns one registered entity type in an already loaded world position.
///
/// The module must request and receive `world.read` and the dedicated
/// `entity.spawn` administrator capability. Spawning is server-only, bounded
/// to eight entities per loaded dimension per server tick, and never loads a
/// chunk. The position must be finite, within build height, inside the world
/// border, and in a loaded chunk. `entity_type_id` must be a registered
/// canonical lowercase `namespace:path` ID. The server-side host rejects
/// player entities.
pub fn spawn_entity(
    world: WorldHandle,
    entity_type_id: &str,
    position: RaycastPoint,
) -> SjmlResult<()> {
    validate_entity_type_id(entity_type_id)?;
    if !valid_raycast_point(position) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity spawn coordinates must be finite and within ±30,000,000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldEntitySpawnRequest {
            world,
            entity_type_id: entity_type_id.to_owned(),
            x: position.x,
            y: position.y,
            z: position.z,
        })?;
        // SAFETY: the encoded request remains allocated and readable for this synchronous import.
        let status = unsafe {
            imported_spawn_entity(query.as_ptr() as usize as u32 as i32, query.len() as i32)
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the entity spawn",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

fn validate_raycast(start: RaycastPoint, end: RaycastPoint) -> SjmlResult<()> {
    if !valid_raycast_point(start) || !valid_raycast_point(end) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "raycast endpoints must be finite and within ±30,000,000",
        ));
    }
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let dz = end.z - start.z;
    if dx * dx + dy * dy + dz * dz > MAX_WORLD_RAY_DISTANCE * MAX_WORLD_RAY_DISTANCE {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world raycast distance cannot exceed 128 blocks",
        ));
    }
    Ok(())
}

fn validate_aabb(bounds: Aabb) -> SjmlResult<()> {
    let coordinates = [
        bounds.min_x,
        bounds.min_y,
        bounds.min_z,
        bounds.max_x,
        bounds.max_y,
        bounds.max_z,
    ];
    if coordinates
        .iter()
        .any(|value| !value.is_finite() || value.abs() > MAX_WORLD_RAY_COORDINATE)
        || bounds.min_x >= bounds.max_x
        || bounds.min_y >= bounds.max_y
        || bounds.min_z >= bounds.max_z
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "collision bounds must be finite, ordered, and within ±30,000,000",
        ));
    }
    let extent = [
        bounds.max_x - bounds.min_x,
        bounds.max_y - bounds.min_y,
        bounds.max_z - bounds.min_z,
    ];
    if extent.iter().any(|length| *length > 16.0) {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "collision bounds cannot exceed 16 blocks on any axis",
        ));
    }
    let first = [
        bounds.min_x.floor(),
        bounds.min_y.floor(),
        bounds.min_z.floor(),
    ];
    let last = [
        previous_float(bounds.max_x).floor(),
        previous_float(bounds.max_y).floor(),
        previous_float(bounds.max_z).floor(),
    ];
    let positions = first
        .into_iter()
        .zip(last)
        .map(|(low, high)| (high - low + 1.0) as usize)
        .try_fold(1_usize, |count, length| count.checked_mul(length));
    if positions.is_none_or(|count| count > MAX_WORLD_COLLISION_BLOCKS) {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "collision bounds exceed the 4096-block query limit",
        ));
    }
    Ok(())
}

fn previous_float(value: f64) -> f64 {
    if value == 0.0 {
        -f64::from_bits(1)
    } else if value > 0.0 {
        f64::from_bits(value.to_bits() - 1)
    } else {
        f64::from_bits(value.to_bits() + 1)
    }
}

fn valid_raycast_point(point: RaycastPoint) -> bool {
    point.x.is_finite()
        && point.y.is_finite()
        && point.z.is_finite()
        && point.x.abs() <= MAX_WORLD_RAY_COORDINATE
        && point.y.abs() <= MAX_WORLD_RAY_COORDINATE
        && point.z.abs() <= MAX_WORLD_RAY_COORDINATE
}

/// Replaces the block at one position with the default state of a registered block.
///
/// The module must request and receive both `world.read` and `world.write`. The world
/// handle is revalidated on the owning server thread. The target must be within build
/// height and its chunk must already be loaded; this operation never loads chunks.
pub fn set_block(world: WorldHandle, position: BlockPos, block_id: &str) -> SjmlResult<()> {
    validate_block_id(block_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        // SAFETY: this synchronous host call copies the 24-byte handle and bounded block ID.
        let status = unsafe {
            imported_set_block(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                position.x,
                position.y,
                position.z,
                block_id.as_ptr() as usize as u32 as i32,
                block_id.len() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the world block mutation",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

/// Spawns one registered simple particle at the center of a loaded block position.
///
/// The module must request and receive `world.read` and `world.write`. The host
/// resolves the world handle on the owning server thread and never loads a chunk.
pub fn spawn_particle(world: WorldHandle, position: BlockPos, particle_id: &str) -> SjmlResult<()> {
    validate_particle_id(particle_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        // SAFETY: the host copies the handle and bounded UTF-8 ID before returning.
        let status = unsafe {
            imported_spawn_particle(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                position.x,
                position.y,
                position.z,
                particle_id.as_ptr() as usize as u32 as i32,
                particle_id.len() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the particle spawn",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

/// Plays a registered sound event at the center of a loaded block position.
///
/// The module must request and receive `world.read` and `world.write`. Volume
/// must be finite and between 0 and 16; pitch must be finite and between 0.5
/// and 2.0. The host uses Minecraft's MASTER sound source and never loads a
/// chunk.
pub fn play_sound(
    world: WorldHandle,
    position: BlockPos,
    sound_id: &str,
    volume: f32,
    pitch: f32,
) -> SjmlResult<()> {
    validate_sound_id(sound_id)?;
    if !volume.is_finite() || !(0.0..=16.0).contains(&volume) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "sound volume must be finite and between 0 and 16",
        ));
    }
    if !pitch.is_finite() || !(0.5..=2.0).contains(&pitch) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "sound pitch must be finite and between 0.5 and 2",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        // SAFETY: the host copies the handle and bounded UTF-8 ID before returning.
        let status = unsafe {
            imported_play_sound(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                position.x,
                position.y,
                position.z,
                sound_id.as_ptr() as usize as u32 as i32,
                sound_id.len() as i32,
                volume.to_bits() as i32,
                pitch.to_bits() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the sound playback",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

fn validate_block_id(id: &str) -> SjmlResult<()> {
    validate_resource_id(id, "block")
}

fn validate_particle_id(id: &str) -> SjmlResult<()> {
    validate_resource_id(id, "particle")
}

fn validate_sound_id(id: &str) -> SjmlResult<()> {
    validate_resource_id(id, "sound")
}

fn validate_entity_type_id(id: &str) -> SjmlResult<()> {
    if id.len() > MAX_WORLD_ENTITY_TYPE_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity type ID exceeds the 256-byte limit",
        ));
    }
    validate_resource_id(id, "entity type")
}

fn validate_resource_id(id: &str, kind: &str) -> SjmlResult<()> {
    if id.len() > MAX_RESOURCE_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            format!("{kind} ID exceeds the 256-byte limit"),
        ));
    }
    let Some((namespace, path)) = id.split_once(':') else {
        return Err(invalid_resource_id(kind));
    };
    let namespace_byte =
        |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte);
    if namespace.is_empty()
        || path.is_empty()
        || !namespace.bytes().all(namespace_byte)
        || !path
            .bytes()
            .all(|byte| namespace_byte(byte) || byte == b'/')
    {
        return Err(invalid_resource_id(kind));
    }
    Ok(())
}

fn invalid_resource_id(kind: &str) -> SjmlError {
    SjmlError::new(
        ErrorCode::InvalidArgument,
        format!("a canonical lowercase namespace:path {kind} ID is required"),
    )
}

fn validate_dimension_id(id: &str) -> SjmlResult<()> {
    if id.len() > MAX_STRING_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "dimension ID exceeds the ABI string limit",
        ));
    }
    let Some((namespace, path)) = id.split_once(':') else {
        return Err(invalid_id());
    };
    let namespace_byte =
        |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte);
    if namespace.is_empty()
        || path.is_empty()
        || !namespace.bytes().all(namespace_byte)
        || !path
            .bytes()
            .all(|byte| namespace_byte(byte) || byte == b'/')
    {
        return Err(invalid_id());
    }
    Ok(())
}

fn invalid_id() -> SjmlError {
    SjmlError::new(
        ErrorCode::InvalidArgument,
        "a canonical namespace:path dimension ID is required",
    )
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
            "could not allocate world output buffer",
        ));
    }
    // SAFETY: initialize the complete allocation so unwritten padding can be borrowed safely.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: the allocation is initialized and remains owned until after stable decoding.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        // Stable values delimit themselves. The remainder is caller-owned output-buffer padding.
        Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the world read",
        ))
    };
    // SAFETY: release this exact allocation once, after host access and decoding have finished.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "world reads require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        collides, play_sound, raycast, set_block, spawn_entity, spawn_particle, Aabb,
        BlockStateBatch, BlockStateQuery, BlockStateSnapshot, RaycastBlockMode, RaycastFluidMode,
        RaycastHit, RaycastPoint, WorldCollisionQuery, WorldEntitySpawnRequest, WorldRaycastQuery,
    };
    use crate::{BlockPos, Direction, ErrorCode, StableDecode, WorldHandle};

    #[test]
    fn raycast_query_matches_the_java_little_endian_contract() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let query = WorldRaycastQuery {
            world,
            start_x: 1.5,
            start_y: 64.25,
            start_z: -3.0,
            end_x: 4.5,
            end_y: 64.25,
            end_z: -3.0,
            block_mode: RaycastBlockMode::Visual as u8,
            fluid_mode: RaycastFluidMode::SourceOnly as u8,
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        for coordinate in [1.5_f64, 64.25, -3.0, 4.5, 64.25, -3.0] {
            expected.extend_from_slice(&coordinate.to_le_bytes());
        }
        expected.extend_from_slice(&[2, 1]);
        assert_eq!(crate::to_bytes(&query).unwrap(), expected);
        assert_eq!(expected.len(), 78);
    }

    #[test]
    fn raycast_response_matches_detached_hit_wire_and_validates_bounds_natively() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let hit = super::RaycastHit {
            position: BlockPos {
                x: 10,
                y: 64,
                z: -3,
            },
            face: Direction::North,
            location: RaycastPoint::new(10.5, 64.25, -3.0),
            block: BlockStateSnapshot {
                block_id: "minecraft:stone".to_owned(),
                state: "minecraft:stone".to_owned(),
            },
        };
        let encoded_hit = crate::to_bytes(&Some(hit.clone())).unwrap();
        assert_eq!(encoded_hit[0], 1);
        assert_eq!(
            RaycastHit::decode(&mut crate::Decoder::new(&encoded_hit[1..]).unwrap()).unwrap(),
            hit
        );

        assert_eq!(
            raycast(
                world,
                RaycastPoint::new(0.0, 0.0, 0.0),
                RaycastPoint::new(f64::NAN, 0.0, 0.0),
                RaycastBlockMode::Collider,
                RaycastFluidMode::None,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            raycast(
                world,
                RaycastPoint::new(0.0, 0.0, 0.0),
                RaycastPoint::new(128.01, 0.0, 0.0),
                RaycastBlockMode::Collider,
                RaycastFluidMode::None,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            raycast(
                world,
                RaycastPoint::new(30_000_001.0, 0.0, 0.0),
                RaycastPoint::new(30_000_001.0, 0.0, 1.0),
                RaycastBlockMode::Collider,
                RaycastFluidMode::None,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            raycast(
                world,
                RaycastPoint::new(0.0, 0.0, 0.0),
                RaycastPoint::new(1.0, 0.0, 0.0),
                RaycastBlockMode::Collider,
                RaycastFluidMode::None,
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn collision_query_matches_the_java_little_endian_contract_and_bounds_work() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let bounds = Aabb::new(
            RaycastPoint::new(-1.5, 63.0, 2.0),
            RaycastPoint::new(0.5, 65.25, 4.0),
        );
        let query = WorldCollisionQuery {
            world,
            min_x: bounds.min_x,
            min_y: bounds.min_y,
            min_z: bounds.min_z,
            max_x: bounds.max_x,
            max_y: bounds.max_y,
            max_z: bounds.max_z,
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        for coordinate in [-1.5_f64, 63.0, 2.0, 0.5, 65.25, 4.0] {
            expected.extend_from_slice(&coordinate.to_le_bytes());
        }
        assert_eq!(crate::to_bytes(&query).unwrap(), expected);
        assert_eq!(expected.len(), 76);
        assert_eq!(
            collides(world, bounds).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            collides(
                world,
                Aabb::new(
                    RaycastPoint::new(0.0, 0.0, 0.0),
                    RaycastPoint::new(f64::INFINITY, 1.0, 1.0),
                ),
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            collides(
                world,
                Aabb::new(
                    RaycastPoint::new(0.0, 0.0, 0.0),
                    RaycastPoint::new(16.1, 1.0, 1.0),
                ),
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            collides(
                world,
                Aabb::new(
                    RaycastPoint::new(0.0, 0.0, 0.0),
                    RaycastPoint::new(0.0, 1.0, 1.0),
                ),
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn batch_query_and_response_match_the_java_little_endian_contract() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let query = BlockStateQuery {
            world,
            positions: vec![BlockPos { x: 1, y: 64, z: 2 }],
        };
        let mut expected_query = Vec::new();
        expected_query.extend_from_slice(&1_u32.to_le_bytes());
        expected_query.extend_from_slice(&1_u64.to_le_bytes());
        expected_query.extend_from_slice(&2_u64.to_le_bytes());
        expected_query.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected_query.extend_from_slice(&1_u32.to_le_bytes());
        expected_query.extend_from_slice(&1_u32.to_le_bytes());
        expected_query.extend_from_slice(&1_i32.to_le_bytes());
        expected_query.extend_from_slice(&64_i32.to_le_bytes());
        expected_query.extend_from_slice(&2_i32.to_le_bytes());
        let encoded_query = crate::to_bytes(&query).unwrap();
        assert_eq!(encoded_query, expected_query);

        let response = BlockStateBatch {
            states: vec![
                Some(BlockStateSnapshot {
                    block_id: "minecraft:stone".to_owned(),
                    state: "minecraft:stone".to_owned(),
                }),
                None,
            ],
        };
        let mut expected_response = Vec::new();
        expected_response.extend_from_slice(&1_u32.to_le_bytes());
        expected_response.extend_from_slice(&2_u32.to_le_bytes());
        expected_response.push(1);
        expected_response.extend_from_slice(&1_u32.to_le_bytes());
        for value in ["minecraft:stone", "minecraft:stone"] {
            expected_response.extend_from_slice(&(value.len() as u32).to_le_bytes());
            expected_response.extend_from_slice(value.as_bytes());
        }
        expected_response.push(0);
        let decoded =
            BlockStateBatch::decode(&mut crate::Decoder::new(&expected_response).unwrap()).unwrap();
        assert_eq!(decoded.states, response.states);
    }

    #[test]
    fn block_mutation_validates_ids_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        for invalid in [
            "stone",
            "Minecraft:stone",
            "minecraft:Bad",
            "minecraft:bad path",
        ] {
            assert_eq!(
                set_block(world, position, invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_block(world, position, "minecraft:stone")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn particle_spawning_validates_ids_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        for invalid in [
            "flame",
            "Minecraft:flame",
            "minecraft:Bad",
            "minecraft:bad path",
        ] {
            assert_eq!(
                spawn_particle(world, position, invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            spawn_particle(world, position, "minecraft:flame")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn sound_playback_validates_id_volume_and_pitch_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        for invalid in [
            "engine",
            "Minecraft:engine",
            "ruby_pack:Bad",
            "ruby_pack:bad path",
        ] {
            assert_eq!(
                play_sound(world, position, invalid, 1.0, 1.0)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for (volume, pitch) in [
            (-1.0, 1.0),
            (16.1, 1.0),
            (f32::NAN, 1.0),
            (1.0, 0.49),
            (1.0, 2.01),
        ] {
            assert_eq!(
                play_sound(world, position, "ruby_pack:content/engine", volume, pitch)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            play_sound(world, position, "ruby_pack:content/engine", 1.0, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn entity_spawning_validates_type_id_and_position_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = RaycastPoint::new(0.5, 64.0, -1.25);
        for invalid in [
            "pig",
            "Minecraft:pig",
            "minecraft:Bad",
            "minecraft:bad path",
        ] {
            assert_eq!(
                spawn_entity(world, invalid, position).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            spawn_entity(
                world,
                "minecraft:pig",
                RaycastPoint::new(f64::NAN, 64.0, 0.0)
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            spawn_entity(world, "minecraft:pig", position)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn entity_spawn_request_matches_the_java_little_endian_contract() {
        let request = WorldEntitySpawnRequest {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            entity_type_id: "minecraft:pig".to_owned(),
            x: 1.5,
            y: 64.0,
            z: -2.25,
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&13_u32.to_le_bytes());
        expected.extend_from_slice(b"minecraft:pig");
        expected.extend_from_slice(&1.5_f64.to_le_bytes());
        expected.extend_from_slice(&64.0_f64.to_le_bytes());
        expected.extend_from_slice(&(-2.25_f64).to_le_bytes());
        assert_eq!(crate::to_bytes(&request).unwrap(), expected);
    }
}
