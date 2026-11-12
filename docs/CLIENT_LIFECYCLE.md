# Physical-client lifecycle and readiness

Client runtime activation and readiness of a particular client feature are
separate milestones. On a physical client, common setup defers `CLIENT`
modules to the client adapter. The adapter binds SJML's client task lane on
Minecraft's main thread, then activates those modules. This is enough for a
module to initialize and register its host subscriptions; SJML does not wait
for a world, player, network login, open screen, or optional native chat
geometry before activating every client module.

Each module still has its own activation result and resource owner. A module
that fails its own initialization is isolated according to package dependency
rules; unrelated packages can continue. Closing the client runtime releases
module-owned subscriptions and resources. The global client runtime is not
restarted for each world or server connection.

Individual APIs can become useful later than the client runtime. Native chat
is one such capability. Its controller subscription may be registered during
module initialization, even if Minecraft has not exposed the GUI/window needed
to create the native geometry adapter yet. Chat screen events begin once the
adapter is ready and observes an active `ChatScreen`. `client_chat::layout`
and `client_chat::set_layout` require that geometry to be ready; before then
they return the host's unavailable-capability error. A missing chat adapter
therefore does not prevent unrelated client modules from starting.

Keep callbacks on the client main thread and treat session objects as
short-lived: world, player, screen, and network availability can change during
the process lifetime. Subscribe to their lifecycle events when available and
release handles when finished. On a dedicated server, physical-client
integration remains isolated in client-only classes and client modules are not
activated.
