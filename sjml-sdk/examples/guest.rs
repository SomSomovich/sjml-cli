#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item(
        "ruby",
        ItemDefinition {
            max_stack_size: 16,
            fire_resistant: true,
        },
    )?;
    registry::register_block(
        "ruby_block",
        BlockDefinition {
            strength: 2.0,
            resistance: 6.0,
            requires_correct_tool: true,
            has_item_form: true,
        },
    )?;
    Ok(())
}

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _abi_major = host::abi_major_version()?;
    host::log(LogLevel::Info, "guest initialized")?;
    events::subscribe("server.started", EventPriority::Normal)?;
    events::subscribe("server.stopping", EventPriority::Normal)?;
    events::subscribe("player.joined", EventPriority::Normal)?;
    events::subscribe("player.left", EventPriority::Normal)?;
    Ok(())
}

#[mod_shutdown]
fn shutdown() -> SjmlResult<()> {
    Ok(())
}

#[event(name = "server.started")]
fn server_started(_: ()) -> SjmlResult<()> {
    host::log(LogLevel::Info, "server is ready")?;
    Ok(())
}

#[event(name = "server.stopping")]
fn server_stopping(_: ()) -> SjmlResult<()> {
    host::log(LogLevel::Info, "server is stopping")?;
    Ok(())
}

#[event(name = "player.joined")]
fn player_joined(message: PlayerJoined) -> SjmlResult<()> {
    host::log(LogLevel::Info, &message.display_name)?;
    Ok(())
}

#[event(name = "player.left")]
fn player_left(message: PlayerLeft) -> SjmlResult<()> {
    host::log(LogLevel::Info, &message.display_name)?;
    Ok(())
}

#[command(name = "demo.echo")]
fn echo(message: String) -> SjmlResult<String> {
    Ok(message)
}

#[service(name = "demo.online_count")]
fn online_count(_: ()) -> SjmlResult<u32> {
    Ok(0)
}
