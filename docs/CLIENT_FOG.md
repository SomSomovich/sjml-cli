# Physical-client fog controls

Modules can keep local fog overrides for the physical Minecraft client with the Rust SDK's
`client_fog` API. The module must declare and receive `client.fog.modify`, have side `client` or
`both`, and run in the client's execution context. The host also checks that calls arrive on the
Minecraft client thread. A dedicated server never provides the client fog imports.

The settings are persistent host state. They do not call a WASM module while Minecraft renders.
Each module owns only its own override, and the host clears those overrides when the runtime closes
or the capability is revoked.

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    client_fog::set_color(0.22, 0.34, 0.48)?;
    client_fog::set_distances(8.0, 72.0)?;
    client_fog::set_shape(FogShape::Sphere)
}
```

## Available settings

- **Color:** `set_color(red, green, blue)` replaces fog RGB. Each component must be finite and
  within `0.0..=1.0`.
- **Distances:** `set_distances(near, far)` sets the fog start and end distances in blocks. Values
  must be finite, `near >= 0`, `far > near`, and `far <= 1024`.
- **Shape:** `set_shape(FogShape::Sphere)` or `set_shape(FogShape::Cylinder)` changes the fog shape.
- Each field has a matching `clear_*` function. `clear_all()` removes all three settings owned by
  the current module.

Fog color is applied at NeoForge's lowest event priority. Distance and shape overrides use
NeoForge's render fog event, which requires cancellation for its setters to take effect. If a module
sets only one of those fields, the host preserves the other incoming event values before
cancelling the event.

## When modules set the same field

Composition is deterministic and happens independently for color, distances, and shape. The host
sorts active runtime module IDs lexicographically; the **greatest ID that set a particular field
wins**. There is no blending or averaging. A module that does not set distances does not affect
another module's distances. Clearing a winning value, revoking its capability, or closing its
runtime reveals the next lower-ID module's setting. Runtime IDs are stable identities supplied by
the host, so the winner does not depend on which module happened to activate first.

An example: if `pack/a/client` sets blue color and `pack/z/client` sets red color, red is shown.
If `pack/z/client` clears its color, blue becomes active again. The same rule is used separately
for fog distance and shape.
