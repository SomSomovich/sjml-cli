# Client action attempts

The physical-client `client.action_attempt` event can veto Minecraft's
supported vanilla attack, use-item, and pick-block keymappings before vanilla
handles them. It uses NeoForge's cancellable
`InputEvent.InteractionKeyMappingTriggered` event, so key rebinding remains
owned by Minecraft and the callback reports the action mapping rather than a
mouse button or keyboard code.

The detached schema-v1 `ClientActionAttempt` contains `action` and `hand`:
`ClientAction::Attack`, `ClientAction::UseItem`, or `ClientAction::PickBlock`,
and `ClientActionHand::Main` or `ClientActionHand::Off`. NeoForge reports use
item once for each hand; attack and pick-block report the main hand. No live
keymapping, player, world, item, or input device object crosses the ABI.

Subscribe from a client module's `#[mod_init]` and return `Ok(true)` to veto:

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    events::subscribe("client.action_attempt", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "client.action_attempt")]
fn prevent_attacks(attempt: ClientActionAttempt) -> SjmlResult<bool> {
    Ok(attempt.action == ClientAction::Attack)
}
```

The module must use `side = "client"` (or another client-capable side) and
request `events.subscribe`, `events.cancel`, and the separate
`client.input.action_cancel` capability. The server administrator must grant
all three capabilities. This callback does not require `client.input` because
it uses its own action-attempt event route.

Callbacks run synchronously on the physical-client main thread and must use
`#[event]`, not `#[event_async]`. If another mod already cancelled the event,
SJML does not call guest handlers or clear that cancellation. Otherwise a true
result from any eligible module cancels the mapping; handlers run in normal
event-priority and activation order. At most 64 callbacks are invoked for one
input event. A malformed result or trapped guest closes only that module.
Minecraft's existing hand-swing setting is left unchanged.

The example package under [`examples/client-action-attempt`](../../examples/client-action-attempt/)
requests the required grants and cancels main-hand attacks.
