//! Runtime client-render subscriptions and bounded GUI/world command batches.

use crate::abi::EventSubscriptionHandle;
#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
#[cfg(target_arch = "wasm32")]
use crate::StableDecode;
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum draw commands accepted during one callback.
pub const MAX_RENDER_COMMANDS: usize = 1024;
/// Maximum complete command batch, including its schema and count fields.
pub const MAX_RENDER_BATCH_BYTES: usize = 1024 * 1024;
/// Maximum encoded bytes in one draw-text string.
pub const MAX_RENDER_TEXT_BYTES: usize = 16 * 1024;
/// Maximum UTF-8 bytes in a package font path or GUI sprite path.
pub const MAX_RENDER_RESOURCE_PATH_BYTES: usize = 256;
/// Maximum encoded bytes in one tooltip.
pub const MAX_TOOLTIP_BYTES: usize = 4096;
/// Maximum combined text, tooltip, and package-resource-path bytes in one GUI batch.
pub const MAX_RENDER_TEXT_BATCH_BYTES: usize = 64 * 1024;
/// Maximum quad faces accepted from one module during one world render callback.
pub const MAX_WORLD_MODEL_QUADS: usize = 256;
/// Maximum vertices accepted from one module during one world render callback.
pub const MAX_WORLD_MODEL_VERTICES: usize = MAX_WORLD_MODEL_QUADS * 4;
#[cfg(target_arch = "wasm32")]
const MAX_RENDER_SUBSCRIPTIONS: usize = 1024;
const RENDER_SCHEMA_VERSION: u32 = 1;
#[cfg(target_arch = "wasm32")]
const HANDLE_BYTES: u32 = 24;

/// Host-scoped runtime render subscription, using the existing event handle kind.
pub type RenderSubscriptionHandle = EventSubscriptionHandle;

/// Client callback layer selected when subscribing to a render event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RenderLayer {
    Hud = 0,
    Screen = 1,
    World = 2,
}

impl RenderLayer {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Hud => "hud",
            Self::Screen => "screen",
            Self::World => "world",
        }
    }

    pub const fn wire_value(self) -> u8 {
        self as u8
    }
}

/// A bounded font choice for GUI text.
///
/// Package fonts are loaded only from this package's resource namespace. The font path refers
/// to a JSON file below `assets/<package-id>/font/`, without the `.json` suffix. Minecraft's
/// three built-in GUI font choices are also available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderFont {
    /// Minecraft's normal proportional font.
    Default,
    /// Minecraft's uniform-width font.
    Uniform,
    /// Minecraft's alternate obfuscated-style font.
    Alt,
    /// A font definition in this package's `assets/<package-id>/font/` directory.
    Package(String),
}

impl RenderFont {
    /// Selects a package-owned font definition, such as `fonts/pixel` for
    /// `assets/<package-id>/font/fonts/pixel.json`.
    pub fn package(path: &str) -> SjmlResult<Self> {
        validate_font_path(path)?;
        Ok(Self::Package(path.to_owned()))
    }

    fn wire_kind(&self) -> u8 {
        match self {
            Self::Default => 0,
            Self::Uniform => 1,
            Self::Alt => 2,
            Self::Package(_) => 3,
        }
    }

    fn wire_path(&self) -> Option<&str> {
        match self {
            Self::Package(path) => Some(path),
            _ => None,
        }
    }
}

/// Camera position for one world-render pass, expressed in absolute Minecraft world coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldRenderFrame {
    pub camera_x: f64,
    pub camera_y: f64,
    pub camera_z: f64,
}

impl WorldRenderFrame {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<Self> {
        const MAX_CAMERA_COORDINATE: f64 = 30_000_000.0;
        if [self.camera_x, self.camera_y, self.camera_z]
            .into_iter()
            .any(|coordinate| !coordinate.is_finite() || coordinate.abs() > MAX_CAMERA_COORDINATE)
        {
            return Err(invalid(
                "world camera coordinates are outside the supported finite range",
            ));
        }
        Ok(self)
    }
}

/// Immutable GUI metrics supplied for one callback invocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFrame {
    /// Scaled GUI width in logical GUI coordinates.
    pub width: u32,
    /// Scaled GUI height in logical GUI coordinates.
    pub height: u32,
    /// Mouse X in logical GUI coordinates; may be outside the screen bounds.
    pub mouse_x: f32,
    /// Mouse Y in logical GUI coordinates; may be outside the screen bounds.
    pub mouse_y: f32,
    /// Render interpolation fraction in `[0, 1]`.
    pub partial_tick: f32,
}

