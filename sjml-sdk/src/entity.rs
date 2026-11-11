//! Reads of loaded non-player entities and capability-gated actions on the owning logical server.
//! This global API takes no world handle. Calls are synchronous on the server
//! thread. Handles are module/context and live-entity-instance scoped and expire when their entity
//! leaves its level, its world unloads, or the module closes.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::{Decoder, MAX_STRING_BYTES, to_bytes};
use crate::{EntityHandle, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

const MAX_EFFECT_ID_BYTES: usize = 256;
pub(crate) const MAX_EFFECT_DURATION_TICKS: u32 = 72_000;
/// Maximum absolute entity velocity component in blocks per tick.
pub const MAX_ENTITY_VELOCITY_COMPONENT: f64 = 64.0;

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
    #[link_name = "discard"]
    fn imported_discard(handle: i32, handle_length: i32) -> i32;
    #[link_name = "velocity"]
    fn imported_velocity(handle: i32, handle_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_velocity"]
    fn imported_set_velocity(
        handle: i32,
        handle_length: i32,
        velocity: i32,
        velocity_length: i32,
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
    use super::{apply_effect, set_velocity, EntityVelocity, MAX_ENTITY_VELOCITY_COMPONENT};
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
}
