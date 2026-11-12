//! Bounded, typed OpenGL bridge for runtime modules with `client.render.opengl`.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::abi::{GlResourceHandle, RenderResourceHandle};
use crate::{ErrorCode, SjmlError, SjmlResult};

pub const MAX_GL_BATCH_BYTES: usize = 1024 * 1024;
pub const MAX_GL_COMMANDS: usize = 1024;
pub const MAX_GL_RESOURCE_DESCRIPTOR_BYTES: usize = 1024 * 1024;
pub const MAX_GL_TEXTURE_DIMENSION: u32 = 4096;
pub const MAX_GL_STRING_BYTES: usize = 65_536;
pub const MAX_GL_UNIFORM_NAME_BYTES: usize = 128;
pub const MAX_GL_CAPABILITIES_BYTES: usize = 65_536;
pub const MAX_GL_REPORT_BYTES: usize = 16 + 4096;
const SCHEMA_VERSION: u32 = 1;
const HANDLE_BYTES: u32 = 24;

/// Stable compact ABI values for buffer bindings; the host maps these to OpenGL enums.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum GlBufferTarget {
    Array = 0,
    ElementArray = 1,
}

impl GlBufferTarget {
    pub const fn wire_value(self) -> u8 {
        self as u8
    }
}

/// Stable compact ABI values for buffer storage usage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum GlBufferUsage {
    StaticDraw = 0,
    DynamicDraw = 1,
    StreamDraw = 2,
}

impl GlBufferUsage {
    pub const fn wire_value(self) -> u8 {
        self as u8
    }
}

/// Stable compact ABI values for framebuffer attachment points.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum GlFramebufferAttachment {
    Color0 = 0,
    Depth = 1,
    Stencil = 2,
    DepthStencil = 3,
}

impl GlFramebufferAttachment {
    pub const fn wire_value(self) -> u8 {
        self as u8
    }
}

/// Raw OpenGL scalar type enums supported by vertex attribute descriptors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GlScalarType {
    Byte = 0x1400,
    UnsignedByte = 0x1401,
    Short = 0x1402,
    UnsignedShort = 0x1403,
    Int = 0x1404,
    UnsignedInt = 0x1405,
    Float = 0x1406,
}

/// Unsigned scalar types allowed for indexed draws.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GlIndexType {
    UnsignedByte = 0x1401,
    UnsignedShort = 0x1403,
    UnsignedInt = 0x1405,
}

/// Raw OpenGL primitive mode enums supported by ABI v1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GlPrimitiveMode {
    Points = 0,
    Lines = 1,
    LineLoop = 2,
    LineStrip = 3,
    Triangles = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

/// Raw OpenGL blend factors supported by ABI v1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GlBlendFactor {
    Zero = 0,
    One = 1,
    SrcColor = 0x0300,
    OneMinusSrcColor = 0x0301,
    SrcAlpha = 0x0302,
    OneMinusSrcAlpha = 0x0303,
    DstAlpha = 0x0304,
    OneMinusDstAlpha = 0x0305,
    DstColor = 0x0306,
    OneMinusDstColor = 0x0307,
    SrcAlphaSaturate = 0x0308,
    ConstantColor = 0x8001,
    OneMinusConstantColor = 0x8002,
    ConstantAlpha = 0x8003,
    OneMinusConstantAlpha = 0x8004,
}

/// Raw OpenGL depth comparison functions supported by ABI v1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GlDepthFunction {
    Never = 0x0200,
    Less = 0x0201,
    Equal = 0x0202,
    LessEqual = 0x0203,
    Greater = 0x0204,
    NotEqual = 0x0205,
    GreaterEqual = 0x0206,
    Always = 0x0207,
}

/// Raw OpenGL cull face values supported by ABI v1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GlCullFace {
    Front = 0x0404,
    Back = 0x0405,
    FrontAndBack = 0x0408,
}

/// Resource descriptors accepted by `gl::create_resource` (descriptor kinds 1–5).
pub enum GlResourceDescriptor<'a> {
    Buffer {
        target: GlBufferTarget,
        usage: GlBufferUsage,
        data: &'a [u8],
    },
    VertexArray,
    Texture {
        width: u32,
        height: u32,
        rgba: &'a [u8],
    },
    Framebuffer,
    Program {
        vertex: &'a str,
        fragment: &'a str,
    },
}