impl RenderFrame {
    #[cfg(any(target_arch = "wasm32", test))]
    fn validate(self) -> SjmlResult<Self> {
        if !self.mouse_x.is_finite() || !self.mouse_y.is_finite() || !self.partial_tick.is_finite()
        {
            return Err(invalid("render frame floats must be finite"));
        }
        if !(0.0..=1.0).contains(&self.partial_tick) {
            return Err(invalid("render partial tick must be within [0, 1]"));
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorldQuad {
    vertices: [[f64; 3]; 4],
    argb: u32,
}

/// Bounded builder for untextured, colored quad faces in absolute world coordinates.
///
/// The builder origin and [`add_world_quad`](Self::add_world_quad) vertices use absolute world
/// positions. [`add_quad`](Self::add_quad) adds local offsets to the origin. On [`draw`](Self::draw),
/// the builder subtracts the current camera position and emits bounded camera-relative vertices.
/// Both forms are limited to 256 faces and 1,024 vertices.
///
/// ```
/// use sjml::prelude::*;
///
/// #[allow(dead_code)]
/// fn install_model_overlay() -> SjmlResult<()> {
///     let _world = render::subscribe(RenderLayer::World, |context| {
///         let camera = context.world_frame().ok_or_else(|| {
///             SjmlError::new(ErrorCode::InvalidArgument, "world camera is unavailable")
///         })?;
///         let mut model = WorldModelBuilder::new([
///             camera.camera_x,
///             camera.camera_y,
///             camera.camera_z,
///         ])?;
///         model.add_quad(
///             [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]],
///             0xFFFF_8040,
///         )?;
///         model.draw(context)
///     })?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct WorldModelBuilder {
    origin: [f64; 3],
    quads: Vec<WorldQuad>,
}

impl WorldModelBuilder {
    /// Creates an empty model at an absolute world origin. Each coordinate is bounded to
    /// +/-30,000,000 world units.
    pub fn new(origin: [f64; 3]) -> SjmlResult<Self> {
        for coordinate in origin {
            validate_world_position(coordinate)?;
        }
        Ok(Self {
            origin,
            quads: Vec::new(),
        })
    }

    /// Adds four local offsets to this builder's absolute world origin. Local coordinates must be
    /// finite and within +/-4,096 units. Vertex order is preserved; colors use packed ARGB.
    pub fn add_quad(&mut self, local_vertices: [[f32; 3]; 4], argb: u32) -> SjmlResult<&mut Self> {
        let mut world_vertices = [[0.0_f64; 3]; 4];
        for (world_vertex, local_vertex) in world_vertices.iter_mut().zip(local_vertices) {
            for local_coordinate in local_vertex {
                validate_world_coordinate(local_coordinate)?;
            }
            let translated: [f64; 3] =
                std::array::from_fn(|axis| self.origin[axis] + f64::from(local_vertex[axis]));
            for coordinate in translated {
                validate_world_position(coordinate)?;
            }
            *world_vertex = translated;
        }
        self.push_quad(world_vertices, argb)
    }

    /// Adds four vertices already expressed as absolute world positions.
    pub fn add_world_quad(&mut self, vertices: [[f64; 3]; 4], argb: u32) -> SjmlResult<&mut Self> {
        for vertex in vertices {
            for coordinate in vertex {
                validate_world_position(coordinate)?;
            }
        }
        self.push_quad(vertices, argb)
    }

    /// Number of faces currently held by the builder.
    pub fn quad_count(&self) -> usize {
        self.quads.len()
    }

    /// Number of vertices currently held by the builder.
    pub fn vertex_count(&self) -> usize {
        self.quads.len() * 4
    }

    /// Adds this model's faces to an active world render callback's bounded command batch after
    /// converting absolute positions to camera-relative coordinates.
    pub fn draw(&self, context: &mut RenderContext) -> SjmlResult<()> {
        let camera = context
            .world_frame()
            .ok_or_else(|| invalid("world model drawing requires a world render callback"))?;
        let camera = [camera.camera_x, camera.camera_y, camera.camera_z];
        for quad in &self.quads {
            let mut relative_vertices = [[0.0; 3]; 4];
            for (relative_vertex, world_vertex) in relative_vertices.iter_mut().zip(quad.vertices) {
                let relative: [f32; 3] =
                    std::array::from_fn(|axis| (world_vertex[axis] - camera[axis]) as f32);
                for coordinate in relative {
                    validate_world_coordinate(coordinate)?;
                }
                *relative_vertex = relative;
            }
            context.draw_world_quad(relative_vertices, quad.argb)?;
        }
        Ok(())
    }

    fn push_quad(&mut self, vertices: [[f64; 3]; 4], argb: u32) -> SjmlResult<&mut Self> {
        if self.quads.len() >= MAX_WORLD_MODEL_QUADS {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "world model quad limit reached",
            ));
        }
        self.quads.push(WorldQuad { vertices, argb });
        Ok(self)
    }
}

/// High-level builder handed to each callback. Its batch is submitted once only after the
/// callback returns successfully.
pub struct RenderContext {
    frame: RenderFrame,
    layer: RenderLayer,
    world_frame: Option<WorldRenderFrame>,
    commands: Vec<RenderCommand>,
    world_commands: Vec<WorldRenderCommand>,
    text_bytes: usize,
}

impl RenderContext {
    #[cfg(any(target_arch = "wasm32", test))]
    fn new(frame: RenderFrame, layer: RenderLayer, world_frame: Option<WorldRenderFrame>) -> Self {
        Self {
            frame,
            layer,
            world_frame,
            commands: Vec::new(),
            world_commands: Vec::new(),
            text_bytes: 0,
        }
    }

    /// GUI dimensions, pointer coordinates, and interpolation fraction. World callbacks use
    /// [`world_frame`](Self::world_frame) for camera data; their GUI dimensions and pointer values
    /// are zero and `-1.0` respectively.
    pub const fn frame(&self) -> RenderFrame {
        self.frame
    }

    /// Camera position for world callbacks, or `None` for HUD and screen callbacks.
    pub const fn world_frame(&self) -> Option<WorldRenderFrame> {
        self.world_frame
    }

    /// Adds a camera-relative world line in logical world units with an ARGB color.
    #[allow(clippy::too_many_arguments)] // Two three-dimensional endpoints and one color define a compact primitive.
    pub fn draw_world_line(
        &mut self,
        from_x: f32,
        from_y: f32,
        from_z: f32,
        to_x: f32,
        to_y: f32,
        to_z: f32,
        argb: u32,
    ) -> SjmlResult<()> {
        self.require_layer(RenderLayer::World)?;
        for value in [from_x, from_y, from_z, to_x, to_y, to_z] {
            validate_world_coordinate(value)?;
        }
        self.push_world(WorldRenderCommand::Line {
            from_x,
            from_y,
            from_z,
            to_x,
            to_y,
            to_z,
            argb,
        })
    }

