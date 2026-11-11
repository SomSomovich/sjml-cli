# Player abilities

`player::abilities(handle)` returns a detached snapshot of the player's
`may_fly`, `flying`, `invulnerable`, and `may_build` flags. It requires the
requested and granted `entity.read` capability and a live player handle in a
logical-server context.

`player::set_flying(handle, enabled)` changes only the current flying flag. It
also requires `player.abilities.write`. Enabling flight succeeds only when the
player's existing `may_fly` flag is already true; disabling flight is allowed
regardless of `may_fly`. A real change is synchronized with Minecraft's
`ServerPlayer.onUpdateAbilities()` call. The host does not change the player's
game mode, `may_fly`, invulnerability, or build permission.

Every call revalidates the module-owned login-session handle on the owning
server thread. Client, static-registration, stale-session, and wrong-thread
calls fail closed. The stable schema-v1 snapshot contains four one-byte
booleans in the order listed above.
