use sjml::chat_messages::{
    self, ChatClickAction, ChatMessageAction, ChatMessageSource, ChatSpan, ChatText,
};
use sjml::ErrorCode;
use std::{cell::Cell, rc::Rc};

#[test]
fn message_sources_and_actions_are_typed_and_bounded() {
    assert_eq!(chat_messages::MAX_CHAT_MESSAGE_BYTES, 4096);
    assert_eq!(chat_messages::MAX_CHAT_MESSAGE_SUBSCRIPTIONS, 128);
    let _: ChatMessageSource = ChatMessageSource::System;
    let _: ChatMessageAction = ChatMessageAction::Hide;
    let text = ChatText::plain("Visit ").unwrap();
    let link = ChatSpan::new("example")
        .unwrap()
        .color(0x44AAFF)
        .unwrap()
        .bold(true)
        .hover_text("Open the website")
        .unwrap()
        .click(ChatClickAction::open_url("https://example.com").unwrap());
    let mut text = text;
    text.push(link).unwrap();
    let _: ChatMessageAction = ChatMessageAction::Replace(text);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_subscriptions_fail_without_invoking_guest_callbacks() {
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    let result = chat_messages::subscribe(move |_| {
        observed.set(true);
        Ok(ChatMessageAction::Hide)
    });
    assert_eq!(result.unwrap_err().code(), ErrorCode::PermissionDenied);
    assert!(!called.get());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn outgoing_chat_rejects_invalid_text_before_requesting_the_client_host() {
    let overlong = "x".repeat(257);
    for message in ["", " \t", "one\ntwo", "/help", "  /help", overlong.as_str()] {
        assert_eq!(
            chat_messages::send(message).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        chat_messages::send("hello").unwrap_err().code(),
        ErrorCode::PermissionDenied
    );
}