    /// Adds a camera-relative outlined box with non-negative extents and an ARGB color.
    #[allow(clippy::too_many_arguments)] // Origin, three extents, and color are the box's complete bounded value.
    pub fn draw_world_box(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        depth: f32,
        argb: u32,
    ) -> SjmlResult<()> {
        self.require_layer(RenderLayer::World)?;
        for value in [x, y, z] {
            validate_world_coordinate(value)?;
        }
        for extent in [width, height, depth] {
            validate_world_extent(extent)?;
        }
        for maximum in [x + width, y + height, z + depth] {
            validate_world_coordinate(maximum)?;
        }
        self.push_world(WorldRenderCommand::Box {
            x,
            y,
            z,
            width,
            height,
            depth,
            argb,
        })
    }

    /// Adds one colored camera-relative quad face in vertex order.
    pub fn draw_world_quad(&mut self, vertices: [[f32; 3]; 4], argb: u32) -> SjmlResult<()> {
        self.require_layer(RenderLayer::World)?;
        for vertex in vertices {
            for coordinate in vertex {
                validate_world_coordinate(coordinate)?;
            }
        }
        self.push_world(WorldRenderCommand::Quad { vertices, argb })
    }

    /// Adds a text draw operation with an ARGB color.
    pub fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        argb: u32,
        shadow: bool,
    ) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_string(text, MAX_RENDER_TEXT_BYTES, "render text")?;
        validate_position(x, y)?;
        self.push(
            RenderCommand::Text {
                text: text.to_owned(),
                x,
                y,
                argb,
                shadow,
            },
            text.len(),
        )
    }

    /// Adds text using one of Minecraft's built-in fonts or a font definition owned by this
    /// package. Package font paths are relative to `assets/<package-id>/font/` and omit `.json`.
    pub fn draw_text_with_font(
        &mut self,
        text: &str,
        font: &RenderFont,
        x: f32,
        y: f32,
        argb: u32,
        shadow: bool,
    ) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_string(text, MAX_RENDER_TEXT_BYTES, "render text")?;
        if let Some(path) = font.wire_path() {
            validate_font_path(path)?;
        }
        validate_position(x, y)?;
        let path = font.wire_path();
        self.push(
            RenderCommand::TextWithFont {
                text: text.to_owned(),
                font_kind: font.wire_kind(),
                font_path: path.map(str::to_owned),
                x,
                y,
                argb,
                shadow,
            },
            text.len() + path.map_or(0, str::len),
        )
    }

    /// Draws a package-owned PNG in `assets/<package-id>/textures/gui/` as a GUI image.
    /// `source_dimensions` are the PNG's pixel dimensions, each limited to 4096. The image can be
    /// scaled to any finite non-negative logical GUI size.
    pub fn draw_sprite(
        &mut self,
        path: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        source_dimensions: (u32, u32),
    ) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_gui_sprite_path(path)?;
        validate_dimensions(x, y, width, height)?;
        let (source_width, source_height) = source_dimensions;
        if source_width == 0 || source_width > 4096 || source_height == 0 || source_height > 4096 {
            return Err(invalid(
                "GUI sprite source dimensions must be within [1, 4096]",
            ));
        }
        self.push(
            RenderCommand::Sprite {
                path: path.to_owned(),
                x,
                y,
                width,
                height,
                source_width,
                source_height,
            },
            path.len(),
        )
    }

    /// Adds a filled rectangle with an ARGB color.
    pub fn draw_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        argb: u32,
    ) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_dimensions(x, y, width, height)?;
        self.push(
            RenderCommand::Rect {
                x,
                y,
                width,
                height,
                argb,
            },
            0,
        )
    }

    /// Adds a background plus a left-to-right progress fill. Fraction is in `[0, 1]`.
    #[allow(clippy::too_many_arguments)] // Coordinates, fraction, and two color channels form one compact draw command.
    pub fn draw_progress(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fraction: f32,
        background_argb: u32,
        foreground_argb: u32,
    ) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_dimensions(x, y, width, height)?;
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(invalid(
                "progress fraction must be finite and within [0, 1]",
            ));
        }
        self.push(
            RenderCommand::Progress {
                x,
                y,
                width,
                height,
                fraction,
                background_argb,
                foreground_argb,
            },
            0,
        )
    }

    /// Adds a module-owned HUD texture draw operation.
    pub fn draw_texture(
        &mut self,
        handle: crate::RenderResourceHandle,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        opacity: f32,
    ) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_dimensions(x, y, width, height)?;
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(invalid("texture opacity must be finite and within [0, 1]"));
        }
        self.push(
            RenderCommand::Texture {
                handle,
                x,
                y,
                width,
                height,
                opacity,
            },
            0,
        )
    }

    /// Adds a bounded tooltip string at logical GUI coordinates.
    pub fn draw_tooltip(&mut self, text: &str, x: f32, y: f32) -> SjmlResult<()> {
        self.require_gui_layer()?;
        validate_string(text, MAX_TOOLTIP_BYTES, "render tooltip")?;
        validate_position(x, y)?;
        self.push(
            RenderCommand::Tooltip {
                text: text.to_owned(),
                x,
                y,
            },
            text.len(),
        )
    }

    fn push(&mut self, command: RenderCommand, added_text_bytes: usize) -> SjmlResult<()> {
        if self.commands.len() + self.world_commands.len() >= MAX_RENDER_COMMANDS {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "HUD render command limit reached",
            ));
        }
        let total_text = self
            .text_bytes
            .checked_add(added_text_bytes)
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::ArithmeticOverflow,
                    "HUD text byte count overflowed",
                )
            })?;
        if total_text > MAX_RENDER_TEXT_BATCH_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "HUD batch text budget exceeded",
            ));
        }
        self.commands.push(command);
        self.text_bytes = total_text;
        match self.encode_batch() {
            Ok(batch) if batch.len() <= MAX_RENDER_BATCH_BYTES => {}
            Ok(_) => {
                self.commands.pop();
                self.text_bytes -= added_text_bytes;
                return Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "HUD render batch exceeds 1 MiB",
                ));
            }
            Err(error) => {
                self.commands.pop();
                self.text_bytes -= added_text_bytes;
                return Err(error);
            }
        }
        Ok(())
    }

    fn encode_batch(&self) -> SjmlResult<Vec<u8>> {
        if self.layer == RenderLayer::World {
            return self.encode_world_batch();
        }
        let mut bytes = Vec::new();
        put_u32(&mut bytes, RENDER_SCHEMA_VERSION);
        put_u32(&mut bytes, self.commands.len() as u32);
        for command in &self.commands {
            match command {
                RenderCommand::Text {
                    text,
                    x,
                    y,
                    argb,
                    shadow,
                } => {
                    bytes.push(0);
                    put_string(&mut bytes, text)?;
                    put_f32(&mut bytes, *x)?;
                    put_f32(&mut bytes, *y)?;
                    put_u32(&mut bytes, *argb);
                    bytes.push(u8::from(*shadow));
                }
                RenderCommand::TextWithFont {
                    text,
                    font_kind,
                    font_path,
                    x,
                    y,
                    argb,
                    shadow,
                } => {
                    bytes.push(5);
                    put_string(&mut bytes, text)?;
                    bytes.push(*font_kind);
                    if let Some(path) = font_path {
                        put_string(&mut bytes, path)?;
                    }
                    put_f32(&mut bytes, *x)?;
                    put_f32(&mut bytes, *y)?;
                    put_u32(&mut bytes, *argb);
                    bytes.push(u8::from(*shadow));
                }
                RenderCommand::Sprite {
                    path,
                    x,
                    y,
                    width,
                    height,
                    source_width,
                    source_height,
                } => {
                    bytes.push(6);
                    put_string(&mut bytes, path)?;
                    for value in [x, y, width, height] {
                        put_f32(&mut bytes, *value)?;
                    }
                    put_u32(&mut bytes, *source_width);
                    put_u32(&mut bytes, *source_height);
                }
                RenderCommand::Rect {
                    x,
                    y,
                    width,
                    height,
                    argb,
                } => {
                    bytes.push(1);
                    for value in [x, y, width, height] {
                        put_f32(&mut bytes, *value)?;
                    }
                    put_u32(&mut bytes, *argb);
                }
                RenderCommand::Progress {
                    x,
                    y,
                    width,
                    height,
                    fraction,
                    background_argb,
                    foreground_argb,
                } => {
                    bytes.push(2);
                    for value in [x, y, width, height, fraction] {
                        put_f32(&mut bytes, *value)?;
                    }
                    put_u32(&mut bytes, *background_argb);
                    put_u32(&mut bytes, *foreground_argb);
                }
                RenderCommand::Texture {
                    handle,
                    x,
                    y,
                    width,
                    height,
                    opacity,
                } => {
                    bytes.push(3);
                    put_u64(&mut bytes, handle.owner_token());
                    put_u64(&mut bytes, handle.context_token());
                    put_u64(&mut bytes, handle.slot_token());
                    for value in [x, y, width, height, opacity] {
                        put_f32(&mut bytes, *value)?;
                    }
                }
                RenderCommand::Tooltip { text, x, y } => {
                    bytes.push(4);
                    put_string(&mut bytes, text)?;
                    put_f32(&mut bytes, *x)?;
                    put_f32(&mut bytes, *y)?;
                }
            }
            if bytes.len() > MAX_RENDER_BATCH_BYTES {
                return Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "HUD render batch exceeds 1 MiB",
                ));
            }
        }
        Ok(bytes)
    }

    fn encode_world_batch(&self) -> SjmlResult<Vec<u8>> {
        let mut bytes = Vec::new();
        put_u32(&mut bytes, RENDER_SCHEMA_VERSION);
        put_u32(&mut bytes, self.world_commands.len() as u32);
        for command in &self.world_commands {
            match command {
                WorldRenderCommand::Line {
                    from_x,
                    from_y,
                    from_z,
                    to_x,
                    to_y,
                    to_z,
                    argb,
                } => {
                    bytes.push(0);
                    for value in [from_x, from_y, from_z, to_x, to_y, to_z] {
                        put_f32(&mut bytes, *value)?;
                    }
                    put_u32(&mut bytes, *argb);
                }
                WorldRenderCommand::Box {
                    x,
                    y,
                    z,
                    width,
                    height,
                    depth,
                    argb,
                } => {
                    bytes.push(1);
                    for value in [x, y, z, width, height, depth] {
                        put_f32(&mut bytes, *value)?;
                    }
                    put_u32(&mut bytes, *argb);
                }
                WorldRenderCommand::Quad { vertices, argb } => {
                    bytes.push(2);
                    for vertex in vertices {
                        for coordinate in vertex {
                            put_f32(&mut bytes, *coordinate)?;
                        }
                    }
                    put_u32(&mut bytes, *argb);
                }
            }
            if bytes.len() > MAX_RENDER_BATCH_BYTES {
                return Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "world render batch exceeds 1 MiB",
                ));
            }
        }
        Ok(bytes)
    }

    fn push_world(&mut self, command: WorldRenderCommand) -> SjmlResult<()> {
        if self.commands.len() + self.world_commands.len() >= MAX_RENDER_COMMANDS {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "world render command limit reached",
            ));
        }
        if matches!(&command, WorldRenderCommand::Quad { .. })
            && self
                .world_commands
                .iter()
                .filter(|existing| matches!(existing, WorldRenderCommand::Quad { .. }))
                .count()
                >= MAX_WORLD_MODEL_QUADS
        {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "world render quad limit reached",
            ));
        }
        self.world_commands.push(command);
        if self.encode_world_batch()?.len() > MAX_RENDER_BATCH_BYTES {
            self.world_commands.pop();
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "world render batch exceeds 1 MiB",
            ));
        }
        Ok(())
    }

    fn require_gui_layer(&self) -> SjmlResult<()> {
        if self.layer == RenderLayer::World {
            return Err(invalid(
                "GUI commands are unavailable in the world render layer",
            ));
        }
        Ok(())
    }

    fn require_layer(&self, expected: RenderLayer) -> SjmlResult<()> {
        if self.layer != expected {
            return Err(invalid(
                "world commands are available only in the world render layer",
            ));
        }
        Ok(())
    }
}

