//! Persistent local fog overrides for physical-client modules.
//!
//! A module can set fog color, start/end distances, and fog shape independently. For each
//! setting, the active module with the lexicographically greatest runtime module ID wins.
//! Clearing or unloading that module reveals the next lower ID's value. Values are stored by
//! the host, so no guest callback runs during rendering.

use crate::{ErrorCode, SjmlError, SjmlResult};

/// Capability required to customize fog on the physical client.
pub const CLIENT_FOG_MODIFY_CAPABILITY: &str = "client.fog.modify";
/// Maximum accepted near or far fog distance, in blocks.
pub const MAX_FOG_DISTANCE: f32 = 1_024.0;

/// Minecraft fog geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum FogShape {
    /// Spherical fog, the default shape in most situations.
    Sphere = 0,
    /// Cylindrical fog, commonly used underwater or in lava.
    Cylinder = 1,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_fog_v1")]
extern "C" {
    #[link_name = "set_color"]
    fn imported_set_color(red: f32, green: f32, blue: f32) -> i32;
    #[link_name = "clear_color"]
    fn imported_clear_color() -> i32;
    #[link_name = "set_distances"]
    fn imported_set_distances(near: f32, far: f32) -> i32;
    #[link_name = "clear_distances"]
    fn imported_clear_distances() -> i32;
    #[link_name = "set_shape"]
    fn imported_set_shape(shape: i32) -> i32;
    #[link_name = "clear_shape"]
    fn imported_clear_shape() -> i32;
    #[link_name = "clear_all"]
    fn imported_clear_all() -> i32;
}

/// Sets this module's persistent fog color. Each component must be finite and in `0.0..=1.0`.
/// Other modules' color settings are resolved by stable runtime ID order on the host.
pub fn set_color(red: f32, green: f32, blue: f32) -> SjmlResult<()> {
    validate_color(red, green, blue)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: this import accepts three f32 values and returns an ABI status code.
        status(
            unsafe { imported_set_color(red, green, blue) },
            "host rejected the fog color",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (red, green, blue);
        Err(unsupported())
    }
}

/// Removes this module's fog color override.
pub fn clear_color() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: this import takes no arguments and returns an ABI status code.
        status(
            unsafe { imported_clear_color() },
            "host rejected clearing the fog color",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

/// Sets this module's persistent fog start and end distances in blocks.
///
/// `near` must be nonnegative, `far` must be greater than `near`, and `far` must not exceed
/// [`MAX_FOG_DISTANCE`]. Both values must be finite.
pub fn set_distances(near: f32, far: f32) -> SjmlResult<()> {
    validate_distances(near, far)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: this import accepts two f32 values and returns an ABI status code.
        status(
            unsafe { imported_set_distances(near, far) },
            "host rejected the fog distances",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (near, far);
        Err(unsupported())
    }
}

/// Removes this module's fog distance override.
pub fn clear_distances() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: this import takes no arguments and returns an ABI status code.
        status(
            unsafe { imported_clear_distances() },
            "host rejected clearing the fog distances",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

/// Sets this module's persistent fog shape.
pub fn set_shape(shape: FogShape) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the enum has stable repr(i32) values matching the host import contract.
        status(
            unsafe { imported_set_shape(shape as i32) },
            "host rejected the fog shape",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = shape;
        Err(unsupported())
    }
}

/// Removes this module's fog shape override.
pub fn clear_shape() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: this import takes no arguments and returns an ABI status code.
        status(
            unsafe { imported_clear_shape() },
            "host rejected clearing the fog shape",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

/// Clears this module's color, distance, and shape overrides together.
pub fn clear_all() -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: this import takes no arguments and returns an ABI status code.
        status(
            unsafe { imported_clear_all() },
            "host rejected clearing fog settings",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

fn validate_color(red: f32, green: f32, blue: f32) -> SjmlResult<()> {
    if [red, green, blue]
        .into_iter()
        .all(|component| component.is_finite() && (0.0..=1.0).contains(&component))
    {
        Ok(())
    } else {
        Err(invalid(
            "fog color components must be finite and within 0..=1",
        ))
    }
}

fn validate_distances(near: f32, far: f32) -> SjmlResult<()> {
    if near.is_finite() && far.is_finite() && near >= 0.0 && far > near && far <= MAX_FOG_DISTANCE {
        Ok(())
    } else {
        Err(invalid(
            "fog distances must be finite, near >= 0, near < far, and far <= 1024",
        ))
    }
}

#[cfg(target_arch = "wasm32")]
fn status(code: i32, message: &str) -> SjmlResult<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(code as u32, message))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "client fog controls require the wasm32-unknown-unknown target",
    )
}

fn invalid(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::{
        clear_all, clear_color, clear_distances, clear_shape, set_color, set_distances, set_shape,
        FogShape, MAX_FOG_DISTANCE,
    };
    use crate::ErrorCode;

    #[test]
    fn validates_color_bounds_before_host_access() {
        for component in [0.0, 0.5, 1.0] {
            assert_eq!(
                set_color(component, component, component)
                    .unwrap_err()
                    .code(),
                ErrorCode::UnknownErrorCode,
                "valid color proceeds to the unavailable native host import"
            );
        }
        for component in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.001, 1.001] {
            assert_eq!(
                set_color(component, 0.5, 0.5).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn validates_distance_bounds_before_host_access() {
        assert_eq!(
            set_distances(0.0, MAX_FOG_DISTANCE).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        for (near, far) in [
            (f32::NAN, 10.0),
            (0.0, f32::INFINITY),
            (-0.01, 10.0),
            (10.0, 10.0),
            (11.0, 10.0),
            (0.0, MAX_FOG_DISTANCE + 1.0),
        ] {
            assert_eq!(
                set_distances(near, far).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn shape_discriminants_are_stable_and_native_imports_fail_closed() {
        assert_eq!(FogShape::Sphere as i32, 0);
        assert_eq!(FogShape::Cylinder as i32, 1);
        for result in [
            set_shape(FogShape::Sphere),
            clear_color(),
            clear_distances(),
            clear_shape(),
            clear_all(),
        ] {
            assert_eq!(result.unwrap_err().code(), ErrorCode::UnknownErrorCode);
        }
    }
}
