# Rust Storage SDK

`sjml::storage` provides typed persistent records in either the active Minecraft save or the local
game profile. Save scopes are stored in `SavedData` below the current save. The explicit
`ClientProfile` scope is stored in a bounded SJML file under the local config directory and is
independent of any world or remote server. Each store is namespaced by the active mod ID, so
modules belonging to one mod share its records. The SDK does not expose raw bytes or JVM objects.

Every host operation checks the module's permissions. Reads and key enumeration require
`storage.read`; writes, removals, and successful migrations require `storage.write`. A migration
may read the old record and then write its replacement, so it needs both permissions. Permissions
are checked by the host on every imported call.

## Scopes

The SDK exposes these logical scopes:

| Rust value | Identity and lifetime |
| --- | --- |
| `StorageScope::ClientProfile` | Local client-profile settings for this mod, independent of the current world and server. Only physical-client modules may use it. |
| `StorageScope::Save` | One record namespace for this mod in the current save, shared by the mod's modules. |
| `StorageScope::dimension(id)` | One canonical `namespace:path` dimension ID. |
| `StorageScope::Player(uuid)` | One player's UUID; data survives logout. |
| `StorageScope::Entity(uuid)` | One entity UUID; survives chunk unloads and dimension transfers, and is removed when NeoForge reports permanent entity removal. |
| `StorageScope::block_entity(dimension, position)` | A block entity at one dimension and block position; survives chunk unloads and is deleted when a tracked removal or replacement is observed. A moved block uses a new position and does not carry the old scope. |

Dimension and block-entity constructors validate lowercase canonical resource IDs. Block positions
use signed 32-bit coordinates. The host does not load chunks as a side effect of storage access.
The client-profile scope is local installation data, not a client-side view of server or world-save
storage. Server contexts reject it, while client contexts reject save, dimension, player, entity,
and block-entity scopes. Client-profile records use the same per-mod record and total-storage limits
as save records. The host stores them under `config/sjml/client-storage/<mod-id>.dat`, loads them
lazily on the client main thread, and atomically replaces the bounded NBT file after a change. File
operations are synchronous on that thread; the host retains no open file handle or mutable cache
between calls.

```rust
use sjml::storage::{self, StorageScope};
use sjml::{StableDecode, StableEncode};

#[derive(StableEncode, StableDecode)]
#[sjml(version = 1, id = "example.counter")]
struct Counter {
    value: u64,
}

fn save_counter(value: u64) -> storage::StorageResult<()> {
    storage::write(&StorageScope::Save, "counter", &Counter { value })
}

fn load_counter() -> storage::StorageResult<Option<Counter>> {
    storage::read(&StorageScope::Save, "counter")
}
```

Keys must contain 1 to 256 non-control UTF-8 bytes. A complete encoded record envelope is limited
to 1 MiB. The host also applies per-mod record-count and total-storage quotas. A successful
individual write replaces one record atomically and either marks its owning `SavedData` dirty or
atomically replaces the client-profile file; a rejected operation does not partially replace the
previous value.

Entity cleanup is tied to permanent `EntityLeaveLevelEvent` removal; level and chunk unloads do not
delete UUID-scoped data. Block-entity cleanup is deferred until after mutation and currently tracks
player break/place events, explosions, pistons, mob block destruction, fluid placement, and
non-simulated tool modifications. It checks that the original block entity was actually removed or
replaced before deleting data, and it skips unloaded chunks without loading them. NeoForge has no
universal hook for arbitrary direct `Level#setBlock` calls: changes made by another mod without one
of the tracked events may leave the old position's block-entity data in the save.

`read` returns `Ok(None)` only when the key is absent. Corrupt data, an incompatible schema,
permission failures, and migration failures are errors. `remove` returns whether a key existed.

## Schema identity and encoding

Both stable derives must be present for persisted record types. `StableEncode` keeps its existing
wire representation: a little-endian `u32` schema version followed by the encoded named fields.
`StableDecode` additionally implements `storage::StableRecord`; this adds metadata but does not
change those stable bytes.

