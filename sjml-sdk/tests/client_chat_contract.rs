use sjml::client_chat::{self, ChatLayout, ChatLayoutHandle, ChatScreenEvent};
use sjml::ErrorCode;
use std::{cell::Cell, rc::Rc};

#[test]
fn layouts_reject_non_finite_or_non_positive_extents() {
    let valid = ChatLayout::new(-8.0, 0.0, 120.0, 56.0).unwrap();
    assert_eq!(
        (valid.x(), valid.y(), valid.width(), valid.height()),
        (-8.0, 0.0, 120.0, 56.0)
    );
    for values in [
        (f32::NAN, 0.0, 120.0, 56.0),
        (0.0, f32::INFINITY, 120.0, 56.0),
        (0.0, 0.0, 0.0, 56.0),
        (0.0, 0.0, -1.0, 56.0),
        (0.0, 0.0, 120.0, f32::NEG_INFINITY),
    ] {
        assert_eq!(
            ChatLayout::new(values.0, values.1, values.2, values.3)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn prelude_supports_typed_client_chat_callbacks() {
    use sjml::prelude::{
        client_chat as chat, ChatLayout as Layout, ChatLayoutHandle as Handle,
        ChatScreenEvent as Event,
    };
    let _: Option<Handle> = None;
    let _: Option<ChatLayoutHandle> = None;
    let rectangle: Layout = ChatLayout::new(0.0, 0.0, 200.0, 100.0).unwrap();
    let event: Event = ChatScreenEvent::Opened {
        viewport_width: 854.0,
        viewport_height: 480.0,
        layout: rectangle,
    };
    assert!(matches!(event, Event::Opened { .. }));
    let _ = chat::layout as fn(Handle) -> sjml::SjmlResult<Layout>;
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_subscriptions_fail_without_invoking_callbacks() {
    let called = Rc::new(Cell::new(false));
    let observed = called.clone();
    let result = client_chat::subscribe(move |_| {
        observed.set(true);
        Ok(true)
    });
    assert_eq!(result.unwrap_err().code(), ErrorCode::PermissionDenied);
    assert!(!called.get());
}
