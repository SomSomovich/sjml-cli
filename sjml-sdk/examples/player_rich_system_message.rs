#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;

#[derive(StableDecode)]
#[sjml(version = 1)]
struct NoArguments {}

#[command(name = "notify.first_online_rich")]
fn notify_first_online_rich(_: NoArguments) -> SjmlResult<String> {
    let Some(player) = player::online()?.into_iter().next() else {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "there is no online player",
        ));
    };

    let hover = RichText::plain("This is a server supplied suggestion")?;
    let action = RichTextNode::new("Run /home")?
        .color(RichTextColor::Aqua)?
        .bold(true)
        .click(RichTextClickAction::suggest_command("/home")?)
        .hover_text(hover)?;
    let detail =
        RichTextNode::new(" to return to your home.")?.color(RichTextColor::rgb(0xC0D0E0)?)?;
    let mut message = RichText::new();
    message.push(RichTextNode::new("Welcome! ")?.color(RichTextColor::Gold)?)?;
    message.push(action.child(detail)?)?;
    player::send_rich_system_message(player, &message)?;
    Ok("rich system message sent".to_owned())
}
