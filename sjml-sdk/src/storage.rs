//! Typed, mod-scoped persistent records in the active Minecraft save.
//!
//! Storage scopes are logical selectors. World-save scopes are persisted under the current save,
//! while [`StorageScope::ClientProfile`] is isolated to this local game profile. Every scope is
//! namespaced by mod ID; modules belonging to the same mod share that namespace. Reads and writes
//! remain permission checked by the host on every operation.

use crate::inventory::BlockPos;
use crate::network::Uuid;
use crate::serialization::{Decoder, Encoder};
use crate::{ErrorCode, SjmlError, StableDecode, StableEncode};
use core::fmt;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Maximum size of a complete persisted record envelope, including schema metadata.
pub const MAX_STORAGE_RECORD_BYTES: usize = crate::MAX_PAYLOAD_BYTES;
/// Maximum encoded payload for an atomic scope-migration batch.
pub const MAX_STORAGE_BATCH_BYTES: usize = crate::MAX_PAYLOAD_BYTES;
/// Maximum UTF-8 key size.
pub const MAX_STORAGE_KEY_BYTES: usize = 256;
/// Maximum encoded schema identity size.
pub const MAX_SCHEMA_ID_BYTES: usize = 256;
/// Maximum number of keys returned for one scope migration.
pub const MAX_STORAGE_KEYS: usize = crate::MAX_COLLECTION_ITEMS;
/// Maximum number of per-key migration-mode overrides retained by one guest instance.
pub const MAX_MIGRATION_MODE_OVERRIDES: usize = crate::MAX_COLLECTION_ITEMS;
/// Maximum number of typed migration edges registered by one module instance.
pub const MAX_STORAGE_MIGRATIONS: usize = 256;
const SCOPE_SCHEMA_VERSION: u32 = 1;
const RECORD_ENVELOPE_VERSION: u32 = 1;

/// Schema metadata emitted by `StableDecode` derive. It does not alter the record's stable bytes.
pub trait StableRecord {
    const SCHEMA_VERSION: u32;

    /// Returns the explicit `#[sjml(id = "...")]` identity or the derived Rust type name.
    fn schema_id() -> &'static str;
}

/// One persistent logical location. Save scopes belong to the active world save; client-profile
/// storage belongs to this local Minecraft installation and is available only to client modules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageScope {
    /// Local profile settings shared by this mod's client modules, independent of a world or server.
    ClientProfile,
    /// Shared by every module of this mod in the current save.
    Save,
    /// Persistent data for one canonical `namespace:path` dimension ID.
    Dimension(String),
    /// Persistent data for one player's UUID; it survives logout.
    Player(Uuid),
    /// Persistent data for one entity UUID; chunk unload and dimension transfer preserve it.
    Entity(Uuid),
    /// Persistent data anchored to a block entity's dimension and position. Moving the block
    /// creates a distinct scope at its new position.
    BlockEntity {
        dimension_id: String,
        position: BlockPos,
    },
}

impl StorageScope {
    /// Creates a validated dimension scope.
    pub fn dimension(dimension_id: impl Into<String>) -> StorageResult<Self> {
        let scope = Self::Dimension(dimension_id.into());
        validate_scope(&scope)?;
        Ok(scope)
    }

    /// Creates a validated block-entity scope.
    pub fn block_entity(
        dimension_id: impl Into<String>,
        position: BlockPos,
    ) -> StorageResult<Self> {
        let scope = Self::BlockEntity {
            dimension_id: dimension_id.into(),
            position,
        };
        validate_scope(&scope)?;
        Ok(scope)
    }
}

impl StableEncode for StorageScope {
    fn encode(&self, encoder: &mut Encoder) -> crate::SjmlResult<()> {
        validate_scope(self).map_err(StorageError::into_sjml_error)?;
        encoder.write_u32(SCOPE_SCHEMA_VERSION)?;
        match self {
            Self::ClientProfile => encoder.write_u8(5),
            Self::Save => encoder.write_u8(0),
            Self::Dimension(id) => {
                encoder.write_u8(1)?;
                id.encode(encoder)
            }
            Self::Player(uuid) => {
                encoder.write_u8(2)?;
                uuid.encode(encoder)
            }
            Self::Entity(uuid) => {
                encoder.write_u8(3)?;
                uuid.encode(encoder)
            }
            Self::BlockEntity {
                dimension_id,
                position,
            } => {
                encoder.write_u8(4)?;
                dimension_id.encode(encoder)?;
                position.encode(encoder)
            }
        }
    }
}

impl StableDecode for StorageScope {
    fn decode(decoder: &mut Decoder<'_>) -> crate::SjmlResult<Self> {
        if decoder.read_u32()? != SCOPE_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported storage scope schema",
            ));
        }
        let scope = match decoder.read_u8()? {
            0 => Self::Save,
            1 => Self::Dimension(String::decode(decoder)?),
            2 => Self::Player(Uuid::decode(decoder)?),
            3 => Self::Entity(Uuid::decode(decoder)?),
            4 => Self::BlockEntity {
                dimension_id: String::decode(decoder)?,
                position: BlockPos::decode(decoder)?,
            },
            5 => Self::ClientProfile,
            _ => {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "unknown storage scope kind",
                ));
            }
        };
        validate_scope(&scope).map_err(StorageError::into_sjml_error)?;
        Ok(scope)
    }
}

/// Whether a registered migration runs implicitly from `read` or only on an explicit migrate call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationMode {
    Automatic,
    Manual,
}

/// Result of an explicit per-scope migration. Records are staged together and committed atomically
/// by the host. If a record fails validation or conversion, no staged update is committed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MigrationReport {
    pub migrated: usize,
    pub unchanged: usize,
    pub failures: Vec<MigrationFailure>,
    /// Whether the whole scope migration completed. This is false when any record failed before
    /// commit; in that case `migrated` is zero and no staged record was written.
    pub committed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MigrationFailure {
    pub key: String,
    pub error: StorageError,
}

/// Distinguishes absent keys from invalid records and schema problems.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageError {
    Host(SjmlError),
    InvalidKey(String),
    InvalidScope(String),
    InvalidSchema(String),
    CorruptRecord(String),
    IncompatibleSchema {
        stored_id: String,
        stored_version: u32,
        expected_id: String,
        expected_version: u32,
    },
    MigrationRequired,
    MigrationPathMissing,
    AmbiguousMigrationPath,
    MigrationCycle,
    DuplicateMigration,
    MigrationFailed(SjmlError),
}

