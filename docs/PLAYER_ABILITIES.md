# Player abilities

`player::abilities(handle)` returns a detached snapshot of the player's
`may_fly`, `flying`, `invulnerable`, and `may_build` flags. It requires the
requested and granted `entity.read` capability and a live player handle in a
logical-server context.

Ability mutations require `player.abilities.write` in addition to `entity.read`.
The SDK exposes separate setters for `may_fly`, `instabuild`, `invulnerable`,
`may_build`, flight speed, and walking speed. Disabling `may_fly` also clears
active flight atomically. `player::set_flying(handle, enabled)` changes only
current flight; enabling flight requires `may_fly` to be enabled first. Ability
speed values must be finite and between `0.0` and `0.5`; Minecraft defaults are
approximately `0.05` for flight and `0.1` for walking. Every real mutation is
synchronized through `ServerPlayer.onUpdateAbilities()`.

Every call revalidates the module-owned login-session handle on the owning
server thread. Client, static-registration, stale-session, and wrong-thread
calls fail closed. The stable schema-v1 snapshot contains four one-byte
booleans in the order listed above.

The snapshot does not include `instabuild` or either speed value. See
[player ability writes](PLAYER_ABILITY_WRITES.md) for the setters and their
validation rules.
