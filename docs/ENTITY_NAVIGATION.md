# Bounded entity navigation

Server modules can ask one loaded non-player Mob to follow a short vanilla path
with `entity::navigate_to`, inspect its detached status and target with
`entity::navigation_snapshot`, and stop the current path with
`entity::stop_navigation`. The host checks the live entity handle, current
dimension, target bounds, and navigation capability before calling Minecraft's
existing path navigator.

```rust
use sjml::{entity, inventory::BlockPos, EntityHandle, SjmlResult};

fn send_mob_to(handle: EntityHandle) -> SjmlResult<bool> {
    entity::navigate_to(handle, BlockPos { x: 12, y: 64, z: -4 }, 1.0)
}

fn stop_if_it_reached(handle: EntityHandle, expected: BlockPos) -> SjmlResult<bool> {
    let status = entity::navigation_snapshot(handle)?;
    if status.active && status.target == Some(expected) {
        entity::stop_navigation(handle)
    } else {
        Ok(false)
    }
}
```

The GameTest example in `sjml-cli/sjml-sdk/examples/gametest/entity_navigation.rs`
starts a path, verifies the detached status and requested target, stops the path,
and checks that navigation is idle. The existing
`entity.navigation.check` GameTest command covers these calls through the guest
ABI.

The module must request and receive both `entity.read` and
`entity.navigation.write`. The Mob must be loaded, belong to the current
logical server, and be within 32 blocks of the target. The target must be
inside the current dimension's build height and world border. Before asking
vanilla pathfinding to run, the host requires the target and the bounded
one-chunk-margin corridor around the start-to-target rectangle to already be
loaded. SJML never requests a chunk load for navigation. Calls run
synchronously on the server thread; clients and static registration cannot use
the operation.

`navigation_snapshot` requires only `entity.read`. It returns an
`EntityNavigationSnapshot` with an `active` flag and an optional copied
`BlockPos` target. The snapshot contains no live Minecraft object or path nodes
and does not load chunks. `stop_navigation` requires both permissions shown
below and returns `true` when it stopped a path that was active at call time;
it returns `false` when there was no active path. It calls vanilla
`PathNavigation.stop()` and does not change AI, target selection, or goals.

The speed multiplier must be finite and within `0.01..=4.0`. Minecraft returns
`true` when it accepts a path and `false` when no path is accepted, including
when the Mob has AI disabled. The API does not expose path nodes, force a
teleport, configure goal selectors, or execute guest code from the pathfinder.
Declare both permissions on the server module for path creation and stopping.
Status reads need only `entity.read`:

```toml
side = "server"
permissions = ["entity.read", "entity.navigation.write"]
```

See the [entity ABI](ABI.md#loaded-non-player-entities) and
[capability configuration](PROJECT_CONFIGURATION.md#permissions-and-capabilities).