pub type StorageResult<T> = core::result::Result<T, StorageError>;

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(error) => write!(formatter, "{error}"),
            Self::InvalidKey(message) => write!(formatter, "invalid storage key: {message}"),
            Self::InvalidScope(message) => write!(formatter, "invalid storage scope: {message}"),
            Self::InvalidSchema(message) => write!(formatter, "invalid storage schema: {message}"),
            Self::CorruptRecord(message) => write!(formatter, "corrupt storage record: {message}"),
            Self::IncompatibleSchema {
                stored_id,
                stored_version,
                expected_id,
                expected_version,
            } => write!(
                formatter,
                "stored schema {stored_id}@{stored_version} is incompatible with {expected_id}@{expected_version}"
            ),
            Self::MigrationRequired => formatter.write_str("this record requires an explicit schema migration"),
            Self::MigrationPathMissing => formatter.write_str("no registered storage migration path reaches the requested schema"),
            Self::AmbiguousMigrationPath => formatter.write_str("more than one storage migration path reaches the requested schema"),
            Self::MigrationCycle => formatter.write_str("storage migration registration would create a cycle"),
            Self::DuplicateMigration => formatter.write_str("the same storage migration is already registered"),
            Self::MigrationFailed(error) => write!(formatter, "storage migration failed: {error}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl StorageError {
    #[cfg(any(target_arch = "wasm32", test))]
    fn host_status(status: u32, message: &'static str) -> Self {
        Self::Host(SjmlError::from_abi_code(status, message))
    }

    fn into_sjml_error(self) -> SjmlError {
        match self {
            Self::Host(error) => error,
            other => SjmlError::new(ErrorCode::InvalidArgument, other.to_string()),
        }
    }
}

/// Reads and decodes one typed record. Missing keys return `Ok(None)`. Automatic registered
/// migration runs only when the registered path and this key's effective mode allow it.
pub fn read<T: StableRecord + StableEncode + StableDecode>(
    scope: &StorageScope,
    key: &str,
) -> StorageResult<Option<T>> {
    validate_key(key)?;
    let Some(bytes) = host_read(scope, key)? else {
        return Ok(None);
    };
    let stored = decode_envelope(&bytes)?;
    let expected = target_schema::<T>();
    validate_schema(&expected)?;
    if stored.schema == expected {
        return decode_record::<T>(&stored).map(Some);
    }

    let override_mode = migration_mode(scope, key)?;
    if override_mode == Some(MigrationMode::Manual) {
        return Err(StorageError::MigrationRequired);
    }
    let registry = MIGRATIONS.with(|registry| registry.borrow().clone());
    let path = if override_mode == Some(MigrationMode::Automatic) {
        registry.find_path(&stored.schema, &expected, true)?
    } else {
        registry.find_path(&stored.schema, &expected, false)?
    };
    let Some(path) = path else {
        if registry
            .find_path(&stored.schema, &expected, true)?
            .is_some()
        {
            return Err(StorageError::MigrationRequired);
        }
        return Err(no_migration(&stored.schema, &expected));
    };

    let upgraded = apply_path(bytes, &path)?;
    let decoded = decode_record::<T>(&decode_envelope(&upgraded)?)?;
    host_write(scope, key, &upgraded)?;
    Ok(Some(decoded))
}

/// Writes one typed record atomically. The host marks the owning `SavedData` dirty only after the
/// complete, bounded envelope has been validated and stored.
pub fn write<T: StableRecord + StableEncode + StableDecode>(
    scope: &StorageScope,
    key: &str,
    value: &T,
) -> StorageResult<()> {
    validate_key(key)?;
    let envelope = encode_record(value)?;
    host_write(scope, key, &envelope)
}

/// Removes one record and reports whether it existed.
pub fn remove(scope: &StorageScope, key: &str) -> StorageResult<bool> {
    validate_key(key)?;
    host_remove(scope, key)
}

/// Registers a typed migration edge that ordinary reads may run automatically.
pub fn register_migration<Old, New, F>(convert: F) -> StorageResult<()>
where
    Old: StableRecord + StableEncode + StableDecode + 'static,
    New: StableRecord + StableEncode + StableDecode + 'static,
    F: Fn(Old) -> crate::SjmlResult<New> + 'static,
{
    register_migration_with_mode::<Old, New, F>(MigrationMode::Automatic, convert)
}

/// Registers a typed migration edge with its default execution mode.
///
/// In automatic mode, a `read` can traverse this edge only if every edge in the unique path is
/// automatic. In manual mode, the edge is used only by `migrate_record`/`migrate_scope`, unless a
/// particular key explicitly overrides its mode with `set_migration_mode`.
pub fn register_migration_with_mode<Old, New, F>(
    mode: MigrationMode,
    convert: F,
) -> StorageResult<()>
where
    Old: StableRecord + StableEncode + StableDecode + 'static,
    New: StableRecord + StableEncode + StableDecode + 'static,
    F: Fn(Old) -> crate::SjmlResult<New> + 'static,
{
    let source = target_schema::<Old>();
    let target = target_schema::<New>();
    validate_schema(&source)?;
    validate_schema(&target)?;
    let migration_source = source.clone();
    let migration_target = target.clone();
    let migration = Rc::new(move |bytes: &[u8]| {
        let source_envelope = decode_envelope(bytes)?;
        if source_envelope.schema != migration_source {
            return Err(incompatible(&source_envelope.schema, &migration_source));
        }
        let old = decode_record::<Old>(&source_envelope)?;
        let next = convert(old).map_err(StorageError::MigrationFailed)?;
        let encoded = encode_record(&next)?;
        if decode_envelope(&encoded)?.schema != migration_target {
            return Err(StorageError::CorruptRecord(
                "migration output does not match its registered target schema".to_owned(),
            ));
        }
        Ok(encoded)
    });
    MIGRATIONS.with(|registry| {
        registry
            .borrow_mut()
            .register(source, target, mode, migration)
    })
}

/// Overrides the migration mode for one key. The setting belongs to this module instance and
/// should be registered again after restart; it never changes the record itself.
pub fn set_migration_mode(
    scope: &StorageScope,
    key: &str,
    mode: MigrationMode,
) -> StorageResult<()> {
    validate_key(key)?;
    let scope = scope_bytes(scope)?;
    MIGRATION_MODES.with(|modes| {
        let mut modes = modes.borrow_mut();
        let map_key = (scope, key.to_owned());
        if !modes.contains_key(&map_key) && modes.len() >= MAX_MIGRATION_MODE_OVERRIDES {
            return Err(StorageError::Host(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "storage migration mode override limit reached",
            )));
        }
        modes.insert(map_key, mode);
        Ok(())
    })
}

/// Removes a per-key mode override so the registered migration-edge defaults apply again.
pub fn clear_migration_mode(scope: &StorageScope, key: &str) -> StorageResult<bool> {
    validate_key(key)?;
    let scope = scope_bytes(scope)?;
    Ok(MIGRATION_MODES.with(|modes| {
        modes
            .borrow_mut()
            .remove(&(scope, key.to_owned()))
            .is_some()
    }))
}