enum RenderCommand {
    Text {
        text: String,
        x: f32,
        y: f32,
        argb: u32,
        shadow: bool,
    },
    TextWithFont {
        text: String,
        font_kind: u8,
        font_path: Option<String>,
        x: f32,
        y: f32,
        argb: u32,
        shadow: bool,
    },
    Sprite {
        path: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        source_width: u32,
        source_height: u32,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        argb: u32,
    },
    Progress {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        fraction: f32,
        background_argb: u32,
        foreground_argb: u32,
    },
    Texture {
        handle: crate::RenderResourceHandle,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        opacity: f32,
    },
    Tooltip {
        text: String,
        x: f32,
        y: f32,
    },
}

enum WorldRenderCommand {
    Line {
        from_x: f32,
        from_y: f32,
        from_z: f32,
        to_x: f32,
        to_y: f32,
        to_z: f32,
        argb: u32,
    },
    Box {
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        depth: f32,
        argb: u32,
    },
    Quad {
        vertices: [[f32; 3]; 4],
        argb: u32,
    },
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CALLBACKS: std::cell::RefCell<Callbacks> = std::cell::RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
type Callback =
    std::rc::Rc<std::cell::RefCell<Box<dyn FnMut(&mut RenderContext) -> SjmlResult<()>>>>;

#[cfg(target_arch = "wasm32")]
struct CallbackEntry {
    layer: RenderLayer,
    callback: Callback,
    handle: Option<RenderSubscriptionHandle>,
}

#[cfg(target_arch = "wasm32")]
struct Callbacks {
    next: Option<u64>,
    entries: std::collections::BTreeMap<u64, CallbackEntry>,
}

#[cfg(target_arch = "wasm32")]
impl Default for Callbacks {
    fn default() -> Self {
        Self {
            next: Some(1),
            entries: std::collections::BTreeMap::new(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Callbacks {
    fn insert<F>(&mut self, layer: RenderLayer, callback: F) -> SjmlResult<u64>
    where
        F: FnMut(&mut RenderContext) -> SjmlResult<()> + 'static,
    {
        if self.entries.len() >= MAX_RENDER_SUBSCRIPTIONS {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "render subscription limit reached",
            ));
        }
        let id = self.next.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "render callback identity space exhausted",
            )
        })?;
        self.next = id.checked_add(1);
        self.entries.insert(
            id,
            CallbackEntry {
                layer,
                callback: std::rc::Rc::new(std::cell::RefCell::new(Box::new(callback))),
                #[cfg(target_arch = "wasm32")]
                handle: None,
            },
        );
        Ok(id)
    }

