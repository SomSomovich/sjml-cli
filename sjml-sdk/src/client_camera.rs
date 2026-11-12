//! Persistent client-local camera controls.
//!
//! Each module may set one FOV multiplier in `0.25..=4.0`; `1.0` leaves the
//! current Minecraft FOV unchanged. Active module multipliers are combined in
//! activation order and the aggregate is capped to `0.25..=4.0`. Values remain
//! active until cleared or the owning runtime closes. This API does not invoke
//! guest code while Minecraft computes FOV each frame.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Capability required by the host to change the local FOV multiplier.
pub const CLIENT_CAMERA_FOV_CAPABILITY: &str = "client.camera.fov";
/// Minimum per-module FOV multiplier.
pub const MIN_FOV_SCALE: f32 = 0.25;
/// Maximum per-module FOV multiplier.
pub const MAX_FOV_SCALE: f32 = 4.0;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_camera_v1")]
extern "C" {
    #[link_name = "set_fov_scale"]
    fn imported_set_fov_scale(scale: f32) -> i32;
    #[link_name = "clear_fov_scale"]
    fn imported_clear_fov_scale() -> i32;
}

/// Sets this module's persistent local FOV multiplier.
///
/// The value must be finite and within `0.25..=4.0`. `1.0` is neutral. The
/// host combines active module multipliers in activation order, caps the
/// aggregate to `0.25..=4.0`, and applies it to Minecraft's already computed
/// dynamic FOV modifier. Call [`clear_fov_scale`] to return this module to
/// neutral.
pub fn set_fov_scale(scale: f32) -> SjmlResult<()> {
    validate_scale(scale)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host import consumes one scalar and returns an ABI status code.
        let status = unsafe { imported_set_fov_scale(scale) };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client FOV scale",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = scale;
        Err(unsupported())
    }
}

/// Removes this module's FOV multiplier, restoring a neutral multiplier of `1.0`.
pub fn clear_fov_scale() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host import takes no arguments and returns an ABI status code.
        let status = unsafe { imported_clear_fov_scale() };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected clearing the client FOV scale",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

fn validate_scale(scale: f32) -> SjmlResult<()> {
    if !scale.is_finite() || !(MIN_FOV_SCALE..=MAX_FOV_SCALE).contains(&scale) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "client FOV scale must be finite and between 0.25 and 4.0",
        ));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "client camera controls require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{clear_fov_scale, set_fov_scale, MAX_FOV_SCALE, MIN_FOV_SCALE};
    use crate::ErrorCode;

    #[test]
    fn validates_scale_bounds_before_host_access() {
        for scale in [MIN_FOV_SCALE, 1.0, MAX_FOV_SCALE] {
            assert_eq!(
                set_fov_scale(scale).unwrap_err().code(),
                ErrorCode::UnknownErrorCode,
                "valid scale {scale} proceeds to the unavailable native host import"
            );
        }
        for scale in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.249, 4.001] {
            assert_eq!(
                set_fov_scale(scale).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "invalid scale {scale} is rejected before host access"
            );
        }
    }

    #[test]
    fn clear_fails_closed_without_a_wasm_host() {
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            clear_fov_scale().unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
