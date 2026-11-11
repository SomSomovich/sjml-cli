//! Reads of loaded non-player entities and capability-gated effect application on the owning
//! logical server. This global API takes no world handle. Calls are synchronous on the server
//! thread. Handles are module/context and live-entity-instance scoped and expire when their entity
//! leaves its level, its world unloads, or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::{to_bytes, Decoder, MAX_STRING_BYTES};
use crate::{EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

const MAX_EFFECT_ID_BYTES: usize = 256;
pub(crate) const MAX_EFFECT_DURATION_TICKS: u32 = 72_000;

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

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_entity_v1")]
extern "C" {
    #[link_name = "loaded"]
    fn imported_loaded(output: i32, capacity: i32) -> i32;
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
        "entity reads require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::apply_effect;
    use crate::{EntityHandle, ErrorCode};

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
}
