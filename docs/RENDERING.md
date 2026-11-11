# Client rendering

SJML exposes client-only HUD, active-screen, and high-level world callbacks in
`render`, plus a bounded OpenGL command bridge in `gl`. GUI callbacks run during
the HUD pass or over the active Minecraft screen. World callbacks can draw
bounded camera-relative lines, outlined boxes, and untextured colored quad faces
after entities render. Guest-declared entities and block entities can also use
bounded colored, textured, or animated cuboid models rendered through
Minecraft's buffered vertex pipelines. Arbitrary vertex formats and
unrestricted OpenGL are not exposed by this high-level model API.

## Entity and block-entity models

Declare the entity or block entity from `#[mod_register]`, then attach one
static or animated model to that same declaration. Models use ARGB cuboids and do not
require `client.render.opengl`. For textured cuboids, call `set_texture` with a
package-owned PNG resource such as `example:textures/entity/clockwork_mite.png`
and include that file at `assets/example/textures/entity/clockwork_mite.png` in
the `.sjml` archive. Minecraft's entity cutout layer maps the full texture to
each cuboid face; the cuboid's ARGB value tints it. Without a texture, the
renderer uses per-vertex colors.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_models() -> SjmlResult<()> {
    registry::register_entity("clockwork_mite", EntityDefinition::default())?;
    let mut entity = EntityModelDefinition::new();
    entity.add_cuboid([-0.25, 0.0, -0.25], [0.25, 0.5, 0.25], 0xFFFF_8040)?;
    entity.add_cuboid([-0.2, 0.5, -0.2], [0.2, 0.9, 0.2], 0xFF80_C0FF)?;
    entity.set_texture("example:textures/entity/clockwork_mite.png")?;
    entity.set_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Y, [0.0, 0.25, 0.0], 30.0, 40, 0,
    )?)?;
    entity.set_cuboid_animation(1, ModelAnimation::sine_rotation(
        ModelAnimationAxis::X, [0.0, 0.5, 0.0], 20.0, 32, 8,
    )?)?;
    registry::register_entity_model("clockwork_mite", entity)?;

    registry::register_block("clockwork_core", BlockDefinition::default())?;
    registry::register_block_entity("clockwork_core", "clockwork_core")?;
    let mut block = BlockEntityModelDefinition::new();
    block.add_cuboid([0.125, 0.125, 0.125], [0.875, 0.875, 0.875], 0xFF40_C0FF)?;
    block.set_texture("example:textures/block/clockwork_core.png")?;
    registry::register_block_entity_model("clockwork_core", block)?;
    Ok(())
}
```

Each model contains 1–32 cuboids; module and host-wide totals are bounded.
Entity model coordinates stay within +/-4 local world units. Block-entity
coordinates stay inside the owning block's unit cube. A model may only target
a type already declared by the same module and may be attached once.
`set_animation` accepts a deterministic host-side sine rotation around a
model-local pivot, along X, Y, or Z. The pivot must remain in the entity's +/-4
local coordinate bounds or the block entity's `[0,1]` bounds; amplitude is
limited to +/-180 degrees, period to 1–12,000 game ticks, and phase to less
than the period. The host evaluates it using game time and partial tick, so
rendering never invokes guest code each frame. `set_cuboid_animation(index, …)`
uses the same bounds and timing limits but rotates just the cuboid at that
zero-based insertion index. Its pivot is in the model's local coordinates, so a
head, arm, or lid can rotate around a joint while other cuboids remain still.
When both animation methods are used, the model-wide rotation is the parent
transform and each cuboid rotation is applied beneath it. Models remain limited
to 32 cuboids, so they can carry at most 32 child animation tracks. The entity
renderer retains its diagnostic wireframe for types without a model.

## Permissions and availability

Each capability is bound only when the module requests it in `sjml.toml` and
receives a matching administrator grant in `config/sjml-permissions.toml`:

In the package manifest:

```toml
[[modules]]
id = "core"
side = "client"
permissions = ["client.render", "client.render.opengl", "client.input"]
```

In the server permissions file:

```toml
[[grant]]
module = "example/core"
capabilities = ["client.render", "client.render.opengl", "client.input"]
```

`client.render` is required for HUD, screen, and world subscription,
unsubscription, and batch submission. `client.render.opengl` is required for GL
resource operations, capability queries, GL command submission, and GUI texture
creation/deletion. Creating a GUI texture and drawing it therefore needs both
permissions. Active-screen click/key callbacks require the separate
`client.input` capability. The host checks the effective grant on each import
and callback.

These APIs are available to the initialized client-side module runtime. Static
registration (`mod_register`) binds authorized signatures but denies calls and
cannot subscribe or access a graphics context. Once imports are bound, the host
checks the current grant on every operation and rejects calls after permission
revocation. Dedicated servers do not load the client rendering implementation.

The rendering import module is `sjml_render_v1`:

| Import | WASM signature | Purpose |
| --- | --- | --- |
| `subscribe` | `(i32 layer_ptr, i32 layer_len, i64 callback_id, i32 handle_out_ptr) -> i32` | Register a closure for a layer; returns a 24-byte event-subscription handle through guest memory |
| `unsubscribe` | `(i32 handle_ptr, i32 handle_len) -> i32` | Revoke one subscription owned by this module runtime |
| `submit` | `(i32 batch_ptr, i32 batch_len) -> i32` | Submit one batch during its active callback |

The host invokes the guest export `sjml_v1_render_dispatch` with one
versioned little-endian payload: render schema `u32`, callback ID `i64`, frame
schema `u32`, layer `u8` (`0` HUD, `1` active screen, `2` world), width and
height `u32`, then mouse X, mouse Y, and partial tick as `f32`. HUD and screen
deliveries are 37 bytes. World deliveries append camera X, Y, and Z as three
`f64` values for 61 bytes total. Unknown schemas/layers, malformed lengths,
out-of-range camera coordinates, and invalid frame floats are rejected.

## GUI callback API

Subscribe at runtime with a closure. `RenderLayer::Hud` runs from NeoForge's
client HUD render event. `RenderLayer::Screen` runs after the active Minecraft
screen renders and receives that event's live scaled pointer coordinates. The
closure receives logical GUI dimensions, mouse coordinates, and the partial
tick through `RenderContext::frame()`:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    let _hud = render::subscribe(RenderLayer::Hud, |ctx| {
        let frame = ctx.frame();
        ctx.draw_text("Status", 8.0, 8.0, 0xFFFF_FFFF, true)?;
        ctx.draw_progress(
            8.0,
            24.0,
            120.0,
            8.0,
            0.75,
            0xFF303030,
            0xFF40C060,
        )?;
        let _logical_size = (frame.width, frame.height);
        Ok(())
    })?;
    Ok(())
}
```

