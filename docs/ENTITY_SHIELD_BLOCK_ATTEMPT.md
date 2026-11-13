# Entity shield-block attempts

Server modules can synchronously veto a vanilla shield block through
`entity.shield_block_attempt`. The host dispatches the callback on the logical
server thread when NeoForge reports that vanilla would block an incoming hit
with a shield. The detached snapshot contains the defender and damage type,
optional causing and direct source UUIDs, the current block flags, incoming
damage, and shield durability damage.

The callback requires `events.subscribe` and `events.cancel`. It may return
`Pass` to preserve the decision from Minecraft and earlier listeners, or
`Cancel` to mark the shield ineligible for this hit. Cancellation does not
cancel the incoming damage; vanilla continues damage processing as if the
shield could not block it. The callback cannot force a block or set either
damage amount. Events already cancelled by another listener and hits that
vanilla did not consider blockable are not dispatched.

```rust
use sjml::prelude::*;
use sjml::shield_block_attempt::{
    EntityShieldBlockAttempt, EntityShieldBlockAttemptAction,
};

#[sjml::events]
mod events {
    use super::*;

    fn shield_block_attempt(
        attempt: EntityShieldBlockAttempt,
    ) -> SjmlResult<EntityShieldBlockAttemptAction> {
        if attempt.damage_type_id == "minecraft:arrow" {
            Ok(EntityShieldBlockAttemptAction::Cancel)
        } else {
            Ok(EntityShieldBlockAttemptAction::Pass)
        }
    }
}
```

Only bounded identifiers, UUID strings, booleans, and finite damage values
cross the ABI. The host caps the payload at 2 KiB and invokes at most 64
authorized callbacks for one hit. Guest errors disable the failing module and
do not cancel the shield block.
