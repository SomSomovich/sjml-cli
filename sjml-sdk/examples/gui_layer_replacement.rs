#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[mod_init]
fn initialize_gui_layer_replacement() -> SjmlResult<()> {
    let _chat_layer = render::suppress_gui_layer("minecraft:chat")?;
    let _replacement = render::subscribe(RenderLayer::Hud, |context| {
        context.draw_rect(8.0, 8.0, 220.0, 32.0, 0xDD10_1820)?;
        context.draw_text("Custom chat panel", 16.0, 17.0, 0xFFFF_FFFF, true)?;
        Ok(())
    })?;
    Ok(())
}
