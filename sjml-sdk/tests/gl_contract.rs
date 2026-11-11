use sjml::gl::{self, GlCommand, GlPrimitiveMode, GlResourceDescriptor};
use sjml::{ErrorCode, GlResourceHandle, RenderResourceHandle};

#[test]
fn gl_sdk_keeps_hud_and_opengl_resource_handles_distinct() {
    let hud: RenderResourceHandle = RenderResourceHandle::new(1, 2, 3, 4).unwrap();
    let gl: GlResourceHandle = GlResourceHandle::new(1, 2, 3, 4).unwrap();
    assert_ne!(RenderResourceHandle::KIND_CODE, GlResourceHandle::KIND_CODE);

    let hud_result = gl::create_hud_texture(1, 1, &[255, 0, 0, 255]);
    assert_eq!(hud_result.unwrap_err().code(), ErrorCode::UnknownErrorCode);
    assert_eq!(
        gl::delete_hud_texture(hud).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );

    let resource = GlResourceDescriptor::VertexArray;
    assert_eq!(
        gl::create_resource(&resource).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        gl::delete_resource(gl).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        gl::capabilities().unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        gl::submit(&[GlCommand::DrawArrays {
            mode: GlPrimitiveMode::Triangles,
            first: 0,
            count: 3,
        }])
        .unwrap_err()
        .code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn resource_descriptors_are_validated_before_host_access() {
    assert_eq!(
        gl::create_hud_texture(1, 1, &[0, 0, 0]).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );

    let invalid_dimensions = GlResourceDescriptor::Texture {
        width: 0,
        height: 1,
        rgba: &[0, 0, 0, 0],
    };
    assert_eq!(
        gl::create_resource(&invalid_dimensions).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}
