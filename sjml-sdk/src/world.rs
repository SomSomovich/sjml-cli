//! Worlds on this module's owning logical server. Reads require requested and granted `world.read`;
//! world mutations, block mutations, and particle spawning additionally require `world.write`. No call loads a
//! dimension or chunk. Client instances expose no server worlds. Handles are module-scoped and
//! limited to 1,024 live worlds.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input, read_length_prefixed_output};
use crate::inventory::{BlockPos, Direction};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
#[cfg(any(target_arch = "wasm32", test))]
use crate::Decoder;
use crate::{
    ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode, WorldHandle, MAX_STRING_BYTES,
};

const MAX_RESOURCE_ID_BYTES: usize = 256;

/// Maximum number of positions accepted by one batched block-state query.
pub const MAX_WORLD_BLOCK_STATES: usize = 1024;
/// Maximum number of properties returned for a single loaded block state.
pub const MAX_WORLD_BLOCK_PROPERTIES: usize = 64;
/// Maximum number of explicitly named block-state properties changed by one call.
pub const MAX_WORLD_BLOCK_PROPERTY_UPDATES: usize = 32;
/// Maximum encoded bytes in one block-property mutation request.
pub const MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES: usize = 10_932;
/// Maximum output bytes reserved for one block-property snapshot.
pub const MAX_WORLD_BLOCK_PROPERTIES_RESPONSE_BYTES: usize = 21_257;
/// Maximum output bytes reserved for one fluid-state snapshot.
pub const MAX_WORLD_FLUID_STATE_RESPONSE_BYTES: usize = 275;
/// Maximum length of one raycast segment in blocks.
pub const MAX_WORLD_RAY_DISTANCE: f64 = 128.0;
/// Maximum absolute coordinate accepted for a raycast endpoint or hit location.
pub const MAX_WORLD_RAY_COORDINATE: f64 = 30_000_000.0;
/// Maximum absolute block coordinate accepted for x/z world queries.
pub const MAX_WORLD_BLOCK_COORDINATE: i32 = 30_000_000;
/// Highest redstone signal returned by [`best_neighbor_signal`].
pub const MAX_WORLD_REDSTONE_SIGNAL: u8 = 15;
/// Maximum encoded optional raycast hit, including the stable snapshot headers.
pub const MAX_WORLD_RAYCAST_RESPONSE_BYTES: usize = 65_857;
/// Minimum launch speed accepted by [`spawn_projectile`], in blocks per tick.
pub const MIN_WORLD_PROJECTILE_SPEED: f64 = 0.1;
/// Maximum launch speed accepted by [`spawn_projectile`], in blocks per tick.
pub const MAX_WORLD_PROJECTILE_SPEED: f64 = 4.0;
/// Maximum vanilla projectiles one module can spawn per logical-server tick.
pub const MAX_WORLD_PROJECTILES_PER_TICK: usize = 4;
/// Maximum ordinary item stacks one module can spawn per logical-server tick.
pub const MAX_WORLD_ITEM_SPAWNS_PER_TICK: usize = 16;
/// Maximum stack count accepted by [`spawn_item`] before the host checks the item's own limit.
pub const MAX_WORLD_ITEM_STACK_COUNT: u32 = 64;
/// Maximum magnitude accepted for [`ItemSpawnVelocity`], in blocks per tick.
pub const MAX_WORLD_ITEM_SPAWN_SPEED: f64 = 4.0;
/// Maximum number of block positions visited by one collision query.
pub const MAX_WORLD_COLLISION_BLOCKS: usize = 4096;
/// Maximum bytes in one canonical entity registry ID accepted by [`spawn_entity`].
pub const MAX_WORLD_ENTITY_TYPE_ID_BYTES: usize = 256;
/// Maximum duration accepted by [`set_weather`], in ticks (24 in-game hours).
pub const MAX_WEATHER_DURATION_TICKS: u32 = 1_728_000;
/// Maximum absolute world-border center coordinate accepted by the host.
pub const MAX_WORLD_BORDER_CENTER: f64 = 29_999_984.0;
/// Maximum world-border diameter accepted by the host.
pub const MAX_WORLD_BORDER_DIAMETER: f64 = 59_999_968.0;
/// Maximum size transition accepted by the host, in milliseconds.
pub const MAX_WORLD_BORDER_LERP_MILLIS: u64 = 604_800_000;
/// Maximum world-border warning lead time accepted by the host, in seconds.
pub const MAX_WORLD_BORDER_WARNING_SECONDS: u32 = 86_400;
/// Maximum world-border warning distance accepted by the host, in blocks.
pub const MAX_WORLD_BORDER_WARNING_BLOCKS: u32 = 59_999_968;
/// Maximum damage buffer accepted by the host, in blocks.
pub const MAX_WORLD_BORDER_DAMAGE_BUFFER: f64 = 1_024.0;
/// Maximum damage applied per block accepted by the host.
pub const MAX_WORLD_BORDER_DAMAGE_PER_BLOCK: f64 = 100.0;
/// Maximum day-time value accepted by [`set_time`].
pub const MAX_WORLD_DAY_TIME_TICKS: i64 = 1_000_000_000_000;
/// Maximum absolute game-time value accepted by [`set_game_time`].
pub const MAX_WORLD_GAME_TIME_TICKS: i64 = 1_000_000_000_000;
/// Stable ABI-v1 size of a world spawn-point request or response.
pub const MAX_WORLD_SPAWN_POINT_BYTES: usize = 20;
/// Maximum delay accepted for a guest-scheduled vanilla block tick, in ticks.
pub const MAX_WORLD_BLOCK_TICK_DELAY_TICKS: u32 = 72_000;
/// Maximum chunk-radius requested by [`loaded_chunks`], in chunks.
pub const MAX_WORLD_CHUNK_RADIUS: u32 = 16;
/// Maximum number of already-loaded chunks returned by one [`loaded_chunks`] call.
pub const MAX_WORLD_CHUNKS: usize = 512;
/// Maximum stable response size for [`loaded_chunks`].
pub const MAX_WORLD_CHUNKS_RESPONSE_BYTES: usize = 8 + MAX_WORLD_CHUNKS * 12;
/// Maximum number of biome IDs returned by one [`biome_ids`] call.
pub const MAX_WORLD_BIOME_IDS: usize = 4_000;
/// Maximum UTF-8 bytes in one biome ID returned by [`biome_ids`].
pub const MAX_WORLD_BIOME_ID_BYTES: usize = 256;
/// Maximum stable response size for [`biome_ids`].
pub const MAX_WORLD_BIOME_IDS_RESPONSE_BYTES: usize =
    8 + MAX_WORLD_BIOME_IDS * (4 + MAX_WORLD_BIOME_ID_BYTES);
/// Maximum number of structure IDs returned for one already-loaded block position.
pub const MAX_WORLD_STRUCTURES_AT: usize = 128;
/// Maximum UTF-8 bytes in one structure ID returned by [`structures_at`].
pub const MAX_WORLD_STRUCTURE_ID_BYTES: usize = 256;
/// Maximum number of structure references examined for one position.
pub const MAX_WORLD_STRUCTURE_REFERENCE_CHECKS: usize = 4_096;
/// Maximum UTF-8 bytes in one block tag ID accepted by [`block_has_tag`].
pub const MAX_WORLD_BLOCK_TAG_ID_BYTES: usize = 256;
/// Maximum output bytes reserved for [`block_has_tag`].
pub const MAX_WORLD_BLOCK_TAG_RESPONSE_BYTES: usize = 6;
/// Maximum UTF-8 bytes accepted for an item ID or tag ID by [`item_has_tag`].
pub const MAX_WORLD_ITEM_TAG_ID_BYTES: usize = 256;
/// Maximum encoded query size for [`item_has_tag`].
pub const MAX_WORLD_ITEM_TAG_QUERY_BYTES: usize = 4 + 24 + 2 * (4 + MAX_WORLD_ITEM_TAG_ID_BYTES);
/// Maximum output bytes reserved for [`item_has_tag`].
pub const MAX_WORLD_ITEM_TAG_RESPONSE_BYTES: usize = 5;
/// Maximum entries enumerated for one block, item, fluid, or entity-type tag.
pub const MAX_WORLD_TAG_MEMBERS: usize = 1_024;
/// Maximum UTF-8 bytes in one tag or registry entry ID.
pub const MAX_WORLD_TAG_ID_BYTES: usize = 256;
/// Maximum encoded query size for [`tag_members`].
pub const MAX_WORLD_TAG_MEMBERS_QUERY_BYTES: usize = 4 + 24 + 4 + 4 + MAX_WORLD_TAG_ID_BYTES;
/// Maximum encoded response size reserved by [`tag_members`].
pub const MAX_WORLD_TAG_MEMBERS_RESPONSE_BYTES: usize =
    9 + MAX_WORLD_TAG_MEMBERS * (4 + MAX_WORLD_TAG_ID_BYTES);
/// Maximum tag-members import output size, including its four-byte response-length prefix.
pub const MAX_WORLD_TAG_MEMBERS_OUTPUT_BYTES: usize = 4 + MAX_WORLD_TAG_MEMBERS_RESPONSE_BYTES;
/// Maximum stable response size reserved by [`structures_at`].
pub const MAX_WORLD_STRUCTURES_AT_RESPONSE_BYTES: usize =
    9 + MAX_WORLD_STRUCTURES_AT * (4 + MAX_WORLD_STRUCTURE_ID_BYTES);

/// Runtime heightmap used by [`heightmap`]. ABI values are stable for world-v1.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum HeightmapType {
    /// Highest non-air block.
    WorldSurface = 0,
    /// Highest block that blocks motion.
    OceanFloor = 1,
    /// Highest motion-blocking block or fluid.
    MotionBlocking = 2,
    /// Highest motion-blocking block or fluid, excluding leaves.
    MotionBlockingNoLeaves = 3,
}

/// The four vanilla difficulty levels shared by a logical server.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum WorldDifficulty {
    /// Hostile mobs do not spawn naturally and hunger cannot reduce health below 10 HP.
    Peaceful = 0,
    /// Easier hostile-mob damage and starvation cannot reduce health below 10 HP.
    Easy = 1,
    /// The standard survival difficulty.
    Normal = 2,
    /// Hard hostile-mob damage; starvation can be fatal.
    Hard = 3,
}

/// Detached configured spawn coordinates and rotation for the vanilla Overworld.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldSpawnPoint {
    /// Block X coordinate.
    pub x: i32,
    /// Block Y coordinate, checked against the live Overworld's build-height range by the host.
    pub y: i32,
    /// Block Z coordinate.
    pub z: i32,
    /// Spawn yaw in degrees.
    pub angle: f32,
}

impl WorldSpawnPoint {
    /// Creates a spawn point after validating the fixed Minecraft world bounds and finite yaw.
    pub fn new(x: i32, y: i32, z: i32, angle: f32) -> SjmlResult<Self> {
        let point = Self { x, y, z, angle };
        point.validate_request()?;
        Ok(point)
    }

    fn validate_request(self) -> SjmlResult<()> {
        if self.x < -MAX_WORLD_BLOCK_COORDINATE
            || self.x > MAX_WORLD_BLOCK_COORDINATE
            || self.z < -MAX_WORLD_BLOCK_COORDINATE
            || self.z > MAX_WORLD_BLOCK_COORDINATE
            || !self.angle.is_finite()
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "spawn x/z must be within Minecraft's world bound and angle must be finite",
            ));
        }
        Ok(())
    }
}

impl TryFrom<i32> for WorldDifficulty {
    type Error = SjmlError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Peaceful),
            1 => Ok(Self::Easy),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Hard),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an unsupported Minecraft difficulty",
            )),
        }
    }
}

/// Detached sky and emitted block-light levels at a loaded position (each in 0..=15).
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct LightSnapshot {
    /// Sky light. Dimensions without sky lighting usually report zero.
    pub sky: u8,
    /// Block-emitted light.
    pub block: u8,
}

/// Detached weather timers and current precipitation state for one loaded dimension.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WeatherSnapshot {
    /// Whether precipitation is currently active.
    pub raining: bool,
    /// Whether thunder is currently active.
    pub thundering: bool,
    /// Ticks until the current clear-weather period ends, or zero when inactive.
    pub clear_time: u32,
    /// Ticks until the rain state changes, or zero when inactive.
    pub rain_time: u32,
    /// Ticks until the thunder state changes, or zero when inactive.
    pub thunder_time: u32,
}

/// Detached world-border settings for one loaded dimension.
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct WorldBorderSnapshot {
    /// Center X coordinate, in blocks.
    pub center_x: f64,
    /// Center Z coordinate, in blocks.
    pub center_z: f64,
    /// Diameter currently applied, in blocks.
    pub current_diameter: f64,
    /// Transition target diameter, in blocks; equals `current_diameter` when stationary.
    pub target_diameter: f64,
    /// Remaining size-transition time, in milliseconds.
    pub remaining_lerp_millis: u64,
    /// Warning lead time, in seconds.
    pub warning_time_seconds: i32,
    /// Warning distance from the border, in blocks.
    pub warning_distance_blocks: i32,
    /// Safe damage buffer inside the border, in blocks.
    pub damage_buffer_blocks: f64,
    /// Damage applied per block outside the safe buffer.
    pub damage_per_block: f64,
}

impl WorldBorderSnapshot {
    #[cfg(target_arch = "wasm32")]
    fn validate(&self) -> SjmlResult<()> {
        if !self.center_x.is_finite()
            || self.center_x.abs() > MAX_WORLD_BORDER_CENTER
            || !self.center_z.is_finite()
            || self.center_z.abs() > MAX_WORLD_BORDER_CENTER
            || !valid_border_diameter(self.current_diameter)
            || !valid_border_diameter(self.target_diameter)
            || self.warning_time_seconds < 0
            || self.warning_distance_blocks < 0
            || !self.damage_buffer_blocks.is_finite()
            || self.damage_buffer_blocks < 0.0
            || !self.damage_per_block.is_finite()
            || self.damage_per_block < 0.0
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned world border values outside the supported ranges",
            ));
        }
        Ok(())
    }
}

fn valid_border_diameter(diameter: f64) -> bool {
    diameter.is_finite() && (1.0..=MAX_WORLD_BORDER_DIAMETER).contains(&diameter)
}

/// Weather preset used by [`set_weather`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum WeatherCondition {
    Clear = 0,
    Rain = 1,
    Thunder = 2,
}

