# Client camera controls

Client modules can set a persistent, module-owned multiplier for the local
player's computed field of view:

```rust
use sjml::{client_camera, SjmlResult};

fn adjust_camera_fov() -> SjmlResult<()> {
    client_camera::set_fov_scale(0.8)?;
    // Later, when this module no longer needs the change:
    client_camera::clear_fov_scale()?;
    Ok(())
}
```

The module must request and receive `client.camera.fov` and run on the physical
client main thread. Each module may set one finite multiplier in `0.25..=4.0`;
`1.0` leaves its contribution neutral. Active multipliers are multiplied in
runtime activation order, with the combined value capped to `0.25..=4.0`.
The host applies the result to Minecraft's already computed dynamic FOV
modifier and clamps the final modifier to `0.05..=4.0`.

The value remains active until the module clears it or its runtime closes. A
module cannot set another module's factor. The frame-time hook does not invoke
guest code. The import is denied during static registration and in dedicated
server contexts. Automated host tests cover permission checks, per-module
ownership, scale aggregation, interaction with Minecraft's dynamic modifier,
and cleanup. A visible effect still needs a display-backed Minecraft client
check.
