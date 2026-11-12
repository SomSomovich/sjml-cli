# Generic entity damage

The SDK function `sjml::entity::damage(handle, amount)` requests a bounded
damage hit against one loaded living non-player entity:

```rust
sjml::entity::damage(pig_handle, 2.0)?;
```

The module must request and receive both `entity.read` and `entity.damage`.
The amount is finite, greater than zero, and at most 64 damage points per
call. Player, non-living, unloaded, stale, foreign, client-side, and
static-registration targets are rejected.

The host uses Minecraft's fixed generic damage source and calls the ordinary
`LivingEntity.hurt` path on the owning server thread. Vanilla invulnerability,
armor/effect mitigation, damage hooks, death, and drops remain authoritative.
The guest cannot select an attacker or damage type, and receives no Java or
Minecraft object. `Ok(())` means the host issued the request; an invulnerable
entity or a canceling game hook may prevent health loss.

The ABI import is `sjml_entity_v1.damage(handle_ptr, handle_len,
amount_bits) -> i32`, where `amount_bits` is the raw `f32` bit pattern passed as
an `i32`. It is not batchable. Malformed handles, unsupported
amounts, unavailable contexts, stale handles, or missing grants return stable
ABI errors.
