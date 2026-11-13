#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "notify.first_online")]
fn notify_first_online(_: NoArguments) -> SjmlResult<String> {
    let Some(player) = player::online()?.into_iter().next() else {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "there is no online player to notify",
        ));
    };
    player::send_system_message(player, "Welcome! This is a literal server message.")?;
    Ok("system message sent".to_owned())
}