Screen callbacks use the same bounded draw commands and are delivered only while
a screen is open:

```rust
#[mod_init]
fn initialize_screen_overlay() -> SjmlResult<()> {
    let _screen = render::subscribe(RenderLayer::Screen, |ctx| {
        let frame = ctx.frame();
        ctx.draw_rect(frame.mouse_x, frame.mouse_y, 120.0, 20.0, 0xCC101820)?;
        ctx.draw_text("Screen overlay", frame.mouse_x + 6.0, frame.mouse_y + 4.0,
            0xFFFF_FFFF, true)?;
        Ok(())
    })?;
    Ok(())
}
```

## Package-owned GUI fonts and sprites

HUD and screen callbacks can draw images and select fonts from their own package
assets without creating OpenGL resources. `draw_sprite` takes a path relative to
`assets/<package-id>/textures/gui/`, the destination size in logical GUI units,
and the source PNG's pixel width and height (each from 1 to 4096). Its source
dimensions must match the PNG. Paths are limited to 256 lowercase ASCII bytes,
must end in `.png`, and cannot contain traversal segments. For example, include
`assets/example/textures/gui/status.png` and draw it like this:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize_gui_assets() -> SjmlResult<()> {
    let pixel_font = RenderFont::package("ui/pixel")?;
    let _hud = render::subscribe(RenderLayer::Hud, move |ctx| {
        ctx.draw_sprite("status.png", 8.0, 8.0, 96.0, 48.0, (192, 96))?;
        ctx.draw_text_with_font("Ready", &pixel_font, 112.0, 8.0,
            0xFFFF_FFFF, true)?;
        Ok(())
    })?;
    Ok(())
}
```

`draw_text_with_font` accepts `RenderFont::Default`, `RenderFont::Uniform`,
`RenderFont::Alt`, or `RenderFont::package("ui/pixel")`. The package form resolves
only under that package's `assets/<package-id>/font/` directory, so the example
font path is `assets/example/font/ui/pixel.json`. Minecraft parses that JSON
with its normal font provider rules; SJML does not expose arbitrary font-manager
objects or allow a module to select another package's font. Font paths omit the
`.json` suffix and have the same bounded lowercase path rules. Sprite and font
resource strings count toward the existing per-frame render string budget.
These commands require `client.render`; package assets are read through the
generated package resource pack and do not require `client.render.opengl`.

Active-screen click, key, and typed-character callbacks use the independent `screen_input` API.
Declare and grant `client.input` to receive them; `client.render` is not needed
unless the module also draws. They run on the client main thread from NeoForge's
pre-input events. Returning `true` consumes that event in Minecraft and stops
delivery to later modules; returning `false` leaves it available to vanilla and
later modules. All callbacks within one module run for the event. Mouse
coordinates are scaled GUI units bounded to ±1,048,576, button numbers are
0–31, and mouse events report current Shift, Control, and Alt modifiers. Key
codes use the range -1–65,535, scan codes 0–65,535, all six defined GLFW
modifier bits are accepted, and the repeat flag distinguishes held keys.
Typed-character callbacks receive one UTF-16 code unit; supplementary Unicode
characters arrive as surrogate pairs. `TextField` joins those pairs into
Unicode scalars, rejects control characters, and stores at most 4,096 UTF-8
bytes. A module may own at most 256 input subscriptions.

```rust
#[mod_init]
fn initialize_screen_input() -> SjmlResult<()> {
    let _input = screen_input::subscribe(|event| match event {
        ScreenInputEvent::MouseButtonPressed { button: 0, .. } => Ok(true),
        ScreenInputEvent::KeyPressed { key_code: 256, .. } => Ok(true),
        _ => Ok(false),
    })?;
    Ok(())
}
```

The SDK also includes a `Button`, single-line `TextField`, `Tooltip`, and
`ProgressBar`. Draw controls from a screen render callback and pass input events
to interactive controls' handlers. Use shared guest state when the render and
input closures need the same field value:

```rust,ignore
use sjml::prelude::*;
use std::{cell::RefCell, rc::Rc};