/// One validated OpenGL command. All resource handles are runtime-scoped and opaque.
pub enum GlCommand {
    UseProgram(GlResourceHandle),
    BindVertexArray(GlResourceHandle),
    BindBuffer {
        target: GlBufferTarget,
        handle: GlResourceHandle,
    },
    BufferSubData {
        target: GlBufferTarget,
        offset: u32,
        data: Vec<u8>,
    },
    VertexAttribute {
        index: u32,
        size: u32,
        scalar_type: GlScalarType,
        normalized: bool,
        stride: u32,
        offset: u32,
    },
    EnableAttribute {
        index: u32,
        enabled: bool,
    },
    BindTexture {
        unit: u32,
        handle: GlResourceHandle,
    },
    BindFramebuffer(Option<GlResourceHandle>),
    AttachFramebufferTexture {
        attachment: GlFramebufferAttachment,
        handle: GlResourceHandle,
    },
    /// Updates a bounded RGBA8 region of the texture bound to the active unit in this batch.
    TextureSubImage {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    /// Checks that the currently bound owned framebuffer is complete.
    CheckFramebufferComplete,
    Viewport {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Scissor {
        enabled: bool,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Blend {
        enabled: bool,
        src_rgb: GlBlendFactor,
        dst_rgb: GlBlendFactor,
        src_alpha: GlBlendFactor,
        dst_alpha: GlBlendFactor,
    },
    Depth {
        test: bool,
        write: bool,
        function: GlDepthFunction,
    },
    Cull {
        enabled: bool,
        face: GlCullFace,
    },
    UniformI32 {
        name: String,
        value: i32,
    },
    UniformF32 {
        name: String,
        value: f32,
    },
    UniformVec4 {
        name: String,
        value: [f32; 4],
    },
    UniformMat4 {
        name: String,
        values: [f32; 16],
    },
    DrawArrays {
        mode: GlPrimitiveMode,
        first: i32,
        count: i32,
    },
    DrawElements {
        mode: GlPrimitiveMode,
        count: i32,
        scalar_type: GlIndexType,
        offset: i32,
    },
    Clear {
        mask: u32,
        color: [f32; 4],
    },
    DeleteResource(GlResourceHandle),
}

/// Immutable snapshot returned by the host GL implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlCapabilities {
    pub major: u32,
    pub minor: u32,
    pub vendor: String,
    pub renderer: String,
    pub extensions: Vec<String>,
}

/// Result of submitting one GL batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlExecutionReport {
    pub executed_count: u32,
    pub gl_error_count: u32,
    pub message: String,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_gl_v1")]
extern "C" {
    #[link_name = "create_resource"]
    fn imported_create_resource(
        kind: i32,
        descriptor_pointer: i32,
        descriptor_length: i32,
        handle_output_pointer: i32,
    ) -> i32;
    #[link_name = "delete_resource"]
    fn imported_delete_resource(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "capabilities"]
    fn imported_capabilities(output_pointer: i32, capacity: i32) -> i64;
    #[link_name = "submit"]
    fn imported_submit(
        commands_pointer: i32,
        commands_length: i32,
        report_pointer: i32,
        capacity: i32,
    ) -> i64;
}

/// Creates a HUD texture whose handle can be passed to [`crate::render::RenderContext::draw_texture`].
pub fn create_hud_texture(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> SjmlResult<RenderResourceHandle> {
    validate_texture(width, height, rgba)?;
    let mut descriptor = Vec::with_capacity(12 + rgba.len());
    put_u32(&mut descriptor, width);
    put_u32(&mut descriptor, height);
    put_bytes(&mut descriptor, rgba)?;
    create_resource_raw(0, &descriptor)
        .and_then(|bytes| crate::from_bytes::<RenderResourceHandle>(&bytes))
}

/// Creates one typed OpenGL resource (buffer, VAO, texture, framebuffer, or program).
pub fn create_resource(descriptor: &GlResourceDescriptor<'_>) -> SjmlResult<GlResourceHandle> {
    let (kind, bytes) = encode_resource_descriptor(descriptor)?;
    create_resource_raw(kind, &bytes)
        .and_then(|bytes| crate::from_bytes::<GlResourceHandle>(&bytes))
}

/// Deletes one HUD texture created by [`create_hud_texture`].
pub fn delete_hud_texture(handle: RenderResourceHandle) -> SjmlResult<()> {
    delete_resource_raw(handle)
}

/// Deletes one GL object created by [`create_resource`].
pub fn delete_resource(handle: GlResourceHandle) -> SjmlResult<()> {
    delete_resource_raw(handle)
}

/// Queries an immutable capability snapshot from the current live client GL context.
pub fn capabilities() -> SjmlResult<GlCapabilities> {
    #[cfg(target_arch = "wasm32")]
    {
        let pointer = allocate_response_buffer(MAX_GL_CAPABILITIES_BYTES)?;
        // SAFETY: output points to a live guest allocation with the advertised 64 KiB capacity.
        let raw =
            unsafe { imported_capabilities(pointer as i32, MAX_GL_CAPABILITIES_BYTES as i32) };
        let result =
            read_packed_response(raw, pointer, MAX_GL_CAPABILITIES_BYTES, decode_capabilities);
        // SAFETY: free the allocation paired with the exact capacity once the host call has ended.
        unsafe { deallocate_host_input(pointer, MAX_GL_CAPABILITIES_BYTES as u32) };
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported())
    }
}

/// Submits a bounded GL batch and returns the host's execution report.
pub fn submit(commands: &[GlCommand]) -> SjmlResult<GlExecutionReport> {
    let encoded = encode_command_batch(commands)?;
    #[cfg(target_arch = "wasm32")]
    {
        let output = allocate_response_buffer(MAX_GL_REPORT_BYTES)?;
        // SAFETY: command bytes are live for this call; output has the full maximum report capacity.
        let raw = unsafe {
            imported_submit(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                output as i32,
                MAX_GL_REPORT_BYTES as i32,
            )
        };
        let result = read_packed_response(raw, output, MAX_GL_REPORT_BYTES, decode_report);
        // SAFETY: free the exact response allocation after the host has returned.
        unsafe { deallocate_host_input(output, MAX_GL_REPORT_BYTES as u32) };
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = encoded;
        Err(unsupported())
    }
}

fn create_resource_raw(kind: i32, descriptor: &[u8]) -> SjmlResult<[u8; HANDLE_BYTES as usize]> {
    if descriptor.len() > MAX_GL_RESOURCE_DESCRIPTOR_BYTES {
        return Err(payload_too_large("GL resource descriptor exceeds 1 MiB"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let output = allocate_response_buffer(HANDLE_BYTES as usize)?;
        // SAFETY: descriptor and output remain live for the synchronous host import.
        let status = unsafe {
            imported_create_resource(
                kind,
                descriptor.as_ptr() as usize as u32 as i32,
                descriptor.len() as u32 as i32,
                output as i32,
            )
        };
        let result = if status == 0 {
            // SAFETY: host wrote exactly the 24-byte handle output allocation.
            let bytes =
                unsafe { core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize) };
            let mut handle = [0; HANDLE_BYTES as usize];
            handle.copy_from_slice(bytes);
            Ok(handle)
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected GL resource creation",
            ))
        };
        // SAFETY: release exactly the handle output allocation after the synchronous host call.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = kind;
        let _ = descriptor;
        Err(unsupported())
    }
}

fn delete_resource_raw<K: crate::HandleKind>(handle: crate::Handle<K>) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut bytes = Vec::with_capacity(HANDLE_BYTES as usize);
        put_u64(&mut bytes, handle.owner_token());
        put_u64(&mut bytes, handle.context_token());
        put_u64(&mut bytes, handle.slot_token());
        // SAFETY: stable handle bytes remain live for the synchronous delete import.
        let status = unsafe {
            imported_delete_resource(bytes.as_ptr() as usize as u32 as i32, bytes.len() as i32)
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected GL resource deletion",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

fn encode_resource_descriptor(descriptor: &GlResourceDescriptor<'_>) -> SjmlResult<(i32, Vec<u8>)> {
    let mut output = Vec::new();
    let kind = match descriptor {
        GlResourceDescriptor::Buffer {
            target,
            usage,
            data,
        } => {
            output.push(target.wire_value());
            output.push(usage.wire_value());
            put_bytes(&mut output, data)?;
            1
        }
        GlResourceDescriptor::VertexArray => 2,
        GlResourceDescriptor::Texture {
            width,
            height,
            rgba,
        } => {
            validate_texture(*width, *height, rgba)?;
            put_u32(&mut output, *width);
            put_u32(&mut output, *height);
            put_bytes(&mut output, rgba)?;
            3
        }
        GlResourceDescriptor::Framebuffer => 4,
        GlResourceDescriptor::Program { vertex, fragment } => {
            if vertex.is_empty() || fragment.is_empty() {
                return Err(invalid(
                    "vertex and fragment shader sources must not be empty",
                ));
            }
            put_string(
                &mut output,
                vertex,
                MAX_GL_STRING_BYTES,
                "vertex shader source",
            )?;
            put_string(
                &mut output,
                fragment,
                MAX_GL_STRING_BYTES,
                "fragment shader source",
            )?;
            5
        }
    };
    if output.len() > MAX_GL_RESOURCE_DESCRIPTOR_BYTES {
        return Err(payload_too_large("GL resource descriptor exceeds 1 MiB"));
    }
    Ok((kind, output))
}

fn validate_texture(width: u32, height: u32, rgba: &[u8]) -> SjmlResult<()> {
    if width == 0
        || height == 0
        || width > MAX_GL_TEXTURE_DIMENSION
        || height > MAX_GL_TEXTURE_DIMENSION
    {
        return Err(invalid("RGBA8 texture dimensions must be in [1, 4096]"));
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| payload_too_large("texture pixel length overflowed"))?;
    if expected > MAX_GL_RESOURCE_DESCRIPTOR_BYTES - 12 || rgba.len() != expected {
        return Err(invalid(
            "RGBA8 texture requires exactly width*height*4 bytes within the descriptor limit",
        ));
    }
    Ok(())
}

fn validate_texture_subimage(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> SjmlResult<()> {
    if width == 0
        || height == 0
        || width > MAX_GL_TEXTURE_DIMENSION
        || height > MAX_GL_TEXTURE_DIMENSION
        || x > MAX_GL_TEXTURE_DIMENSION
        || y > MAX_GL_TEXTURE_DIMENSION
        || x.checked_add(width)
            .is_none_or(|right| right > MAX_GL_TEXTURE_DIMENSION)
        || y.checked_add(height)
            .is_none_or(|bottom| bottom > MAX_GL_TEXTURE_DIMENSION)
    {
        return Err(invalid(
            "RGBA8 subimage region must fit within the maximum 4096x4096 texture",
        ));
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| payload_too_large("texture subimage length overflowed"))?;
    if expected > MAX_GL_BATCH_BYTES - 29 || rgba.len() != expected {
        return Err(invalid(
            "RGBA8 subimage requires exactly width*height*4 bytes within the GL batch limit",
        ));
    }
    Ok(())
}

fn encode_command_batch(commands: &[GlCommand]) -> SjmlResult<Vec<u8>> {
    if commands.len() > MAX_GL_COMMANDS {
        return Err(payload_too_large("GL command count exceeds 1024"));
    }
    let mut output = Vec::new();
    put_u32(&mut output, SCHEMA_VERSION);
    put_u32(&mut output, commands.len() as u32);
    for command in commands {
        match command {
            GlCommand::UseProgram(handle) => {
                output.push(0);
                put_handle(&mut output, *handle);
            }
            GlCommand::BindVertexArray(handle) => {
                output.push(1);
                put_handle(&mut output, *handle);
            }
            GlCommand::BindBuffer { target, handle } => {
                output.push(2);
                output.push(target.wire_value());
                put_handle(&mut output, *handle);
            }
            GlCommand::BufferSubData {
                target,
                offset,
                data,
            } => {
                output.push(3);
                output.push(target.wire_value());
                put_u32(&mut output, *offset);
                put_bytes(&mut output, data)?;
            }
            GlCommand::VertexAttribute {
                index,
                size,
                scalar_type,
                normalized,
                stride,
                offset,
            } => {
                if !(1..=4).contains(size) {
                    return Err(invalid("vertex attribute size must be in [1, 4]"));
                }
                output.push(4);
                put_u32(&mut output, *index);
                put_u32(&mut output, *size);
                put_u32(&mut output, *scalar_type as u32);
                output.push(u8::from(*normalized));
                put_u32(&mut output, *stride);
                put_u32(&mut output, *offset);
            }
            GlCommand::EnableAttribute { index, enabled } => {
                output.push(5);
                put_u32(&mut output, *index);
                output.push(u8::from(*enabled));
            }
            GlCommand::BindTexture { unit, handle } => {
                output.push(6);
                put_u32(&mut output, *unit);
                put_handle(&mut output, *handle);
            }
            GlCommand::BindFramebuffer(handle) => {
                output.push(7);
                if let Some(handle) = handle {
                    output.push(1);
                    put_handle(&mut output, *handle);
                } else {
                    output.push(0);
                }
            }
            GlCommand::AttachFramebufferTexture { attachment, handle } => {
                output.push(8);
                output.push(attachment.wire_value());
                put_handle(&mut output, *handle);
            }
            GlCommand::TextureSubImage {
                x,
                y,
                width,
                height,
                rgba,
            } => {
                validate_texture_subimage(*x, *y, *width, *height, rgba)?;
                output.push(22);
                put_u32(&mut output, *x);
                put_u32(&mut output, *y);
                put_u32(&mut output, *width);
                put_u32(&mut output, *height);
                put_bytes(&mut output, rgba)?;
            }
            GlCommand::CheckFramebufferComplete => output.push(23),
            GlCommand::Viewport {
                x,
                y,
                width,
                height,
            } => {
                if *width < 0 || *height < 0 {
                    return Err(invalid("viewport dimensions must not be negative"));
                }
                output.push(9);
                put_i32(&mut output, *x);
                put_i32(&mut output, *y);
                put_i32(&mut output, *width);
                put_i32(&mut output, *height);
            }
            GlCommand::Scissor {
                enabled,
                x,
                y,
                width,
                height,
            } => {
                if *width < 0 || *height < 0 {
                    return Err(invalid("scissor dimensions must not be negative"));
                }
                output.push(10);
                output.push(u8::from(*enabled));
                put_i32(&mut output, *x);
                put_i32(&mut output, *y);
                put_i32(&mut output, *width);
                put_i32(&mut output, *height);
            }
            GlCommand::Blend {
                enabled,
                src_rgb,
                dst_rgb,
                src_alpha,
                dst_alpha,
            } => {
                output.push(11);
                output.push(u8::from(*enabled));
                for factor in [src_rgb, dst_rgb, src_alpha, dst_alpha] {
                    put_u32(&mut output, *factor as u32);
                }
            }
            GlCommand::Depth {
                test,
                write,
                function,
            } => {
                output.push(12);
                output.push(u8::from(*test));
                output.push(u8::from(*write));
                put_u32(&mut output, *function as u32);
            }
            GlCommand::Cull { enabled, face } => {
                output.push(13);
                output.push(u8::from(*enabled));
                put_u32(&mut output, *face as u32);
            }
            GlCommand::UniformI32 { name, value } => {
                output.push(14);
                put_uniform_name(&mut output, name)?;
                put_i32(&mut output, *value);
            }
            GlCommand::UniformF32 { name, value } => {
                require_finite(*value, "uniform float")?;
                output.push(15);
                put_uniform_name(&mut output, name)?;
                put_f32(&mut output, *value);
            }
            GlCommand::UniformVec4 { name, value } => {
                output.push(16);
                put_uniform_name(&mut output, name)?;
                for component in value {
                    require_finite(*component, "uniform vec4 component")?;
                    put_f32(&mut output, *component);
                }
            }
            GlCommand::UniformMat4 { name, values } => {
                output.push(17);
                put_uniform_name(&mut output, name)?;
                for component in values {
                    require_finite(*component, "uniform mat4 component")?;
                    put_f32(&mut output, *component);
                }
            }
            GlCommand::DrawArrays { mode, first, count } => {
                if *first < 0 || *count < 0 {
                    return Err(invalid("draw array first/count must not be negative"));
                }
                output.push(18);
                put_u32(&mut output, *mode as u32);
                put_i32(&mut output, *first);
                put_i32(&mut output, *count);
            }
            GlCommand::DrawElements {
                mode,
                count,
                scalar_type,
                offset,
            } => {
                if *count < 0 || *offset < 0 {
                    return Err(invalid("draw element count/offset must not be negative"));
                }
                let alignment = match scalar_type {
                    GlIndexType::UnsignedByte => 1,
                    GlIndexType::UnsignedShort => 2,
                    GlIndexType::UnsignedInt => 4,
                };
                if *offset % alignment != 0 {
                    return Err(invalid(
                        "draw element offset is not aligned to its scalar type",
                    ));
                }
                output.push(19);
                put_u32(&mut output, *mode as u32);
                put_i32(&mut output, *count);
                put_u32(&mut output, *scalar_type as u32);
                put_i32(&mut output, *offset);
            }
            GlCommand::Clear { mask, color } => {
                if mask & !0x4500 != 0 {
                    return Err(invalid("clear mask contains unsupported bits"));
                }
                output.push(20);
                put_u32(&mut output, *mask);
                for component in color {
                    require_finite(*component, "clear color")?;
                    put_f32(&mut output, *component);
                }
            }
            GlCommand::DeleteResource(handle) => {
                output.push(21);
                put_handle(&mut output, *handle);
            }
        }
        if output.len() > MAX_GL_BATCH_BYTES {
            return Err(payload_too_large("GL command batch exceeds 1 MiB"));
        }
    }
    Ok(output)
}

#[cfg(target_arch = "wasm32")]
fn allocate_response_buffer(length: usize) -> SjmlResult<u32> {
    let pointer = allocate_host_input(length as u32);
    if pointer == 0 {
        return Err(payload_too_large(
            "could not allocate GL import output buffer",
        ));
    }
    // SAFETY: allocator returned a live writable span of exactly `length` bytes.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, length) };
    Ok(pointer)
}

#[cfg(target_arch = "wasm32")]
fn read_packed_response<T>(
    raw: i64,
    pointer: u32,
    capacity: usize,
    decode: fn(&[u8]) -> SjmlResult<T>,
) -> SjmlResult<T> {
    let bits = raw as u64;
    let status = bits as u32;
    if status != 0 {
        return Err(SjmlError::from_abi_code(
            status,
            "host rejected GL bridge operation",
        ));
    }
    let written = (bits >> 32) as u32 as usize;
    if written > capacity {
        return Err(SjmlError::new(
            ErrorCode::MemoryOutOfBounds,
            "host GL response exceeds its output capacity",
        ));
    }
    // SAFETY: host import wrote no more than the checked capacity in the live allocation.
    let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, written) };
    decode(bytes)
}

#[cfg(target_arch = "wasm32")]
fn decode_capabilities(bytes: &[u8]) -> SjmlResult<GlCapabilities> {
    let mut input = Reader::new(bytes)?;
    if input.u32()? != SCHEMA_VERSION {
        return Err(abi_error(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported GL capabilities schema",
        ));
    }
    let major = input.u32()?;
    let minor = input.u32()?;
    let vendor = input.string(16_384)?;
    let renderer = input.string(16_384)?;
    let count = input.count(1024)?;
    let mut extensions = Vec::with_capacity(count);
    for _ in 0..count {
        extensions.push(input.string(MAX_GL_UNIFORM_NAME_BYTES)?);
    }
    input.finish()?;
    let total: usize =
        vendor.len() + renderer.len() + extensions.iter().map(String::len).sum::<usize>();
    if total > MAX_GL_CAPABILITIES_BYTES {
        return Err(payload_too_large("GL capability strings exceed 64 KiB"));
    }
    Ok(GlCapabilities {
        major,
        minor,
        vendor,
        renderer,
        extensions,
    })
}

#[cfg(target_arch = "wasm32")]
fn decode_report(bytes: &[u8]) -> SjmlResult<GlExecutionReport> {
    let mut input = Reader::new(bytes)?;
    if input.u32()? != SCHEMA_VERSION {
        return Err(abi_error(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported GL execution report schema",
        ));
    }
    let executed_count = input.u32()?;
    let gl_error_count = input.u32()?;
    let message = input.string(4096)?;
    input.finish()?;
    Ok(GlExecutionReport {
        executed_count,
        gl_error_count,
        message,
    })
}

#[cfg(target_arch = "wasm32")]
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

#[cfg(target_arch = "wasm32")]
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> SjmlResult<Self> {
        if bytes.len() > MAX_GL_CAPABILITIES_BYTES {
            return Err(payload_too_large("GL response exceeds 64 KiB"));
        }
        Ok(Self { bytes, offset: 0 })
    }
    fn raw(&mut self, length: usize) -> SjmlResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid("GL response length overflowed"))?;
        if end > self.bytes.len() {
            return Err(abi_error(
                ErrorCode::MemoryOutOfBounds,
                "truncated GL response",
            ));
        }
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }
    fn u32(&mut self) -> SjmlResult<u32> {
        Ok(u32::from_le_bytes(self.raw(4)?.try_into().unwrap()))
    }
    fn count(&mut self, max: usize) -> SjmlResult<usize> {
        let count = self.u32()? as usize;
        if count > max {
            return Err(payload_too_large(
                "GL response collection exceeds its limit",
            ));
        }
        Ok(count)
    }
    fn string(&mut self, max: usize) -> SjmlResult<String> {
        let length = self.count(max)?;
        let text = core::str::from_utf8(self.raw(length)?)
            .map_err(|_| abi_error(ErrorCode::InvalidUtf8, "GL response string is not UTF-8"))?;
        Ok(text.to_owned())
    }
    fn finish(&self) -> SjmlResult<()> {
        if self.offset != self.bytes.len() {
            return Err(invalid("GL response has trailing bytes"));
        }
        Ok(())
    }
}