/// Explicitly migrates one record regardless of its automatic/manual setting. Returns `false`
/// for a missing or already-current record. The original remains intact if any step or write fails.
pub fn migrate_record<T: StableRecord + StableEncode + StableDecode>(
    scope: &StorageScope,
    key: &str,
) -> StorageResult<bool> {
    validate_key(key)?;
    let Some(bytes) = host_read(scope, key)? else {
        return Ok(false);
    };
    let stored = decode_envelope(&bytes)?;
    let target = target_schema::<T>();
    validate_schema(&target)?;
    if stored.schema == target {
        let _ = decode_record::<T>(&stored)?;
        return Ok(false);
    }
    let registry = MIGRATIONS.with(|registry| registry.borrow().clone());
    let path = registry
        .find_path(&stored.schema, &target, true)?
        .ok_or_else(|| no_migration(&stored.schema, &target))?;
    let upgraded = apply_path(bytes, &path)?;
    let _ = decode_record::<T>(&decode_envelope(&upgraded)?)?;
    host_write(scope, key, &upgraded)?;
    Ok(true)
}

/// Explicitly migrates every key in a scope to `T`. This call never runs implicitly from `read`.
/// All changed values are staged before a single host batch write. If any key fails validation or
/// conversion, the report contains the failures, `committed` is false, and no record is changed.
pub fn migrate_scope<T: StableRecord + StableEncode + StableDecode>(
    scope: &StorageScope,
) -> StorageResult<MigrationReport> {
    let keys = host_keys(scope)?;
    let target = target_schema::<T>();
    validate_schema(&target)?;
    let registry = MIGRATIONS.with(|registry| registry.borrow().clone());
    let mut report = MigrationReport::default();
    let mut updates = Vec::new();
    // Include the count prefix and each key/envelope length prefix in the bounded ABI batch.
    let mut batch_bytes = core::mem::size_of::<u32>();

    for key in keys {
        let result = (|| {
            let Some(bytes) = host_read(scope, &key)? else {
                return Err(StorageError::CorruptRecord(
                    "host listed a storage key that has no record".to_owned(),
                ));
            };
            let stored = decode_envelope(&bytes)?;
            if stored.schema.id != target.id {
                // Scopes may contain records with unrelated stable identities. A typed scope
                // migration only targets this schema family and leaves every other record alone.
                report.unchanged += 1;
                return Ok(None);
            }
            if stored.schema == target {
                let _ = decode_record::<T>(&stored)?;
                report.unchanged += 1;
                return Ok(None);
            }

            let path = registry
                .find_path(&stored.schema, &target, true)?
                .ok_or_else(|| no_migration(&stored.schema, &target))?;
            let upgraded = apply_path(bytes, &path)?;
            let _ = decode_record::<T>(&decode_envelope(&upgraded)?)?;
            let entry_bytes = core::mem::size_of::<u32>()
                .checked_add(key.len())
                .and_then(|length| length.checked_add(core::mem::size_of::<u32>()))
                .and_then(|length| length.checked_add(upgraded.len()))
                .ok_or_else(|| {
                    StorageError::Host(SjmlError::new(
                        ErrorCode::ArithmeticOverflow,
                        "storage migration batch length overflowed",
                    ))
                })?;
            let next_batch_bytes = batch_bytes.checked_add(entry_bytes).ok_or_else(|| {
                StorageError::Host(SjmlError::new(
                    ErrorCode::ArithmeticOverflow,
                    "storage migration batch length overflowed",
                ))
            })?;
            if next_batch_bytes > MAX_STORAGE_BATCH_BYTES {
                return Err(StorageError::Host(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "atomic storage migration batch exceeds the 1 MiB ABI limit",
                )));
            }
            batch_bytes = next_batch_bytes;
            Ok(Some((key.clone(), upgraded)))
        })();

        match result {
            Ok(Some(update)) => updates.push(update),
            Ok(None) => {}
            Err(error) => report.failures.push(MigrationFailure { key, error }),
        }
    }

    if !report.failures.is_empty() {
        return Ok(report);
    }
    if !updates.is_empty() {
        host_write_batch(scope, &updates)?;
    }
    report.migrated = updates.len();
    report.committed = true;
    Ok(report)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Schema {
    id: String,
    version: u32,
}

type MigrationConverter = Rc<dyn Fn(&[u8]) -> StorageResult<Vec<u8>>>;

#[derive(Clone)]
struct MigrationEdge {
    source: Schema,
    target: Schema,
    mode: MigrationMode,
    apply: MigrationConverter,
}

#[derive(Clone, Default)]
struct MigrationRegistry {
    edges: Vec<MigrationEdge>,
}

impl MigrationRegistry {
    fn register(
        &mut self,
        source: Schema,
        target: Schema,
        mode: MigrationMode,
        apply: MigrationConverter,
    ) -> StorageResult<()> {
        if self
            .edges
            .iter()
            .any(|edge| edge.source == source && edge.target == target)
        {
            return Err(StorageError::DuplicateMigration);
        }
        if self.edges.len() >= MAX_STORAGE_MIGRATIONS {
            return Err(StorageError::Host(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "storage migration registration limit reached",
            )));
        }
        if self.has_path(&target, &source) {
            return Err(StorageError::MigrationCycle);
        }
        self.edges.push(MigrationEdge {
            source,
            target,
            mode,
            apply,
        });
        if self.has_ambiguous_path() {
            self.edges.pop();
            return Err(StorageError::AmbiguousMigrationPath);
        }
        Ok(())
    }

    fn find_path(
        &self,
        source: &Schema,
        target: &Schema,
        allow_manual: bool,
    ) -> StorageResult<Option<Vec<MigrationEdge>>> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        self.collect_paths(
            source,
            target,
            allow_manual,
            &mut visited,
            &mut Vec::new(),
            &mut paths,
        );
        match paths.len() {
            0 => Ok(None),
            1 => Ok(paths.pop()),
            _ => Err(StorageError::AmbiguousMigrationPath),
        }
    }

    fn collect_paths(
        &self,
        current: &Schema,
        target: &Schema,
        allow_manual: bool,
        visited: &mut HashSet<Schema>,
        path: &mut Vec<MigrationEdge>,
        paths: &mut Vec<Vec<MigrationEdge>>,
    ) {
        if paths.len() > 1 || !visited.insert(current.clone()) {
            return;
        }
        for edge in &self.edges {
            if &edge.source != current || (!allow_manual && edge.mode == MigrationMode::Manual) {
                continue;
            }
            path.push(edge.clone());
            if &edge.target == target {
                paths.push(path.clone());
            } else {
                self.collect_paths(&edge.target, target, allow_manual, visited, path, paths);
            }
            path.pop();
            if paths.len() > 1 {
                break;
            }
        }
        visited.remove(current);
    }

    fn has_path(&self, source: &Schema, target: &Schema) -> bool {
        fn visit(
            registry: &MigrationRegistry,
            current: &Schema,
            target: &Schema,
            visited: &mut HashSet<Schema>,
        ) -> bool {
            if current == target {
                return true;
            }
            if !visited.insert(current.clone()) {
                return false;
            }
            let found = registry.edges.iter().any(|edge| {
                edge.source == *current && visit(registry, &edge.target, target, visited)
            });
            visited.remove(current);
            found
        }

        visit(self, source, target, &mut HashSet::new())
    }

    fn has_ambiguous_path(&self) -> bool {
        let mut nodes = HashSet::new();
        for edge in &self.edges {
            nodes.insert(edge.source.clone());
            nodes.insert(edge.target.clone());
        }
        for source in &nodes {
            for target in &nodes {
                if source == target {
                    continue;
                }
                let mut paths = Vec::new();
                self.collect_paths(
                    source,
                    target,
                    true,
                    &mut HashSet::new(),
                    &mut Vec::new(),
                    &mut paths,
                );
                if paths.len() > 1 {
                    return true;
                }
            }
        }
        false
    }
}

