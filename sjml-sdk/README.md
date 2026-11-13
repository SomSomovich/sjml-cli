# `sjml-sdk`

`sjml-sdk` is the Rust guest API for Somuch Java Mod Loader (SJML). Its Rust
crate name is `sjml`; guest projects normally depend on it like this:

```toml
[dependencies]
sjml-sdk = "0.76.0"
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
the [docs.rs API reference](https://docs.rs/sjml-sdk/0.76.0/sjml/) lists the
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
SDK 0.76.0 adds bounded friction, movement factors, emitted light, lava
ignition, piston reactions, and allowlisted sound groups for registered blocks.
See `docs/WORLD_TAG_MEMBERS.md`, `docs/WORLD_BLOCK_SHAPES.md`, and
`docs/WORLD_BLOCK_BEHAVIOR.md`.
