#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _release_veto = screen_input::subscribe_mouse_released(|event| {
        // A custom slider can commit its final value here and consume the release so the
        // vanilla screen does not treat it as an unrelated click.
        let _release = (event.mouse_x, event.mouse_y, event.button, event.modifiers);
        Ok(false)
    })?;
    Ok(())
}
