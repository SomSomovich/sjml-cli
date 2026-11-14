# Physical-client resource reload notifications

`client.resources_reloaded` notifies a client-side SJML module after Minecraft has successfully
applied a resource-pack reload. It carries a single positive generation ID; resource managers,
pack objects, textures, and other Java objects never cross the ABI.

## Subscribe

Declare `events.subscribe` in the module permissions, then subscribe from client-side initialization:

```rust
use sjml::{events, mod_init, EventPriority, SjmlResult};

#[mod_init]
fn init() -> SjmlResult<()> {
    let _subscription = events::subscribe(
        "client.resources_reloaded",
        EventPriority::Normal,
    )?;
    Ok(())
}
```

Use the typed asynchronous callback to enqueue work after the reload callback returns:

```rust
use sjml::{event_async, ClientResourcesReloaded, SjmlResult};

#[event_async(name = "client.resources_reloaded")]
async fn on_resources_reloaded(event: ClientResourcesReloaded) -> SjmlResult<()> {
    // Rebuild module-local state from the newly applied resources.
    let _generation = event.generation;
    Ok(())
}
```

The payload schema is version 1 and contains `generation: u64`. The host emits IDs in
`1..=i64::MAX`; they increase for every completed notification and never wrap. The initial client
resource load is included. If the counter reaches its maximum, later notifications are omitted.

## Timing and thread behavior

NeoForge 21.1.265 fires `RegisterClientReloadListenersEvent` once while constructing `Minecraft`.
The SJML listener implements Minecraft's `ResourceManagerReloadListener`; its
`onResourceManagerReload` runs on the reload's game executor. It only captures up to 256 pack IDs,
each at most 256 UTF-8 bytes, and returns without invoking guest code or waiting on another future.

That listener runs before the whole reload pipeline has reported success. A small mixin observes
Minecraft's `LoadingOverlay.render` completion callback: Minecraft calls it with an empty result
only after `ReloadInstance.checkExceptions()` succeeds, and with the failure otherwise. SJML only
records that outcome there; it does not call guest code or wait. It holds a bounded candidate
snapshot and waits for a later `ClientTickEvent.Post` where Minecraft's loading overlay is gone. It
compares the active resource manager's pack IDs with the candidate before assigning a generation
and publishing the event. Failed reloads are discarded even if the attempted pack IDs remain
active, and rollback's successful fallback reload receives its own generation. Overlapping reload
requests are coalesced by Minecraft; SJML reports the final active pack set once.

The event is a non-cancellable notification. The host does not invoke a module while NeoForge is
applying reload listeners, so module work cannot hold the reload future open. The event export is
entered briefly from the client tick; `#[event_async]` queues the user future on the module task
lane and returns. That task lane is cooperative and runs on the client thread, so break expensive
work into short chunks and yield between them. The API does not wait on a worker or run an entire
module callback inside the reload apply stage. A regular synchronous `#[event]` handler can use the
render/client thread for its whole body.

The generation is local to this physical Minecraft instance, not a network or server revision.
Mods can listen for the same reload independently; the SJML callback does not change pack order or
cancel the reload.

## Lifecycle and limits

The user's event subscription is scoped to the physical-client activation and is removed when that
runtime context closes. The single NeoForge listener is attached to Minecraft's client resource
manager for that manager's lifetime; Minecraft closes that manager on shutdown. SJML keeps only the
bounded pending pack-ID snapshot, clears it on client shutdown and exposes a context-teardown clear
hook. No listener is registered per module.

If a candidate pack list has more than 256 entries or an invalid/oversized ID, SJML skips its
notification without failing Minecraft's resource reload. If active pack IDs differ from the
candidate at the next eligible client tick, SJML discards the candidate. The callback is only
available in the physical-client execution context and requires `events.subscribe`.

The deterministic host-side tracker tests cover delayed completion, rollback mismatch, increasing
IDs, context cleanup, shutdown cleanup, and limits. The same tracker is used by the live client
adapter, making the test reproducible without a display-dependent GameTest.
