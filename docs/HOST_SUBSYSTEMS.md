# Host subsystem lifecycle

SJML host bootstrap and runtime activation use a dependency graph to initialize shared services and resources owned by each execution context. The graph runs on the host lifecycle thread. It does not schedule guest callbacks or load a physical-client adapter on a dedicated server.

## Scope and ordering

Common subsystems are initialized once for a `PackageRuntimeActivationService`. They cover the capability policy, staged content registrations, the WASM runtime, and the activation service gate. `SjmlMod` starts this graph during common setup, before it defers client module activation. Direct callers of `activate` also pass through the same gate.

Every `Activation` opens a separate context scope for each selected `ExecutionContext`. This allows two activations to use the same context independently. The context graph owns the `ExecutionScope`, `ModuleServiceRouter`, optional session lane, negotiated network runtime, and active module instances. A context can close without closing other contexts held by the same activation. A later activation opens fresh state and resources.

Definitions are validated before the graph can run. Every dependency ID must exist, common subsystems cannot depend on context subsystems, and a context dependency must apply to every context where its dependant applies. A stable topological sort resolves dependencies first and uses subsystem ID order for otherwise independent definitions. Cycles are rejected with a deterministic cycle path.

Required dependencies initialize before their dependant. A required dependency failure marks the dependant `FAILED` and skips its initializer. Optional dependencies are observed only when already initialized; they do not trigger initialization or block their dependant. This keeps the server-session lane optional while common server setup activates modules before a logical server is bound. `bindServer` and client task binding start the lane when it becomes available, and unbinding stops that lane without stopping guest modules.

## States and failures

Each registered subsystem follows these states:

| State | Meaning |
| --- | --- |
| `REGISTERED` | Definition is known and its initializer has not run. |
| `INITIALIZING` | Its initializer is running. |
| `READY` | Initialization succeeded and dependent subsystems may use it. |
| `FAILED` | Initialization or cleanup failed; the diagnostic retains the subsystem and cause. |
| `STOPPED` | The scope closed or the subsystem was explicitly stopped. A stopped context subsystem can be started again while its scope remains open. |

An initializer failure triggers cleanup for partial initialization, marks only that subsystem failed, and allows independent branches to continue. On shutdown, ready subsystems stop in reverse dependency order. Cleanup failures are recorded and do not prevent remaining subsystems from stopping. Closing a context is idempotent.

The graph exposes state maps and localized diagnostics for startup logging and tests. `PackageRuntimeActivationService.Activation.subsystemStates(context)` reports the live or stopped state for a particular activation context.

## Test coverage

`HostSubsystemGraphTest` checks deterministic ordering, missing definitions, cycle paths, isolated and optional failures, partial-initialization cleanup, reverse-order teardown, cleanup-error continuation, and client-only scope availability. `PackageRuntimeActivationServiceTest` checks that the graph owns real context resources and that an unavailable optional server session does not block module activation.