/// Registry whose data-driven tag members [`tag_members`] enumerates.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum TagRegistry {
    /// Minecraft block registry.
    Block = 0,
    /// Minecraft item registry.
    Item = 1,
    /// Minecraft fluid registry.
    Fluid = 2,
    /// Minecraft entity-type registry.
    EntityType = 3,
}

/// Stable allowlist of boolean vanilla world gamerules. Values are ABI IDs and are not Minecraft
/// registry or implementation ordinals.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum WorldGameRule {
    /// `doDaylightCycle`.
    DaylightCycle = 0,
    /// `doWeatherCycle`.
    WeatherCycle = 1,
    /// `doMobSpawning`.
    MobSpawning = 2,
    /// `mobGriefing`.
    MobGriefing = 3,
    /// `keepInventory`.
    KeepInventory = 4,
    /// `doFireTick`.
    FireTick = 5,
    /// `doMobLoot`.
    MobLoot = 6,
    /// `doTileDrops`.
    BlockDrops = 7,
    /// `doEntityDrops`.
    EntityDrops = 8,
    /// `naturalRegeneration`.
    NaturalRegeneration = 9,
    /// `disableRaids`.
    DisableRaids = 10,
    /// `announceAdvancements`.
    AnnounceAdvancements = 11,
    /// `showDeathMessages`.
    ShowDeathMessages = 12,
    /// `doInsomnia`.
    Insomnia = 13,
    /// `doImmediateRespawn`.
    ImmediateRespawn = 14,
    /// `doPatrolSpawning`.
    PatrolSpawning = 15,
    /// `doTraderSpawning`.
    TraderSpawning = 16,
    /// `doWardenSpawning`.
    WardenSpawning = 17,
}

/// Stable allowlist of vanilla integer gamerules available through [`integer_game_rule`].
/// The IDs are separate from [`WorldGameRule`] and are not Minecraft implementation ordinals.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum WorldIntegerGameRule {
    /// `maxEntityCramming`; write values are 0 through 128.
    MaxEntityCramming = 0,
    /// `playersSleepingPercentage`; write values are 0 through 100.
    PlayersSleepingPercentage = 1,
    /// `randomTickSpeed`; write values are 0 through 4,096.
    RandomTickSpeed = 2,
    /// `spawnRadius`; write values are 0 through 64.
    SpawnRadius = 3,
}

impl WorldIntegerGameRule {
    /// Inclusive lower bound accepted by [`set_integer_game_rule`].
    pub const fn min_value(self) -> i32 {
        0
    }

    /// Inclusive upper bound accepted by [`set_integer_game_rule`].
    pub const fn max_value(self) -> i32 {
        match self {
            Self::MaxEntityCramming => 128,
            Self::PlayersSleepingPercentage => 100,
            Self::RandomTickSpeed => 4096,
            Self::SpawnRadius => 64,
        }
    }
}

/// Immutable, detached block identity and state value returned by [`block_states`].
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockStateSnapshot {
    pub block_id: String,
    pub state: String,
}

/// Detached fluid registry state at one already-loaded block position.
#[derive(Debug, Clone, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct FluidStateSnapshot {
    /// Canonical registered fluid ID such as `minecraft:water`.
    pub fluid_id: String,
    /// Vanilla fluid level in 0..=8; source states normally report 8.
    pub amount: u8,
    /// Whether Minecraft marks this state as a source.
    pub source: bool,
    /// Intrinsic `FluidState::getOwnHeight()` in blocks, bounded to 0..=1.
    pub height: f32,
}

/// One immutable, detached, canonically named property on a loaded block state.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockProperty {
    pub name: String,
    pub value: String,
}

/// Finite world-space point used by [`raycast`].
#[derive(Debug, Clone, Copy, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct RaycastPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// One supported built-in throwable. Vanilla collision behavior is preserved.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum VanillaProjectileKind {
    Snowball = 0,
    Egg = 1,
}

/// Per-tick launch velocity for a vanilla snowball or egg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectileVelocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Bounded initial velocity for one ordinary world-spawned item stack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemSpawnVelocity {
    /// Initial X velocity in blocks per tick.
    pub x: f64,
    /// Initial Y velocity in blocks per tick.
    pub y: f64,
    /// Initial Z velocity in blocks per tick.
    pub z: f64,
}

impl ItemSpawnVelocity {
    /// Creates a velocity value; [`spawn_item`] validates finite components and the 4.0 magnitude bound.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl ProjectileVelocity {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
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

/// Chunk coordinates in a detached loaded-chunk query result.
#[derive(Debug, Clone, Copy, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct ChunkPos {
    /// X coordinate of the chunk.
    pub x: i32,
    /// Z coordinate of the chunk.
    pub z: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct LoadedChunksQuery {
    world: WorldHandle,
    center: BlockPos,
    radius: u32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, StableEncode)]
#[sjml(version = 1)]
struct LoadedChunksResponse {
    chunks: Vec<ChunkPos>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for LoadedChunksResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported loaded-chunk response schema",
            ));
        }
        let count = decoder.read_length(MAX_WORLD_CHUNKS)?;
        let mut chunks = Vec::new();
        chunks.try_reserve_exact(count).map_err(|_| {
            SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not reserve memory for loaded chunk results",
            )
        })?;
        for _ in 0..count {
            chunks.push(ChunkPos::decode(decoder)?);
        }
        Ok(Self { chunks })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct BiomeIdsQuery {
    world: WorldHandle,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, StableEncode)]
#[sjml(version = 1)]
struct BiomeIdsResponse {
    ids: Vec<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct StructuresAtQuery {
    world: WorldHandle,
    position: BlockPos,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct BlockTagQuery {
    world: WorldHandle,
    position: BlockPos,
    tag_id: String,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct ItemTagQuery {
    world: WorldHandle,
    item_id: String,
    tag_id: String,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct TagMembersQuery {
    world: WorldHandle,
    registry_type: u32,
    tag_id: String,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct StructuresAtResponse {
    result: Option<Vec<String>>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for StructuresAtResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported structure-at response schema",
            ));
        }
        let available = match decoder.read_u8()? {
            0 => false,
            1 => true,
            _ => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "structure-at availability flag must be 0 or 1",
                ));
            }
        };
        let count = decoder.read_length(MAX_WORLD_STRUCTURES_AT)?;
        if !available && count != 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "unavailable structure-at response must have an empty result list",
            ));
        }
        let mut ids = Vec::new();
        ids.try_reserve_exact(count).map_err(|_| {
            SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not reserve memory for structure results",
            )
        })?;
        for _ in 0..count {
            let length = decoder.read_length(MAX_WORLD_STRUCTURE_ID_BYTES)?;
            let id = String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
                SjmlError::new(
                    ErrorCode::InvalidUtf8,
                    "structure ID contains malformed UTF-8",
                )
            })?;
            ids.push(id);
        }
        validate_structure_ids(&ids)?;
        Ok(Self {
            result: available.then_some(ids),
        })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct BlockTagResponse {
    result: Option<bool>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for BlockTagResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported block-tag response schema",
            ));
        }
        let available = decoder.read_u8()?;
        let contains = decoder.read_u8()?;
        match (available, contains) {
            (0, 0) => Ok(Self { result: None }),
            (1, 0) => Ok(Self {
                result: Some(false),
            }),
            (1, 1) => Ok(Self { result: Some(true) }),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "block-tag response flags are invalid",
            )),
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct ItemTagResponse {
    contains: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug)]
