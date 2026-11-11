use sjml::render::{self, RenderFrame, RenderLayer, RenderSubscriptionHandle};
use sjml::{ErrorCode, EventSubscriptionHandle, RenderResourceHandle};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn prelude_exports_the_runtime_render_surface() {
    use sjml::prelude::{
        render as prelude_render, RenderLayer as PreludeLayer,
        RenderSubscriptionHandle as PreludeHandle, WorldRenderFrame as PreludeWorldFrame,
    };

    let _: PreludeLayer = PreludeLayer::Hud;
    let _: Option<PreludeHandle> = None;
    let _: Option<PreludeWorldFrame> = None;
    assert_eq!(prelude_render::RenderLayer::Hud.wire_name(), "hud");
    assert_eq!(prelude_render::RenderLayer::Hud.wire_value(), 0);
    assert_eq!(prelude_render::RenderLayer::World.wire_value(), 2);

    let frame = RenderFrame {
        width: 854,
        height: 480,
        mouse_x: -1.0,
        mouse_y: 17.5,
        partial_tick: 0.25,
    };
    assert_eq!(frame.width, 854);
    assert_eq!(frame.height, 480);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_render_subscription_reports_unsupported_imports() {
    let callback_called = Rc::new(Cell::new(false));
    let called = callback_called.clone();
    let result = render::subscribe(RenderLayer::Hud, move |context| {
        called.set(true);
        context.draw_text("hello", 2.0, 3.0, 0xffff_ffff, true)?;
        Ok(())
    });
    assert_eq!(result.unwrap_err().code(), ErrorCode::UnknownErrorCode);
    assert!(!callback_called.get());

    let handle = RenderSubscriptionHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        render::unsubscribe(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );

    let _: Option<RenderResourceHandle> = None;
    let _: Option<EventSubscriptionHandle> = None;
}