fn put_handle<K: crate::HandleKind>(output: &mut Vec<u8>, handle: crate::Handle<K>) {
    put_u64(output, handle.owner_token());
    put_u64(output, handle.context_token());
    put_u64(output, handle.slot_token());
}
fn put_bytes(output: &mut Vec<u8>, bytes: &[u8]) -> SjmlResult<()> {
    let length = u32::try_from(bytes.len())
        .map_err(|_| payload_too_large("GL byte vector exceeds u32 length"))?;
    let fits_limit = output
        .len()
        .checked_add(core::mem::size_of::<u32>())
        .and_then(|current| current.checked_add(bytes.len()))
        .is_some_and(|final_length| final_length <= MAX_GL_BATCH_BYTES);
    if !fits_limit {
        return Err(payload_too_large("GL payload exceeds 1 MiB"));
    }
    put_u32(output, length);
    output.extend_from_slice(bytes);
    Ok(())
}
fn put_string(output: &mut Vec<u8>, value: &str, maximum: usize, label: &str) -> SjmlResult<()> {
    if value.len() > maximum {
        return Err(payload_too_large(&format!(
            "{label} exceeds its UTF-8 byte limit"
        )));
    }
    if value.as_bytes().contains(&0) {
        return Err(invalid("GL strings must not contain NUL bytes"));
    }
    put_bytes(output, value.as_bytes())
}
fn put_uniform_name(output: &mut Vec<u8>, name: &str) -> SjmlResult<()> {
    let bytes = name.as_bytes();
    let valid_start = bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_');
    if !valid_start
        || bytes.len() > MAX_GL_UNIFORM_NAME_BYTES
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return Err(invalid(
            "uniform names must match [A-Za-z_][A-Za-z0-9_]{0,127}",
        ));
    }
    put_string(output, name, MAX_GL_UNIFORM_NAME_BYTES, "uniform name")
}
fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn put_i32(output: &mut Vec<u8>, value: i32) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn put_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn put_f32(output: &mut Vec<u8>, value: f32) {
    output.extend_from_slice(&value.to_bits().to_le_bytes());
}
fn require_finite(value: f32, label: &str) -> SjmlResult<()> {
    if !value.is_finite() {
        return Err(invalid(&format!("{label} must be finite")));
    }
    Ok(())
}
fn invalid(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
fn payload_too_large(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::PayloadTooLarge, message)
}
#[cfg(target_arch = "wasm32")]
fn abi_error(code: ErrorCode, message: &'static str) -> SjmlError {
    SjmlError::new(code, message)
}
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "OpenGL imports require wasm32-unknown-unknown and SJML GL host imports",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle() -> GlResourceHandle {
        GlResourceHandle::new(1, 2, 3, 4).unwrap()
    }

    #[test]
    fn descriptors_use_compact_u8_codes_and_exact_rgba_byte_vectors() {
        let (kind, bytes) = encode_resource_descriptor(&GlResourceDescriptor::Buffer {
            target: GlBufferTarget::ElementArray,
            usage: GlBufferUsage::DynamicDraw,
            data: &[7, 8],
        })
        .unwrap();
        assert_eq!(kind, 1);
        assert_eq!(bytes, [1, 1, 2, 0, 0, 0, 7, 8]);

        let (kind, texture) = encode_resource_descriptor(&GlResourceDescriptor::Texture {
            width: 1,
            height: 1,
            rgba: &[1, 2, 3, 4],
        })
        .unwrap();
        assert_eq!(kind, 3);
        assert_eq!(&texture[..12], &[1, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0]);
        assert_eq!(&texture[12..], &[1, 2, 3, 4]);
        assert_eq!(
            encode_resource_descriptor(&GlResourceDescriptor::VertexArray).unwrap(),
            (2, vec![])
        );
    }

    #[test]
    fn command_batch_encodes_all_tags_and_optional_default_framebuffer() {
        let commands = vec![
            GlCommand::UseProgram(handle()),
            GlCommand::BindVertexArray(handle()),
            GlCommand::BindBuffer {
                target: GlBufferTarget::Array,
                handle: handle(),
            },
            GlCommand::BufferSubData {
                target: GlBufferTarget::ElementArray,
                offset: 4,
                data: vec![5, 6],
            },
            GlCommand::VertexAttribute {
                index: 0,
                size: 2,
                scalar_type: GlScalarType::Float,
                normalized: false,
                stride: 8,
                offset: 0,
            },
            GlCommand::EnableAttribute {
                index: 1,
                enabled: true,
            },
            GlCommand::BindTexture {
                unit: 0,
                handle: handle(),
            },
            GlCommand::BindFramebuffer(None),
            GlCommand::AttachFramebufferTexture {
                attachment: GlFramebufferAttachment::Depth,
                handle: handle(),
            },
            GlCommand::TextureSubImage {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                rgba: vec![1, 2, 3, 4],
            },
            GlCommand::CheckFramebufferComplete,
            GlCommand::Viewport {
                x: -1,
                y: 2,
                width: 640,
                height: 480,
            },
            GlCommand::Scissor {
                enabled: true,
                x: 1,
                y: 2,
                width: 3,
                height: 4,
            },
            GlCommand::Blend {
                enabled: true,
                src_rgb: GlBlendFactor::SrcAlpha,
                dst_rgb: GlBlendFactor::OneMinusSrcAlpha,
                src_alpha: GlBlendFactor::One,
                dst_alpha: GlBlendFactor::Zero,
            },
            GlCommand::Depth {
                test: true,
                write: false,
                function: GlDepthFunction::LessEqual,
            },
            GlCommand::Cull {
                enabled: true,
                face: GlCullFace::Back,
            },
            GlCommand::UniformI32 {
                name: "u_count".to_owned(),
                value: 4,
            },
            GlCommand::UniformF32 {
                name: "u_alpha".to_owned(),
                value: 0.5,
            },
            GlCommand::UniformVec4 {
                name: "u_color".to_owned(),
                value: [1.0, 0.0, 0.0, 1.0],
            },
            GlCommand::UniformMat4 {
                name: "u_matrix".to_owned(),
                values: [0.0; 16],
            },
            GlCommand::DrawArrays {
                mode: GlPrimitiveMode::Triangles,
                first: 0,
                count: 3,
            },
            GlCommand::DrawElements {
                mode: GlPrimitiveMode::Triangles,
                count: 3,
                scalar_type: GlIndexType::UnsignedShort,
                offset: 0,
            },
            GlCommand::Clear {
                mask: 0x4000,
                color: [0.0, 0.0, 0.0, 1.0],
            },
            GlCommand::DeleteResource(handle()),
        ];
        let bytes = encode_command_batch(&commands).unwrap();
        assert_eq!(&bytes[..8], &[1, 0, 0, 0, 24, 0, 0, 0]);
        assert_eq!(bytes[8], 0);
        let bind_framebuffer = bytes.iter().position(|byte| *byte == 7).unwrap();
        assert_eq!(bytes[bind_framebuffer + 1], 0);
    }

    #[test]
    fn vec4_uniform_encodes_exactly_four_finite_components() {
        let encoded = encode_command_batch(&[GlCommand::UniformVec4 {
            name: "u".to_owned(),
            value: [1.0, 2.0, 3.0, 4.0],
        }])
        .unwrap();
        assert_eq!(encoded.len(), 30);
        assert_eq!(
            &encoded[14..],
            &[0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 128, 64,]
        );
    }

    #[test]
    fn texture_subimage_rejects_regions_and_payloads_that_cannot_fit_abi_limits() {
        for command in [
            GlCommand::TextureSubImage {
                x: 0,
                y: 0,
                width: 0,
                height: 1,
                rgba: vec![],
            },
            GlCommand::TextureSubImage {
                x: 4096,
                y: 0,
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            },
            GlCommand::TextureSubImage {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
                rgba: vec![0; 3],
            },
        ] {
            assert_eq!(
                encode_command_batch(&[command]).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn gl_payload_validation_rejects_empty_names_and_preserves_output_on_overflow() {
        assert_eq!(
            encode_command_batch(&[GlCommand::UniformI32 {
                name: String::new(),
                value: 1
            }])
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            encode_resource_descriptor(&GlResourceDescriptor::Program {
                vertex: "",
                fragment: "fragment"
            })
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );

        let mut output = vec![1, 2, 3];
        let oversized = vec![0; MAX_GL_BATCH_BYTES];
        assert_eq!(
            put_bytes(&mut output, &oversized).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(output, [1, 2, 3]);
    }

    #[test]
    fn command_batch_rejects_invalid_ranges_nonfinite_values_and_over_limits() {
        for command in [
            GlCommand::Viewport {
                x: 0,
                y: 0,
                width: -1,
                height: 1,
            },
            GlCommand::DrawArrays {
                mode: GlPrimitiveMode::Triangles,
                first: -1,
                count: 1,
            },
            GlCommand::DrawElements {
                mode: GlPrimitiveMode::Triangles,
                count: 1,
                scalar_type: GlIndexType::UnsignedShort,
                offset: 1,
            },
        ] {
            assert_eq!(
                encode_command_batch(&[command]).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            encode_command_batch(&[GlCommand::UniformF32 {
                name: "u".into(),
                value: f32::NAN
            }])
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(encode_command_batch(&[]).unwrap().len(), 8);
        let oversized: Vec<_> = std::iter::repeat_with(|| GlCommand::BindFramebuffer(None))
            .take(MAX_GL_COMMANDS + 1)
            .collect();
        assert_eq!(
            encode_command_batch(&oversized).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_gl_imports_are_unsupported_after_local_validation() {
        assert_eq!(
            create_hud_texture(1, 1, &[1, 2, 3, 4]).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            create_hud_texture(1, 1, &[1, 2, 3]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            create_resource(&GlResourceDescriptor::VertexArray)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            delete_resource(handle()).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            capabilities().unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            submit(&[GlCommand::DrawArrays {
                mode: GlPrimitiveMode::Triangles,
                first: 0,
                count: 3
            }])
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
