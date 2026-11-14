# Player movement state

Use `player::movement_state` to read a detached snapshot of an online server player's movement flags and fall distance:

```rust
let movement = player::movement_state(player_handle)?;
if movement.on_ground && movement.sprinting {
    // The server currently considers the player grounded and sprinting.
}
```

`PlayerMovementState` contains `on_ground`, `sprinting`, `crouching`, `swimming`, and `fall_distance`. These values come from the current vanilla player entity; `on_ground` is vanilla's grounded state, and `crouching` reflects the entity's crouching state rather than raw key input. `fall_distance` must be finite and in `0..=1,000,000` blocks.

The read requires the declared and granted `entity.read` capability, a live online player handle, and the owning logical-server thread. Client contexts and static registration cannot read live state. The host revalidates the login session on each call. This API is read-only and does not load a world, look up other entities, or change player state.

The schema-v1 response is 12 little-endian bytes: schema `u32` (`1`), four canonical `u8` flags in order (`on_ground`, `sprinting`, `crouching`, `swimming`), and the `fall_distance` IEEE-754 `f32` bits. The `sjml_player_v1` import is `movement_state(handle_ptr, handle_len, output_ptr, output_capacity)` and returns a standard SJML ABI status code.
