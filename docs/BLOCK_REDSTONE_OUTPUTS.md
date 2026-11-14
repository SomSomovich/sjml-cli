# Static redstone output for registered blocks

Registered blocks can provide weak redstone, direct redstone, and analog
comparator output for an exact declared block state. Values are static registry
declarations; Minecraft does not call guest code while it evaluates a redstone
network.

```rust
registry::set_block_state_behavior(
    "signal_block",
    &[("powered", "true")],
    registry::BlockStateBehavior::new()
        .redstone_signal(12)?
        .direct_redstone_signal(15)?
        .comparator_signal(7)?,
)?;
```

Each output is optional and uses the inclusive range `0..=15`. `redstone_signal`
implements the weak signal returned by the vanilla block signal query;
`direct_redstone_signal` implements the direct-neighbor signal query. They are
independent, and either one makes the configured state a redstone source.
`comparator_signal` marks the state as an analog-output source and supplies its
value to vanilla comparators. It does not make the block a redstone source by
itself. Setting a field to zero is an explicit zero output; leaving it unset
retains vanilla behavior.

The selector must name the complete declared state. The existing
`redstone_conductor` option remains separate and controls whether a state
conducts incoming signals; it does not set emitted output. Vanilla neighbor
updates and comparator behavior still govern when these static values are
queried.