struct TagMembersResponse {
    members: Option<Vec<String>>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for ItemTagResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported item-tag response schema",
            ));
        }
        let contains = match decoder.read_u8()? {
            0 => false,
            1 => true,
            _ => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "item-tag membership flag must be 0 or 1",
                ));
            }
        };
        Ok(Self { contains })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for TagMembersResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported tag-members response schema",
            ));
        }
        let available = match decoder.read_u8()? {
            0 => false,
            1 => true,
            _ => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "tag-members availability flag must be 0 or 1",
                ));
            }
        };
        let count = decoder.read_length(MAX_WORLD_TAG_MEMBERS)?;
        if !available && count != 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "an absent tag must have an empty member list",
            ));
        }
        let mut members = Vec::new();
        members.try_reserve_exact(count).map_err(|_| {
            SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not reserve memory for tag members",
            )
        })?;
        for _ in 0..count {
            let length = decoder.read_length(MAX_WORLD_TAG_ID_BYTES)?;
            let id = String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
                SjmlError::new(
                    ErrorCode::InvalidUtf8,
                    "tag member ID contains malformed UTF-8",
                )
            })?;
            members.push(id);
        }
        validate_tag_member_ids(&members)?;
        Ok(Self {
            members: available.then_some(members),
        })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for BiomeIdsResponse {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let schema = decoder.read_u32()?;
        if schema != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported biome-ID response schema",
            ));
        }
        let count = decoder.read_length(MAX_WORLD_BIOME_IDS)?;
        let mut ids = Vec::new();
        ids.try_reserve_exact(count).map_err(|_| {
            SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not reserve memory for biome-ID results",
            )
        })?;
        for _ in 0..count {
            let length = decoder.read_length(MAX_WORLD_BIOME_ID_BYTES)?;
            let id = String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
                SjmlError::new(ErrorCode::InvalidUtf8, "biome ID contains malformed UTF-8")
            })?;
            ids.push(id);
        }
        validate_biome_ids(&ids)?;
        Ok(Self { ids })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct BlockPropertiesQuery {
    world: WorldHandle,
    position: BlockPos,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockPropertiesResponse {
    properties: Option<Vec<BlockProperty>>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct FluidStateQuery {
    world: WorldHandle,
    position: BlockPos,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode, StableDecode)]
#[sjml(version = 1)]
struct FluidStateResponse {
    state: Option<FluidStateSnapshot>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct BlockPropertiesMutation {
    world: WorldHandle,
    position: BlockPos,
    expected_block_id: String,
    properties: Vec<BlockProperty>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode, StableDecode)]
#[sjml(version = 1)]
struct DaylightCycleSnapshot {
    enabled: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(StableDecode)]
#[sjml(version = 1)]
struct DifficultySnapshot {
    difficulty: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldGameRuleQuery {
    world: WorldHandle,
    rule_id: u32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldGameRuleMutation {
    world: WorldHandle,
    rule_id: u32,
    enabled: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableDecode)]
#[sjml(version = 1)]
struct WorldGameRuleSnapshot {
    enabled: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldIntegerGameRuleQuery {
    world: WorldHandle,
    rule_id: u32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldIntegerGameRuleMutation {
    world: WorldHandle,
    rule_id: u32,
    value: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableDecode)]
#[sjml(version = 1)]
struct WorldIntegerGameRuleSnapshot {
    value: i32,
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

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldProjectileSpawnRequest {
    world: WorldHandle,
    projectile: u8,
    x: f64,
    y: f64,
    z: f64,
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(StableEncode)]
#[sjml(version = 1)]
struct WorldItemSpawnRequest {
    world: WorldHandle,
    item_id: String,
    count: u32,
    x: f64,
    y: f64,
    z: f64,
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_world_v1")]
extern "C" {
    #[link_name = "get"]
    fn imported_get(id: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "loaded"]
    fn imported_loaded(output: i32, capacity: i32) -> i32;
    #[link_name = "loaded_chunks"]
    fn imported_loaded_chunks(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "dimension_id"]
    fn imported_dimension_id(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "game_time"]
    fn imported_game_time(handle: i32, length: i32, output: i32) -> i32;
    #[link_name = "get_difficulty"]
    fn imported_get_difficulty(output: i32, capacity: i32) -> i32;
    #[link_name = "set_difficulty"]
    fn imported_set_difficulty(difficulty: i32) -> i32;
    #[link_name = "spawn_point"]
    fn imported_spawn_point(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_spawn_point"]
    fn imported_set_spawn_point(handle: i32, length: i32, point: i32, point_length: i32) -> i32;
    #[link_name = "day_time"]
    fn imported_day_time(handle: i32, length: i32, output: i32) -> i32;
    #[link_name = "daylight_cycle_enabled"]
    fn imported_daylight_cycle_enabled(handle: i32, length: i32, output: i32, capacity: i32)
        -> i32;
    #[link_name = "game_rule"]
    fn imported_game_rule(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "integer_game_rule"]
    fn imported_integer_game_rule(query: i32, query_length: i32, output: i32, capacity: i32)
        -> i32;
    #[link_name = "biome_at"]
    fn imported_biome_at(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "biome_ids"]
    fn imported_biome_ids(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "structures_at"]
    fn imported_structures_at(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "block_has_tag"]
    fn imported_block_has_tag(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "item_has_tag"]
    fn imported_item_has_tag(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "tag_members"]
    fn imported_tag_members(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "heightmap"]
    fn imported_heightmap(
        handle: i32,
        handle_length: i32,
        heightmap_type: i32,
        x: i32,
        z: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "light_at"]
    fn imported_light_at(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "best_neighbor_signal"]
    fn imported_best_neighbor_signal(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "weather"]
    fn imported_weather(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "border"]
    fn imported_border(handle: i32, length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_weather"]
    fn imported_set_weather(handle: i32, length: i32, condition: i32, duration_ticks: i32) -> i32;
    #[link_name = "set_border_center"]
    fn imported_set_border_center(handle: i32, length: i32, values: i32, values_length: i32)
        -> i32;
    #[link_name = "set_border_size"]
    fn imported_set_border_size(handle: i32, length: i32, values: i32, values_length: i32) -> i32;
    #[link_name = "set_border_warnings"]
    fn imported_set_border_warnings(
        handle: i32,
        length: i32,
        values: i32,
        values_length: i32,
    ) -> i32;
    #[link_name = "set_border_damage"]
    fn imported_set_border_damage(handle: i32, length: i32, values: i32, values_length: i32)
        -> i32;
    #[link_name = "set_time"]
    fn imported_set_time(handle: i32, length: i32, day_time_low: i32, day_time_high: i32) -> i32;
    #[link_name = "set_game_time"]
    fn imported_set_game_time(
        handle: i32,
        length: i32,
        game_time_low: i32,
        game_time_high: i32,
    ) -> i32;
    #[link_name = "set_daylight_cycle_enabled"]
    fn imported_set_daylight_cycle_enabled(handle: i32, length: i32, enabled: i32) -> i32;
    #[link_name = "set_game_rule"]
    fn imported_set_game_rule(query: i32, query_length: i32) -> i32;
    #[link_name = "set_integer_game_rule"]
    fn imported_set_integer_game_rule(query: i32, query_length: i32) -> i32;
    #[link_name = "block_states"]
    fn imported_block_states(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "block_properties"]
    fn imported_block_properties(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "fluid_at"]
    fn imported_fluid_at(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "raycast"]
    fn imported_raycast(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "collides"]
    fn imported_collides(query: i32, query_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "spawn_entity"]
    fn imported_spawn_entity(query: i32, query_length: i32) -> i32;
    #[link_name = "spawn_projectile"]
    fn imported_spawn_projectile(query: i32, query_length: i32) -> i32;
    #[link_name = "spawn_item"]
    fn imported_spawn_item(query: i32, query_length: i32) -> i32;
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
    #[link_name = "set_block_properties"]
    fn imported_set_block_properties(request: i32, request_length: i32) -> i32;
    #[link_name = "schedule_block_tick"]
    fn imported_schedule_block_tick(
        handle: i32,
        handle_length: i32,
        x: i32,
        y: i32,
        z: i32,
        expected_block_id: i32,
        expected_block_id_length: i32,
        delay_ticks: i32,
    ) -> i32;
    #[link_name = "notify_neighbors_at"]
    fn imported_notify_neighbors_at(handle: i32, handle_length: i32, x: i32, y: i32, z: i32)
        -> i32;
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

/// Lists chunks that are already loaded in a square around `center` on this module's server world.
/// The center chunk is included when loaded; `center.y` is ignored. `chunk_radius` is measured in
/// chunks and must be at most 16. Results are sorted by chunk X then Z and are either complete or
/// an error when more than 512 chunks match. The host checks loaded-chunk residency without
/// requesting absent chunks, so this query never loads a chunk or dimension. Requires `world.read`
/// and a live server-side [`WorldHandle`].
pub fn loaded_chunks(
    world: WorldHandle,
    center: BlockPos,
    chunk_radius: u32,
) -> SjmlResult<Vec<ChunkPos>> {
    validate_loaded_chunk_query(center, chunk_radius)?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&LoadedChunksQuery {
            world,
            center,
            radius: chunk_radius,
        })?;
        let response: LoadedChunksResponse = read_output(
            MAX_WORLD_CHUNKS_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the encoded query and output buffer remain live for the synchronous import.
                unsafe {
                    imported_loaded_chunks(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        validate_loaded_chunk_results(center, chunk_radius, &response.chunks)?;
        Ok(response.chunks)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

fn validate_loaded_chunk_query(center: BlockPos, radius: u32) -> SjmlResult<()> {
    if center.x.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
        || center.z.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "loaded-chunk center x/z must be within +/-30000000",
        ));
    }
    if radius > MAX_WORLD_CHUNK_RADIUS {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "loaded-chunk radius must be within 0..=16 chunks",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_loaded_chunk_results(
    center: BlockPos,
    radius: u32,
    chunks: &[ChunkPos],
) -> SjmlResult<()> {
    if chunks.len() > MAX_WORLD_CHUNKS {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "host returned more than 512 loaded chunks",
        ));
    }
    let center_x = center.x >> 4;
    let center_z = center.z >> 4;
    let mut previous: Option<ChunkPos> = None;
    for chunk in chunks {
        let in_range = (i64::from(chunk.x) - i64::from(center_x)).unsigned_abs()
            <= u64::from(radius)
            && (i64::from(chunk.z) - i64::from(center_z)).unsigned_abs() <= u64::from(radius);
        if !in_range
            || chunk.x.unsigned_abs() > (MAX_WORLD_BLOCK_COORDINATE as u32 >> 4)
            || chunk.z.unsigned_abs() > (MAX_WORLD_BLOCK_COORDINATE as u32 >> 4)
        {
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned a loaded chunk outside the requested world bounds",
            ));
        }
        if let Some(previous) = previous {
            if previous.x > chunk.x || (previous.x == chunk.x && previous.z >= chunk.z) {
                return Err(SjmlError::new(
                    ErrorCode::UnknownErrorCode,
                    "host returned duplicate or unsorted loaded chunks",
                ));
            }
        }
        previous = Some(*chunk);
    }
    Ok(())
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

/// Reads the server-wide vanilla difficulty. Requires `world.read` and the owning logical-server
/// thread; difficulty is global to the server rather than scoped to a dimension handle.
pub fn difficulty() -> SjmlResult<WorldDifficulty> {
    #[cfg(target_arch = "wasm32")]
    {
        let snapshot: DifficultySnapshot = read_output(8, |output, capacity| {
            // SAFETY: the fixed-size output remains live through this synchronous server import.
            unsafe { imported_get_difficulty(output, capacity) }
        })?;
        WorldDifficulty::try_from(snapshot.difficulty)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Sets the server-wide vanilla difficulty. Requires `world.write` and the owning logical-server
/// thread. This respects the world's difficulty lock and Minecraft's hardcore restriction.
pub fn set_difficulty(difficulty: WorldDifficulty) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the enum is a closed four-value repr validated again by the host.
        let status = unsafe { imported_set_difficulty(difficulty as i32) };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the server difficulty change",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = difficulty;
        Err(unsupported_target())
    }
}

/// Reads the raw configured vanilla Overworld spawn point and its yaw.
///
/// Requires `world.read` and a live handle returned for `minecraft:overworld`. The host reads the
/// saved level data directly and does not perform Minecraft's heightmap fallback for an out-of-
/// border spawn, so this call never inspects or loads terrain.
pub fn spawn_point(world: WorldHandle) -> SjmlResult<WorldSpawnPoint> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        let point: WorldSpawnPoint =
            read_output(MAX_WORLD_SPAWN_POINT_BYTES as u32, |output, capacity| {
                // SAFETY: the stable handle and fixed-size output remain live through the host call.
                unsafe {
                    imported_spawn_point(
                        encoded.as_ptr() as usize as u32 as i32,
                        encoded.len() as i32,
                        output,
                        capacity,
                    )
                }
            })?;
        if point.x < -MAX_WORLD_BLOCK_COORDINATE
            || point.x > MAX_WORLD_BLOCK_COORDINATE
            || point.z < -MAX_WORLD_BLOCK_COORDINATE
            || point.z > MAX_WORLD_BLOCK_COORDINATE
            || !point.angle.is_finite()
        {
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned an invalid configured Overworld spawn point",
            ));
        }
        Ok(point)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Sets the vanilla Overworld spawn point and yaw on the owning server thread.
///
/// Requires a live `minecraft:overworld` handle plus `world.read` to validate that handle and
/// `world.write` to change the spawn. X/Z and the finite yaw are validated here; the host checks
/// the loaded Overworld's current build-height range and world border before changing anything.
pub fn set_spawn_point(world: WorldHandle, spawn_point: WorldSpawnPoint) -> SjmlResult<()> {
    spawn_point.validate_request()?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&world)?;
        let encoded_point = to_bytes(&spawn_point)?;
        // SAFETY: both stable byte buffers remain live through this synchronous mutation.
        let status = unsafe {
            imported_set_spawn_point(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as i32,
                encoded_point.as_ptr() as usize as u32 as i32,
                encoded_point.len() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the Overworld spawn-point change",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
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

/// Reads the current absolute daylight-cycle time in ticks for a loaded world.
///
/// Requires `world.read`. Unlike [`game_time`], this value follows the daylight-cycle gamerule and
/// may stop advancing when that rule is disabled. This method never loads a dimension or chunk.
pub fn day_time(handle: WorldHandle) -> SjmlResult<i64> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        read_output(8, |output, _| {
            // SAFETY: the stable handle encoding and 8-byte output remain live for this call.
            unsafe { imported_day_time(encoded.as_ptr() as usize as u32 as i32, 24, output) }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads whether vanilla `doDaylightCycle` is enabled for a loaded world.
///
/// Requires `world.read` and a logical-server context. The value is the addressed world's
/// persisted vanilla gamerule, read on the server thread; this call does not load a dimension.
pub fn daylight_cycle_enabled(handle: WorldHandle) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let snapshot: DaylightCycleSnapshot = read_output(5, |output, capacity| {
            // SAFETY: the handle bytes and fixed versioned output buffer remain live through the import.
            unsafe {
                imported_daylight_cycle_enabled(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        Ok(snapshot.enabled)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Reads one allowlisted boolean vanilla gamerule for a loaded logical-server world.
///
/// Requires `world.read`. The read is server-thread-only, applies to the selected dimension, and
/// never loads a world or chunk. The [`WorldGameRule`] enum intentionally excludes integer rules.
pub fn game_rule(handle: WorldHandle, rule: WorldGameRule) -> SjmlResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldGameRuleQuery {
            world: handle,
            rule_id: rule as u32,
        })?;
        let snapshot: WorldGameRuleSnapshot = read_output(5, |output, capacity| {
            // SAFETY: the stable request and fixed-size destination stay live for the sync host call.
            unsafe {
                imported_game_rule(
                    query.as_ptr() as usize as u32 as i32,
                    query.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        Ok(snapshot.enabled)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, rule);
        Err(unsupported_target())
    }
}

/// Reads the exact integer value of one allowlisted vanilla gamerule on a loaded dimension.
///
/// Requires `world.read`. Reads report the value currently stored by Minecraft, including values
/// changed outside SJML; the documented per-rule range is enforced when this SDK writes a value.
/// The call is server-thread-only and never loads a world or chunk.
pub fn integer_game_rule(handle: WorldHandle, rule: WorldIntegerGameRule) -> SjmlResult<i32> {
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldIntegerGameRuleQuery {
            world: handle,
            rule_id: rule as u32,
        })?;
        let snapshot: WorldIntegerGameRuleSnapshot = read_output(8, |output, capacity| {
            // SAFETY: the stable request and fixed-size destination stay live for the sync host call.
            unsafe {
                imported_integer_game_rule(
                    query.as_ptr() as usize as u32 as i32,
                    query.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        Ok(snapshot.value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, rule);
        Err(unsupported_target())
    }
}

/// Reads the canonical biome ID at a loaded world position. Returns `None` when the position is
/// outside build height or its chunk is not already loaded; the call never loads a chunk.
pub fn biome_at(world: WorldHandle, position: BlockPos) -> SjmlResult<Option<String>> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        let id: String = read_output((4 + MAX_RESOURCE_ID_BYTES) as u32, |output, capacity| {
            // SAFETY: the handle and scalar position remain live for this synchronous import.
            unsafe {
                imported_biome_at(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    position.x,
                    position.y,
                    position.z,
                    output,
                    capacity,
                )
            }
        })?;
        if id.is_empty() {
            return Ok(None);
        }
        validate_resource_id(&id, "biome")?;
        Ok(Some(id))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

/// Lists all registered biome IDs for this module's loaded server world in lexical order.
///
/// Requires `world.read` and a live server-side [`WorldHandle`]. The registry is read directly;
/// chunks are never loaded or queried. Results are complete or an error if more than
/// [`MAX_WORLD_BIOME_IDS`] biomes are registered.
pub fn biome_ids(world: WorldHandle) -> SjmlResult<Vec<String>> {
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&BiomeIdsQuery { world })?;
        let response: BiomeIdsResponse = read_output(
            MAX_WORLD_BIOME_IDS_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the versioned query and bounded destination remain live for this sync call.
                unsafe {
                    imported_biome_ids(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        validate_biome_ids(&response.ids)?;
        Ok(response.ids)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Returns the structures whose generated pieces contain `position` in this server world.
///
/// Requires `world.read`. Both the position's chunk and every referenced structure-start chunk
/// must already be loaded; otherwise this returns `Ok(None)` and requests no chunk loads. A
/// loaded position outside the dimension's build height also returns `None`. `Some(vec![])`
/// means the loaded position was checked and no structure piece contains it. IDs are canonical,
/// lexically sorted, and bounded to [`MAX_WORLD_STRUCTURES_AT`] entries.
pub fn structures_at(world: WorldHandle, position: BlockPos) -> SjmlResult<Option<Vec<String>>> {
    validate_world_position(position)?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&StructuresAtQuery { world, position })?;
        let response: StructuresAtResponse = read_output(
            MAX_WORLD_STRUCTURES_AT_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the versioned query and bounded output remain live for this synchronous import.
                unsafe {
                    imported_structures_at(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if let Some(ids) = &response.result {
            validate_structure_ids(ids)?;
        }
        Ok(response.result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Checks whether the block at `position` belongs to a registered block tag such as
/// `minecraft:logs`. Requires `world.read`. Returns `None` for unloaded/out-of-height positions and
/// never loads chunks; `Some(false)` means the position was available and the block is not a member.
pub fn block_has_tag(
    world: WorldHandle,
    position: BlockPos,
    tag_id: &str,
) -> SjmlResult<Option<bool>> {
    validate_world_position(position)?;
    if tag_id.len() > MAX_WORLD_BLOCK_TAG_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "block tag ID exceeds the 256-byte limit",
        ));
    }
    validate_resource_id(tag_id, "block tag")?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&BlockTagQuery {
            world,
            position,
            tag_id: tag_id.to_owned(),
        })?;
        let response: BlockTagResponse = read_output(
            MAX_WORLD_BLOCK_TAG_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the versioned query and fixed-size output remain live for this synchronous import.
                unsafe {
                    imported_block_has_tag(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        Ok(response.result)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Checks whether a canonical item ID belongs to a vanilla, mod, or data-pack item tag.
///
/// Requires a live logical-server world handle and `world.read`. Membership uses the current
/// server registry tag set and reflects resource reloads. Unknown item IDs and absent tags return
/// `false`; this registry-only query does not inspect inventories or load chunks.
pub fn item_has_tag(world: WorldHandle, item_id: &str, tag_id: &str) -> SjmlResult<bool> {
    if item_id.len() > MAX_WORLD_ITEM_TAG_ID_BYTES || tag_id.len() > MAX_WORLD_ITEM_TAG_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "item and item-tag IDs are limited to 256 UTF-8 bytes each",
        ));
    }
    validate_resource_id(item_id, "item")?;
    validate_resource_id(tag_id, "item tag")?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&ItemTagQuery {
            world,
            item_id: item_id.to_owned(),
            tag_id: tag_id.to_owned(),
        })?;
        let response: ItemTagResponse = read_output(
            MAX_WORLD_ITEM_TAG_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the versioned query and fixed-size output live for this synchronous import.
                unsafe {
                    imported_item_has_tag(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        Ok(response.contains)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Returns sorted canonical members of a current vanilla, mod, or data-pack registry tag.
///
/// Supports the block, item, fluid, and entity-type registries. Requires a live logical-server
/// world handle and `world.read`. `None` means the tag is absent; `Some(empty)` is a registered
/// tag with no members. The result is capped at [`MAX_WORLD_TAG_MEMBERS`] entries and each ID at
/// [`MAX_WORLD_TAG_ID_BYTES`] bytes. Calls observe the current server registries after resource
/// reloads. This operation reads registry data only and never loads chunks.
pub fn tag_members(
    world: WorldHandle,
    registry_type: TagRegistry,
    tag_id: &str,
) -> SjmlResult<Option<Vec<String>>> {
    if tag_id.len() > MAX_WORLD_TAG_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "tag ID exceeds the 256-byte limit",
        ));
    }
    validate_resource_id(tag_id, "tag")?;
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&TagMembersQuery {
            world,
            registry_type: registry_type as u32,
            tag_id: tag_id.to_owned(),
        })?;
        let response: TagMembersResponse = read_length_prefixed_output(
            MAX_WORLD_TAG_MEMBERS_OUTPUT_BYTES as u32,
            |output, capacity| {
                // SAFETY: the versioned query and bounded output live for this synchronous import.
                unsafe {
                    imported_tag_members(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        if let Some(members) = &response.members {
            validate_tag_member_ids(members)?;
        }
        Ok(response.members)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, registry_type);
        Err(unsupported_target())
    }
}

/// Returns sorted block IDs in one current block tag.
pub fn block_tag_members(world: WorldHandle, tag_id: &str) -> SjmlResult<Option<Vec<String>>> {
    tag_members(world, TagRegistry::Block, tag_id)
}

/// Returns sorted item IDs in one current item tag.
pub fn item_tag_members(world: WorldHandle, tag_id: &str) -> SjmlResult<Option<Vec<String>>> {
    tag_members(world, TagRegistry::Item, tag_id)
}

/// Returns sorted fluid IDs in one current fluid tag.
pub fn fluid_tag_members(world: WorldHandle, tag_id: &str) -> SjmlResult<Option<Vec<String>>> {
    tag_members(world, TagRegistry::Fluid, tag_id)
}

/// Returns sorted entity-type IDs in one current entity-type tag.
pub fn entity_type_tag_members(
    world: WorldHandle,
    tag_id: &str,
) -> SjmlResult<Option<Vec<String>>> {
    tag_members(world, TagRegistry::EntityType, tag_id)
}

fn validate_world_position(position: BlockPos) -> SjmlResult<()> {
    if position.x < -MAX_WORLD_BLOCK_COORDINATE
        || position.x > MAX_WORLD_BLOCK_COORDINATE
        || position.z < -MAX_WORLD_BLOCK_COORDINATE
        || position.z > MAX_WORLD_BLOCK_COORDINATE
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world query x/z coordinates must be within Minecraft's +/-30,000,000 bound",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_structure_ids(ids: &[String]) -> SjmlResult<()> {
    if ids.len() > MAX_WORLD_STRUCTURES_AT {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "host returned more than 128 structure IDs",
        ));
    }
    let mut previous: Option<&str> = None;
    for id in ids {
        if id.len() > MAX_WORLD_STRUCTURE_ID_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "structure ID exceeds the 256-byte limit",
            ));
        }
        validate_resource_id(id, "structure")?;
        if previous.is_some_and(|previous| previous >= id.as_str()) {
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned duplicate or unsorted structure IDs",
            ));
        }
        previous = Some(id);
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_biome_ids(ids: &[String]) -> SjmlResult<()> {
    if ids.len() > MAX_WORLD_BIOME_IDS {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "host returned more than 4000 biome IDs",
        ));
    }
    let mut previous: Option<&str> = None;
    for id in ids {
        validate_resource_id(id, "biome")?;
        if previous.is_some_and(|previous| previous >= id.as_str()) {
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned duplicate or unsorted biome IDs",
            ));
        }
        previous = Some(id);
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_tag_member_ids(ids: &[String]) -> SjmlResult<()> {
    if ids.len() > MAX_WORLD_TAG_MEMBERS {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "host returned more than 1024 tag members",
        ));
    }
    let mut previous: Option<&str> = None;
    for id in ids {
        if id.len() > MAX_WORLD_TAG_ID_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "tag member ID exceeds the 256-byte limit",
            ));
        }
        validate_resource_id(id, "tag member")?;
        if previous.is_some_and(|previous| previous >= id.as_str()) {
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned duplicate or unsorted tag members",
            ));
        }
        previous = Some(id);
    }
    Ok(())
}

/// Reads a vanilla runtime heightmap value at block x/z coordinates.
///
/// Requires `world.read` and a valid module-scoped handle. The four runtime maps are available;
/// world-generation-only maps are intentionally excluded. Coordinates outside +/-30,000,000 are
/// rejected. An unloaded chunk or a chunk without the requested primed map returns `None`; no
/// chunk is requested or generated. The value is the highest Y included by the selected map
/// (`min_build_height - 1` for a fully empty column).
pub fn heightmap(
    world: WorldHandle,
    heightmap_type: HeightmapType,
    x: i32,
    z: i32,
) -> SjmlResult<Option<i32>> {
    if !(-MAX_WORLD_BLOCK_COORDINATE..=MAX_WORLD_BLOCK_COORDINATE).contains(&x)
        || !(-MAX_WORLD_BLOCK_COORDINATE..=MAX_WORLD_BLOCK_COORDINATE).contains(&z)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world x/z coordinates must be within +/-30000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        read_output(1 + 4, |output, capacity| {
            // SAFETY: the encoded handle and fixed optional-i32 output stay live for this call.
            unsafe {
                imported_heightmap(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    heightmap_type as i32,
                    x,
                    z,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, heightmap_type);
        Err(unsupported_target())
    }
}

/// Reads sky and block light at a loaded world position. Requires `world.read` and never loads a
/// chunk. Returns `None` outside build height or when the position's chunk is not already loaded.
/// x/z coordinates outside +/-30,000,000 are rejected; returned light values are each 0..=15.
pub fn light_at(world: WorldHandle, position: BlockPos) -> SjmlResult<Option<LightSnapshot>> {
    if !(-MAX_WORLD_BLOCK_COORDINATE..=MAX_WORLD_BLOCK_COORDINATE).contains(&position.x)
        || !(-MAX_WORLD_BLOCK_COORDINATE..=MAX_WORLD_BLOCK_COORDINATE).contains(&position.z)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world x/z coordinates must be within +/-30000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        let light: Option<LightSnapshot> = read_output(1 + 4 + 2, |output, capacity| {
            // SAFETY: the handle, coordinates, and fixed output stay live for this call.
            unsafe {
                imported_light_at(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    position.x,
                    position.y,
                    position.z,
                    output,
                    capacity,
                )
            }
        })?;
        if light.is_some_and(|value| value.sky > 15 || value.block > 15) {
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned a light level outside 0..=15",
            ));
        }
        Ok(light)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Reads vanilla's maximum neighboring redstone signal at a block position.
///
/// Requires `world.read` and runs on the owning logical-server thread. The result is Minecraft's
/// `SignalGetter::getBestNeighborSignal` value in `0..=15`, considering the six blocks adjacent to
/// `position` with vanilla's direct-power behavior. The host verifies that the target is inside
/// build height and that every block position vanilla may inspect is within the world border and
/// already in a full loaded chunk before reading. It returns `None` if that bounded neighborhood
/// is unavailable and never requests or generates a chunk. x/z coordinates outside
/// +/-30,000,000 are rejected.
pub fn best_neighbor_signal(world: WorldHandle, position: BlockPos) -> SjmlResult<Option<u8>> {
    if !(-MAX_WORLD_BLOCK_COORDINATE..=MAX_WORLD_BLOCK_COORDINATE).contains(&position.x)
        || !(-MAX_WORLD_BLOCK_COORDINATE..=MAX_WORLD_BLOCK_COORDINATE).contains(&position.z)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world x/z coordinates must be within +/-30000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        let signal: Option<u32> = read_output(1 + 4, |output, capacity| {
            // SAFETY: the handle, coordinates, and fixed output stay live for this call.
            unsafe {
                imported_best_neighbor_signal(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    position.x,
                    position.y,
                    position.z,
                    output,
                    capacity,
                )
            }
        })?;
        match signal {
            Some(value) if value <= u32::from(MAX_WORLD_REDSTONE_SIGNAL) => Ok(Some(value as u8)),
            Some(_) => Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "host returned a redstone signal outside 0..=15",
            )),
            None => Ok(None),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = world;
        Err(unsupported_target())
    }
}

/// Reads the current weather state and timers of a loaded dimension. Requires `world.read` and
/// executes on the owning logical-server thread.
pub fn weather(handle: WorldHandle) -> SjmlResult<WeatherSnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let snapshot: WeatherSnapshot = read_output(4 + 2 + 3 * 4, |output, capacity| {
            // SAFETY: the encoded handle and fixed-size output allocation stay live for this call.
            unsafe {
                imported_weather(
                    encoded.as_ptr() as usize as u32 as i32,
                    24,
                    output,
                    capacity,
                )
            }
        })?;
        if snapshot.clear_time > i32::MAX as u32
            || snapshot.rain_time > i32::MAX as u32
            || snapshot.thunder_time > i32::MAX as u32
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a weather timer beyond the supported bound",
            ));
        }
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets clear, rain, or thunder weather for a bounded duration in a loaded dimension.
///
/// Requires both `world.read` and `world.write`; the host never loads a dimension or chunk. A
/// duration of 1 through 1,728,000 ticks is accepted. `Clear` schedules the clear-weather timer;
/// `Rain` and `Thunder` set the requested weather and use that duration for the vanilla weather
/// transition timer.
pub fn set_weather(
    handle: WorldHandle,
    condition: WeatherCondition,
    duration_ticks: u32,
) -> SjmlResult<()> {
    if !(1..=MAX_WEATHER_DURATION_TICKS).contains(&duration_ticks) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "weather duration must be within 1..=1728000 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the stable handle and scalar arguments remain valid for this synchronous import.
        let status = unsafe {
            imported_set_weather(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                condition as u32 as i32,
                duration_ticks as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the world weather update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, condition);
        Err(unsupported_target())
    }
}

/// Reads the live world's center, size transition, warnings, and damage settings.
///
/// Requires `world.read` and a loaded world handle. The transition time is the non-negative
/// number of milliseconds remaining; a stationary border has no remaining transition.
pub fn border(handle: WorldHandle) -> SjmlResult<WorldBorderSnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let snapshot: WorldBorderSnapshot = read_output(68, |output, capacity| {
            // SAFETY: the handle and fixed output buffer stay live for this synchronous import.
            unsafe {
                imported_border(
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        snapshot.validate()?;
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets the center of a loaded world's border. Requires `world.read` and `world.write`.
pub fn set_border_center(handle: WorldHandle, center_x: f64, center_z: f64) -> SjmlResult<()> {
    if !center_x.is_finite()
        || center_x.abs() > MAX_WORLD_BORDER_CENTER
        || !center_z.is_finite()
        || center_z.abs() > MAX_WORLD_BORDER_CENTER
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world border center must be finite and within ±29999984",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut values = Vec::with_capacity(20);
        values.extend_from_slice(&1_u32.to_le_bytes());
        values.extend_from_slice(&center_x.to_le_bytes());
        values.extend_from_slice(&center_z.to_le_bytes());
        set_border_request(handle, &values, imported_set_border_center)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets a border diameter immediately or starts a bounded size transition. Requires `world.read`
/// and `world.write`; transition time is in milliseconds, with zero meaning immediate.
pub fn set_border_size(
    handle: WorldHandle,
    diameter: f64,
    transition_millis: u64,
) -> SjmlResult<()> {
    if !valid_border_diameter(diameter) || transition_millis > MAX_WORLD_BORDER_LERP_MILLIS {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world border size or transition time is outside its supported range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut values = Vec::with_capacity(20);
        values.extend_from_slice(&1_u32.to_le_bytes());
        values.extend_from_slice(&diameter.to_le_bytes());
        values.extend_from_slice(&transition_millis.to_le_bytes());
        set_border_request(handle, &values, imported_set_border_size)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets border warning lead time and distance. Requires `world.read` and `world.write`.
pub fn set_border_warnings(
    handle: WorldHandle,
    warning_time_seconds: u32,
    warning_distance_blocks: u32,
) -> SjmlResult<()> {
    if warning_time_seconds > MAX_WORLD_BORDER_WARNING_SECONDS
        || warning_distance_blocks > MAX_WORLD_BORDER_WARNING_BLOCKS
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world border warning time or distance is outside its supported range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut values = Vec::with_capacity(12);
        values.extend_from_slice(&1_u32.to_le_bytes());
        values.extend_from_slice(&warning_time_seconds.to_le_bytes());
        values.extend_from_slice(&warning_distance_blocks.to_le_bytes());
        set_border_request(handle, &values, imported_set_border_warnings)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Sets the safe damage buffer and damage per block for a border. Requires `world.read` and
/// `world.write`; both finite values are bounded by the host-supported ranges.
pub fn set_border_damage(
    handle: WorldHandle,
    damage_buffer_blocks: f64,
    damage_per_block: f64,
) -> SjmlResult<()> {
    if !damage_buffer_blocks.is_finite()
        || !(0.0..=MAX_WORLD_BORDER_DAMAGE_BUFFER).contains(&damage_buffer_blocks)
        || !damage_per_block.is_finite()
        || !(0.0..=MAX_WORLD_BORDER_DAMAGE_PER_BLOCK).contains(&damage_per_block)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world border damage values are outside their supported finite ranges",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mut values = Vec::with_capacity(20);
        values.extend_from_slice(&1_u32.to_le_bytes());
        values.extend_from_slice(&damage_buffer_blocks.to_le_bytes());
        values.extend_from_slice(&damage_per_block.to_le_bytes());
        set_border_request(handle, &values, imported_set_border_damage)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

#[cfg(target_arch = "wasm32")]
type BorderMutationImport = unsafe extern "C" fn(i32, i32, i32, i32) -> i32;

#[cfg(target_arch = "wasm32")]
fn set_border_request(
    handle: WorldHandle,
    values: &[u8],
    import: BorderMutationImport,
) -> SjmlResult<()> {
    let encoded_handle = to_bytes(&handle)?;
    // SAFETY: handle and bounded request bytes live through this synchronous host call.
    let status = unsafe {
        import(
            encoded_handle.as_ptr() as usize as u32 as i32,
            encoded_handle.len() as i32,
            values.as_ptr() as usize as u32 as i32,
            values.len() as i32,
        )
    };
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the world border update",
        ))
    }
}

/// Sets the absolute daylight-cycle time for a loaded world.
///
/// Requires both `world.read` (for the handle) and `world.write`. Values from zero through
/// [`MAX_WORLD_DAY_TIME_TICKS`] are accepted. This does not change `game_time` or the daylight-cycle
/// gamerule; when that gamerule is enabled, `day_time` resumes advancing on the next world tick.
/// The operation is server-thread-only and never loads a dimension or chunk.
pub fn set_time(handle: WorldHandle, day_time: i64) -> SjmlResult<()> {
    if !(0..=MAX_WORLD_DAY_TIME_TICKS).contains(&day_time) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "day time must be within 0..=1000000000000 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the stable handle and split i64 argument remain valid for this synchronous call.
        let status = unsafe {
            imported_set_time(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                day_time as u32 as i32,
                (day_time >> 32) as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the world time update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, day_time);
        Err(unsupported_target())
    }
}

/// Sets the absolute elapsed game-time clock shared by dimensions on this loaded world’s server.
///
/// Requires both `world.read` (for the scoped handle) and `world.write`. Values from zero through
/// [`MAX_WORLD_GAME_TIME_TICKS`] are accepted. This changes `game_time`, not `day_time`; it runs on
/// the owning server thread and never loads a dimension or chunk.
pub fn set_game_time(handle: WorldHandle, game_time: i64) -> SjmlResult<()> {
    if !(0..=MAX_WORLD_GAME_TIME_TICKS).contains(&game_time) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "game time must be within 0..=1000000000000 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the stable handle and split i64 argument remain valid for this synchronous call.
        let status = unsafe {
            imported_set_game_time(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                game_time as u32 as i32,
                (game_time >> 32) as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the world game-time update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, game_time);
        Err(unsupported_target())
    }
}

/// Enables or disables vanilla `doDaylightCycle` for a loaded world.
///
/// Requires both `world.read` (for the world handle) and `world.write`. This changes the actual
/// persisted Minecraft gamerule on the server thread; it does not change the current `day_time`.
/// When disabled, daylight time stops advancing. Client and static registration contexts are
/// rejected, and the host does not load a dimension or chunk.
pub fn set_daylight_cycle_enabled(handle: WorldHandle, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        // SAFETY: the encoded handle and boolean argument remain valid for this synchronous call.
        let status = unsafe {
            imported_set_daylight_cycle_enabled(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                i32::from(enabled),
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the daylight-cycle gamerule update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, enabled);
        Err(unsupported_target())
    }
}

/// Sets one allowlisted boolean vanilla gamerule on a loaded world.
///
/// Requires both `world.read` (to validate the handle) and `world.write`. The update runs on the
/// logical server thread and uses Minecraft's persisted `GameRules` state. It never loads a world
/// or chunk. Integer gamerules are intentionally outside this API.
pub fn set_game_rule(handle: WorldHandle, rule: WorldGameRule, enabled: bool) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let mutation = to_bytes(&WorldGameRuleMutation {
            world: handle,
            rule_id: rule as u32,
            enabled,
        })?;
        // SAFETY: the complete stable mutation stays live for the synchronous host call.
        let status = unsafe {
            imported_set_game_rule(
                mutation.as_ptr() as usize as u32 as i32,
                mutation.len() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the world gamerule update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, rule, enabled);
        Err(unsupported_target())
    }
}

/// Sets one allowlisted vanilla integer gamerule on a loaded world.
///
/// Requires both `world.read` (to validate the handle) and `world.write`. Values must be within
/// the inclusive range for the selected [`WorldIntegerGameRule`]. Writes use Minecraft's persisted
/// `GameRules` state and run on the logical server thread.
pub fn set_integer_game_rule(
    handle: WorldHandle,
    rule: WorldIntegerGameRule,
    value: i32,
) -> SjmlResult<()> {
    if value < rule.min_value() || value > rule.max_value() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "integer gamerule value is outside the allowed range for this rule",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let mutation = to_bytes(&WorldIntegerGameRuleMutation {
            world: handle,
            rule_id: rule as u32,
            value,
        })?;
        // SAFETY: the complete stable mutation stays live for the synchronous host call.
        let status = unsafe {
            imported_set_integer_game_rule(
                mutation.as_ptr() as usize as u32 as i32,
                mutation.len() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the integer world gamerule update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, rule);
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

/// Reads the canonically named state properties at one already-loaded block position.
///
/// Returns `None` when the position is outside build height or its chunk is not already loaded;
/// this query never loads a chunk. Values are detached strings in lexicographic property-name
/// order. The operation requires `world.read` and the logical server context.
pub fn block_properties(
    world: WorldHandle,
    position: BlockPos,
) -> SjmlResult<Option<Vec<BlockProperty>>> {
    if position.x.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
        || position.z.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block x/z coordinates must be within ±30000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&BlockPropertiesQuery { world, position })?;
        let response: BlockPropertiesResponse = read_output(
            MAX_WORLD_BLOCK_PROPERTIES_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: the encoded query and owned output allocation remain live for this import.
                unsafe {
                    imported_block_properties(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        validate_block_properties(response.properties)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

/// Reads the detached fluid state at one already-loaded block position.
///
/// Returns `None` when the position is outside build height or its chunk is not already loaded;
/// the host never loads a chunk. `height` is the fluid state's intrinsic own-height value, bounded
/// to 0..=1 blocks and independent of neighboring block shapes. Requires `world.read` and the
/// logical server context.
pub fn fluid_at(world: WorldHandle, position: BlockPos) -> SjmlResult<Option<FluidStateSnapshot>> {
    if position.x.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
        || position.z.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block x/z coordinates must be within ±30000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&FluidStateQuery { world, position })?;
        let response: FluidStateResponse = read_output(
            MAX_WORLD_FLUID_STATE_RESPONSE_BYTES as u32,
            |output, capacity| {
                // SAFETY: query and bounded output live through the synchronous world import.
                unsafe {
                    imported_fluid_at(
                        query.as_ptr() as usize as u32 as i32,
                        query.len() as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        validate_fluid_state(response.state)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_fluid_state(
    state: Option<FluidStateSnapshot>,
) -> SjmlResult<Option<FluidStateSnapshot>> {
    let Some(state) = state else {
        return Ok(None);
    };
    validate_resource_id(&state.fluid_id, "fluid")?;
    if state.amount > 8 || !state.height.is_finite() || !(0.0..=1.0).contains(&state.height) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned a fluid amount outside 0..=8 or own-height outside 0..=1",
        ));
    }
    let empty = state.fluid_id == "minecraft:empty";
    if (empty && (state.amount != 0 || state.source || state.height != 0.0))
        || (!empty && state.amount == 0)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned inconsistent empty or non-empty fluid values",
        ));
    }
    Ok(Some(state))
}

#[cfg(any(target_arch = "wasm32", test))]
fn validate_block_properties(
    properties: Option<Vec<BlockProperty>>,
) -> SjmlResult<Option<Vec<BlockProperty>>> {
    let Some(properties) = properties else {
        return Ok(None);
    };
    if properties.len() > MAX_WORLD_BLOCK_PROPERTIES {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned more than 64 block properties",
        ));
    }
    let mut previous_name: Option<&str> = None;
    for property in &properties {
        if property.name.is_empty()
            || property.name.len() > 64
            || !property
                .name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || property.value.is_empty()
            || property.value.len() > 256
            || property.value.chars().any(char::is_control)
            || previous_name.is_some_and(|previous| previous >= property.name.as_str())
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned malformed or non-canonical block properties",
            ));
        }
        previous_name = Some(&property.name);
    }
    Ok(Some(properties))
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

/// Launches one vanilla snowball or egg in an already-loaded server-world position.
///
/// The module must request and receive `world.read` and the separate
/// `projectile.spawn` administrator capability. Only vanilla snowballs and eggs
/// are accepted; their normal collision behavior remains unchanged. Positions
/// must be finite, inside build height and the world border, and in loaded
/// chunks. Velocity components must be finite and their magnitude must be
/// between 0.1 and 4.0 blocks per tick. Each module can launch at most four per
/// logical-server tick. No call loads a chunk.
pub fn spawn_projectile(
    world: WorldHandle,
    projectile: VanillaProjectileKind,
    position: RaycastPoint,
    velocity: ProjectileVelocity,
) -> SjmlResult<()> {
    if !valid_raycast_point(position) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "projectile coordinates must be finite and within ±30,000,000",
        ));
    }
    if !valid_projectile_velocity(velocity) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "projectile velocity must be finite and between 0.1 and 4.0 blocks per tick",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldProjectileSpawnRequest {
            world,
            projectile: projectile as u8,
            x: position.x,
            y: position.y,
            z: position.z,
            velocity_x: velocity.x,
            velocity_y: velocity.y,
            velocity_z: velocity.z,
        })?;
        // SAFETY: the encoded request remains allocated and readable for this synchronous import.
        let status = unsafe {
            imported_spawn_projectile(query.as_ptr() as usize as u32 as i32, query.len() as i32)
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the vanilla projectile spawn",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, projectile);
        Err(unsupported_target())
    }
}

/// Spawns one ordinary item stack in an already-loaded logical-server world.
///
/// The module must request and receive `world.read` and `world.write`. The item ID may name any
/// item registered on the current server, including another mod's item. The host constructs only
/// the item's default stack: this API cannot set NBT or arbitrary data components. Count must be
/// 1..=64 and may not exceed the registered item's stack limit. Position must be finite, inside
/// build height and the world border, and fully inside already-loaded chunks. Velocity may be zero
/// and its magnitude is limited to 4.0 blocks per tick. Each module may spawn at most 16 stacks per
/// server tick; the call never loads a chunk.
pub fn spawn_item(
    world: WorldHandle,
    item_id: &str,
    count: u32,
    position: RaycastPoint,
    velocity: ItemSpawnVelocity,
) -> SjmlResult<()> {
    validate_resource_id(item_id, "item")?;
    if count == 0 || count > MAX_WORLD_ITEM_STACK_COUNT {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "item stack count must be in 1..=64",
        ));
    }
    if !valid_raycast_point(position) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "item coordinates must be finite and within ±30,000,000",
        ));
    }
    if !valid_item_spawn_velocity(velocity) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "item velocity must be finite and have magnitude at most 4.0 blocks per tick",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let query = to_bytes(&WorldItemSpawnRequest {
            world,
            item_id: item_id.to_owned(),
            count,
            x: position.x,
            y: position.y,
            z: position.z,
            velocity_x: velocity.x,
            velocity_y: velocity.y,
            velocity_z: velocity.z,
        })?;
        // SAFETY: the encoded request remains allocated and readable for this synchronous import.
        let status = unsafe {
            imported_spawn_item(query.as_ptr() as usize as u32 as i32, query.len() as i32)
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the ordinary item spawn",
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

fn valid_projectile_velocity(velocity: ProjectileVelocity) -> bool {
    if !velocity.x.is_finite() || !velocity.y.is_finite() || !velocity.z.is_finite() {
        return false;
    }
    let speed = velocity.x.hypot(velocity.y).hypot(velocity.z);
    (MIN_WORLD_PROJECTILE_SPEED..=MAX_WORLD_PROJECTILE_SPEED).contains(&speed)
}

fn valid_item_spawn_velocity(velocity: ItemSpawnVelocity) -> bool {
    velocity.x.is_finite()
        && velocity.y.is_finite()
        && velocity.z.is_finite()
        && velocity.x.abs() <= MAX_WORLD_ITEM_SPAWN_SPEED
        && velocity.y.abs() <= MAX_WORLD_ITEM_SPAWN_SPEED
        && velocity.z.abs() <= MAX_WORLD_ITEM_SPAWN_SPEED
        && velocity.x.hypot(velocity.y).hypot(velocity.z) <= MAX_WORLD_ITEM_SPAWN_SPEED
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

/// Changes only the explicitly named properties of the block currently at `position`.
///
/// The expected block ID must still match when the server applies the update. Property names and
/// values should come from [`block_properties`]; the host checks each against the block's
/// registered Minecraft property and its enum or numeric range. The call requires `world.read`
/// and `world.write`, runs synchronously on the server thread, and never loads a chunk. It rejects
/// unloaded positions and accepts 1 through [`MAX_WORLD_BLOCK_PROPERTY_UPDATES`] unique properties.
/// Neighbor and client updates use fixed Minecraft flags selected by the host.
pub fn set_block_properties(
    world: WorldHandle,
    position: BlockPos,
    expected_block_id: &str,
    properties: &[BlockProperty],
) -> SjmlResult<()> {
    validate_block_id(expected_block_id)?;
    if position.x.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
        || position.z.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block x/z coordinates must be within ±30000000",
        ));
    }
    validate_block_property_updates(properties)?;
    #[cfg(target_arch = "wasm32")]
    {
        let mut ordered = properties.to_vec();
        ordered.sort_by(|left, right| left.name.cmp(&right.name));
        let mutation = to_bytes(&BlockPropertiesMutation {
            world,
            position,
            expected_block_id: expected_block_id.to_owned(),
            properties: ordered,
        })?;
        if mutation.len() > MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "block-property mutation exceeds its bounded request size",
            ));
        }
        // SAFETY: the encoded mutation remains live for this synchronous host call.
        let status = unsafe {
            imported_set_block_properties(
                mutation.as_ptr() as usize as u32 as i32,
                mutation.len() as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the block-property update",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position, expected_block_id, properties);
        Err(unsupported_target())
    }
}

