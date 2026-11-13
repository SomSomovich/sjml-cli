# `sjml-sdk`

`sjml-sdk` is the Rust guest API for Somuch Java Mod Loader (SJML). Its Rust
crate name is `sjml`; guest projects normally depend on it like this:

```toml
[dependencies]
sjml-sdk = "0.97.0"
```

SJML guests target `wasm32-unknown-unknown`. The SDK also builds natively so
guest logic and data types can be tested, but native builds do not provide the
host imports or WebAssembly callback exports.

SDK 0.88.0 adds per-state friction for registered blocks, bounded loaded-only fluid tick scheduling, and sampled client cursor movement. See [block-state friction](../docs/BLOCK_STATE_FRICTION.md), [world fluid ticks](../docs/WORLD_FLUID_TICKS.md), and [client cursor movement](../docs/CLIENT_CURSOR_MOVED.md).

Registered blocks can subscribe to their own scheduled ticks with
`events::subscribe("block.tick", ...)`; the callback receives the detached
[`BlockTick`](https://docs.rs/sjml-sdk/0.83.0/sjml/events/struct.BlockTick.html)
snapshot. See the [block tick guide](../docs/BLOCK_TICK.md) for ownership,
thread, capability, and quota limits.

Owners can opt registered blocks into vanilla random-tick selection during
`#[mod_register]` with `registry::set_block_random_ticking`, then subscribe to
the separate `block.random_tick` event and its `BlockRandomTick` snapshot. Both
tick callbacks share a per-server-tick host budget; see the same guide for the
full contract.

Physical-client modules can veto vanilla attack, use-item, and pick-block
actions through `client.action_attempt`. The synchronous callback requires
`events.subscribe`, `events.cancel`, and `client.input.action_cancel`; see the
[client action attempts guide](../docs/CLIENT_ACTION_ATTEMPTS.md).

The `scoreboard` module can assign or clear a fixed vanilla display slot for
an existing objective under `scoreboard.write`; see the
[scoreboard guide](../docs/SCOREBOARD.md) for stable slot codes and the
server-thread contract.

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
entity-death vetoes, and manifest-declared local client commands with bounded
dynamic suggestions. See
[loaded chunks](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_LOADED_CHUNKS.md),
[entity raycasts](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_RAYCAST.md),
[entity death attempts](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_DEATH_ATTEMPTS.md),
[client commands and bounded local suggestions](https://github.com/SomSomovich/sjml-cli/blob/master/docs/CLIENT_COMMANDS.md),
[spatial entity queries](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ENTITY_SPATIAL_QUERY.md),
[integer game rules](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_INTEGER_GAME_RULES.md),
[block property writes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/WORLD_BLOCK_PROPERTIES_WRITE.md),
[item-stack snapshots](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ITEM_STACK_SNAPSHOT.md),
[player inventory writes](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_INVENTORY_WRITES.md),
[player inventory stack moves](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PLAYER_INVENTORY_MOVE.md),
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

SDK 0.55.0 adds module-owned integer-component access on live player inventory
stacks, static item rarity and enchantability declarations, bounded fire-tick operations for
loaded non-player entities, cancellable block-break attempts, observe-only
global key notices, active-screen key vetoes, looping client sounds, and
server titles/subtitles. See [player inventory components](../docs/PLAYER_INVENTORY_COMPONENTS.md),
[item rarity and enchantability](../docs/ITEM_RARITY.md), [entity fire ticks](../docs/ENTITY_FIRE_TICKS.md),
[block-break attempts](../docs/BLOCK_BREAK_ATTEMPTS.md), [client key events](../docs/CLIENT_KEY_EVENTS.md),
[client audio](../docs/CLIENT_AUDIO.md), and [player titles](../docs/PLAYER_TITLES.md).

SDK 0.56.0 adds safe synchronous cancellation of player item drops,
owner-scoped updates to the active read-only server menu, live non-player
glowing/silent flags, and static item enchantability. See [player drop
attempts](../docs/PLAYER_DROP_ATTEMPTS.md), [menu updates](../docs/MENU_UPDATES.md),
and [entity state flags](../docs/ENTITY_STATE_FLAGS.md).

SDK 0.58.0 adds synchronous cancellable projectile-impact events, bounded
plain custom names for loaded non-player entities, registered food properties,
and editable server menus with allowlisted slots and item-conserving transfers.
See [projectile impact attempts](../docs/PROJECTILE_IMPACT_ATTEMPT.md),
[entity custom names](../docs/ENTITY_CUSTOM_NAMES.md), [item food](../docs/ITEM_FOOD.md),
and [editable menus](../docs/EDITABLE_MENUS.md).

SDK 0.59.0 adds advancement control and custom-stat access, difficulty and
Overworld spawn-point control, cancellable entity heal/effect/join attempts,
bounded block-inventory names and transfers, food-consumption effects,
independent model-axis animation, and exact client GUI-layer suppression. See
the matching API guides in `../docs/` for permissions, limits, and examples.

SDK 0.60.0 adds loaded-Mob AI toggles, player respawn-point read/set/clear,
literal custom names on existing player-inventory stacks, and loaded-only
vanilla neighboring-redstone-signal reads. See the matching API guides in
`../docs/` for capability requirements and chunk/thread behavior.

SDK 0.61.0 adds selected-hotbar-slot control, bounded reads of active effects
on loaded living entities, reads/writes for existing scoreboard objectives,
and the cancellable `PlayerSleepAttempt` event. See the matching API guides
in `../docs/` for grants, bounds, and event semantics.

SDK 0.62.0 adds the synchronous physical-client `client.item.tooltip` event
and bounded literal lines through `ItemTooltipLines`. It requires
`events.subscribe` and the separate `client.item.tooltip.modify` grant; see
`../docs/ITEM_TOOLTIP_EVENTS.md` for the contract and example.

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
the [docs.rs API reference](https://docs.rs/sjml-sdk/0.83.0/sjml/) lists the
public Rust types and functions. The [ABI v1 specification](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ABI.md)
describes the raw WebAssembly interface. For `sjml.toml`, project layout, and
administrator grants, see the [project configuration guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/PROJECT_CONFIGURATION.md)
and [manifest schema](https://github.com/SomSomovich/sjml-cli/blob/master/docs/MANIFEST_SCHEMA.md).

The full workspace and command-line tool are in the
[sjml-cli repository](https://github.com/SomSomovich/sjml-cli).

SDK 0.63.0 also provides the physical-client `client.chat.send_attempt` event, which can veto the original outgoing message without changing Minecraft’s signed-chat flow. Reading outgoing text requires `client.chat.send.observe`; vetoing also requires `events.subscribe` and `events.cancel`. See the outgoing chat guide in the workspace documentation.

SDK 0.64.0 adds bounded `client_screen::open` screens with literal text and up to four vanilla buttons. Button actions are dispatched directly to the owning module as `client.screen.action`; opening requires `client.ui.screen`, and the callback does not require `events.subscribe`. See `docs/CLIENT_SCREEN.md` in the workspace documentation.


SDK 0.66.0 adds schema-v3 client form screens with up to four UTF-8-limited text fields and one scrollable selection list of up to 32 options. Submit values are delivered directly to the owner through `client.screen.form_submit`; schema-v1 button screens and schema-v2 text forms remain compatible. See `docs/CLIENT_SCREEN.md` in the workspace documentation.

SDK 0.67.0 adds schema-v4 checkbox settings forms, same-module boolean item
components, detached world-fluid snapshots, local physical-client FOV scaling,
and server/client tick notifications. Network channel declarations now support
optional local token-bucket limits without changing their wire format. See the
focused guides in the workspace `docs/` directory.

SDK 0.73.0 adds loaded-only block tag checks and current-registry item tag
membership queries through the world API. Both require `world.read`; block
queries preserve `None` for unavailable chunks and item queries do not load
chunks or inspect inventories. See `docs/WORLD_BLOCK_TAGS.md` and
`docs/WORLD_ITEM_TAGS.md` in the workspace.

SDK 0.75.0 adds bounded `world::tag_members` enumeration for block, item,
fluid, and entity-type tags, with typed convenience functions, plus fixed
full-cube, empty, and single-cuboid shapes for registered blocks. Tag results
are sorted, limited to 1,024 members, and distinguish absent tags from present
empty tags. Block shapes are finite, normalized, and set during registration.
SDK 0.77.0 adds the owner-scoped synchronous `block.use` callback for
registered blocks. ABI 1.68.0 / SDK 0.76.0 added bounded friction, movement
factors, emitted light, lava ignition, piston reactions, and allowlisted sound
groups for registered blocks.
See `docs/WORLD_TAG_MEMBERS.md`, `docs/WORLD_BLOCK_SHAPES.md`, and
`docs/WORLD_BLOCK_BEHAVIOR.md`; see `docs/BLOCK_USE.md` in the workspace for
the event contract.

SDK 0.78.0 adds an owner-scoped synchronous `item.use_on_block` callback for registered items. The block owner gets first refusal through `block.use`; if it passes, only the held item owner receives the detached item and block snapshot. See `docs/ITEM_USE_ON_BLOCK.md` in the workspace.

SDK 0.79.0 adds an owner-scoped synchronous `item.use_on_entity` callback for registered items. It runs after the `interactAt` stage and `player.interaction` veto, before vanilla entity/item interaction. See `docs/ITEM_USE_ON_ENTITY.md` in the workspace.


SDK 0.83.0 adds owner-scoped player boss bars, detached non-player living-entity vitals with bounded healing, and exact-state emitted-light overrides for registered blocks. See `docs/PLAYER_BOSS_BARS.md`, `docs/ENTITY_HEALTH.md`, and `docs/BLOCK_STATE_LIGHT.md` in the workspace.

SDK 0.82.0 adds static geometry overrides for exact Boolean/integer block-state combinations; see `docs/BLOCK_STATE_SHAPES.md` in the workspace.
The state-aware block API also supports exact-state sound, redstone conductor, suffocation, view-blocking, emissive-rendering, and post-process overrides through `registry::set_block_state_behavior`; see `docs/BLOCK_STATE_BEHAVIOR.md` in the workspace. Minecraft 1.21.1 exposes friction, movement factors, lava ignition, piston reaction, and destroy speed as block-wide behavior, so these remain configured by `BlockBehavior`.
SDK 0.81.0 adds bounded Boolean and integer state properties for registered blocks; see `docs/BLOCK_STATE_PROPERTIES.md` in the workspace.
SDK 0.80.0 adds bounded static vanilla and modded attribute modifiers for registered items, including operation and equipment-slot selection. See `docs/ITEM_ATTRIBUTE_MODIFIERS.md` in the workspace.
The block-entity API also supports a bounded typed persistent key/value store for block entities registered by the calling module. It requires separate data read/write grants; see `docs/BLOCK_ENTITY_PERSISTENT_DATA.md` in the workspace.
The world API also supports capability-gated launches of vanilla snowballs and eggs with bounded velocity; see `docs/VANILLA_PROJECTILES.md` in the workspace. Vanilla collision behavior is preserved.

`world::spawn_item` creates one ordinary default item stack from the server's current item registry. It requires `world.read` and `world.write`, accepts a loaded position and a finite velocity bounded to 4 blocks per tick, and has a 16-stack-per-module/tick quota. It does not accept custom NBT or arbitrary data components; see `docs/WORLD_ITEM_SPAWN.md`.

Server-side modules can create one module-owned vanilla boss bar per live player with
`player_bossbar::create`, update it through its `PlayerBossBarId`, and remove it explicitly.
The API requires `entity.read` and `player.bossbar.write`, and cleans bars on logout, module
deactivation, and server stop. See `docs/PLAYER_BOSS_BARS.md` in the workspace.

Server-side modules can set or clear their own literal player-list header and
footer with `player::set_tab_list` and `player::clear_tab_list`. The host
recomposes active SJML module contributions in deterministic order and removes
only the calling module's text on clear or shutdown. Both calls require
`entity.read` and `player.ui.tab_list`; see `docs/PLAYER_TAB_LIST.md` in the
workspace.

SDK 0.87.0 separates permission to change an entity's glowing outline from
permission to change its silent flag. `entity::set_glowing` now requires
`entity.read` and `entity.glowing.write`; `entity::set_silent` continues to use
`entity.state.write`. Existing modules that call `set_glowing` must add the new
capability to both their manifest and the administrator grant. See
`docs/ENTITY_STATE_FLAGS.md` and `docs/PROJECT_CONFIGURATION.md` in the
workspace.