fn install_controls() -> SjmlResult<()> {
    let button = Rc::new(Button::new(
        WidgetRect::new(20.0, 20.0, 100.0, 20.0)?, "Apply"
    )?);
    let field = Rc::new(RefCell::new(
        TextField::new(WidgetRect::new(20.0, 48.0, 180.0, 20.0)?, 256)?
            .with_placeholder("Type a name")?
    ));

    let input_button = button.clone();
    let input_field = field.clone();
    let _input = screen_input::subscribe(move |event| {
        if input_button.handle_input(event) {
            return Ok(true);
        }
        Ok(input_field.borrow_mut().handle_input(event).consumes_input())
    })?;

    let draw_button = button.clone();
    let draw_field = field.clone();
    let _screen = render::subscribe(RenderLayer::Screen, move |ctx| {
        draw_button.draw(ctx)?;
        draw_field.borrow().draw(ctx)?;
        Ok(())
    })?;
    Ok(())
}
```

`Button::handle_input` activates on a left-button press inside its bounds.
`TextField::handle_input` supports insertion, Unicode-aware backspace/delete,
cursor movement, focus, and Enter submission. `TextFieldAction::consumes_input`
can be returned directly from the module's screen-input callback.

`Tooltip` displays its bounded text while the pointer is inside its hover
rectangle. It accepts at most 4 KiB of UTF-8 text. `ProgressBar` accepts a finite
fraction from `0.0` to `1.0`; its rectangle must have positive width and height.
Both draw with the existing bounded `RenderContext` commands:

```rust,ignore
use sjml::prelude::*;

