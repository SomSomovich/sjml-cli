use sjml::chat_messages::{self, ChatMessageAction, ChatMessageSource};
use sjml::ErrorCode;
use std::{cell::Cell, rc::Rc};

#[test]
fn message_sources_and_actions_are_typed_and_bounded() {
    assert_eq!(chat_messages::MAX_CHAT_MESSAGE_BYTES, 4096);
    assert_eq!(chat_messages::MAX_CHAT_MESSAGE_SUBSCRIPTIONS, 128);
    let _: ChatMessageSource = ChatMessageSource::System;
    let _: ChatMessageAction = ChatMessageAction::Hide;
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
