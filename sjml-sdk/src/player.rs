//! Scoped access to online player sessions on the owning logical server. Snapshots require requested
//! and granted `entity.read`; effect application also requires `entity.effect.apply`. Player handles
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
use crate::{ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode, StableEncode};

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

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_player_v1")]
extern "C" {
    #[link_name = "online"]
    fn imported_online(output: i32, capacity: i32) -> i32;
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
        "player reads require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::apply_effect;
    use crate::{ErrorCode, PlayerHandle};

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
}