    fn get(&self, id: u64) -> SjmlResult<(RenderLayer, Callback)> {
        self.entries
            .get(&id)
            .map(|entry| (entry.layer, entry.callback.clone()))
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "render callback is no longer subscribed",
                )
            })
    }

    fn remove(&mut self, id: u64) {
        self.entries.remove(&id);
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_render_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(
        layer_pointer: i32,
        layer_length: i32,
        callback_id: i64,
        handle_output_pointer: i32,
    ) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
    #[link_name = "submit"]
    fn imported_submit(batch_pointer: i32, batch_length: i32) -> i32;
}

/// Subscribes a closure to the requested render layer.
pub fn subscribe<F>(layer: RenderLayer, callback: F) -> SjmlResult<RenderSubscriptionHandle>
where
    F: FnMut(&mut RenderContext) -> SjmlResult<()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        let id = CALLBACKS.with(|registry| registry.borrow_mut().insert(layer, callback))?;
        let name = layer.wire_name().as_bytes();
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().remove(id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate render subscription output",
            ));
        }
        // SAFETY: initialize the full live handle output allocation before the host writes it.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: the layer slice and exact 24-byte output allocation remain live during the import.
        let status = unsafe {
            imported_subscribe(
                name.as_ptr() as i32,
                name.len() as i32,
                id as i64,
                output as i32,
            )
        };
        let result = if status == 0 {
            // SAFETY: the host call initialized the complete output span.
            crate::from_bytes::<RenderSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected render subscription",
            ))
        };
        // SAFETY: release the exact allocation returned above after the synchronous host call.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CALLBACKS.with(|registry| {
                    if let Some(entry) = registry.borrow_mut().entries.get_mut(&id) {
                        entry.handle = Some(handle);
                    }
                });
                Ok(handle)
            }
            Err(error) => {
                CALLBACKS.with(|registry| registry.borrow_mut().remove(id));
                Err(error)
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (layer, callback);
        Err(unsupported())
    }
}