thread_local! {
    static MIGRATIONS: RefCell<MigrationRegistry> = RefCell::new(MigrationRegistry::default());
    static MIGRATION_MODES: RefCell<HashMap<(Vec<u8>, String), MigrationMode>> = RefCell::new(HashMap::new());
}

fn migration_mode(scope: &StorageScope, key: &str) -> StorageResult<Option<MigrationMode>> {
    let encoded_scope = scope_bytes(scope)?;
    Ok(MIGRATION_MODES.with(|modes| {
        modes
            .borrow()
            .get(&(encoded_scope, key.to_owned()))
            .copied()
    }))
}

fn apply_path(mut bytes: Vec<u8>, path: &[MigrationEdge]) -> StorageResult<Vec<u8>> {
    for edge in path {
        bytes = (edge.apply)(&bytes)?;
        let envelope = decode_envelope(&bytes)?;
        if envelope.schema != edge.target {
            return Err(StorageError::CorruptRecord(
                "migration output schema does not match its registered target".to_owned(),
            ));
        }
    }
    Ok(bytes)
}

fn target_schema<T: StableRecord>() -> Schema {
    Schema {
        id: T::schema_id().to_owned(),
        version: T::SCHEMA_VERSION,
    }
}

struct RecordEnvelope {
    schema: Schema,
    value: Vec<u8>,
}

fn encode_record<T: StableRecord + StableEncode>(value: &T) -> StorageResult<Vec<u8>> {
    let schema = target_schema::<T>();
    validate_schema(&schema)?;
    let value = crate::to_bytes(value).map_err(StorageError::Host)?;
    ensure_value_schema(&value, &schema)?;
    let schema_id = schema.id.clone();
    let mut encoder = Encoder::new();
    encoder
        .write_u32(RECORD_ENVELOPE_VERSION)
        .and_then(|()| schema_id.encode(&mut encoder))
        .and_then(|()| encoder.write_u32(schema.version))
        .and_then(|()| encoder.write_length(value.len()))
        .and_then(|()| encoder.write_raw(&value))
        .map_err(StorageError::Host)?;
    Ok(encoder.into_bytes())
}

fn decode_envelope(bytes: &[u8]) -> StorageResult<RecordEnvelope> {
    let mut decoder = Decoder::new(bytes).map_err(corrupt)?;
    if decoder.read_u32().map_err(corrupt)? != RECORD_ENVELOPE_VERSION {
        return Err(StorageError::CorruptRecord(
            "unsupported storage record envelope version".to_owned(),
        ));
    }
    let id = String::decode(&mut decoder).map_err(corrupt)?;
    validate_schema_id(&id).map_err(|error| match error {
        StorageError::InvalidSchema(message) => StorageError::CorruptRecord(message),
        other => other,
    })?;
    let version = decoder.read_u32().map_err(corrupt)?;
    if version == 0 {
        return Err(StorageError::CorruptRecord(
            "record schema version must be greater than zero".to_owned(),
        ));
    }
    let value_length = decoder
        .read_length(MAX_STORAGE_RECORD_BYTES)
        .map_err(corrupt)?;
    let value = decoder.read_raw(value_length).map_err(corrupt)?.to_vec();
    if decoder.remaining() != 0 {
        return Err(StorageError::CorruptRecord(
            "storage record envelope has trailing bytes".to_owned(),
        ));
    }
    let schema = Schema { id, version };
    ensure_value_schema(&value, &schema)?;
    Ok(RecordEnvelope { schema, value })
}

fn decode_record<T: StableRecord + StableDecode>(envelope: &RecordEnvelope) -> StorageResult<T> {
    let expected = target_schema::<T>();
    validate_schema(&expected)?;
    if envelope.schema != expected {
        return Err(incompatible(&envelope.schema, &expected));
    }
    crate::from_bytes(&envelope.value).map_err(corrupt)
}

fn validate_schema(schema: &Schema) -> StorageResult<()> {
    validate_schema_id(&schema.id)?;
    if schema.version == 0 {
        return Err(StorageError::InvalidSchema(
            "record schema version must be greater than zero".to_owned(),
        ));
    }
    Ok(())
}

fn ensure_value_schema(value: &[u8], schema: &Schema) -> StorageResult<()> {
    if value.len() < core::mem::size_of::<u32>() {
        return Err(StorageError::CorruptRecord(
            "stable record is missing its schema version".to_owned(),
        ));
    }
    let encoded_version = u32::from_le_bytes(value[..4].try_into().expect("four bytes checked"));
    if encoded_version != schema.version {
        return Err(StorageError::CorruptRecord(
            "record envelope version disagrees with StableEncode bytes".to_owned(),
        ));
    }
    Ok(())
}

fn incompatible(stored: &Schema, expected: &Schema) -> StorageError {
    StorageError::IncompatibleSchema {
        stored_id: stored.id.clone(),
        stored_version: stored.version,
        expected_id: expected.id.clone(),
        expected_version: expected.version,
    }
}

fn no_migration(stored: &Schema, expected: &Schema) -> StorageError {
    if stored.id == expected.id && stored.version != expected.version {
        StorageError::MigrationPathMissing
    } else {
        incompatible(stored, expected)
    }
}

fn validate_schema_id(id: &str) -> StorageResult<()> {
    if id.is_empty()
        || id.len() > MAX_SCHEMA_ID_BYTES
        || id.chars().all(char::is_whitespace)
        || id.chars().any(char::is_control)
    {
        return Err(StorageError::InvalidSchema(
            "schema identity must contain 1..=256 non-blank, non-control UTF-8 bytes".to_owned(),
        ));
    }
    Ok(())
}

fn validate_key(key: &str) -> StorageResult<()> {
    if key.is_empty() || key.len() > MAX_STORAGE_KEY_BYTES || key.chars().any(char::is_control) {
        return Err(StorageError::InvalidKey(
            "key must contain 1..=256 non-control UTF-8 bytes".to_owned(),
        ));
    }
    Ok(())
}

fn validate_scope(scope: &StorageScope) -> StorageResult<()> {
    match scope {
        StorageScope::ClientProfile
        | StorageScope::Save
        | StorageScope::Player(_)
        | StorageScope::Entity(_) => Ok(()),
        StorageScope::Dimension(id)
        | StorageScope::BlockEntity {
            dimension_id: id, ..
        } => validate_dimension_id(id),
    }
}