Use an explicit stable identity for records expected to live across releases:

```rust
#[derive(StableEncode, StableDecode)]
#[sjml(version = 2, id = "example.counter")]
struct CounterV2 {
    value: u64,
    label: String,
}
```

The identity must be 1 to 256 non-blank UTF-8 bytes and contain no control characters. If `id` is omitted,
the SDK uses `core::any::type_name::<T>()` as a convenience fallback. Rust type paths can change
when code is renamed or moved, so the fallback is unsuitable for long-lived save compatibility.
Keep an explicit `id` and increment `version` when the stable field encoding changes.

The storage envelope is independent of the record's stable payload. It is encoded as:

1. little-endian `u32` envelope version (`1`);
2. stable UTF-8 string `schema_id` (`u32` byte length followed by bytes);
3. little-endian `u32` record schema version;
4. little-endian `u32` stable payload byte length;
5. the existing `StableEncode` payload bytes.

The SDK rejects trailing envelope bytes, invalid IDs, zero schema versions, and a mismatch between
the envelope version and the first `u32` in the stable payload. The host validates the same payload
version before persistence.

## Migrations

Register typed edges during module initialization. Each migration receives a decoded old value and
returns a new value; callers never manipulate envelope bytes.

```rust
use sjml::storage::{self, MigrationMode};
use sjml::{StableDecode, StableEncode};

#[derive(StableEncode, StableDecode)]
#[sjml(version = 1, id = "example.counter")]
struct CounterV1 {
    value: u32,
}

#[derive(StableEncode, StableDecode)]
#[sjml(version = 2, id = "example.counter")]
struct CounterV2 {
    value: u64,
    label: String,
}

#[derive(StableEncode, StableDecode)]
#[sjml(version = 3, id = "example.counter")]
struct CounterV3 {
    value: u64,
    label: String,
    archived: bool,
}

fn register_counter_migrations() -> storage::StorageResult<()> {
    storage::register_migration::<CounterV1, CounterV2, _>(|old| {
        Ok(CounterV2 { value: u64::from(old.value), label: "default".into() })
    })?;
    storage::register_migration_with_mode::<CounterV2, CounterV3, _>(
        MigrationMode::Manual,
        |old| Ok(CounterV3 { value: old.value, label: old.label, archived: false }),
    )
}
```

`register_migration` defaults the edge to automatic. `register_migration_with_mode` sets the
default mode for that edge. A normal `read::<T>` automatically migrates only when there is exactly
one complete path from the stored schema to `T` and every edge is automatic. If the only path
contains manual edges, `read` returns `MigrationRequired` and leaves the old record untouched.
`set_migration_mode(scope, key, mode)` overrides the defaults for one key; use
`clear_migration_mode` to return to the registered defaults. Overrides live in that module
instance and must be installed again after restart.

Explicit migration ignores automatic/manual mode:

```rust
storage::migrate_record::<CounterV3>(&StorageScope::Save, "counter")?;
let report = storage::migrate_scope::<CounterV3>(&StorageScope::Save)?;
```

The scope operation stages the complete migration before writing and only targets records whose
stable schema ID matches `T`; records with another valid schema ID are left untouched and counted
as unchanged. Corrupt or unclassifiable records are failures, because the SDK cannot safely decide
whether they belong to the target schema. Its `MigrationReport` includes a key and typed error for
each failed record. If any converter, decode, validation, or host write fails, no records in that
scope are changed. `MigrationReport.committed` is true after a complete migration, including when
there was nothing to update; it is false when pre-commit failures abort the scope, and then
`migrated` is zero. A successful migration commits all changed records in one atomic host batch.
Scope migration batches are limited to 1 MiB of encoded entries, so a larger migration is rejected
before any writes. A host batch-write error is returned as an error, and the atomic host operation
leaves the scope unchanged. A record migration is atomic on its own as well.

