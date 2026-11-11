use sjml::client_input::{subscribe_scroll, MouseScrollEvent};
use sjml::ErrorCode;

#[test]
fn mouse_scroll_event_exposes_fractional_horizontal_and_vertical_deltas() {
    let event = MouseScrollEvent::new(-0.25, 1.5).unwrap();

    assert_eq!(event.delta_x, -0.25);
    assert_eq!(event.delta_y, 1.5);
    assert!(MouseScrollEvent::new(f64::NAN, 0.0).is_err());
    assert!(MouseScrollEvent::new(0.0, f64::INFINITY).is_err());
    assert!(MouseScrollEvent::new(65.0, 0.0).is_err());
    assert!(MouseScrollEvent::new(0.0, 0.0).is_err());
}

#[test]
fn native_mouse_scroll_subscription_fails_closed() {
    let error = subscribe_scroll(|_| Ok(())).unwrap_err();

    assert_eq!(error.code(), ErrorCode::PermissionDenied);
}