fn validate_dimension_id(id: &str) -> StorageResult<()> {
    let valid_namespace =
        |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte);
    let Some((namespace, path)) = id.split_once(':') else {
        return Err(StorageError::InvalidScope(
            "dimension ID must use canonical namespace:path form".to_owned(),
        ));
    };
    if id.len() > 256
        || namespace.is_empty()
        || path.is_empty()
        || !namespace.bytes().all(valid_namespace)
        || !path
            .bytes()
            .all(|byte| valid_namespace(byte) || byte == b'/')
    {
        return Err(StorageError::InvalidScope(
            "dimension ID must use canonical namespace:path form".to_owned(),
        ));
    }
    Ok(())
}

fn scope_bytes(scope: &StorageScope) -> StorageResult<Vec<u8>> {
    validate_scope(scope)?;
    crate::to_bytes(scope).map_err(StorageError::Host)
}

fn corrupt(error: SjmlError) -> StorageError {
    StorageError::CorruptRecord(error.to_string())
}

#[cfg(all(not(target_arch = "wasm32"), test))]
thread_local! {
    static TEST_RECORDS: TestRecordStore = RefCell::new(HashMap::new());
    static TEST_BATCH_WRITE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TEST_BATCH_WRITE_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(all(not(target_arch = "wasm32"), test))]
type TestRecordStore = RefCell<HashMap<(Vec<u8>, String), Vec<u8>>>;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_storage_v1")]
extern "C" {
    #[link_name = "read"]
    fn imported_read(
        scope: i32,
        scope_length: i32,
        key: i32,
        key_length: i32,
        output: i32,
        capacity: i32,
    ) -> i64;
    #[link_name = "write"]
    fn imported_write(
        scope: i32,
        scope_length: i32,
        key: i32,
        key_length: i32,
        value: i32,
        value_length: i32,
    ) -> i32;
    #[link_name = "write_batch"]
    fn imported_write_batch(
        scope: i32,
        scope_length: i32,
        entries: i32,
        entries_length: i32,
    ) -> i32;
    #[link_name = "remove"]
    fn imported_remove(scope: i32, scope_length: i32, key: i32, key_length: i32) -> i64;
    #[link_name = "keys"]
    fn imported_keys(scope: i32, scope_length: i32, output: i32, capacity: i32) -> i64;
}

fn host_read(scope: &StorageScope, key: &str) -> StorageResult<Option<Vec<u8>>> {
    #[cfg(target_arch = "wasm32")]
    {
        let scope = scope_bytes(scope)?;
        read_raw_output("storage record read", |output, capacity| {
            // SAFETY: stable scope/key buffers and output allocation remain live for this call.
            unsafe {
                imported_read(
                    scope.as_ptr() as usize as u32 as i32,
                    scope.len() as i32,
                    key.as_ptr() as usize as u32 as i32,
                    key.len() as i32,
                    output,
                    capacity,
                )
            }
        })
    }
    #[cfg(all(not(target_arch = "wasm32"), test))]
    {
        let scope = scope_bytes(scope)?;
        Ok(TEST_RECORDS.with(|records| records.borrow().get(&(scope, key.to_owned())).cloned()))
    }
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        let _ = (scope, key);
        Err(unsupported_target())
    }
}

fn host_write(scope: &StorageScope, key: &str, value: &[u8]) -> StorageResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let scope = scope_bytes(scope)?;
        if value.len() > MAX_STORAGE_RECORD_BYTES {
            return Err(StorageError::Host(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "storage record exceeds the 1 MiB limit",
            )));
        }
        // SAFETY: stable scope, key, and value buffers remain live through the synchronous call.
        let status = unsafe {
            imported_write(
                scope.as_ptr() as usize as u32 as i32,
                scope.len() as i32,
                key.as_ptr() as usize as u32 as i32,
                key.len() as i32,
                value.as_ptr() as usize as u32 as i32,
                value.len() as i32,
            )
        } as u32;
        if status == 0 {
            Ok(())
        } else {
            Err(StorageError::host_status(
                status,
                "host rejected the storage write",
            ))
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), test))]
    {
        let scope = scope_bytes(scope)?;
        if value.len() > MAX_STORAGE_RECORD_BYTES {
            return Err(StorageError::Host(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "storage record exceeds the 1 MiB limit",
            )));
        }
        TEST_RECORDS.with(|records| {
            records
                .borrow_mut()
                .insert((scope, key.to_owned()), value.to_vec());
        });
        Ok(())
    }
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        let _ = (scope, key, value);
        Err(unsupported_target())
    }
}

fn host_write_batch(scope: &StorageScope, entries: &[(String, Vec<u8>)]) -> StorageResult<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let scope_bytes = scope_bytes(scope)?;
    let encoded_entries = encode_batch_entries(entries)?;

    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the scope and encoded entry buffers remain live through this synchronous call.
        let status = unsafe {
            imported_write_batch(
                scope_bytes.as_ptr() as usize as u32 as i32,
                scope_bytes.len() as i32,
                encoded_entries.as_ptr() as usize as u32 as i32,
                encoded_entries.len() as i32,
            )
        } as u32;
        if status == 0 {
            Ok(())
        } else {
            Err(StorageError::host_status(
                status,
                "host rejected the atomic storage batch write",
            ))
        }
    }
    #[cfg(all(not(target_arch = "wasm32"), test))]
    {
        let _ = encoded_entries;
        TEST_BATCH_WRITE_CALLS.with(|calls| calls.set(calls.get() + 1));
        if TEST_BATCH_WRITE_FAILURE.with(std::cell::Cell::get) {
            return Err(StorageError::Host(SjmlError::new(
                ErrorCode::ServiceBusy,
                "injected atomic storage batch failure",
            )));
        }

        TEST_RECORDS.with(|records| {
            let mut staged = records.borrow().clone();
            for (key, value) in entries {
                if value.len() > MAX_STORAGE_RECORD_BYTES {
                    return Err(StorageError::Host(SjmlError::new(
                        ErrorCode::PayloadTooLarge,
                        "storage record exceeds the 1 MiB limit",
                    )));
                }
                staged.insert((scope_bytes.clone(), key.clone()), value.clone());
            }
            *records.borrow_mut() = staged;
            Ok(())
        })
    }
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        let _ = (scope_bytes, encoded_entries);
        Err(unsupported_target())
    }
}

fn encode_batch_entries(entries: &[(String, Vec<u8>)]) -> StorageResult<Vec<u8>> {
    if entries.len() > MAX_STORAGE_KEYS {
        return Err(StorageError::Host(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "storage migration batch exceeds its entry limit",
        )));
    }
    let mut seen = HashSet::with_capacity(entries.len());
    let mut encoder = Encoder::new();
    encoder
        .write_length(entries.len())
        .map_err(StorageError::Host)?;
    for (key, value) in entries {
        validate_key(key)?;
        if !seen.insert(key.as_str()) {
            return Err(StorageError::CorruptRecord(
                "storage migration batch contains a duplicate key".to_owned(),
            ));
        }
        if value.len() > MAX_STORAGE_RECORD_BYTES {
            return Err(StorageError::Host(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "storage record exceeds the 1 MiB limit",
            )));
        }
        let _ = decode_envelope(value)?;
        let key_bytes = key.as_bytes();
        encoder
            .write_length(key_bytes.len())
            .and_then(|()| encoder.write_raw(key_bytes))
            .and_then(|()| encoder.write_length(value.len()))
            .and_then(|()| encoder.write_raw(value))
            .map_err(StorageError::Host)?;
    }
    if encoder.as_slice().len() > MAX_STORAGE_BATCH_BYTES {
        return Err(StorageError::Host(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "atomic storage migration batch exceeds the 1 MiB ABI limit",
        )));
    }
    Ok(encoder.into_bytes())
}

