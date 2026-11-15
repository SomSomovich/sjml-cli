# Creating explosions

Server modules with the separate `world.explosion` capability can create a
vanilla explosion in a loaded world:

```rust
use sjml::{world, world::ExplosionInteraction, SjmlResult, WorldHandle};

fn blast(world: WorldHandle, x: f64, y: f64, z: f64) -> SjmlResult<()> {
    world::create_explosion(
        world,
        world::RaycastPoint::new(x, y, z),
        4.0,
        false,
        ExplosionInteraction::Block,
    )
}
```

The module must request and receive `world.read` and `world.explosion`.
Coordinates must be finite and within ±30,000,000; power must be finite and in
`0 < power <= 16`, where vanilla TNT uses 4 and a charged creeper uses 6. The
center must be inside build height and the world border, and every chunk touched
by the horizontal blast area must already be loaded. The call never requests a
chunk load.

`ExplosionInteraction::Block` uses vanilla block destruction and drops, subject
to the world's explosion game rules. `ExplosionInteraction::None` destroys no
blocks while nearby entities still receive normal explosion damage. The `fire`
flag uses vanilla's random fire-placement chance. The explosion has no source
entity, so vanilla attribution and `mobGriefing` entity-grief checks behave as
for an unattributed explosion. A module's own subscribed `world.explosion_attempt`
callback also runs for the guest-created explosion and can veto it.

Each module can create at most two explosions per logical-server tick; further
attempts in the same tick return `SERVICE_BUSY`. The request uses ABI schema 1:
version, module-owned world handle, `f64` x/y/z, `f32` power, a Boolean fire
flag, and an interaction code (`0` none, `1` block). Every value is validated
before world access, and unloaded chunks, out-of-bounds positions, or missing
capabilities return an error.

GameTests place stone blocks, invoke a real guest-created explosion, and verify
that the destructive interaction removes blocks while the non-destructive one
leaves them intact. Unit tests cover the stable codec, bounds, capability
enforcement, quota, static/client denial, and malformed-length rejection.