The guest registry belongs to one module instance. Duplicate edges, cycles, and graph additions
that make any source-to-target path ambiguous are rejected when registered. Missing links yield
`MigrationPathMissing` for another version of the same identity, or `IncompatibleSchema` for a
different identity. Automatic reads do not run migrations registered as manual unless the specific
key is overridden to automatic. There is no global migration registry shared across modules.

## Errors

`StorageError` distinguishes:

- `Host(SjmlError)` for permissions, quotas, unsupported guest targets, and host-side failures;
- `InvalidKey`, `InvalidScope`, and `InvalidSchema` for invalid guest input or record metadata;
- `CorruptRecord` for malformed stored envelope/payload bytes or invalid host output;
- `IncompatibleSchema` when a different record identity is requested;
- `MigrationPathMissing`, `MigrationRequired`, and `AmbiguousMigrationPath` for version paths;
- `MigrationCycle`, `DuplicateMigration`, and `MigrationFailed` for registration/converter errors.

## Host ABI v1

All pointers are wasm32 guest-memory offsets. The guest buffers remain live for each synchronous
call. `scope` is the versioned stable scope encoding above; `key` is its UTF-8 bytes without a
length prefix. The host derives a canonical internal identity from the decoded scope:
`save`, `dimension:<id>`, `player:<uuid>`, `entity:<uuid>`,
`block_entity:<dimension>:<x>:<y>:<z>`, or `client_profile`.

The scope wire header is a little-endian `u32` schema version (`1`) followed by a `u8` kind tag:
`0` save; `1` dimension followed by a stable UTF-8 string; `2` player followed by the existing
16-byte `Uuid` representation; `3` entity followed by the same UUID bytes; `4` block entity
followed by a stable dimension string and the existing versioned `BlockPos` encoding; or `5`
client profile with no additional payload. The `BlockPos` encoding is unchanged from
`sjml::inventory`. The host routes tag `5` only from physical-client contexts and denies it in
server contexts; ordinary save-scope routing is unchanged.

Imports use module `sjml_storage_v1`:

| Import | Signature | Result |
| --- | --- | --- |
| `read` | `(scope_ptr, scope_len, key_ptr, key_len, out_ptr, capacity) -> i64` | Low `u32`: status. High `u32`: bytes written. Status 0 and length 0 means absent. |
| `write` | `(scope_ptr, scope_len, key_ptr, key_len, envelope_ptr, envelope_len) -> i32` | `0` on success; otherwise the stable ABI error code. |
| `write_batch` | `(scope_ptr, scope_len, entries_ptr, entries_len) -> i32` | `0` on success; otherwise the stable ABI error code. The complete batch is validated and applied atomically. |
| `remove` | `(scope_ptr, scope_len, key_ptr, key_len) -> i64` | Low `u32`: status. High `u32`: `0` absent or `1` removed. |
| `keys` | `(scope_ptr, scope_len, out_ptr, capacity) -> i64` | Low `u32`: status. High `u32`: encoded `Vec<String>` byte length. Empty scope is an encoded zero-length vector, not a zero-byte result. |

The host must verify the full output fits `capacity` before writing any bytes. It must return an
error without truncating when it does not fit. `read` output is the complete record envelope;
`keys` output uses the SDK's stable vector/string encoding. The complete envelope and host output
buffers are bounded at 1 MiB.

`write_batch` requires `storage.write`. Its `entries` buffer is a little-endian `u32` entry count,
followed by each entry as a `u32` UTF-8 key byte length and key bytes, then a `u32` record-envelope
byte length and the complete envelope. Keys must be valid, non-empty, and unique. The host bounds
the entire encoded batch at 1 MiB, validates all keys, envelopes, namespace limits, and quotas
before changing storage, then commits all records together. Save-backed batches mark `SavedData`
dirty once; client-profile batches use one atomic file replacement. Any rejected entry or quota
failure leaves every record in the scope unchanged. `write_batch` is used by the SDK for atomic
scope migrations; ordinary `write` remains a one-record operation.