fn install_status_widgets() -> SjmlResult<()> {
    let tooltip = Tooltip::new(
        WidgetRect::new(20.0, 20.0, 100.0, 20.0)?,
        "Additional status details",
    )?;
    let mut progress = ProgressBar::new(
        WidgetRect::new(20.0, 52.0, 160.0, 8.0)?,
        0.65,
    )?
    .colors(0xFF303030, 0xFF40C060);
    progress.set_fraction(0.75)?;

    let _screen = render::subscribe(RenderLayer::Screen, move |ctx| {
        progress.draw(ctx)?;
        let _tooltip_visible = tooltip.draw(ctx)?;
        Ok(())
    })?;
    Ok(())
}
```

`RenderLayer::World` runs at NeoForge's `AFTER_ENTITIES` stage. Its
`RenderContext::world_frame()` value reports the camera's absolute world
position. `draw_world_line` and `draw_world_box` take camera-relative logical
world coordinates; boxes are outlines. The adapter uses the event pose stack
and line buffer only during the callback:

```rust
#[mod_init]
fn initialize_world_overlay() -> SjmlResult<()> {
    let _world = render::subscribe(RenderLayer::World, |ctx| {
        let camera = ctx.world_frame().ok_or_else(|| {
            SjmlError::new(ErrorCode::InvalidArgument, "world camera is unavailable")
        })?;
        ctx.draw_world_line(-1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0xFFFF_4040)?;
        ctx.draw_world_box(-0.5, -0.5, -0.5, 1.0, 1.0, 1.0, 0xFF40_C0FF)?;

        let mut model = WorldModelBuilder::new([
            camera.camera_x,
            camera.camera_y,
            camera.camera_z,
        ])?;
        model.add_quad(
            [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]],
            0xC0_FF_8040,
        )?;
        model.add_world_quad(
            [
                [camera.camera_x - 1.0, camera.camera_y - 1.0, camera.camera_z + 2.0],
                [camera.camera_x + 1.0, camera.camera_y - 1.0, camera.camera_z + 2.0],
                [camera.camera_x + 1.0, camera.camera_y + 1.0, camera.camera_z + 2.0],
                [camera.camera_x - 1.0, camera.camera_y + 1.0, camera.camera_z + 2.0],
            ],
            0xFF40_80FF,
        )?;
        model.draw(ctx)?;
        Ok(())
    })?;
    Ok(())
}
```

The subscription closure is `FnMut(&mut RenderContext) -> SjmlResult<()>` and
must be `'static`. `render::subscribe` returns a module-scoped
`RenderSubscriptionHandle` (the existing event-subscription handle kind).
Retain the handle if the module needs to call `render::unsubscribe`; dropping a
copied handle value does not unsubscribe. The host revokes every subscription
when its module runtime closes. A module may register at most 1,024 callbacks
across all three layers.

`RenderContext::frame()` carries scaled GUI width and height and the event's
partial tick. World callbacks receive zero GUI dimensions and `-1.0` pointer
sentinels, with camera data in `world_frame()`. The Screen layer reports the
active screen event's mouse coordinates. The HUD layer uses `-1.0` sentinels
because NeoForge's HUD render event does not carry pointer coordinates. Partial
tick and all render floats must be finite, and partial tick is in `[0, 1]`.

The SDK builds one complete, layer-specific command batch per callback and
submits it only after the closure returns `Ok(())`. A failed closure sends no
partial batch; the host aborts the remaining callbacks for that module's layer
pass and passes the failure to the module owner. Each successful callback
submits exactly one batch, including an empty batch when it draws nothing.

GUI commands are emitted and drawn in order:

- `draw_text(text, x, y, argb, shadow)` draws text.
- `draw_rect(x, y, width, height, argb)` draws a filled rectangle.
- `draw_progress(...)` draws a background and a left-to-right fill.
- `draw_texture(handle, ...)` draws a module-owned GUI texture.
- `draw_tooltip(text, x, y)` draws a tooltip at logical GUI coordinates.
- `draw_world_line(from_x, from_y, from_z, to_x, to_y, to_z, argb)` draws a
  camera-relative world line.
- `draw_world_box(x, y, z, width, height, depth, argb)` draws a
  camera-relative outlined box.