fn validate_block_property_updates(properties: &[BlockProperty]) -> SjmlResult<()> {
    if properties.is_empty() || properties.len() > MAX_WORLD_BLOCK_PROPERTY_UPDATES {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block-property update count must be within 1..=32",
        ));
    }
    let mut names = std::collections::HashSet::with_capacity(properties.len());
    for property in properties {
        if property.name.is_empty()
            || property.name.len() > 64
            || !property
                .name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            || property.value.is_empty()
            || property.value.len() > 256
            || property.value.chars().any(char::is_control)
            || !names.insert(property.name.as_str())
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "block-property updates must use unique canonical names and printable bounded values",
            ));
        }
    }
    Ok(())
}

/// Schedules one ordinary Minecraft block tick for an expected block at a loaded position.
///
/// The module must request and receive `world.read` and `world.write`. The host rejects
/// unloaded positions and mismatched block IDs without loading a chunk. The delay must be
/// between 1 and [`MAX_WORLD_BLOCK_TICK_DELAY_TICKS`] ticks. Calls are limited to 16 per
/// module per server tick. Minecraft de-duplicates matching pending ticks according to its
/// normal tick scheduler. If the matching block is an SJML-registered block owned by this
/// module, its `block.tick` event callback runs on the server thread; vanilla blocks do not emit
/// this event. See the block tick guide for callback bounds and snapshot fields.
pub fn schedule_block_tick(
    world: WorldHandle,
    position: BlockPos,
    expected_block_id: &str,
    delay_ticks: u32,
) -> SjmlResult<()> {
    validate_block_id(expected_block_id)?;
    if !(1..=MAX_WORLD_BLOCK_TICK_DELAY_TICKS).contains(&delay_ticks) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scheduled block tick delay must be within 1..=72000 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        // SAFETY: the host copies the handle and bounded UTF-8 block ID before returning.
        let status = unsafe {
            imported_schedule_block_tick(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                position.x,
                position.y,
                position.z,
                expected_block_id.as_ptr() as usize as u32 as i32,
                expected_block_id.len() as i32,
                delay_ticks as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the scheduled block tick",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, position);
        Err(unsupported_target())
    }
}

/// Runs the normal vanilla neighbor update around a block at a loaded position.
///
/// The module must request and receive both `world.read` and `world.write`. The call runs
/// synchronously on the owning server thread and requires the target to be within build height,
/// inside the world border, and in an already loaded chunk. Every in-height direct neighbor must
/// also be loaded, so vanilla's six neighbor reads cannot cause a chunk load. An unloaded target
/// or direct neighbor is rejected. Vanilla neighbor callbacks run normally and may update blocks
/// according to Minecraft's rules.
pub fn notify_neighbors_at(world: WorldHandle, position: BlockPos) -> SjmlResult<()> {
    if position.x.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
        || position.z.unsigned_abs() > MAX_WORLD_BLOCK_COORDINATE as u32
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "neighbor update x/z coordinates must be within ±30000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&world)?;
        // SAFETY: the fixed-size handle and coordinates remain live for this synchronous host call.
        let status = unsafe {
            imported_notify_neighbors_at(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as i32,
                position.x,
                position.y,
                position.z,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the vanilla neighbor update",
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
        biome_ids, block_has_tag, block_properties, collides, day_time, daylight_cycle_enabled,
        entity_type_tag_members, fluid_at, fluid_tag_members, integer_game_rule, item_has_tag,
        item_tag_members, loaded_chunks, notify_neighbors_at, play_sound, raycast,
        schedule_block_tick, set_block, set_block_properties, set_daylight_cycle_enabled,
        set_game_time, set_integer_game_rule, set_time, spawn_entity, spawn_item, spawn_particle,
        spawn_projectile, structures_at, tag_members, validate_biome_ids,
        validate_block_property_updates, validate_loaded_chunk_query,
        validate_loaded_chunk_results, validate_structure_ids, validate_tag_member_ids,
        validate_world_position, Aabb, BiomeIdsQuery, BiomeIdsResponse, BlockPropertiesMutation,
        BlockPropertiesQuery, BlockPropertiesResponse, BlockProperty, BlockStateBatch,
        BlockStateQuery, BlockStateSnapshot, BlockTagQuery, BlockTagResponse, ChunkPos,
        DaylightCycleSnapshot, FluidStateQuery, FluidStateResponse, ItemSpawnVelocity,
        ItemTagQuery, ItemTagResponse, LoadedChunksQuery, LoadedChunksResponse, ProjectileVelocity,
        RaycastBlockMode, RaycastFluidMode, RaycastHit, RaycastPoint, StructuresAtQuery,
        StructuresAtResponse, TagMembersQuery, TagMembersResponse, TagRegistry,
        VanillaProjectileKind, WeatherCondition, WeatherSnapshot, WorldCollisionQuery,
        WorldEntitySpawnRequest, WorldGameRule, WorldGameRuleMutation, WorldGameRuleQuery,
        WorldGameRuleSnapshot, WorldIntegerGameRule, WorldIntegerGameRuleMutation,
        WorldIntegerGameRuleQuery, WorldIntegerGameRuleSnapshot, WorldItemSpawnRequest,
        WorldProjectileSpawnRequest, WorldRaycastQuery, MAX_WEATHER_DURATION_TICKS,
        MAX_WORLD_BIOME_IDS, MAX_WORLD_BIOME_IDS_RESPONSE_BYTES, MAX_WORLD_BIOME_ID_BYTES,
        MAX_WORLD_BLOCK_PROPERTIES, MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES,
        MAX_WORLD_BLOCK_PROPERTY_UPDATES, MAX_WORLD_BLOCK_TAG_ID_BYTES,
        MAX_WORLD_BLOCK_TAG_RESPONSE_BYTES, MAX_WORLD_BLOCK_TICK_DELAY_TICKS,
        MAX_WORLD_DAY_TIME_TICKS, MAX_WORLD_FLUID_STATE_RESPONSE_BYTES, MAX_WORLD_GAME_TIME_TICKS,
        MAX_WORLD_ITEM_SPAWNS_PER_TICK, MAX_WORLD_ITEM_SPAWN_SPEED, MAX_WORLD_ITEM_STACK_COUNT,
        MAX_WORLD_ITEM_TAG_ID_BYTES, MAX_WORLD_ITEM_TAG_QUERY_BYTES,
        MAX_WORLD_ITEM_TAG_RESPONSE_BYTES, MAX_WORLD_PROJECTILE_SPEED, MAX_WORLD_STRUCTURES_AT,
        MAX_WORLD_STRUCTURES_AT_RESPONSE_BYTES, MAX_WORLD_STRUCTURE_ID_BYTES,
        MAX_WORLD_TAG_ID_BYTES, MAX_WORLD_TAG_MEMBERS, MAX_WORLD_TAG_MEMBERS_OUTPUT_BYTES,
        MAX_WORLD_TAG_MEMBERS_QUERY_BYTES, MAX_WORLD_TAG_MEMBERS_RESPONSE_BYTES,
        MIN_WORLD_PROJECTILE_SPEED,
    };
    use crate::{BlockPos, Direction, ErrorCode, StableDecode, WorldHandle};

    #[test]
    fn weather_snapshot_has_a_versioned_little_endian_contract() {
        let snapshot = WeatherSnapshot {
            raining: true,
            thundering: false,
            clear_time: 0,
            rain_time: 12_000,
            thunder_time: 40,
        };
        let encoded = crate::to_bytes(&snapshot).unwrap();
        assert_eq!(
            encoded,
            [
                1, 0, 0, 0, // schema
                1, 0, // rain, thunder
                0, 0, 0, 0, // clear
                0xe0, 0x2e, 0, 0, // rain
                40, 0, 0, 0, // thunder
            ]
        );
        assert_eq!(
            WeatherSnapshot::decode(&mut crate::Decoder::new(&encoded).unwrap()).unwrap(),
            snapshot
        );
    }

    #[test]
    fn loaded_chunk_query_and_response_match_the_host_codec() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let center = BlockPos {
            x: -17,
            y: 72,
            z: 31,
        };
        let query = crate::to_bytes(&LoadedChunksQuery {
            world,
            center,
            radius: 4,
        })
        .unwrap();
        assert_eq!(48, query.len());
        assert_eq!(1_u32.to_le_bytes(), query[0..4]);
        assert_eq!(1_u32.to_le_bytes(), query[28..32]);
        assert_eq!((-17_i32).to_le_bytes(), query[32..36]);
        assert_eq!(4_u32.to_le_bytes(), query[44..48]);

        let chunks = vec![
            ChunkPos { x: -2, z: 7 },
            ChunkPos { x: -1, z: -4 },
            ChunkPos { x: 3, z: 1 },
        ];
        let encoded = crate::to_bytes(&LoadedChunksResponse {
            chunks: chunks.clone(),
        })
        .unwrap();
        assert_eq!(8 + 3 * 12, encoded.len());
        assert_eq!(
            chunks,
            crate::from_bytes::<LoadedChunksResponse>(&encoded)
                .unwrap()
                .chunks
        );
    }

    #[test]
    fn loaded_chunk_query_and_results_are_bounded_and_canonical() {
        let center = BlockPos { x: -1, y: 0, z: 15 };
        assert!(validate_loaded_chunk_query(center, 16).is_ok());
        assert_eq!(
            ErrorCode::InvalidArgument,
            validate_loaded_chunk_query(center, 17).unwrap_err().code()
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            validate_loaded_chunk_query(
                BlockPos {
                    x: i32::MAX,
                    y: 0,
                    z: 0,
                },
                0,
            )
            .unwrap_err()
            .code()
        );

        let valid = [ChunkPos { x: -1, z: -1 }, ChunkPos { x: 0, z: 0 }];
        assert!(validate_loaded_chunk_results(center, 1, &valid).is_ok());
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            validate_loaded_chunk_results(center, 1, &[ChunkPos { x: 10, z: 0 }])
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            validate_loaded_chunk_results(
                center,
                1,
                &[ChunkPos { x: 0, z: 0 }, ChunkPos { x: 0, z: 0 }]
            )
            .unwrap_err()
            .code()
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            loaded_chunks(WorldHandle::new(1, 2, 3, 4).unwrap(), center, 17)
                .unwrap_err()
                .code()
        );

        let oversized_response = [1_u32.to_le_bytes(), 513_u32.to_le_bytes()].concat();
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            crate::from_bytes::<LoadedChunksResponse>(&oversized_response)
                .unwrap_err()
                .code(),
            "response count is capped before allocating or reading entries"
        );
    }

    #[test]
    fn biome_id_query_response_and_enumeration_are_bounded_and_canonical() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let query = crate::to_bytes(&BiomeIdsQuery { world }).unwrap();
        assert_eq!(28, query.len());
        assert_eq!(1_u32.to_le_bytes(), query[0..4]);

        let ids = vec!["alpha:forest".to_owned(), "minecraft:plains".to_owned()];
        let encoded = crate::to_bytes(&BiomeIdsResponse { ids: ids.clone() }).unwrap();
        assert_eq!(
            ids,
            crate::from_bytes::<BiomeIdsResponse>(&encoded).unwrap().ids
        );
        assert_eq!(
            1_040_008, MAX_WORLD_BIOME_IDS_RESPONSE_BYTES,
            "the maximum response remains below the one MiB ABI payload cap"
        );
        assert_eq!(256, MAX_WORLD_BIOME_ID_BYTES);

        assert!(validate_biome_ids(&ids).is_ok());
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            validate_biome_ids(&["zeta:forest".to_owned(), "alpha:plains".to_owned()])
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            validate_biome_ids(&["Minecraft:Plains".to_owned()])
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            validate_biome_ids(&[format!("mod:{}", "a".repeat(MAX_WORLD_BIOME_ID_BYTES))])
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            biome_ids(world).unwrap_err().code(),
            "native targets fail closed without fabricating registered biomes"
        );

        let oversized_count = [
            1_u32.to_le_bytes(),
            (MAX_WORLD_BIOME_IDS as u32 + 1).to_le_bytes(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            crate::from_bytes::<BiomeIdsResponse>(&oversized_count)
                .unwrap_err()
                .code(),
            "response count is bounded before allocating strings"
        );
    }

    #[test]
    fn structure_query_response_and_enumeration_are_bounded_and_canonical() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos {
            x: -12,
            y: 80,
            z: 34,
        };
        let query = crate::to_bytes(&StructuresAtQuery { world, position }).unwrap();
        assert_eq!(44, query.len());
        assert_eq!(1_u32.to_le_bytes(), query[0..4]);
        assert_eq!(1_u32.to_le_bytes(), query[28..32], "nested BlockPos schema");
        assert_eq!((-12_i32).to_le_bytes(), query[32..36]);
        assert_eq!(80_i32.to_le_bytes(), query[36..40]);
        assert_eq!(34_i32.to_le_bytes(), query[40..44]);

        let ids = vec![
            "minecraft:ancient_city".to_owned(),
            "minecraft:stronghold".to_owned(),
        ];
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&1_u32.to_le_bytes());
        encoded.push(1);
        encoded.extend_from_slice(&(ids.len() as u32).to_le_bytes());
        for id in &ids {
            encoded.extend_from_slice(&(id.len() as u32).to_le_bytes());
            encoded.extend_from_slice(id.as_bytes());
        }
        assert_eq!(
            Some(ids.clone()),
            crate::from_bytes::<StructuresAtResponse>(&encoded)
                .unwrap()
                .result
        );

        let unavailable = [
            1_u32.to_le_bytes().as_slice(),
            [0_u8].as_slice(),
            0_u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            None,
            crate::from_bytes::<StructuresAtResponse>(&unavailable)
                .unwrap()
                .result
        );
        assert_eq!(
            9 + MAX_WORLD_STRUCTURES_AT * (4 + MAX_WORLD_STRUCTURE_ID_BYTES),
            MAX_WORLD_STRUCTURES_AT_RESPONSE_BYTES
        );
        assert!(validate_structure_ids(&ids).is_ok());
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            validate_structure_ids(&["zeta:ruin".to_owned(), "alpha:fort".to_owned()])
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            validate_structure_ids(&["Minecraft:Stronghold".to_owned()])
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            validate_structure_ids(&[format!("mod:{}", "a".repeat(MAX_WORLD_STRUCTURE_ID_BYTES))])
                .unwrap_err()
                .code()
        );
        let too_many: Vec<String> = (0..=MAX_WORLD_STRUCTURES_AT)
            .map(|index| format!("mod:structure_{index:03}"))
            .collect();
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            validate_structure_ids(&too_many).unwrap_err().code()
        );

        assert_eq!(
            ErrorCode::InvalidArgument,
            validate_world_position(BlockPos {
                x: 30_000_001,
                y: 0,
                z: 0
            })
            .unwrap_err()
            .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            structures_at(world, BlockPos { x: 0, y: 64, z: 0 })
                .unwrap_err()
                .code(),
            "native targets fail closed without fabricating structure results"
        );

        let invalid_flag = [
            1_u32.to_le_bytes().as_slice(),
            [2_u8].as_slice(),
            0_u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::InvalidArgument,
            crate::from_bytes::<StructuresAtResponse>(&invalid_flag)
                .unwrap_err()
                .code()
        );
        let unavailable_with_count = [
            1_u32.to_le_bytes().as_slice(),
            [0_u8].as_slice(),
            1_u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::InvalidArgument,
            crate::from_bytes::<StructuresAtResponse>(&unavailable_with_count)
                .unwrap_err()
                .code()
        );
        let oversized_count = [
            1_u32.to_le_bytes().as_slice(),
            [1_u8].as_slice(),
            (MAX_WORLD_STRUCTURES_AT as u32 + 1)
                .to_le_bytes()
                .as_slice(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            crate::from_bytes::<StructuresAtResponse>(&oversized_count)
                .unwrap_err()
                .code()
        );
    }

    #[test]
    fn block_tag_query_and_response_match_the_host_wire_contract() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos {
            x: -12,
            y: 80,
            z: 34,
        };
        let tag_id = "minecraft:logs";
        let query = crate::to_bytes(&BlockTagQuery {
            world,
            position,
            tag_id: tag_id.to_owned(),
        })
        .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(-12_i32).to_le_bytes());
        expected.extend_from_slice(&80_i32.to_le_bytes());
        expected.extend_from_slice(&34_i32.to_le_bytes());
        expected.extend_from_slice(&(tag_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(tag_id.as_bytes());
        assert_eq!(expected, query);

        for (available, contains, expected_result) in
            [(0_u8, 0_u8, None), (1, 0, Some(false)), (1, 1, Some(true))]
        {
            let encoded = [1_u32.to_le_bytes().as_slice(), &[available, contains]].concat();
            assert_eq!(
                expected_result,
                crate::from_bytes::<BlockTagResponse>(&encoded)
                    .unwrap()
                    .result
            );
        }
        assert_eq!(MAX_WORLD_BLOCK_TAG_RESPONSE_BYTES, 6);
        assert_eq!(MAX_WORLD_BLOCK_TAG_ID_BYTES, 256);
        assert_eq!(
            ErrorCode::InvalidArgument,
            block_has_tag(world, position, "Minecraft:logs")
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            block_has_tag(
                world,
                position,
                &format!("mod:{}", "a".repeat(MAX_WORLD_BLOCK_TAG_ID_BYTES))
            )
            .unwrap_err()
            .code()
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            block_has_tag(
                world,
                BlockPos {
                    x: 30_000_001,
                    y: 64,
                    z: 0
                },
                "minecraft:logs"
            )
            .unwrap_err()
            .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            block_has_tag(world, position, tag_id).unwrap_err().code(),
            "native targets must not fabricate Minecraft tag membership"
        );
        let invalid = [1_u32.to_le_bytes().as_slice(), &[0_u8, 1_u8]].concat();
        assert_eq!(
            ErrorCode::InvalidArgument,
            crate::from_bytes::<BlockTagResponse>(&invalid)
                .unwrap_err()
                .code()
        );
    }

    #[test]
    fn item_tag_query_and_response_match_the_host_wire_contract() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let item_id = "minecraft:oak_planks";
        let tag_id = "minecraft:planks";
        let query = crate::to_bytes(&ItemTagQuery {
            world,
            item_id: item_id.to_owned(),
            tag_id: tag_id.to_owned(),
        })
        .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&(item_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(item_id.as_bytes());
        expected.extend_from_slice(&(tag_id.len() as u32).to_le_bytes());
        expected.extend_from_slice(tag_id.as_bytes());
        assert_eq!(expected, query);
        assert_eq!(MAX_WORLD_ITEM_TAG_QUERY_BYTES, 548);
        assert_eq!(MAX_WORLD_ITEM_TAG_ID_BYTES, 256);
        assert_eq!(MAX_WORLD_ITEM_TAG_RESPONSE_BYTES, 5);
        assert!(
            !crate::from_bytes::<ItemTagResponse>(&[1_u32.to_le_bytes().as_slice(), &[0]].concat())
                .unwrap()
                .contains
        );
        assert!(
            crate::from_bytes::<ItemTagResponse>(&[1_u32.to_le_bytes().as_slice(), &[1]].concat())
                .unwrap()
                .contains
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            item_has_tag(world, "Minecraft:oak_planks", tag_id)
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            item_has_tag(
                world,
                &format!("mod:{}", "a".repeat(MAX_WORLD_ITEM_TAG_ID_BYTES)),
                tag_id
            )
            .unwrap_err()
            .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            item_has_tag(world, item_id, tag_id).unwrap_err().code(),
            "native targets must not fabricate Minecraft item-tag membership"
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            crate::from_bytes::<ItemTagResponse>(&[1_u32.to_le_bytes().as_slice(), &[2]].concat())
                .unwrap_err()
                .code()
        );
    }

    #[test]
    fn registry_tag_member_query_and_response_match_the_host_wire_contract() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let query = crate::to_bytes(&TagMembersQuery {
            world,
            registry_type: TagRegistry::EntityType as u32,
            tag_id: "minecraft:skeletons".to_owned(),
        })
        .unwrap();
        let mut expected_query = Vec::new();
        expected_query.extend_from_slice(&1_u32.to_le_bytes());
        expected_query.extend_from_slice(&1_u64.to_le_bytes());
        expected_query.extend_from_slice(&2_u64.to_le_bytes());
        expected_query.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected_query.extend_from_slice(&(TagRegistry::EntityType as u32).to_le_bytes());
        expected_query.extend_from_slice(&("minecraft:skeletons".len() as u32).to_le_bytes());
        expected_query.extend_from_slice(b"minecraft:skeletons");
        assert_eq!(expected_query, query);
        assert_eq!(MAX_WORLD_TAG_MEMBERS_QUERY_BYTES, 292);
        assert_eq!(MAX_WORLD_TAG_MEMBERS_RESPONSE_BYTES, 266_249);
        assert_eq!(MAX_WORLD_TAG_MEMBERS_OUTPUT_BYTES, 266_253);
        assert_eq!(MAX_WORLD_TAG_MEMBERS, 1024);
        assert_eq!(MAX_WORLD_TAG_ID_BYTES, 256);

        let available = [
            1_u32.to_le_bytes().as_slice(),
            &[1_u8],
            2_u32.to_le_bytes().as_slice(),
            18_u32.to_le_bytes().as_slice(),
            b"minecraft:skeleton".as_slice(),
            25_u32.to_le_bytes().as_slice(),
            b"minecraft:wither_skeleton".as_slice(),
        ]
        .concat();
        assert_eq!(
            Some(vec![
                "minecraft:skeleton".to_owned(),
                "minecraft:wither_skeleton".to_owned()
            ]),
            crate::from_bytes::<TagMembersResponse>(&available)
                .unwrap()
                .members
        );
        let empty = [
            1_u32.to_le_bytes().as_slice(),
            &[1_u8],
            0_u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            Some(Vec::new()),
            crate::from_bytes::<TagMembersResponse>(&empty)
                .unwrap()
                .members
        );
        let absent = [
            1_u32.to_le_bytes().as_slice(),
            &[0_u8],
            0_u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            None,
            crate::from_bytes::<TagMembersResponse>(&absent)
                .unwrap()
                .members
        );

        let unsorted = [
            1_u32.to_le_bytes().as_slice(),
            &[1_u8],
            2_u32.to_le_bytes().as_slice(),
            25_u32.to_le_bytes().as_slice(),
            b"minecraft:wither_skeleton".as_slice(),
            18_u32.to_le_bytes().as_slice(),
            b"minecraft:skeleton".as_slice(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            crate::from_bytes::<TagMembersResponse>(&unsorted)
                .unwrap_err()
                .code()
        );
        let unavailable_with_members = [
            1_u32.to_le_bytes().as_slice(),
            &[0_u8],
            1_u32.to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::InvalidArgument,
            crate::from_bytes::<TagMembersResponse>(&unavailable_with_members)
                .unwrap_err()
                .code()
        );
        let oversized = [
            1_u32.to_le_bytes().as_slice(),
            &[1_u8],
            (MAX_WORLD_TAG_MEMBERS as u32 + 1).to_le_bytes().as_slice(),
        ]
        .concat();
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            crate::from_bytes::<TagMembersResponse>(&oversized)
                .unwrap_err()
                .code()
        );
        let too_many = (0..=MAX_WORLD_TAG_MEMBERS)
            .map(|index| format!("sjml_test:entry_{index:04}"))
            .collect::<Vec<_>>();
        assert_eq!(
            ErrorCode::PayloadTooLarge,
            validate_tag_member_ids(&too_many).unwrap_err().code()
        );
        assert_eq!(
            ErrorCode::InvalidArgument,
            tag_members(world, TagRegistry::Block, "#minecraft:logs")
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            tag_members(world, TagRegistry::Block, "minecraft:logs")
                .unwrap_err()
                .code(),
            "native targets must not fabricate registry tag contents"
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            entity_type_tag_members(world, "minecraft:skeletons")
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            fluid_tag_members(world, "minecraft:water")
                .unwrap_err()
                .code()
        );
        assert_eq!(
            ErrorCode::UnknownErrorCode,
            item_tag_members(world, "minecraft:planks")
                .unwrap_err()
                .code()
        );
    }

    #[test]
    fn boolean_game_rule_request_and_snapshot_match_host_wire_schema() {
        assert_eq!(WorldGameRule::DaylightCycle as u32, 0);
        assert_eq!(WorldGameRule::WeatherCycle as u32, 1);
        assert_eq!(WorldGameRule::KeepInventory as u32, 4);
        assert_eq!(WorldGameRule::WardenSpawning as u32, 17);

        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let query = crate::to_bytes(&WorldGameRuleQuery {
            world,
            rule_id: WorldGameRule::KeepInventory as u32,
        })
        .unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&4_u32.to_le_bytes());
        assert_eq!(query, expected);
        assert_eq!(query.len(), 32);

        let mutation = crate::to_bytes(&WorldGameRuleMutation {
            world,
            rule_id: WorldGameRule::KeepInventory as u32,
            enabled: false,
        })
        .unwrap();
        assert_eq!(mutation.len(), 33);
        assert_eq!(mutation.last(), Some(&0));
        let snapshot = crate::from_bytes::<WorldGameRuleSnapshot>(&[1, 0, 0, 0, 1]).unwrap();
        assert!(snapshot.enabled);
    }

    #[test]
    fn integer_game_rule_request_mutation_and_range_match_host_contract() {
        assert_eq!(WorldIntegerGameRule::MaxEntityCramming as u32, 0);
        assert_eq!(WorldIntegerGameRule::PlayersSleepingPercentage as u32, 1);
        assert_eq!(WorldIntegerGameRule::RandomTickSpeed as u32, 2);
        assert_eq!(WorldIntegerGameRule::SpawnRadius as u32, 3);
        assert_eq!(WorldIntegerGameRule::RandomTickSpeed.min_value(), 0);
        assert_eq!(WorldIntegerGameRule::RandomTickSpeed.max_value(), 4096);

        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let query = crate::to_bytes(&WorldIntegerGameRuleQuery {
            world,
            rule_id: WorldIntegerGameRule::RandomTickSpeed as u32,
        })
        .unwrap();
        assert_eq!(query.len(), 32);
        assert_eq!(&query[28..32], &2_u32.to_le_bytes());

        let mutation = crate::to_bytes(&WorldIntegerGameRuleMutation {
            world,
            rule_id: WorldIntegerGameRule::SpawnRadius as u32,
            value: 64,
        })
        .unwrap();
        assert_eq!(mutation.len(), 36);
        assert_eq!(&mutation[28..32], &3_u32.to_le_bytes());
        assert_eq!(&mutation[32..36], &64_i32.to_le_bytes());

        let snapshot = crate::from_bytes::<WorldIntegerGameRuleSnapshot>(&[
            1, 0, 0, 0, 0xff, 0xff, 0xff, 0xff,
        ])
        .unwrap();
        assert_eq!(
            snapshot.value, -1,
            "reads preserve Minecraft values set outside SDK bounds"
        );

        assert_eq!(
            integer_game_rule(world, WorldIntegerGameRule::RandomTickSpeed)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_integer_game_rule(world, WorldIntegerGameRule::RandomTickSpeed, -1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_integer_game_rule(world, WorldIntegerGameRule::RandomTickSpeed, 4097)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_integer_game_rule(world, WorldIntegerGameRule::RandomTickSpeed, 0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn set_weather_validates_duration_before_host_access() {
        let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(WeatherCondition::Clear as u32, 0);
        assert_eq!(WeatherCondition::Rain as u32, 1);
        assert_eq!(WeatherCondition::Thunder as u32, 2);
        for duration in [0, MAX_WEATHER_DURATION_TICKS + 1] {
            assert_eq!(
                super::set_weather(handle, WeatherCondition::Rain, duration)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            super::set_weather(handle, WeatherCondition::Clear, 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn day_time_read_and_set_time_validate_the_mutation_bound_before_host_access() {
        let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            day_time(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        for day_time in [-1, MAX_WORLD_DAY_TIME_TICKS + 1] {
            assert_eq!(
                set_time(handle, day_time).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_time(handle, 0).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_time(handle, MAX_WORLD_DAY_TIME_TICKS)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn game_time_read_and_set_game_time_validate_mutation_bound_before_host_access() {
        let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            super::game_time(handle).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        for game_time in [-1, MAX_WORLD_GAME_TIME_TICKS + 1] {
            assert_eq!(
                set_game_time(handle, game_time).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_game_time(handle, 0).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_game_time(handle, MAX_WORLD_GAME_TIME_TICKS)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn daylight_cycle_snapshot_has_a_versioned_boolean_contract() {
        let enabled = crate::to_bytes(&DaylightCycleSnapshot { enabled: true }).unwrap();
        assert_eq!(enabled, [1, 0, 0, 0, 1]);
        let disabled = crate::to_bytes(&DaylightCycleSnapshot { enabled: false }).unwrap();
        assert_eq!(disabled, [1, 0, 0, 0, 0]);
        assert!(daylight_cycle_enabled(WorldHandle::new(1, 2, 3, 4).unwrap()).is_err());
    }

    #[test]
    fn daylight_cycle_write_requires_the_wasm_host() {
        let handle = WorldHandle::new(1, 2, 3, 4).unwrap();
        for enabled in [false, true] {
            assert_eq!(
                set_daylight_cycle_enabled(handle, enabled)
                    .unwrap_err()
                    .code(),
                ErrorCode::UnknownErrorCode
            );
        }
    }

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
    fn block_property_mutation_matches_java_schema_and_is_explicit() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let mutation = BlockPropertiesMutation {
            world,
            position: BlockPos { x: 4, y: 64, z: -3 },
            expected_block_id: "minecraft:oak_stairs".to_owned(),
            properties: vec![
                BlockProperty {
                    name: "facing".to_owned(),
                    value: "east".to_owned(),
                },
                BlockProperty {
                    name: "waterlogged".to_owned(),
                    value: "true".to_owned(),
                },
            ],
        };
        let encoded = crate::to_bytes(&mutation).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        for coordinate in [4_i32, 64, -3] {
            expected.extend_from_slice(&coordinate.to_le_bytes());
        }
        put_test_string(&mut expected, "minecraft:oak_stairs");
        expected.extend_from_slice(&2_u32.to_le_bytes());
        for (name, value) in [("facing", "east"), ("waterlogged", "true")] {
            expected.extend_from_slice(&1_u32.to_le_bytes());
            put_test_string(&mut expected, name);
            put_test_string(&mut expected, value);
        }
        assert_eq!(encoded, expected);
        assert!(encoded.len() <= MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES);
    }

    #[test]
    fn block_property_writes_reject_invalid_input_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        let property = |name: &str, value: &str| BlockProperty {
            name: name.to_owned(),
            value: value.to_owned(),
        };
        assert_eq!(
            set_block_properties(world, position, "stone", &[property("axis", "x")])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_block_properties(
                world,
                BlockPos {
                    x: i32::MAX,
                    ..position
                },
                "minecraft:oak_log",
                &[property("axis", "x")]
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_block_properties(world, position, "minecraft:oak_log", &[])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_block_property_updates(&[property("axis", "x"), property("axis", "y")])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_block_property_updates(&[property("Axis", "x")])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_block_property_updates(&[property("axis", "")])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let too_many: Vec<_> = (0..=MAX_WORLD_BLOCK_PROPERTY_UPDATES)
            .map(|index| property(&format!("p{index:02}"), "x"))
            .collect();
        assert_eq!(
            validate_block_property_updates(&too_many)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_block_properties(
                world,
                position,
                "minecraft:oak_log",
                &[property("axis", "x")]
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    fn put_test_string(output: &mut Vec<u8>, value: &str) {
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn scheduled_block_tick_validates_id_and_delay_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = BlockPos { x: 0, y: 64, z: 0 };
        for invalid in [
            "stone",
            "Minecraft:stone",
            "minecraft:Bad",
            "minecraft:bad path",
        ] {
            assert_eq!(
                schedule_block_tick(world, position, invalid, 20)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for invalid in [0, MAX_WORLD_BLOCK_TICK_DELAY_TICKS + 1] {
            assert_eq!(
                schedule_block_tick(world, position, "minecraft:stone", invalid)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            schedule_block_tick(world, position, "minecraft:stone", 20)
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

    #[test]
    fn projectile_spawn_request_matches_the_java_little_endian_contract() {
        let request = WorldProjectileSpawnRequest {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            projectile: VanillaProjectileKind::Snowball as u8,
            x: 1.5,
            y: 64.0,
            z: -2.25,
            velocity_x: 1.25,
            velocity_y: 0.5,
            velocity_z: -0.25,
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.push(0);
        expected.extend_from_slice(&1.5_f64.to_le_bytes());
        expected.extend_from_slice(&64.0_f64.to_le_bytes());
        expected.extend_from_slice(&(-2.25_f64).to_le_bytes());
        expected.extend_from_slice(&1.25_f64.to_le_bytes());
        expected.extend_from_slice(&0.5_f64.to_le_bytes());
        expected.extend_from_slice(&(-0.25_f64).to_le_bytes());
        assert_eq!(crate::to_bytes(&request).unwrap(), expected);
    }

    #[test]
    fn item_spawn_request_matches_the_java_little_endian_contract() {
        let request = WorldItemSpawnRequest {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            item_id: "minecraft:diamond".to_owned(),
            count: 3,
            x: 1.5,
            y: 64.0,
            z: -2.25,
            velocity_x: 0.25,
            velocity_y: 1.0,
            velocity_z: -0.5,
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&17_u32.to_le_bytes());
        expected.extend_from_slice(b"minecraft:diamond");
        expected.extend_from_slice(&3_u32.to_le_bytes());
        expected.extend_from_slice(&1.5_f64.to_le_bytes());
        expected.extend_from_slice(&64.0_f64.to_le_bytes());
        expected.extend_from_slice(&(-2.25_f64).to_le_bytes());
        expected.extend_from_slice(&0.25_f64.to_le_bytes());
        expected.extend_from_slice(&1.0_f64.to_le_bytes());
        expected.extend_from_slice(&(-0.5_f64).to_le_bytes());
        assert_eq!(crate::to_bytes(&request).unwrap(), expected);
    }

    #[test]
    fn item_spawn_rejects_invalid_ids_counts_positions_and_velocity_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let position = RaycastPoint::new(0.5, 64.0, 0.5);
        let velocity = ItemSpawnVelocity::new(0.0, 0.0, 0.0);
        for id in ["", "diamond", "Minecraft:diamond", "minecraft:bad path"] {
            assert_eq!(
                spawn_item(world, id, 1, position, velocity)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for count in [0, MAX_WORLD_ITEM_STACK_COUNT + 1] {
            assert_eq!(
                spawn_item(world, "minecraft:diamond", count, position, velocity)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for velocity in [
            ItemSpawnVelocity::new(4.01, 0.0, 0.0),
            ItemSpawnVelocity::new(3.0, 3.0, 0.0),
            ItemSpawnVelocity::new(f64::INFINITY, 0.0, 0.0),
            ItemSpawnVelocity::new(f64::NAN, 0.0, 0.0),
        ] {
            assert_eq!(
                spawn_item(world, "minecraft:diamond", 1, position, velocity)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            spawn_item(
                world,
                "minecraft:diamond",
                1,
                RaycastPoint::new(f64::NAN, 64.0, 0.5),
                ItemSpawnVelocity::new(0.0, 0.0, 0.0),
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(MAX_WORLD_ITEM_SPAWNS_PER_TICK, 16);
        assert_eq!(MAX_WORLD_ITEM_SPAWN_SPEED, 4.0);
    }

    #[test]
    fn projectile_spawn_rejects_bad_positions_and_speeds_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        let kind = VanillaProjectileKind::Egg;
        let position = RaycastPoint::new(0.5, 64.0, 0.5);
        for velocity in [
            ProjectileVelocity::new(0.0, 0.0, 0.0),
            ProjectileVelocity::new(4.01, 0.0, 0.0),
            ProjectileVelocity::new(f64::INFINITY, 0.0, 0.0),
            ProjectileVelocity::new(f64::NAN, 0.0, 0.0),
        ] {
            assert_eq!(
                spawn_projectile(world, kind, position, velocity)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            spawn_projectile(
                world,
                kind,
                RaycastPoint::new(30_000_001.0, 64.0, 0.5),
                ProjectileVelocity::new(1.0, 0.0, 0.0),
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            spawn_projectile(
                world,
                kind,
                position,
                ProjectileVelocity::new(MIN_WORLD_PROJECTILE_SPEED, 0.0, 0.0),
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(MAX_WORLD_PROJECTILE_SPEED, 4.0);
    }

    #[test]
    fn block_properties_query_matches_the_java_versioned_position_contract() {
        let query = BlockPropertiesQuery {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            position: BlockPos { x: -3, y: 64, z: 8 },
        };
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&1_u64.to_le_bytes());
        expected.extend_from_slice(&2_u64.to_le_bytes());
        expected.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.extend_from_slice(&(-3_i32).to_le_bytes());
        expected.extend_from_slice(&64_i32.to_le_bytes());
        expected.extend_from_slice(&8_i32.to_le_bytes());
        assert_eq!(crate::to_bytes(&query).unwrap(), expected);
    }

    #[test]
    fn block_property_value_has_a_stable_versioned_schema_and_host_values_are_checked() {
        let property = BlockProperty {
            name: "axis".to_owned(),
            value: "x".to_owned(),
        };
        let encoded_property = crate::to_bytes(&property).unwrap();
        let mut expected_property = 1_u32.to_le_bytes().to_vec();
        expected_property.extend_from_slice(&4_u32.to_le_bytes());
        expected_property.extend_from_slice(b"axis");
        expected_property.extend_from_slice(&1_u32.to_le_bytes());
        expected_property.extend_from_slice(b"x");
        assert_eq!(encoded_property, expected_property);
        assert_eq!(
            crate::from_bytes::<BlockProperty>(&encoded_property).unwrap(),
            property
        );

        let mut encoded_response = 1_u32.to_le_bytes().to_vec();
        encoded_response.push(1);
        encoded_response.extend_from_slice(&1_u32.to_le_bytes());
        encoded_response.extend_from_slice(&encoded_property);
        assert_eq!(
            crate::from_bytes::<BlockPropertiesResponse>(&encoded_response)
                .unwrap()
                .properties,
            Some(vec![property])
        );

        assert_eq!(
            super::validate_block_properties(Some(vec![BlockProperty {
                name: "UpperCase".to_owned(),
                value: "x".to_owned(),
            }]))
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            super::validate_block_properties(Some(vec![
                BlockProperty {
                    name: "axis".to_owned(),
                    value: "x".to_owned()
                },
                BlockProperty {
                    name: "axis".to_owned(),
                    value: "y".to_owned()
                },
            ]))
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        let too_many = (0..=MAX_WORLD_BLOCK_PROPERTIES)
            .map(|index| BlockProperty {
                name: format!("property_{index:02}"),
                value: "value".to_owned(),
            })
            .collect();
        assert_eq!(
            super::validate_block_properties(Some(too_many))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn fluid_state_query_and_snapshot_match_the_stable_little_endian_contract() {
        let query = FluidStateQuery {
            world: WorldHandle::new(1, 2, 3, 4).unwrap(),
            position: BlockPos { x: -3, y: 64, z: 8 },
        };
        let mut expected_query = Vec::new();
        expected_query.extend_from_slice(&1_u32.to_le_bytes());
        expected_query.extend_from_slice(&1_u64.to_le_bytes());
        expected_query.extend_from_slice(&2_u64.to_le_bytes());
        expected_query.extend_from_slice(&((4_u64 << 32) | 3).to_le_bytes());
        expected_query.extend_from_slice(&1_u32.to_le_bytes());
        expected_query.extend_from_slice(&(-3_i32).to_le_bytes());
        expected_query.extend_from_slice(&64_i32.to_le_bytes());
        expected_query.extend_from_slice(&8_i32.to_le_bytes());
        assert_eq!(crate::to_bytes(&query).unwrap(), expected_query);

        let state = super::FluidStateSnapshot {
            fluid_id: "minecraft:water".to_owned(),
            amount: 4,
            source: false,
            height: 0.5,
        };
        let encoded_state = crate::to_bytes(&state).unwrap();
        assert_eq!(
            encoded_state,
            [
                1_u32.to_le_bytes().as_slice(),
                15_u32.to_le_bytes().as_slice(),
                b"minecraft:water",
                &[4, 0],
                &0.5_f32.to_le_bytes(),
            ]
            .concat()
        );
        assert_eq!(
            crate::from_bytes::<super::FluidStateSnapshot>(&encoded_state).unwrap(),
            state
        );

        let encoded_response = crate::to_bytes(&FluidStateResponse {
            state: Some(state.clone()),
        })
        .unwrap();
        assert_eq!(
            crate::from_bytes::<FluidStateResponse>(&encoded_response)
                .unwrap()
                .state,
            Some(state)
        );
        assert_eq!(MAX_WORLD_FLUID_STATE_RESPONSE_BYTES, 275);
    }

    #[test]
    fn fluid_state_response_values_and_query_coordinates_are_checked() {
        let valid = super::FluidStateSnapshot {
            fluid_id: "minecraft:water".to_owned(),
            amount: 8,
            source: true,
            height: 1.0,
        };
        assert_eq!(
            super::validate_fluid_state(Some(valid.clone())).unwrap(),
            Some(valid)
        );
        assert_eq!(super::validate_fluid_state(None).unwrap(), None);
        for state in [
            super::FluidStateSnapshot {
                fluid_id: "Minecraft:water".to_owned(),
                amount: 8,
                source: true,
                height: 1.0,
            },
            super::FluidStateSnapshot {
                fluid_id: "minecraft:water".to_owned(),
                amount: 9,
                source: true,
                height: 1.0,
            },
            super::FluidStateSnapshot {
                fluid_id: "minecraft:water".to_owned(),
                amount: 4,
                source: false,
                height: f32::NAN,
            },
            super::FluidStateSnapshot {
                fluid_id: "minecraft:empty".to_owned(),
                amount: 0,
                source: false,
                height: 0.5,
            },
        ] {
            assert_eq!(
                super::validate_fluid_state(Some(state)).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }

        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            fluid_at(
                world,
                BlockPos {
                    x: i32::MIN,
                    y: 0,
                    z: 0,
                },
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            fluid_at(world, BlockPos { x: 0, y: 64, z: 0 })
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn block_property_queries_reject_out_of_range_coordinates_before_host_access() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            block_properties(
                world,
                BlockPos {
                    x: i32::MIN,
                    y: 0,
                    z: 0
                }
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn notify_neighbors_rejects_out_of_range_coordinates_and_requires_wasm_host() {
        let world = WorldHandle::new(1, 2, 3, 4).unwrap();
        assert_eq!(
            notify_neighbors_at(
                world,
                BlockPos {
                    x: i32::MIN,
                    y: 0,
                    z: 0,
                },
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            notify_neighbors_at(world, BlockPos { x: 0, y: 64, z: 0 })
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