/// Removes one live render subscription from the host and guest callback registry.
pub fn unsubscribe(handle: RenderSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let id = CALLBACKS
            .with(|registry| {
                registry
                    .borrow()
                    .entries
                    .iter()
                    .find_map(|(&id, entry)| (entry.handle == Some(handle)).then_some(id))
            })
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "render subscription is not active in this module",
                )
            })?;
        let bytes = to_bytes(&handle)?;
        // SAFETY: stable handle bytes remain live for the complete synchronous host call.
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected render unsubscription",
            ));
        }
        CALLBACKS.with(|registry| registry.borrow_mut().remove(id));
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

#[cfg(target_arch = "wasm32")]
struct RenderDelivery {
    callback_id: u64,
    layer: RenderLayer,
    frame: RenderFrame,
    world_frame: Option<WorldRenderFrame>,
}

#[cfg(target_arch = "wasm32")]
impl StableDecode for RenderDelivery {
    fn decode(decoder: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != RENDER_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported render dispatch schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        if callback_id == 0 {
            return Err(invalid("render callback identity is zero"));
        }
        if decoder.read_u32()? != RENDER_SCHEMA_VERSION {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported render frame schema",
            ));
        }
        let layer = decode_render_layer(decoder.read_u8()?)?;
        let width = decoder.read_u32()?;
        let height = decoder.read_u32()?;
        let mouse_x = decoder.read_f32()?;
        let mouse_y = decoder.read_f32()?;
        let partial_tick = decoder.read_f32()?;
        let frame = RenderFrame {
            width,
            height,
            mouse_x,
            mouse_y,
            partial_tick,
        }
        .validate()?;
        let world_frame = if layer == RenderLayer::World {
            Some(
                WorldRenderFrame {
                    camera_x: decoder.read_f64()?,
                    camera_y: decoder.read_f64()?,
                    camera_z: decoder.read_f64()?,
                }
                .validate()?,
            )
        } else {
            None
        };
        Ok(Self {
            callback_id,
            layer,
            frame,
            world_frame,
        })
    }
}

/// Dispatches one host render callback, then submits its complete HUD batch only on success.
///
/// # Safety
/// The host must pass a live initialized input allocation from `sjml_v1_alloc` and keep it alive
/// until return. It releases the input and response with the paired ABI dealloc exports.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_render_dispatch"]
pub unsafe extern "C" fn dispatch(pointer: u32, length: u32) -> u64 {
    // SAFETY: the host guarantees that the callback input is the exact live initialized span.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: RenderDelivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant render dispatch is forbidden",
                ));
            }
            struct DispatchGuard;
            impl Drop for DispatchGuard {
                fn drop(&mut self) {
                    DISPATCHING.with(|active| active.set(false));
                }
            }
            let _guard = DispatchGuard;
            let (registered_layer, callback) =
                CALLBACKS.with(|registry| registry.borrow().get(delivery.callback_id))?;
            if registered_layer != delivery.layer {
                return Err(invalid(
                    "render callback layer does not match its subscription",
                ));
            }
            let mut context =
                RenderContext::new(delivery.frame, delivery.layer, delivery.world_frame);
            callback.borrow_mut()(&mut context)?;
            let batch = context.encode_batch()?;
            // SAFETY: `batch` remains live and immutable for the synchronous host import.
            let status = imported_submit(
                batch.as_ptr() as usize as u32 as i32,
                batch.len() as u32 as i32,
            );
            if status == 0 {
                Ok(())
            } else {
                Err(SjmlError::from_abi_code(
                    status as u32,
                    "host rejected HUD render batch",
                ))
            }
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "render subscriptions require wasm32-unknown-unknown and SJML render imports",
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_render_layer(value: u8) -> SjmlResult<RenderLayer> {
    match value {
        0 => Ok(RenderLayer::Hud),
        1 => Ok(RenderLayer::Screen),
        2 => Ok(RenderLayer::World),
        _ => Err(invalid("unknown render callback layer")),
    }
}

fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

fn validate_position(x: f32, y: f32) -> SjmlResult<()> {
    if !x.is_finite() || !y.is_finite() {
        return Err(invalid("render coordinates must be finite"));
    }
    Ok(())
}

fn validate_dimensions(x: f32, y: f32, width: f32, height: f32) -> SjmlResult<()> {
    validate_position(x, y)?;
    if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
        return Err(invalid("render dimensions must be finite and non-negative"));
    }
    Ok(())
}

fn validate_world_coordinate(value: f32) -> SjmlResult<()> {
    const MAX_WORLD_RELATIVE_COORDINATE: f32 = 4096.0;
    if !value.is_finite() || value.abs() > MAX_WORLD_RELATIVE_COORDINATE {
        return Err(invalid(
            "world coordinates must be finite and within +/-4096 camera-relative units",
        ));
    }
    Ok(())
}

fn validate_world_position(value: f64) -> SjmlResult<()> {
    const MAX_WORLD_POSITION: f64 = 30_000_000.0;
    if !value.is_finite() || value.abs() > MAX_WORLD_POSITION {
        return Err(invalid(
            "world model positions must be finite and within +/-30,000,000 world units",
        ));
    }
    Ok(())
}

fn validate_world_extent(value: f32) -> SjmlResult<()> {
    if !value.is_finite() || !(0.0..=4096.0).contains(&value) {
        return Err(invalid("world extents must be finite and within [0, 4096]"));
    }
    Ok(())
}

fn validate_string(value: &str, maximum: usize, label: &'static str) -> SjmlResult<()> {
    if value.len() > maximum {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            format!("{label} exceeds its UTF-8 byte limit"),
        ));
    }
    Ok(())
}

