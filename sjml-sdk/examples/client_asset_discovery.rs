#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::client_assets;
use sjml::prelude::*;

#[client_command(name = "assets.list")]
fn list_package_assets(prefix: String) -> SjmlResult<String> {
    let result = client_assets::list(&prefix, 60)?;
    let mut message = result.paths.join(", ");
    if result.truncated {
        message.push_str(" (some paths omitted; query a narrower prefix)");
    }
    Ok(message)
}