- `WorldModelBuilder` adds bounded colored quad faces and transforms absolute
  world-space vertices to camera-relative coordinates during `draw`.

The stable batch uses schema version 1 and little-endian values. The SDK and host
bound each batch to 1,024 commands and 1 MiB, each text value to 16 KiB UTF-8,
each tooltip to 4 KiB, and combined text per batch to 64 KiB. GUI coordinates and
dimensions must be finite and within 1,048,576 logical units; world coordinates
and box extents must be finite and within 4,096 camera-relative units.
Dimensions cannot be negative. Progress fraction and texture
opacity must be in `[0, 1]`. The client adapter clips rectangles to the visible
GUI. It isolates drawing failures per command, while a malformed or over-limit
batch is rejected before drawing.

The world layer accepts at most 1,024 total commands and 256 quad faces
(1,024 vertices) per callback. `WorldModelBuilder::new` and `add_world_quad`
accept absolute world positions bounded to +/-30,000,000 units; `add_quad`
accepts local offsets bounded to +/-4,096 units and translates them by the
builder origin. During `draw`, each face is converted using the callback's live
camera position and rejected if any camera-relative vertex exceeds +/-4,096
units. Quad vertices are submitted in the supplied order as untextured
position-color geometry through Minecraft's `RenderType.debugQuads()` on the
world render thread.

World batches use the same 8-byte schema/count header. Tag `0` carries six
camera-relative `f32` line coordinates and one ARGB `u32`; tag `1` carries
camera-relative box origin and extents as six `f32` values and one ARGB `u32`;
tag `2` carries four ordered camera-relative XYZ `f32` vertices and one ARGB
`u32`. Unknown tags, non-finite coordinates, excessive quad counts, and
commands outside the camera-relative bounds are rejected before the client
adapter draws them.

The host dispatches callbacks on the client render thread and does not retain
Minecraft `GuiGraphics` objects after a render call. Subscription changes are
runtime operations; draw-batch submission is permitted only during the active
callback on the render thread. Closing or disabling the module removes its
callback route and invalidates its subscription handles.

## GUI textures and OpenGL handles

GUI textures and OpenGL objects use distinct opaque handle kinds:

| Handle kind | SDK type | Use |
| ---: | --- | --- |
| 11 `RENDER_RESOURCE` | `RenderResourceHandle` | RGBA GUI texture passed to `RenderContext::draw_texture` |
| 12 `GL_RESOURCE` | `GlResourceHandle` | Buffer, vertex array, GL texture, framebuffer, or shader program used by `gl` commands |

The handle types are not interchangeable. `gl::create_hud_texture(width,
height, rgba)` returns kind 11; `gl::create_resource(...)` returns kind 12.
`gl::delete_hud_texture` and `gl::delete_resource` retain that distinction.
Handles are owner- and runtime-scoped; they cannot be transferred to another
module. Resource creation reserves the handle immediately and queues the GPU
work for a render-context drain. A texture is drawable after its queued creation
has completed; attempting to draw it sooner returns a per-command failure.

The GL bridge accepts only typed, versioned descriptors and commands. It does
not expose raw JNI objects, arbitrary pointers, or unrestricted LWJGL calls.
Supported resources are:

- buffers with `Array` or `ElementArray` targets and `StaticDraw`, `DynamicDraw`,
  or `StreamDraw` usage;
- vertex arrays;
- 2D RGBA8 textures;
- framebuffers; and
- shader programs compiled from vertex and fragment source.

The command set covers resource/program binding, buffer data updates, vertex
attributes, 2D texture units, bounded texture subimage uploads, framebuffer
attachment and completeness checks, viewport and scissor,
blend/depth/cull state, scalar/vector/matrix uniforms, array and indexed draws,
clear, and resource deletion. The host validates the complete batch, resource
ownership and kinds, enum values, dimensions, and draw prerequisites before it
enqueues or executes effects. A draw requires a batch-owned vertex array and
program; indexed draws also require a batch-owned element buffer. Texture units
are limited to 0 through 31, and vertex attribute indices to the host-supported
range.

