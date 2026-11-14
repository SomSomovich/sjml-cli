#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _motion_listener = client_input::subscribe_cursor_motion(|movement| {
        // Each callback preserves one native cursor event in physical window pixels.
        let _position_and_delta = (movement.x, movement.y, movement.delta_x, movement.delta_y);
        Ok(())
    })?;
    Ok(())
}