fn host_remove(scope: &StorageScope, key: &str) -> StorageResult<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let scope = scope_bytes(scope)?;
        // SAFETY: stable scope and key buffers remain live through this synchronous call.
        let result = unsafe {
            imported_remove(
                scope.as_ptr() as usize as u32 as i32,
                scope.len() as i32,
                key.as_ptr() as usize as u32 as i32,
                key.len() as i32,
            )
        } as u64;
        decode_removed(result as i64)
    }
    #[cfg(all(not(target_arch = "wasm32"), test))]
    {
        let scope = scope_bytes(scope)?;
        Ok(TEST_RECORDS.with(|records| {
            records
                .borrow_mut()
                .remove(&(scope, key.to_owned()))
                .is_some()
        }))
    }
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        let _ = (scope, key);
        Err(unsupported_target())
    }
}

fn host_keys(scope: &StorageScope) -> StorageResult<Vec<String>> {
    #[cfg(target_arch = "wasm32")]
    {
        let scope = scope_bytes(scope)?;
        let bytes = read_raw_output("storage key enumeration", |output, capacity| {
            // SAFETY: stable scope buffer and output allocation remain live for this call.
            unsafe {
                imported_keys(
                    scope.as_ptr() as usize as u32 as i32,
                    scope.len() as i32,
                    output,
                    capacity,
                )
            }
        })?
        .ok_or_else(|| {
            StorageError::CorruptRecord("host returned an empty key-list encoding".to_owned())
        })?;
        decode_keys(&bytes)
    }
    #[cfg(all(not(target_arch = "wasm32"), test))]
    {
        let scope = scope_bytes(scope)?;
        let mut keys = TEST_RECORDS.with(|records| {
            records
                .borrow()
                .keys()
                .filter(|&(record_scope, _)| record_scope == &scope)
                .map(|(_, key)| key.clone())
                .collect::<Vec<_>>()
        });
        keys.sort();
        Ok(keys)
    }
    #[cfg(all(not(target_arch = "wasm32"), not(test)))]
    {
        let _ = scope;
        Err(unsupported_target())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_keys(bytes: &[u8]) -> StorageResult<Vec<String>> {
    let mut keys: Vec<String> = crate::from_bytes(bytes).map_err(corrupt)?;
    if keys.len() > MAX_STORAGE_KEYS {
        return Err(StorageError::CorruptRecord(
            "host returned more storage keys than the configured limit".to_owned(),
        ));
    }
    for key in &keys {
        validate_key(key)?;
    }
    keys.sort();
    if keys.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(StorageError::CorruptRecord(
            "host returned a duplicate storage key".to_owned(),
        ));
    }
    Ok(keys)
}

#[cfg(target_arch = "wasm32")]
fn read_raw_output(
    operation: &'static str,
    invoke: impl FnOnce(i32, i32) -> i64,
) -> StorageResult<Option<Vec<u8>>> {
    use crate::abi::{allocate_host_input, deallocate_host_input};

    let capacity = MAX_STORAGE_RECORD_BYTES as u32;
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(StorageError::Host(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate storage output buffer",
        )));
    }
    // SAFETY: initialize the exact live allocation before the host writes its bounded output.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let result = invoke(pointer as i32, capacity as i32);
    let decoded = match decode_output_length(result, operation, capacity) {
        Ok(Some(length)) => {
            // SAFETY: the complete output span is within the live allocation and initialized above.
            let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, length) };
            Ok(Some(bytes.to_vec()))
        }
        Ok(None) => Ok(None),
        Err(error) => Err(error),
    };
    // SAFETY: free exactly the allocation created above, after decoding has finished.
    unsafe { deallocate_host_input(pointer, capacity) };
    decoded
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_output_length(
    result: i64,
    operation: &'static str,
    capacity: u32,
) -> StorageResult<Option<usize>> {
    let result = result as u64;
    let status = result as u32;
    let length = (result >> 32) as u32;
    if status != 0 {
        let message = match operation {
            "storage key enumeration" => "host rejected storage key enumeration",
            _ => "host rejected the storage read",
        };
        return Err(StorageError::host_status(status, message));
    }
    if length > capacity {
        return Err(StorageError::CorruptRecord(
            "host storage output length exceeds the provided capacity".to_owned(),
        ));
    }
    if length == 0 {
        Ok(None)
    } else {
        Ok(Some(length as usize))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_removed(result: i64) -> StorageResult<bool> {
    let result = result as u64;
    let status = result as u32;
    if status != 0 {
        return Err(StorageError::host_status(
            status,
            "host rejected the storage removal",
        ));
    }
    match (result >> 32) as u32 {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(StorageError::CorruptRecord(
            "host returned an invalid storage removal result".to_owned(),
        )),
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
fn unsupported_target() -> StorageError {
    StorageError::Host(SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "storage operations require wasm32-unknown-unknown and host storage imports",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StableDecode, StableEncode};

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.counter")]
    struct CounterV1 {
        value: u32,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 2, id = "tests.counter")]
    struct CounterV2 {
        value: u64,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 3, id = "tests.counter")]
    struct CounterV3 {
        value: u64,
        label: String,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.other")]
    struct OtherV1 {
        flag: bool,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.branch_a")]
    struct BranchA {
        value: u32,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.branch_b")]
    struct BranchB {
        value: u32,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.branch_c")]
    struct BranchC {
        value: u32,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.cycle_a")]
    struct CycleA {
        value: u32,
    }

    #[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
    #[sjml(version = 1, id = "tests.cycle_b")]
    struct CycleB {
        value: u32,
    }

    fn reset() {
        TEST_RECORDS.with(|records| records.borrow_mut().clear());
        TEST_BATCH_WRITE_CALLS.with(|calls| calls.set(0));
        TEST_BATCH_WRITE_FAILURE.with(|should_fail| should_fail.set(false));
        MIGRATIONS.with(|registry| *registry.borrow_mut() = MigrationRegistry::default());
        MIGRATION_MODES.with(|modes| modes.borrow_mut().clear());
    }

    fn raw(scope: &StorageScope, key: &str) -> Vec<u8> {
        host_read(scope, key).unwrap().unwrap()
    }

    #[test]
    fn typed_envelope_checks_schema_metadata_against_stable_bytes() {
        reset();
        let envelope = encode_record(&CounterV1 { value: 21 }).unwrap();
        let decoded = decode_envelope(&envelope).unwrap();
        assert_eq!(decoded.schema.id, "tests.counter");
        assert_eq!(decoded.schema.version, 1);
        assert_eq!(decode_record::<CounterV1>(&decoded).unwrap().value, 21);

        let mut corrupted = envelope;
        let version_offset = 4 + 4 + "tests.counter".len();
        corrupted[version_offset..version_offset + 4].copy_from_slice(&2_u32.to_le_bytes());
        assert!(matches!(
            decode_envelope(&corrupted),
            Err(StorageError::CorruptRecord(_))
        ));
    }

    #[test]
    fn remove_reports_presence_and_makes_the_key_missing() {
        reset();
        let scope = StorageScope::Save;
        assert!(!remove(&scope, "count").unwrap());
        write(&scope, "count", &CounterV1 { value: 3 }).unwrap();
        assert!(remove(&scope, "count").unwrap());
        assert_eq!(read::<CounterV1>(&scope, "count").unwrap(), None);
        assert!(!remove(&scope, "count").unwrap());
    }

    #[test]
    fn automatic_migration_uses_a_unique_chain_and_persists_only_final_value() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "count", &CounterV1 { value: 42 }).unwrap();
        register_migration::<CounterV1, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();
        register_migration::<CounterV2, CounterV3, _>(|old| {
            Ok(CounterV3 {
                value: old.value,
                label: "migrated".to_owned(),
            })
        })
        .unwrap();

        assert_eq!(
            read::<CounterV3>(&scope, "count").unwrap(),
            Some(CounterV3 {
                value: 42,
                label: "migrated".to_owned(),
            })
        );
        assert_eq!(
            decode_envelope(&raw(&scope, "count"))
                .unwrap()
                .schema
                .version,
            3
        );
    }

    #[test]
    fn manual_migration_is_skipped_by_read_and_runs_when_explicitly_requested() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "count", &CounterV1 { value: 9 }).unwrap();
        register_migration_with_mode::<CounterV1, CounterV2, _>(MigrationMode::Manual, |old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();

        assert_eq!(
            read::<CounterV2>(&scope, "count").unwrap_err(),
            StorageError::MigrationRequired
        );
        assert_eq!(
            decode_envelope(&raw(&scope, "count"))
                .unwrap()
                .schema
                .version,
            1
        );
        assert!(migrate_record::<CounterV2>(&scope, "count").unwrap());
        assert_eq!(
            read::<CounterV2>(&scope, "count").unwrap(),
            Some(CounterV2 { value: 9 })
        );
        assert!(!migrate_record::<CounterV2>(&scope, "count").unwrap());
    }

    #[test]
    fn a_key_can_override_and_clear_the_registered_migration_mode() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "count", &CounterV1 { value: 7 }).unwrap();
        register_migration::<CounterV1, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();
        set_migration_mode(&scope, "count", MigrationMode::Manual).unwrap();
        assert_eq!(
            read::<CounterV2>(&scope, "count").unwrap_err(),
            StorageError::MigrationRequired
        );
        assert!(clear_migration_mode(&scope, "count").unwrap());
        assert_eq!(
            read::<CounterV2>(&scope, "count").unwrap(),
            Some(CounterV2 { value: 7 })
        );
        assert!(!clear_migration_mode(&scope, "count").unwrap());
    }

    #[test]
    fn registration_rejects_cycles_and_ambiguous_paths() {
        reset();
        register_migration::<CycleA, CycleB, _>(|old| Ok(CycleB { value: old.value })).unwrap();
        assert_eq!(
            register_migration::<CycleB, CycleA, _>(|old| Ok(CycleA { value: old.value }))
                .unwrap_err(),
            StorageError::MigrationCycle
        );

        reset();
        register_migration::<CounterV1, BranchA, _>(|old| Ok(BranchA { value: old.value }))
            .unwrap();
        register_migration::<CounterV1, BranchB, _>(|old| Ok(BranchB { value: old.value }))
            .unwrap();
        register_migration::<BranchA, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();
        assert_eq!(
            register_migration::<BranchB, CounterV2, _>(|old| {
                Ok(CounterV2 {
                    value: u64::from(old.value),
                })
            })
            .unwrap_err(),
            StorageError::AmbiguousMigrationPath
        );
        assert!(register_migration::<BranchB, BranchC, _>(|old| {
            Ok(BranchC { value: old.value })
        })
        .is_ok());
    }

    #[test]
    fn failed_migration_preserves_the_original_record() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "count", &CounterV1 { value: 88 }).unwrap();
        let original = raw(&scope, "count");
        register_migration::<CounterV1, CounterV2, _>(|_| {
            Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "rejected by migration",
            ))
        })
        .unwrap();

        assert!(matches!(
            read::<CounterV2>(&scope, "count"),
            Err(StorageError::MigrationFailed(_))
        ));
        assert_eq!(raw(&scope, "count"), original);
        assert_eq!(
            read::<CounterV1>(&scope, "count").unwrap(),
            Some(CounterV1 { value: 88 })
        );
    }

    #[test]
    fn explicit_scope_migration_leaves_unrelated_schema_records_untouched() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "old", &CounterV1 { value: 5 }).unwrap();
        write(&scope, "current", &CounterV2 { value: 6 }).unwrap();
        write(&scope, "other", &OtherV1 { flag: true }).unwrap();
        register_migration::<CounterV1, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();

        let report = migrate_scope::<CounterV2>(&scope).unwrap();
        assert_eq!(report.migrated, 1);
        assert_eq!(report.unchanged, 2);
        assert!(report.failures.is_empty());
        assert!(report.committed);
        assert_eq!(
            decode_envelope(&raw(&scope, "old")).unwrap().schema.version,
            2
        );
        assert_eq!(
            decode_envelope(&raw(&scope, "other")).unwrap().schema.id,
            "tests.other"
        );
        assert_eq!(TEST_BATCH_WRITE_CALLS.with(std::cell::Cell::get), 1);
    }

    #[test]
    fn converter_failure_on_later_key_discards_every_staged_record() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "a_good", &CounterV1 { value: 5 }).unwrap();
        write(&scope, "z_bad", &CounterV1 { value: 8 }).unwrap();
        let good_before = raw(&scope, "a_good");
        let bad_before = raw(&scope, "z_bad");
        register_migration::<CounterV1, CounterV2, _>(|old| {
            if old.value == 8 {
                Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "test converter rejects this key",
                ))
            } else {
                Ok(CounterV2 {
                    value: u64::from(old.value),
                })
            }
        })
        .unwrap();

        let report = migrate_scope::<CounterV2>(&scope).unwrap();
        assert_eq!(report.migrated, 0);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].key, "z_bad");
        assert!(!report.committed);
        assert_eq!(raw(&scope, "a_good"), good_before);
        assert_eq!(raw(&scope, "z_bad"), bad_before);
        assert_eq!(TEST_BATCH_WRITE_CALLS.with(std::cell::Cell::get), 0);
    }

    #[test]
    fn corrupt_later_key_discards_every_staged_record() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "a_good", &CounterV1 { value: 5 }).unwrap();
        let good_before = raw(&scope, "a_good");
        TEST_RECORDS.with(|records| {
            records.borrow_mut().insert(
                (scope_bytes(&scope).unwrap(), "z_broken".to_owned()),
                vec![1, 2, 3],
            );
        });
        register_migration::<CounterV1, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();

        let report = migrate_scope::<CounterV2>(&scope).unwrap();
        assert_eq!(report.migrated, 0);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].key, "z_broken");
        assert!(!report.committed);
        assert_eq!(raw(&scope, "a_good"), good_before);
        assert_eq!(raw(&scope, "z_broken"), [1, 2, 3]);
        assert_eq!(TEST_BATCH_WRITE_CALLS.with(std::cell::Cell::get), 0);
    }

    #[test]
    fn scope_migration_stages_all_records_then_commits_with_one_batch_call() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "first", &CounterV1 { value: 5 }).unwrap();
        write(&scope, "second", &CounterV1 { value: 8 }).unwrap();
        register_migration::<CounterV1, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();

        let report = migrate_scope::<CounterV2>(&scope).unwrap();
        assert_eq!(report.migrated, 2);
        assert_eq!(report.unchanged, 0);
        assert!(report.failures.is_empty());
        assert!(report.committed);
        assert_eq!(TEST_BATCH_WRITE_CALLS.with(std::cell::Cell::get), 1);
        assert_eq!(
            read::<CounterV2>(&scope, "first").unwrap(),
            Some(CounterV2 { value: 5 })
        );
        assert_eq!(
            read::<CounterV2>(&scope, "second").unwrap(),
            Some(CounterV2 { value: 8 })
        );
    }

    #[test]
    fn host_batch_failure_preserves_every_original_record() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "first", &CounterV1 { value: 5 }).unwrap();
        write(&scope, "second", &CounterV1 { value: 8 }).unwrap();
        let first = raw(&scope, "first");
        let second = raw(&scope, "second");
        register_migration::<CounterV1, CounterV2, _>(|old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();
        TEST_BATCH_WRITE_FAILURE.with(|should_fail| should_fail.set(true));

        assert!(matches!(
            migrate_scope::<CounterV2>(&scope),
            Err(StorageError::Host(error)) if error.code() == ErrorCode::ServiceBusy
        ));
        assert_eq!(raw(&scope, "first"), first);
        assert_eq!(raw(&scope, "second"), second);
        assert_eq!(TEST_BATCH_WRITE_CALLS.with(std::cell::Cell::get), 1);
    }

    #[test]
    fn explicit_scope_migration_runs_manual_edges_without_ordinary_read_side_effects() {
        reset();
        let scope = StorageScope::Save;
        write(&scope, "count", &CounterV1 { value: 12 }).unwrap();
        register_migration_with_mode::<CounterV1, CounterV2, _>(MigrationMode::Manual, |old| {
            Ok(CounterV2 {
                value: u64::from(old.value),
            })
        })
        .unwrap();

        assert_eq!(
            read::<CounterV2>(&scope, "count").unwrap_err(),
            StorageError::MigrationRequired
        );
        assert_eq!(
            decode_envelope(&raw(&scope, "count"))
                .unwrap()
                .schema
                .version,
            1
        );
        let report = migrate_scope::<CounterV2>(&scope).unwrap();
        assert_eq!(report.migrated, 1);
        assert!(report.committed);
        assert_eq!(
            read::<CounterV2>(&scope, "count").unwrap(),
            Some(CounterV2 { value: 12 })
        );
    }

    #[test]
    fn missing_keys_are_distinct_from_corruption_and_schema_mismatch() {
        reset();
        let scope = StorageScope::Save;
        assert_eq!(read::<CounterV1>(&scope, "missing").unwrap(), None);
        write(&scope, "other", &OtherV1 { flag: false }).unwrap();
        assert!(matches!(
            read::<CounterV1>(&scope, "other"),
            Err(StorageError::IncompatibleSchema { .. })
        ));

        TEST_RECORDS.with(|records| {
            records.borrow_mut().insert(
                (scope_bytes(&scope).unwrap(), "broken".to_owned()),
                vec![1, 2, 3],
            );
        });
        assert!(matches!(
            read::<CounterV1>(&scope, "broken"),
            Err(StorageError::CorruptRecord(_))
        ));

        write(&scope, "needs_migration", &CounterV1 { value: 1 }).unwrap();
        assert_eq!(
            read::<CounterV2>(&scope, "needs_migration").unwrap_err(),
            StorageError::MigrationPathMissing
        );
    }

    #[test]
    fn key_lists_are_validated_sorted_and_deduplicated() {
        let encoded = crate::to_bytes(&vec!["z".to_owned(), "a".to_owned()]).unwrap();
        assert_eq!(decode_keys(&encoded).unwrap(), vec!["a", "z"]);
        let duplicate = crate::to_bytes(&vec!["a".to_owned(), "a".to_owned()]).unwrap();
        assert!(matches!(
            decode_keys(&duplicate),
            Err(StorageError::CorruptRecord(_))
        ));
    }

    #[test]
    fn atomic_batch_encoding_is_bounded_and_rejects_duplicate_keys() {
        let value = encode_record(&CounterV1 { value: 7 }).unwrap();
        let entries = vec![("count".to_owned(), value.clone())];
        let encoded = encode_batch_entries(&entries).unwrap();
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 5);
        assert_eq!(&encoded[8..13], b"count");
        assert_eq!(
            u32::from_le_bytes(encoded[13..17].try_into().unwrap()) as usize,
            value.len()
        );
        assert_eq!(&encoded[17..], value.as_slice());

        assert_eq!(
            encode_batch_entries(&[
                ("count".to_owned(), value.clone()),
                ("count".to_owned(), value),
            ])
            .unwrap_err(),
            StorageError::CorruptRecord(
                "storage migration batch contains a duplicate key".to_owned()
            )
        );
    }

    #[test]
    fn packed_host_results_decode_status_length_and_removed_flag() {
        assert_eq!(
            decode_output_length((19_u64 << 32) as i64, "storage record read", 64).unwrap(),
            Some(19)
        );
        assert_eq!(
            decode_output_length(0, "storage record read", 64).unwrap(),
            None
        );
        assert!(matches!(
            decode_output_length((65_u64 << 32) as i64, "storage record read", 64),
            Err(StorageError::CorruptRecord(_))
        ));
        let denied = ErrorCode::PermissionDenied.abi_code();
        assert!(matches!(
            decode_output_length(i64::from(denied), "storage record read", 64),
            Err(StorageError::Host(error)) if error.code() == ErrorCode::PermissionDenied
        ));
        assert!(!decode_removed(0).unwrap());
        assert!(decode_removed((1_u64 << 32) as i64).unwrap());
        assert!(matches!(
            decode_removed((2_u64 << 32) as i64),
            Err(StorageError::CorruptRecord(_))
        ));
    }
}
