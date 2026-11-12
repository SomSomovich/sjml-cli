# `sjml-sdk`

`sjml-sdk` is the Rust guest API for Somuch Java Mod Loader (SJML). Its Rust
crate name is `sjml`; guest projects normally depend on it like this:

```toml
[dependencies]
sjml-sdk = "0.54.0"
```

SJML guests target `wasm32-unknown-unknown`. The SDK also builds natively so
guest logic and data types can be tested, but native builds do not provide the
host imports or WebAssembly callback exports.

SDK 0.43.0 added bounded same-dimension player teleportation, non-player entity
removal, loaded-world weather controls, and stonecutting recipes. It also
includes client-only native chat layout and panel theming, a bounded
server-side `player.moved` notification, and bounded transfers between loaded
block inventory slots. These APIs require their documented capabilities; see
the [SDK guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SDK_GUIDE.md),
[chat API contract](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CHAT.md),
[player teleport](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_TELEPORT.md),
[entity removal](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_REMOVAL.md),
[world weather](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_WEATHER.md),
[stonecutting recipes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/STONECUTTING.md),
[movement event](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_MOVED_EVENT.md),
and [inventory mutations](https://github.com/SomSomovich/sjml-cli/blob/master/docs/INVENTORY_MUTATIONS.md).

SDK 0.44.0 adds read-only player vitals, bounded server-player game-mode
changes, loaded-world biome and daylight-cycle time access, bounded player
motion reads and velocity writes, smithing-transform recipe registration, and
the non-cancellable `player.respawned` event. See
[player vitals](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_VITALS.md),
[player game mode](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_GAME_MODE.md),
[world biomes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_BIOMES.md),
[world time](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_TIME.md),
[player motion](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_MOTION.md),
[smithing transform recipes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SMITHING_TRANSFORM.md),
and [player respawn events](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_RESPAWNED_EVENT.md).

SDK 0.45.0 adds detached player equipment snapshots, `doDaylightCycle`
gamerule access, client-local chat-message callbacks with capability-gated
display filtering, and bounded client toast notifications. See
[player equipment](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_EQUIPMENT.md),
[daylight-cycle gamerule](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_DAYLIGHT_CYCLE.md),
[client chat messages](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_CHAT_MESSAGES.md),
and [client toast notifications](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_NOTIFICATIONS.md).

SDK 0.46.0 adds player ability snapshots and guarded flight changes, ordinary
signed client chat sending, client-local sound playback, and read/write access
to loaded world-border settings. See
[player abilities](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_ABILITIES.md),
[client audio](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_AUDIO.md),
and [world border](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_BORDER.md).

SDK 0.47.0 adds observational mouse-wheel subscriptions, read-only player
inventory snapshots, bounded reads and base-value writes for selected vanilla
living-entity attributes, and loaded block-state property snapshots. See
[mouse-wheel input](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_INPUT_SCROLL.md),
[player inventory](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_INVENTORY.md),
[live entity attributes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_ATTRIBUTES.md),
and [block-state properties](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_BLOCK_PROPERTIES.md).

SDK 0.48.0 adds allowlisted boolean world gamerules, bounded generic damage to
living non-player entities, literal player action-bar messages, named client
keybindings, and opt-in active-screen scrolling. See
[world gamerules](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_GAME_RULES.md),
[entity damage](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_DAMAGE.md),
[player action bars](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_ACTIONBAR.md),
[client keybindings](https://github.com/SomSomovich/sjml-cli/blob/master/docs/KEYBINDINGS.md),
and [active-screen scrolling](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SCREEN_INPUT_SCROLL.md).

SDK 0.49.0 adds quota-limited scheduled ticks for matching blocks in loaded
chunks, removal of registered effects from loaded non-player entities,
synchronous vetoes for right-click block/entity/item interactions, and
scaled GUI cursor coordinates on the physical client. See
[scheduled block ticks](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_SCHEDULED_TICKS.md),
[effect removal](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_EFFECT_REMOVAL.md),
[player interaction vetoes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_INTERACTIONS.md),
and [cursor position](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CURSOR_POSITION.md).

SDK 0.51.0 adds loaded-entity spatial queries and raycasts, loaded-chunk
enumeration, allowlisted integer game-rule access, expected-state-guarded
block-property writes, item-stack snapshots, server-player attribute access,
player inventory slot and durability writes, synchronous block-placement and
entity-death vetoes, and manifest-declared local client commands. See
[loaded chunks](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_LOADED_CHUNKS.md),
[entity raycasts](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_RAYCAST.md),
[entity death attempts](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_DEATH_ATTEMPTS.md),
[client commands](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_COMMANDS.md),
[spatial entity queries](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_SPATIAL_QUERY.md),
[integer game rules](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_INTEGER_GAME_RULES.md),
[block property writes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_BLOCK_PROPERTIES_WRITE.md),
[item-stack snapshots](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ITEM_STACK_SNAPSHOT.md),
[player inventory writes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_INVENTORY_WRITES.md),
and [block placement vetoes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/BLOCK_PLACE_ATTEMPTS.md).

SDK 0.52.0 adds bounded static villager trade offers, cancellable pre-mitigation
entity damage attempts, cross-dimension teleportation into an already-loaded
chunk, and shared elapsed game-time writes. See [villager trades](https://github.com/SomSomovich/sjml-cli/blob/master/docs/VILLAGER_TRADES.md),
[damage attempts](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_DAMAGE_ATTEMPTS.md),
[cross-dimension teleport](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_DIMENSION_TELEPORT.md),
and [shared game time](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_GAME_TIME.md).

SDK 0.53.0 adds bounded local rich chat-message replacements with formatting,
hover text, and safe URL/suggest/copy click actions. Rewriting requires the
separate
`client.chat.rewrite` grant; the older one-byte Show/Hide result remains
compatible. See the [chat-message API contract](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_CHAT_MESSAGES.md).

SDK 0.54.0 adds bounded biome-ID enumeration from a loaded world's registry,
module-owned integer item components for loaded block inventories, and the
cancellable `entity.target_attempt` event. The first two are restricted to
server contexts and declared capabilities; target vetoes require both event
subscription and cancellation grants. See the [biome IDs](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_BIOME_IDS.md),
[integer item components](https://github.com/SomSomovich/sjml-cli/blob/master/docs/INTEGER_ITEM_COMPONENTS.md),
and [entity target attempts](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_TARGET_ATTEMPTS.md).

```rust
use sjml::prelude::*;

#[mod_init]
fn initialize() -> SjmlResult<()> {
    host::log(LogLevel::Info, "guest initialized")?;
    Ok(())
}
```

See the [SDK guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SDK_GUIDE.md)
for callback macros, API modules, capability requirements, bounds, and examples;
the [docs.rs API reference](https://docs.rs/sjml-sdk/0.54.0/sjml/) lists the
public Rust types and functions. The [ABI v1 specification](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ABI.md)
describes the raw WebAssembly interface. For `sjml.toml`, project layout, and
administrator grants, see the [project configuration guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PROJECT_CONFIGURATION.md)
and [manifest schema](https://github.com/SomSomovich/sjml-cli/blob/master/docs/MANIFEST_SCHEMA.md).

The full workspace and command-line tool are in the
[sjml-cli repository](https://github.com/SomSomovich/sjml-cli).