ABI v1 bounds a descriptor and GL batch to 1 MiB and each batch to 1,024
commands. RGBA texture dimensions are limited to 4,096 per side. The client host
allows at most 64 live GL objects and 16 MiB of descriptor data per runtime;
the queued-operation limit is 256 operations and 4 MiB. GUI texture storage has
its own 64-texture and 16 MiB limits. Shader source is bounded to 64 KiB per
stage. These are implementation limits in addition to the guest SDK's input
validation.

The GL import module is `sjml_gl_v1`:

| Import | WASM signature | Purpose |
| --- | --- | --- |
| `create_resource` | `(i32 kind, i32 descriptor_ptr, i32 descriptor_len, i32 handle_out_ptr) -> i32` | Create a GUI texture (kind 0) or GL resource (kinds 1–5) |
| `delete_resource` | `(i32 handle_ptr, i32 handle_len) -> i32` | Queue deletion of an owned kind-11 or kind-12 resource |
| `capabilities` | `(i32 output_ptr, i32 capacity) -> i64` | Return a packed status/length and immutable live-context snapshot |
| `submit` | `(i32 commands_ptr, i32 commands_len, i32 report_ptr, i32 capacity) -> i64` | Validate and execute or queue a bounded command batch |

For the `i64` results, the low 32 bits contain the ABI status code and the high
32 bits contain the number of response bytes written. Capability and execution
report payloads use schema version 1.

The `kind` values for `create_resource` are: 0 GUI texture, 1 buffer, 2 vertex
array, 3 GL texture, 4 framebuffer, and 5 shader program. The command variants
are `UseProgram`, `BindVertexArray`, `BindBuffer`, `BufferSubData`,
`VertexAttribute`, `EnableAttribute`, `BindTexture`, `BindFramebuffer`,
`AttachFramebufferTexture`, `Viewport`, `Scissor`, `Blend`, `Depth`, `Cull`,
`UniformI32`, `UniformF32`, `UniformVec4`, `UniformMat4`, `DrawArrays`,
`DrawElements`, `Clear`, and `DeleteResource`. The bridge rejects unsupported
enum values and malformed or trailing batch bytes. Vertex attribute indices
and texture units are limited to 0–31, vertex stride to 2,048 bytes, and draw
ranges to 1,000,000 vertices/indices. `TextureSubImage` requires a texture
bound earlier in the batch, an in-bounds region, and tightly packed RGBA8 data;
the host temporarily sets pixel-unpack state and restores it after upload.
`CheckFramebufferComplete` requires an owned framebuffer bound in the batch and
reports the actual driver completeness status. These append wire tags 22 and
23, preserving existing command tag assignments. A clear command accepts only
the color, depth, and stencil clear-mask bits.

GL creation, deletion, and out-of-context submissions are queued for the client
render thread. At a live render callback the host drains pending operations
before and after guest dispatch, so resources created by that callback can be
used in its frame. A queued `gl::submit` returns a report with zero commands
executed and a queue message; it does not acknowledge later GPU execution.
`gl::capabilities()` requires a current client GL context. Commands executed
with a current context return an execution report with the number of commands
issued and any GL errors.

GL execution is sequential rather than transactional. If the driver reports an
error during a batch, execution stops and the report gives the number of
commands issued; effects from earlier commands are not rolled back. Errors in
queued resource creation are reported through the host failure handler and
invalidate the failed resource handle.

The OpenGL executor captures and restores the program, VAO, buffer and
framebuffer bindings, touched texture bindings, viewport, scissor, blend/depth/
cull/scissor enables and relevant functions, and clear color around each
command batch. A batch can still affect the framebuffer through its drawing or
clear commands; state restoration does not undo rendered pixels. GPU resources
are owned by one module runtime. Closing the runtime invalidates its handles and
queues cleanup for the next available client render context when immediate
cleanup is not possible.

## Verification boundary

The source wires HUD callbacks into NeoForge's client GUI event, screen callbacks
into `ScreenEvent.Render.Post`, and world callbacks into
`RenderLevelStageEvent.Stage.AFTER_ENTITIES`. Typed GL operations use a client
render-context executor. A live Minecraft client with a display-backed OpenGL
context has not been run. Visual output, shader compilation, world occlusion and
translucency, and interaction with other mods' render state remain unverified.
