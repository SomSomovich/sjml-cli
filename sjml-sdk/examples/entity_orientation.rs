#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

/// Reads, updates, then restores the detached orientation of a loaded non-player entity.
pub fn set_entity_orientation(handle: EntityHandle) -> SjmlResult<()> {
    let original = entity::orientation(handle)?;
    let desired = EntityOrientation::new(90.0, -20.0, 120.0)?;
    entity::set_orientation(handle, desired)?;
    entity::set_orientation(handle, original)
}