fn validate_resource_path(value: &str, maximum: usize, label: &'static str) -> SjmlResult<()> {
    if value.is_empty()
        || value.len() > maximum
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'/' | b'_' | b'-' | b'.')
        })
        || value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(invalid(label));
    }
    Ok(())
}

fn validate_gui_sprite_path(value: &str) -> SjmlResult<()> {
    validate_resource_path(value, MAX_RENDER_RESOURCE_PATH_BYTES, "GUI sprite path")?;
    if !value.ends_with(".png") {
        return Err(invalid("GUI sprite path must name a PNG file"));
    }
    Ok(())
}

fn validate_font_path(value: &str) -> SjmlResult<()> {
    validate_resource_path(value, MAX_RENDER_RESOURCE_PATH_BYTES, "package font path")?;
    if value.ends_with(".json") {
        return Err(invalid("package font path must omit the .json suffix"));
    }
    Ok(())
}

fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn put_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}
fn put_f32(output: &mut Vec<u8>, value: f32) -> SjmlResult<()> {
    if !value.is_finite() {
        return Err(invalid("render values must be finite"));
    }
    put_u32(output, value.to_bits());
    Ok(())
}
fn put_string(output: &mut Vec<u8>, value: &str) -> SjmlResult<()> {
    validate_string(value, MAX_RENDER_TEXT_BYTES, "render string")?;
    put_u32(output, value.len() as u32);
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> RenderFrame {
        RenderFrame {
            width: 800,
            height: 600,
            mouse_x: 20.0,
            mouse_y: 30.0,
            partial_tick: 0.5,
        }
    }

    #[test]
    fn render_layers_have_stable_wire_values_and_reject_unknown_values() {
        assert_eq!(RenderLayer::Hud.wire_value(), 0);
        assert_eq!(RenderLayer::Hud.wire_name(), "hud");
        assert_eq!(RenderLayer::Screen.wire_value(), 1);
        assert_eq!(RenderLayer::Screen.wire_name(), "screen");
        assert_eq!(RenderLayer::World.wire_value(), 2);
        assert_eq!(RenderLayer::World.wire_name(), "world");
        assert_eq!(decode_render_layer(0).unwrap(), RenderLayer::Hud);
        assert_eq!(decode_render_layer(1).unwrap(), RenderLayer::Screen);
        assert_eq!(decode_render_layer(2).unwrap(), RenderLayer::World);
        assert_eq!(
            decode_render_layer(3).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn hud_commands_encode_one_versioned_little_endian_batch_in_order() {
        let mut context = RenderContext::new(frame(), RenderLayer::Hud, None);
        context
            .draw_text("ok", 1.0, 2.0, 0xff00_ffff, true)
            .unwrap();
        context.draw_rect(3.0, 4.0, 5.0, 6.0, 0x1122_3344).unwrap();
        context
            .draw_progress(1.0, 2.0, 3.0, 4.0, 0.25, 5, 6)
            .unwrap();
        context.draw_tooltip("tip", 7.0, 8.0).unwrap();
        let bytes = context.encode_batch().unwrap();
        assert_eq!(&bytes[..8], &[1, 0, 0, 0, 4, 0, 0, 0]);
        assert_eq!(bytes[8], 0);
        assert_eq!(bytes[28], 1);
        assert_eq!(bytes[49], 2);
        assert_eq!(bytes[78], 4);
    }

    #[test]
    fn package_fonts_and_gui_sprites_encode_as_bounded_package_owned_commands() {
        let mut context = RenderContext::new(frame(), RenderLayer::Hud, None);
        let font = RenderFont::package("fonts/pixel").unwrap();
        context
            .draw_text_with_font("SJML", &font, 2.0, 3.0, 0xffff_ffff, true)
            .unwrap();
        context
            .draw_sprite("hud/status.png", 8.0, 9.0, 32.0, 16.0, (64, 32))
            .unwrap();

        let bytes = context.encode_batch().unwrap();
        assert_eq!(bytes[8], 5);
        assert_eq!(bytes[17], 3); // package-font selector follows the text string
        let sprite_tag = 8 + 1 + 4 + 4 + 1 + 4 + "fonts/pixel".len() + 8 + 4 + 1;
        assert_eq!(bytes[sprite_tag], 6);
        assert!(bytes
            .windows("hud/status.png".len())
            .any(|window| window == "hud/status.png".as_bytes()));

        assert!(RenderFont::package("../foreign").is_err());
        assert!(RenderFont::package("fonts/pixel.json").is_err());
        assert!(context
            .draw_sprite("../outside.png", 0.0, 0.0, 8.0, 8.0, (16, 16))
            .is_err());
        assert!(context
            .draw_sprite("hud/icon.png", 0.0, 0.0, 8.0, 8.0, (4097, 16))
            .is_err());
    }

    #[test]
    fn render_builder_rejects_nonfinite_negative_and_over_limit_values() {
        let mut context = RenderContext::new(frame(), RenderLayer::Hud, None);
        assert_eq!(
            context
                .draw_rect(0.0, 0.0, -1.0, 1.0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            context
                .draw_text("x", f32::NAN, 0.0, 0, false)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            context
                .draw_progress(0.0, 0.0, 1.0, 1.0, 2.0, 0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            context
                .draw_tooltip(&"x".repeat(MAX_TOOLTIP_BYTES + 1), 0.0, 0.0)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        for _ in 0..MAX_RENDER_COMMANDS {
            context.draw_rect(0.0, 0.0, 0.0, 0.0, 0).unwrap();
        }
        assert_eq!(
            context.draw_rect(0.0, 0.0, 0.0, 0.0, 0).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn frame_rejects_nonfinite_values_and_partial_tick_outside_unit_interval() {
        assert_eq!(
            RenderFrame {
                partial_tick: f32::NAN,
                ..frame()
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            RenderFrame {
                partial_tick: -0.1,
                ..frame()
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn world_commands_encode_camera_relative_lines_and_boxes_in_a_bounded_batch() {
        let mut context = RenderContext::new(
            RenderFrame {
                width: 0,
                height: 0,
                mouse_x: -1.0,
                mouse_y: -1.0,
                partial_tick: 0.25,
            },
            RenderLayer::World,
            Some(WorldRenderFrame {
                camera_x: 12.5,
                camera_y: 64.0,
                camera_z: -8.25,
            }),
        );
        context
            .draw_world_line(0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0xff12_3456)
            .unwrap();
        context
            .draw_world_box(-1.0, 0.0, 2.0, 1.0, 2.0, 3.0, 0x8045_6789)
            .unwrap();

        let bytes = context.encode_batch().unwrap();

        assert_eq!(&bytes[..8], &[1, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(bytes[8], 0);
        assert_eq!(bytes[37], 1);
        assert_eq!(bytes.len(), 66);
    }

    #[test]
    fn world_model_builder_encodes_local_and_camera_relative_quad_faces() {
        let local_face = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let world_space_face = [
            [99.0, 64.0, -99.0],
            [100.0, 64.0, -99.0],
            [100.0, 65.0, -99.0],
            [99.0, 65.0, -99.0],
        ];
        let mut model = WorldModelBuilder::new([102.0, 66.0, -100.0]).unwrap();
        model
            .add_quad(local_face, 0xff12_3456)
            .unwrap()
            .add_world_quad(world_space_face, 0xff65_4321)
            .unwrap();
        assert_eq!(model.quad_count(), 2);
        assert_eq!(model.vertex_count(), 8);

        let mut context = RenderContext::new(
            RenderFrame {
                width: 0,
                height: 0,
                mouse_x: -1.0,
                mouse_y: -1.0,
                partial_tick: 0.25,
            },
            RenderLayer::World,
            Some(WorldRenderFrame {
                camera_x: 100.0,
                camera_y: 64.0,
                camera_z: -100.0,
            }),
        );
        model.draw(&mut context).unwrap();
        let bytes = context.encode_world_batch().unwrap();

        assert_eq!(&bytes[..8], &[1, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(bytes[8], 2);
        assert_eq!(f32::from_le_bytes(bytes[9..13].try_into().unwrap()), 2.0);
        assert_eq!(f32::from_le_bytes(bytes[13..17].try_into().unwrap()), 2.0);
        assert_eq!(f32::from_le_bytes(bytes[17..21].try_into().unwrap()), 0.0);
        assert_eq!(&bytes[57..61], &0xff12_3456_u32.to_le_bytes());
        assert_eq!(bytes[61], 2);
        assert_eq!(f32::from_le_bytes(bytes[62..66].try_into().unwrap()), -1.0);
        assert_eq!(bytes.len(), 8 + 2 * (1 + 12 * 4 + 4));
    }

    #[test]
    fn world_model_builder_and_callback_enforce_geometry_bounds() {
        let mut model = WorldModelBuilder::new([0.0; 3]).unwrap();
        let zero_face = [[0.0_f64; 3]; 4];
        let zero_face_camera_relative = [[0.0_f32; 3]; 4];
        for _ in 0..MAX_WORLD_MODEL_QUADS {
            model.add_world_quad(zero_face, 0).unwrap();
        }
        assert_eq!(model.quad_count(), MAX_WORLD_MODEL_QUADS);
        assert_eq!(model.vertex_count(), MAX_WORLD_MODEL_VERTICES);
        assert_eq!(
            model.add_world_quad(zero_face, 0).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );

        assert_eq!(
            WorldModelBuilder::new([f64::INFINITY, 0.0, 0.0])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            WorldModelBuilder::new([30_000_000.0, 0.0, 0.0])
                .unwrap()
                .add_quad([[1.0, 0.0, 0.0]; 4], 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            WorldModelBuilder::new([0.0; 3])
                .unwrap()
                .add_world_quad([[f64::NAN, 0.0, 0.0]; 4], 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut context = RenderContext::new(
            frame(),
            RenderLayer::World,
            Some(WorldRenderFrame {
                camera_x: 0.0,
                camera_y: 64.0,
                camera_z: 0.0,
            }),
        );
        for _ in 0..MAX_WORLD_MODEL_QUADS {
            context
                .draw_world_quad(zero_face_camera_relative, 0)
                .unwrap();
        }
        assert_eq!(
            context
                .draw_world_quad(zero_face_camera_relative, 0)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn world_commands_enforce_layer_and_coordinate_bounds() {
        let mut gui = RenderContext::new(frame(), RenderLayer::Hud, None);
        assert_eq!(
            gui.draw_world_line(0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );

        let mut world = RenderContext::new(
            frame(),
            RenderLayer::World,
            Some(WorldRenderFrame {
                camera_x: 0.0,
                camera_y: 64.0,
                camera_z: 0.0,
            }),
        );
        assert_eq!(
            world
                .draw_world_line(4097.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            world
                .draw_world_box(0.0, 0.0, 0.0, f32::NAN, 1.0, 1.0, 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            world
                .draw_text("GUI", 0.0, 0.0, 0, false)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn world_camera_frame_rejects_nonfinite_or_unbounded_coordinates() {
        assert_eq!(
            WorldRenderFrame {
                camera_x: f64::INFINITY,
                camera_y: 0.0,
                camera_z: 0.0,
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            WorldRenderFrame {
                camera_x: 30_000_001.0,
                camera_y: 0.0,
                camera_z: 0.0,
            }
            .validate()
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }
}
