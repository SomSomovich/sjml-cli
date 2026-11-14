# Entity air supply

The SDK reads and changes the current air supply of a live, loaded non-player `LivingEntity`:

```rust
let air = entity::air_supply(entity_handle)?;
entity::set_air_supply(entity_handle, 180)?;
```

`entity::air_supply` returns a detached schema-v1 `EntityAirSupply { current, max }` value. The
current value can range from `-20` through the entity's maximum because vanilla uses the negative
range during its drowning damage window. The maximum is read from the current entity and bounded
to `0..=1,000,000`.

## Permissions and safety

Reading requires `entity.read`. Writing requires both `entity.read` and the separate
`entity.air.write` capability. Declare the capability in the module manifest and grant it to the
module on the server.

Every operation validates the module-owned handle, its generation, the current live entity, its
living-entity type, and the owning logical-server thread. The setter accepts only nonnegative ticks
no greater than the entity's current maximum. It changes the current value only; it never changes
the maximum. Both calls operate on already loaded entities and do not allocate handles or load
chunks. Client modules and static registration cannot access live air supply.

## ABI and SDK

The imports use `sjml_entity_v1`:

- `air_supply(handle_ptr, handle_length, output_ptr, output_capacity)`
- `set_air_supply(handle_ptr, handle_length, ticks)`

The schema-v1 read response is exactly 12 little-endian bytes: schema `u32` (`1`), current air
`i32`, and maximum air `i32`. The SDK methods are:

- `entity::air_supply(handle) -> SjmlResult<EntityAirSupply>`
- `entity::set_air_supply(handle, ticks: u32) -> SjmlResult<()>`

The setter rejects values above the maximum observed by the host at call time.
