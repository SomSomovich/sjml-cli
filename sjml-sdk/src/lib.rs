//! Rust guest SDK for SJML ABI v1.
//!
//! Guest crates should target `wasm32-unknown-unknown`. The crate remains
//! natively compilable so its data types and callback logic can be unit-tested;
//! host imports and generated ABI exports are only available on wasm32.
//!
//! Invalid public macro forms are compile-time errors:
//!
//! ```compile_fail
//! use sjml::{mod_init, SjmlResult};
//! #[mod_init]
//! fn invalid(_argument: u32) -> SjmlResult<()> { Ok(()) }
//! ```

//! ```compile_fail
//! use sjml::{mod_register, SjmlResult};
//! #[mod_register]
//! fn invalid(_argument: u32) -> SjmlResult<()> { Ok(()) }
//! ```
//!
//! ```compile_fail
//! use sjml::{event, SjmlResult};
//! #[event(name = "demo.async")]
//! async fn invalid(_: ()) -> SjmlResult<()> { Ok(()) }
//! ```
//!
//! ```compile_fail
//! use sjml::{command, SjmlResult};
//! #[command(name = "demo.no_input")]
//! fn invalid() -> SjmlResult<()> { Ok(()) }
//! ```
//!
//! ```compile_fail
//! use sjml::{service, SjmlResult};
//! #[service(name = "Bad ID")]
//! fn invalid(_: ()) -> SjmlResult<()> { Ok(()) }
//! ```
//!
//! ```compile_fail
//! use sjml::StableEncode;
//! #[derive(StableEncode)]
//! struct MissingVersion { value: u32 }
//! ```
//!
//! ```compile_fail
//! use sjml::StableDecode;
//! #[derive(StableDecode)]
//! #[sjml(version = 1)]
//! enum Unsupported { One }
//! ```
//!
//! ```compile_fail
//! use sjml::EntityHandle;
//! use sjml::WorldHandle;
//! let entity = EntityHandle::new(1, 2, 3, 4).unwrap();
//! let _world: WorldHandle = entity;
//! ```

#![cfg_attr(
    target_arch = "wasm32",
    doc = r#"
The raw callback ABI requires an unsafe call from Rust.

```compile_fail,E0133
use sjml::abi::{allocate_host_input, deallocate_host_input};
let pointer = allocate_host_input(1);
deallocate_host_input(pointer, 1);
deallocate_host_input(pointer, 1);
```

```compile_fail,E0133
sjml::abi::dealloc_response(1, 1);
```

```compile_fail,E0133
sjml::__private::dispatch_payload(0, 1, |_: u8| Ok(()));
```

```compile_fail,E0133
use sjml::{event, SjmlResult};
#[event(name = "demo.read")]
fn read(_: u8) -> SjmlResult<()> { Ok(()) }
__sjml_v1_event_read(0, 1);
```
"#
)]

extern crate self as sjml;

#[cfg(all(target_arch = "wasm32", not(target_os = "unknown")))]
compile_error!("SJML guest modules require wasm32-unknown-unknown; WASI targets are not supported");

pub mod abi;
pub mod batch;
pub mod block_entity;
pub mod chat_messages;
pub mod client_assets;
pub mod client_audio;
pub mod client_camera;
pub mod client_chat;
pub mod client_chat_history;
pub mod client_input;
pub mod client_keybindings;
pub mod client_localization;
pub mod client_music;
pub mod client_notifications;
pub mod client_resources_reloaded;
pub mod client_screen;
pub mod client_sound_started;
pub mod command_suggestions;
pub mod entity;
pub mod entity_attributes;
pub mod entity_data;
pub mod entity_left_level;
pub mod entity_teleport_attempt;
mod error;
pub mod events;
pub mod gl;
pub mod host;
pub mod inventory;
pub mod menu;
pub mod model_animation;
pub mod network;
pub mod player;
pub mod player_advancement;
pub mod player_bossbar;
pub mod player_inventory;
pub mod player_statistics;
pub mod recipe;
pub mod registry;
pub mod render;
pub mod scoreboard;
pub mod screen_input;
mod serialization;
pub mod shield_block_attempt;
pub mod storage;
pub mod task;
pub mod widgets;
pub mod world;
pub mod world_chunk_lifecycle;
pub mod world_save_events;

pub use abi::{
    BlockEntityHandle, BlockEntityTag, BlockHandle, BlockTag, EntityHandle, EntityTag,
    EventSubscriptionHandle, EventSubscriptionTag, GlResourceHandle, GlResourceTag, Handle,
    HandleKind, ItemStackHandle, ItemStackTag, NetworkChannelHandle, NetworkChannelTag,
    PlayerHandle, PlayerTag, RenderResourceHandle, RenderResourceTag, StorageScopeHandle,
    StorageScopeTag, TaskHandle, TaskTag, WorldHandle, WorldTag,
};
pub use batch::{Batch, BatchValue, CallResult};
pub use block_entity::{
    BlockEntity, BlockEntitySnapshot, NbtValue, PersistentDataValue, SignSide, SignTextLines,
    MAX_PERSISTENT_DATA_KEY_BYTES, MAX_PERSISTENT_DATA_STRING_BYTES, MAX_SIGN_TEXT_LINE_BYTES,
    MAX_SIGN_TEXT_RESPONSE_BYTES,
};
pub use chat_messages::{
    ChatClickAction, ChatMessage, ChatMessageAction, ChatMessageSource,
    ChatMessageSubscriptionHandle, ChatSpan, ChatText, MAX_CHAT_INTERACTION_BYTES,
    MAX_CHAT_INTERACTION_VALUE_BYTES, MAX_CHAT_TEXT_SPANS,
};
pub use client_assets::{
    list as list_client_assets, ClientAssetList, CLIENT_ASSETS_READ_CAPABILITY,
    MAX_CLIENT_ASSET_PATH_BYTES, MAX_CLIENT_ASSET_PREFIX_BYTES, MAX_CLIENT_ASSET_RESPONSE_BYTES,
    MAX_CLIENT_ASSET_RESULTS,
};
pub use client_audio::SoundCategory;
pub use client_camera::{
    clear_fov_scale, set_fov_scale, CLIENT_CAMERA_FOV_CAPABILITY, MAX_FOV_SCALE, MIN_FOV_SCALE,
};
pub use client_chat::{ChatLayout, ChatLayoutHandle, ChatScreenEvent, ChatTheme};
pub use client_chat_history::{
    read_recent as read_recent_client_chat_history, ChatHistoryEntry, ChatHistorySnapshot,
    CLIENT_CHAT_HISTORY_READ_CAPABILITY, MAX_CLIENT_CHAT_HISTORY_MESSAGES,
    MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES, MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES,
};
pub use client_input::{
    cursor_position, is_key_down, is_mouse_button_down, subscribe_cursor_motion,
    subscribe_cursor_moved, subscribe_global_keys, unsubscribe_cursor_motion,
    unsubscribe_cursor_moved, unsubscribe_global_keys, ClientCursorMotionEvent,
    ClientCursorMovedEvent, ClientInputAction, ClientInputEvent, ClientInputSubscriptionHandle,
    CursorMotionSubscriptionHandle, CursorMovedSubscriptionHandle, CursorPosition,
    GlobalKeyObservation, GlobalKeyObservationHandle, MouseScrollEvent,
    MouseScrollSubscriptionHandle, MAX_CURSOR_COORDINATE,
    MAX_CURSOR_MOTION_CALLBACKS_PER_MODULE_PER_TICK, MAX_CURSOR_MOTION_COORDINATE,
    MAX_CURSOR_MOTION_DELTA, MAX_CURSOR_MOTION_SUBSCRIPTIONS,
    MAX_CURSOR_MOTION_SUBSCRIPTIONS_PER_MODULE, MAX_CURSOR_MOVED_DELTA,
    MAX_CURSOR_MOVED_SUBSCRIPTIONS, MAX_CURSOR_MOVED_SUBSCRIPTIONS_PER_MODULE, MAX_GLFW_KEY_CODE,
    MAX_MOUSE_BUTTON,
};
pub use client_keybindings::{
    Context as KeybindingContext, Key as KeybindingKey, KeybindingDefinition,
    KeybindingSubscriptionHandle, Modifier as KeybindingModifier,
    MAX_CLIENT_KEYBINDING_SUBSCRIPTIONS,
};
pub use client_localization::{
    translate as translate_client_text, CLIENT_LOCALIZATION_READ_CAPABILITY, MAX_TRANSLATION_BYTES,
    MAX_TRANSLATION_KEY_BYTES,
};
pub use client_music::{
    subscribe_music_selection, unsubscribe_music_selection, ClientMusicSelectionEvent,
    MusicSelectionDecision, CLIENT_MUSIC_CAPABILITY, MAX_MUSIC_DELAY_TICKS,
    MAX_MUSIC_SOUND_ID_BYTES,
};
pub use client_notifications::NotificationKind;
pub use client_resources_reloaded::{
    ClientResourcesReloaded, EVENT_NAME as CLIENT_RESOURCES_RELOADED_EVENT_NAME,
    MAX_GENERATION as MAX_CLIENT_RESOURCE_RELOAD_GENERATION,
};
pub use client_screen::{
    ClientScreenButton, ClientScreenCheckbox, ClientScreenDefinition, ClientScreenFormDefinition,
    ClientScreenListFormDefinition, ClientScreenMultiSelectFormDefinition,
    ClientScreenMultiSelectionList, ClientScreenNumericSlider, ClientScreenProgressBar,
    ClientScreenSelectionList, ClientScreenSelectionOption, ClientScreenSettingsFormDefinition,
    ClientScreenSliderFormDefinition, ClientScreenTextField, ClientScreenWithProgressBars,
    MAX_CLIENT_SCREEN_PROGRESS_BARS, MAX_CLIENT_SCREEN_PROGRESS_BAR_ABS_VALUE,
};
pub use client_sound_started::{
    ClientSoundCategory, ClientSoundStarted, EVENT_NAME as CLIENT_SOUND_STARTED_EVENT_NAME,
    MAX_CALLBACKS_PER_TICK as MAX_CLIENT_SOUND_STARTED_CALLBACKS_PER_TICK,
    MAX_PAYLOAD_BYTES as MAX_CLIENT_SOUND_STARTED_PAYLOAD_BYTES,
    MAX_SOUND_ID_BYTES as MAX_CLIENT_SOUND_ID_BYTES,
    OBSERVE_CAPABILITY as CLIENT_AUDIO_OBSERVE_CAPABILITY,
};
pub use command_suggestions::{
    ClientCommandSuggestionRequest, CommandSuggestionArgument, CommandSuggestionRequest,
    CommandSuggestions, CLIENT_COMMAND_SUGGEST_CAPABILITY,
    MAX_CLIENT_COMMAND_SUGGESTION_PARTIAL_BYTES, MAX_COMMAND_SUGGESTIONS,
    MAX_COMMAND_SUGGESTION_BYTES, MAX_COMMAND_SUGGESTION_PARTIAL_BYTES,
};
pub use entity::{
    add_villager_offer, age_ticks, clear_equipment, equipment_drop_chance, is_baby,
    remove_villager_offer, set_age_ticks, set_baby, set_equipment_drop_chance,
    set_villager_offer_special_price, set_villager_offer_uses, try_equip_default,
    update_villager_offer_demand, villager_offers, ActiveEffect, EntityAirSupply, EntityBounds,
    EntityEquipment, EntityEquipmentSlot, EntityEquipmentStack, EntityOrientation, EntityRotation,
    EntitySnapshot, EntityStateSnapshot, EntityTargetSnapshot, EntityVelocity, EntityVitals,
    VillagerOffer, VillagerOfferItem, VillagerTradeOfferDefinition, AGE_WRITE_CAPABILITY,
    AIR_SUPPLY_WRITE_CAPABILITY, AI_CONTROL_CAPABILITY, EQUIPMENT_DROP_CHANCE_WRITE_CAPABILITY,
    HEALTH_WRITE_CAPABILITY, INVULNERABILITY_WRITE_CAPABILITY, LOOT_PICKUP_WRITE_CAPABILITY,
    MAX_ACTIVE_ENTITY_EFFECTS, MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES,
    MAX_ENTITY_AGE_RESPONSE_BYTES, MAX_ENTITY_AGE_TICKS, MAX_ENTITY_AIR_SUPPLY,
    MAX_ENTITY_AIR_SUPPLY_RESPONSE_BYTES, MAX_ENTITY_BOUNDS_COORDINATE,
    MAX_ENTITY_BOUNDS_RESPONSE_BYTES, MAX_ENTITY_EFFECT_ID_BYTES, MAX_ENTITY_EQUIPMENT_DAMAGE,
    MAX_ENTITY_EQUIPMENT_DROP_CHANCE_RESPONSE_BYTES, MAX_ENTITY_EQUIPMENT_ITEM_ID_BYTES,
    MAX_ENTITY_EQUIPMENT_RESPONSE_BYTES, MAX_ENTITY_EQUIPMENT_STACK_COUNT,
    MAX_ENTITY_HEAD_YAW_DEGREES, MAX_ENTITY_HEAL_AMOUNT, MAX_ENTITY_NAVIGATION_RANGE,
    MAX_ENTITY_NAVIGATION_SPEED, MAX_ENTITY_PITCH_DEGREES, MAX_ENTITY_STATE_RESPONSE_BYTES,
    MAX_ENTITY_TARGET_COORDINATE, MAX_ENTITY_TARGET_ID_BYTES, MAX_ENTITY_TARGET_RANGE,
    MAX_ENTITY_TARGET_RESPONSE_BYTES, MAX_ENTITY_TARGET_SNAPSHOT_RESPONSE_BYTES,
    MAX_ENTITY_TARGET_UUID_BYTES, MAX_ENTITY_TELEPORT_COORDINATE, MAX_ENTITY_VELOCITY_COMPONENT,
    MAX_ENTITY_VITALS_VALUE, MAX_ENTITY_YAW_DEGREES, MAX_SPATIAL_QUERY_RADIUS,
    MAX_SPATIAL_QUERY_RESULTS, MAX_VILLAGER_RUNTIME_OFFERS_PER_MERCHANT,
    MAX_VILLAGER_RUNTIME_OFFERS_PER_MODULE, MAX_VILLAGER_RUNTIME_OFFER_DEMAND,
    MAX_VILLAGER_RUNTIME_OFFER_ITEM_COUNT, MAX_VILLAGER_RUNTIME_OFFER_PATH_BYTES,
    MAX_VILLAGER_RUNTIME_OFFER_PRICE_MULTIPLIER, MAX_VILLAGER_RUNTIME_OFFER_REQUEST_BYTES,
    MAX_VILLAGER_RUNTIME_OFFER_XP, MAX_VILLAGER_TRADE_DEMAND, MAX_VILLAGER_TRADE_ID_BYTES,
    MAX_VILLAGER_TRADE_ITEM_COUNT, MAX_VILLAGER_TRADE_OFFERS, MAX_VILLAGER_TRADE_RESPONSE_BYTES,
    MAX_VILLAGER_TRADE_SPECIAL_PRICE, MAX_VILLAGER_TRADE_USES, MIN_ENTITY_AGE_TICKS,
    MIN_ENTITY_AIR_SUPPLY, NAVIGATION_WRITE_CAPABILITY, PERSISTENCE_WRITE_CAPABILITY,
    ROTATION_WRITE_CAPABILITY, TARGET_WRITE_CAPABILITY, TELEPORT_CAPABILITY,
    VILLAGER_TRADE_WRITE_CAPABILITY,
};
pub use entity_attributes::{
    AttributeValues, LivingAttribute, MAX_ATTRIBUTE_MODIFIER_KEY_BYTES,
    MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT,
};
pub use entity_left_level::{
    EntityLeftLevel, ENTITY_LEFT_LEVEL_EVENT_NAME, MAX_ENTITY_LEFT_LEVEL_CALLBACKS,
    MAX_ENTITY_LEFT_LEVEL_PAYLOAD_BYTES, MAX_ENTITY_LEFT_LEVEL_STRING_BYTES,
};
pub use entity_teleport_attempt::{
    EntityTeleportAttempt, EntityTeleportAttemptAction, EntityTeleportCause,
    ENTITY_TELEPORT_ATTEMPT_CANCEL_CAPABILITY, MAX_ENTITY_TELEPORT_ATTEMPT_COORDINATE,
    MAX_ENTITY_TELEPORT_ATTEMPT_ID_BYTES,
};
pub use error::{ErrorCode, SjmlError, SjmlResult};
pub use events::BlockGrowAttempt;
pub use events::EntityBreedAttempt;
pub use events::PlayerAttackAttempt;
pub use events::PlayerItemPickupAttempt;
pub use events::PlayerStatisticChanged;
pub use events::{AdvancementProgressOperation, PlayerAdvancementProgressEvent};
pub use events::{BlockDropsAttempt, BlockDropsBreakerSnapshot, BlockDropsItemSnapshot};
pub use events::{
    BlockFarmlandTrampleAttempt, BlockFertilizeAttempt, BlockFireSpreadAttempt,
    BlockFireSpreadCause, BlockFluidPlaceAttempt, BlockRandomTick, BlockTick, BlockTickProperty,
};
pub use events::{
    BlockUseAction, ClientAction, ClientActionAttempt, ClientActionHand, ClientChatSendAttempt,
    ClientItemTooltip, ClientScreenAction, ClientScreenFormSubmit, ClientScreenFormValue,
    ClientScreenMultiSelectFormSubmit, ClientTick, EntityDropsAttempt, EntityDropsItemSnapshot,
    EntityFallAttempt, EntityKnockbackAttempt, EntityMountAttempt, EntityMountTarget, EntityTick,
    ItemTooltipLines, ItemUseAction, ModuleBlockUse, ModuleItemAttackAttempt, ModuleItemUse,
    ModuleItemUseOnBlock, ModuleItemUseOnEntity, PlayerItemBroken, PlayerItemCrafted,
    PlayerItemUseAttempt, PlayerPotionUseAttempt, PlayerPotionUseAttemptAction, PlayerSleepAttempt,
    PlayerTick, PlayerVillagerTraded, WorldTick, CLIENT_ACTION_CANCEL_CAPABILITY,
    MAX_CLIENT_ACTION_ATTEMPT_CALLBACKS, MAX_PLAYER_EQUIPMENT_ITEM_COUNT,
    MAX_PLAYER_POTION_USE_ATTEMPT_CALLBACKS, MAX_PLAYER_POTION_USE_DURATION_TICKS,
    MAX_PLAYER_POTION_USE_ITEM_COUNT, MAX_PLAYER_STATISTIC_CHANGED_CALLBACKS,
    MAX_PLAYER_STATISTIC_ID_BYTES, PLAYER_ADVANCEMENT_OBSERVE_CAPABILITY,
    PLAYER_EQUIPMENT_OBSERVE_CAPABILITY, PLAYER_STATISTIC_OBSERVE_CAPABILITY,
};
pub use events::{
    FishedDropSnapshot, PlayerFishedAttempt, MAX_FISHED_DROPS, MAX_FISHED_ITEM_COUNT,
};
pub use events::{
    PlayerBreakSpeed, PlayerBreakSpeedAction, MAX_PLAYER_BREAK_SPEED,
    MAX_PLAYER_BREAK_SPEED_CALLBACKS, PLAYER_BREAK_SPEED_MODIFY_CAPABILITY,
};
pub use events::{
    PlayerEnchanted, PlayerEnchantedEnchantment, MAX_PLAYER_ENCHANTED_CALLBACKS,
    MAX_PLAYER_ENCHANTED_ENCHANTMENTS, MAX_PLAYER_ENCHANTED_ID_BYTES,
    MAX_PLAYER_ENCHANTED_ITEM_VALUE, MAX_PLAYER_ENCHANTED_LEVEL,
    MAX_PLAYER_ENCHANTED_PAYLOAD_BYTES, PLAYER_ENCHANTED_EVENT_NAME,
    PLAYER_ENCHANTMENTS_OBSERVE_CAPABILITY,
};
pub use events::{PlayerEquipmentChanged, PlayerEquipmentItemSnapshot, PlayerEquipmentSlot};
pub use events::{
    PlayerExperienceChangeAction, PlayerExperienceChangeAttempt,
    MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT, MAX_PLAYER_EXPERIENCE_LEVEL,
    MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT,
};
pub use events::{
    PlayerExperienceLevelChangeAction, PlayerExperienceLevelChangeAttempt,
    MAX_PLAYER_EXPERIENCE_LEVEL_DELTA, MAX_PLAYER_EXPERIENCE_LEVEL_SNAPSHOT,
    MIN_PLAYER_EXPERIENCE_LEVEL_DELTA,
};
pub use events::{
    BLOCK_FIRE_SPREAD_ATTEMPT_EVENT_NAME, MAX_BLOCK_FIRE_SPREAD_ATTEMPT_AGE,
    MAX_BLOCK_FIRE_SPREAD_ATTEMPT_CALLBACKS,
    MAX_BLOCK_FIRE_SPREAD_ATTEMPT_CALLBACKS_PER_SERVER_TICK,
};
pub use events::{
    MAX_BLOCK_DROPS_ATTEMPT_CALLBACKS, MAX_BLOCK_DROPS_ATTEMPT_DROPS,
    MAX_BLOCK_DROPS_ATTEMPT_DURABILITY, MAX_BLOCK_DROPS_ATTEMPT_EXPERIENCE,
    MAX_BLOCK_DROPS_ATTEMPT_ITEM_COUNT, MAX_BLOCK_DROPS_ATTEMPT_PAYLOAD_BYTES,
};
pub use gl::{
    GlBlendFactor, GlBufferTarget, GlBufferUsage, GlCapabilities, GlCommand, GlCullFace,
    GlDepthFunction, GlExecutionReport, GlFramebufferAttachment, GlIndexType, GlPrimitiveMode,
    GlResourceDescriptor, GlScalarType,
};
pub use inventory::{
    BlockPos, Direction, InventoryEndpoint, InventorySnapshot, ItemStackDescriptor,
    ItemStackSnapshot, MAX_ITEM_TRANSFER_COUNT,
};
pub use menu::{
    EditableMenuDefinition, EditableMenuEndpoint, EditableMenuSlot, EditableMenuStack, MenuAction,
    MenuDefinition, MenuItem,
};
pub use model_animation::{
    register_block_entity_model_keyframe_track,
    register_block_entity_model_transform_keyframe_track, register_entity_model_keyframe_track,
    register_entity_model_transform_keyframe_track, ModelAnimationTarget, ModelKeyframe,
    ModelKeyframeTrack, ModelTransformKeyframe, ModelTransformKeyframeTrack, ModelTransformKind,
    MAX_MODEL_KEYFRAMES, MAX_MODEL_KEYFRAME_DURATION_TICKS, MAX_MODEL_SCALE, MIN_MODEL_SCALE,
};
pub use network::{
    ChannelRef, LogicalSide, NetworkContext, NetworkContextSnapshot, NetworkDirection,
    NetworkSubscriptionHandle, Player as NetworkPlayer, Uuid,
};
pub use player::{
    active_effects, air_supply as player_air_supply, clear_respawn_point,
    clear_team as clear_player_team, custom_stat, grant_advancement, increment_custom_stat,
    movement_state as player_movement_state, reset_fall_distance as reset_player_fall_distance,
    respawn_point, revoke_advancement, selected_hotbar_slot, send_rich_system_message,
    send_system_message, set_air_supply as set_player_air_supply, set_custom_stat, set_flying,
    set_flying_speed, set_instabuild, set_invulnerable, set_may_build, set_may_fly,
    set_respawn_point, set_rotation, set_selected_hotbar_slot, set_team as set_player_team,
    set_walking_speed, team as player_team, PlayerAbilities, PlayerAirSupply, PlayerCustomStat,
    PlayerEquipment, PlayerEquipmentStack, PlayerGameMode, PlayerMotion, PlayerMovementState,
    PlayerRespawnPoint, PlayerSnapshot, PlayerTitleTimings, PlayerVelocity, PlayerVitals, RichText,
    RichTextClickAction, RichTextColor, RichTextNode,
    AIR_SUPPLY_WRITE_CAPABILITY as PLAYER_AIR_SUPPLY_WRITE_CAPABILITY,
    FALL_DISTANCE_WRITE_CAPABILITY as PLAYER_FALL_DISTANCE_WRITE_CAPABILITY,
    MAX_PLAYER_ABILITY_SPEED, MAX_PLAYER_ACTIVE_EFFECTS, MAX_PLAYER_ACTIVE_EFFECTS_RESPONSE_BYTES,
    MAX_PLAYER_ADVANCEMENT_ID_BYTES, MAX_PLAYER_FALL_DISTANCE, MAX_PLAYER_HOTBAR_SLOT_INDEX,
    MAX_PLAYER_RESPAWN_ANGLE, MAX_PLAYER_RESPAWN_COORDINATE, MAX_PLAYER_RESPAWN_DIMENSION_ID_BYTES,
    MAX_PLAYER_RESPAWN_POINT_BYTES, MAX_PLAYER_RESPAWN_SNAPSHOT_BYTES,
    MAX_PLAYER_RICH_SYSTEM_MESSAGE_DEPTH, MAX_PLAYER_RICH_SYSTEM_MESSAGE_NODES,
    MAX_PLAYER_RICH_SYSTEM_MESSAGE_PAYLOAD_BYTES, MAX_PLAYER_RICH_SYSTEM_MESSAGE_STRING_BYTES,
    MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES, MAX_PLAYER_ROTATION_YAW,
    MAX_PLAYER_STATISTIC_INCREMENT, MAX_PLAYER_STATISTIC_VALUE,
    MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES, MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS,
    MAX_PLAYER_TEAM_NAME_BYTES, MAX_PLAYER_TEAM_RESPONSE_BYTES, MAX_PLAYER_TITLE_TEXT_BYTES,
    MAX_PLAYER_TITLE_TEXT_CODE_POINTS, MAX_PLAYER_TITLE_TIMING_TICKS,
    MAX_PLAYER_VELOCITY_COMPONENT, MIN_PLAYER_ABILITY_SPEED,
};
pub use player_advancement::{
    grant_criterion as grant_advancement_criterion, progress as player_advancement_progress,
    revoke_criterion as revoke_advancement_criterion, PlayerAdvancementCriterion,
    PlayerAdvancementProgress, MAX_PLAYER_ADVANCEMENT_CRITERIA,
    MAX_PLAYER_ADVANCEMENT_CRITERION_ID_BYTES, MAX_PLAYER_ADVANCEMENT_PROGRESS_BYTES,
};
pub use player_inventory::{
    clear_item_glint_override as clear_player_inventory_item_glint_override,
    give_item as give_player_item, move_stack as move_player_inventory_stack,
    set_count as set_player_inventory_count,
    set_item_glint_override as set_player_inventory_item_glint_override, PlayerInventorySlotUpdate,
    MAX_PLAYER_INVENTORY_GIVE_COUNT, MAX_PLAYER_INVENTORY_ITEM_ID_BYTES,
    MAX_PLAYER_INVENTORY_MOVE_COUNT, MAX_PLAYER_INVENTORY_SLOTS, MAX_PLAYER_INVENTORY_SLOT_COUNT,
    MAX_PLAYER_INVENTORY_SLOT_INDEX,
};
pub use player_statistics::{
    increment as increment_player_statistic, read as player_statistic,
    set_value as set_player_statistic_value, PlayerStatistic, StatisticFamily,
    MAX_INCREMENT as MAX_PLAYER_REGISTRY_STATISTIC_INCREMENT, MAX_STATISTIC_ID_BYTES,
    MAX_VALUE as MAX_PLAYER_REGISTRY_STATISTIC_VALUE,
};
pub use recipe::{
    CraftingSlot, RecipeEvaluation, RecipeMatchInput, RECIPE_CALLBACK_SCHEMA_VERSION,
};
pub use registry::{
    register_fluid_with_type_properties, register_fluid_with_visuals_and_type_properties,
    set_block_state_behavior, set_item_fire_resistant, set_item_glint_override, set_item_no_repair,
    set_item_repair_tag, BlockAabb, BlockBehavior, BlockEntityModelDefinition, BlockPushReaction,
    BlockShape, BlockShapeBoxes, BlockSound, BlockStateBehavior, BlockStateProperty,
    EntityModelDefinition, FluidTypeProperties, IntegerComponentDefinition, ItemFood,
    ItemFoodEffect, ItemFoodEffects, ItemRarity, ModelAnimation, ModelAnimationAxis, ModelCuboid,
    OreGenerationDefinition, OreReplaceableBlocks, VillagerTradeItem, VillagerTradeOffer,
    MAX_BLOCK_SHAPE_BOXES, MAX_BOUND_MODEL_CUBOIDS, MAX_ITEM_FOOD_EFFECTS,
    MAX_ITEM_FOOD_EFFECT_DURATION_TICKS, MAX_ITEM_FOOD_EFFECT_ID_BYTES,
    MAX_ITEM_REPAIR_TAG_ID_BYTES,
};
pub use render::{
    suppress_gui_layer, unsuppress_gui_layer, GuiLayerSuppressionHandle, RenderContext, RenderFont,
    RenderFrame, RenderLayer, RenderSubscriptionHandle, WorldModelBuilder, WorldRenderFrame,
    MAX_GUI_LAYER_ID_BYTES, MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS,
    MAX_WORLD_MODEL_VERTICES,
};
pub use scoreboard::{
    clear_display_slot as scoreboard_clear_display_slot,
    create_objective as scoreboard_create_objective, objective_score as scoreboard_objective_score,
    remove_objective as scoreboard_remove_objective, score as scoreboard_score,
    set_display_slot as scoreboard_set_display_slot,
    set_objective_display_slot as scoreboard_set_objective_display_slot,
    set_objective_score as scoreboard_set_objective_score, set_score as scoreboard_set_score,
    DisplaySlot, DISPLAY_SLOT_COUNT, MAX_OWNED_OBJECTIVE_DISPLAY_BYTES,
    MAX_OWNED_OBJECTIVE_DISPLAY_CODE_POINTS, MAX_OWNED_OBJECTIVE_ID_BYTES,
    MAX_SCOREBOARD_ENTRY_BYTES, MAX_SCOREBOARD_OBJECTIVE_BYTES, MAX_SCORE_VALUE,
};
pub use screen_input::{
    subscribe_mouse_dragged, subscribe_mouse_released, unsubscribe_key_pressed,
    unsubscribe_mouse_dragged, unsubscribe_mouse_released, ScreenInputEvent,
    ScreenInputSubscriptionHandle, ScreenKeyPressedEvent, ScreenKeyPressedSubscriptionHandle,
    ScreenMouseDraggedEvent, ScreenMouseDraggedSubscriptionHandle, ScreenMouseReleasedEvent,
    ScreenMouseReleasedSubscriptionHandle, MAX_SCREEN_DRAG_DELTA, MAX_SCREEN_SCROLL_DELTA,
};
pub use serialization::{
    from_bytes, to_bytes, Decoder, Encoder, StableDecode, StableEncode, MAX_COLLECTION_ITEMS,
    MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
};
pub use shield_block_attempt::{EntityShieldBlockAttempt, EntityShieldBlockAttemptAction};
pub use sjml_macros::{
    client_command, client_command_suggestions, client_music_selection, command,
    command_suggestions, compute, event, event_async, mod_init, mod_register, mod_shutdown, recipe,
    service,
};
pub use sjml_macros::{StableDecode, StableEncode};
pub use storage::{
    MigrationFailure, MigrationMode, MigrationReport, StableRecord, StorageError, StorageResult,
    StorageScope,
};
pub use task::GuestTaskId;
pub use widgets::{
    Button, ProgressBar, TextField, TextFieldAction, Tooltip, WidgetRect, MAX_TEXT_FIELD_BYTES,
};
pub use world::{
    block_collision_boxes, block_destroy_speed, fluid_flow_at, spawn_particles,
    BiomeClimateSnapshot, BiomePrecipitation, BiomeTemperatureModifier, BlockCollisionBox,
    BlockProperty, BlockStateSnapshot, BlockUpdateOptions, ChunkPos, FluidFlowSnapshot,
    FluidStateSnapshot, HeightmapType, ItemSpawnVelocity, LightSnapshot, ParticleSpawnOptions,
    TagRegistry, WorldBorderSnapshot, WorldDifficulty, WorldGameRule, WorldIntegerGameRule,
    WorldSpawnPoint, MAX_WORLD_BIOME_CLIMATE_RESPONSE_BYTES, MAX_WORLD_BIOME_IDS,
    MAX_WORLD_BIOME_IDS_RESPONSE_BYTES, MAX_WORLD_BIOME_ID_BYTES, MAX_WORLD_BLOCK_COLLISION_BOXES,
    MAX_WORLD_BLOCK_COLLISION_BOXES_RESPONSE_BYTES, MAX_WORLD_BLOCK_COORDINATE,
    MAX_WORLD_BLOCK_DESTROY_SPEED, MAX_WORLD_BLOCK_DESTROY_SPEED_RESPONSE_BYTES,
    MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES, MAX_WORLD_BLOCK_PROPERTY_UPDATES,
    MAX_WORLD_BLOCK_STATES, MAX_WORLD_BLOCK_TICK_DELAY_TICKS, MAX_WORLD_BORDER_CENTER,
    MAX_WORLD_BORDER_DAMAGE_BUFFER, MAX_WORLD_BORDER_DAMAGE_PER_BLOCK, MAX_WORLD_BORDER_DIAMETER,
    MAX_WORLD_BORDER_LERP_MILLIS, MAX_WORLD_BORDER_WARNING_BLOCKS,
    MAX_WORLD_BORDER_WARNING_SECONDS, MAX_WORLD_CHUNKS, MAX_WORLD_CHUNKS_RESPONSE_BYTES,
    MAX_WORLD_CHUNK_RADIUS, MAX_WORLD_FLUID_FLOW_COMPONENT, MAX_WORLD_FLUID_FLOW_LENGTH,
    MAX_WORLD_FLUID_FLOW_RESPONSE_BYTES, MAX_WORLD_FLUID_STATE_RESPONSE_BYTES,
    MAX_WORLD_GAME_TIME_TICKS, MAX_WORLD_ITEM_SPAWNS_PER_TICK, MAX_WORLD_ITEM_SPAWN_SPEED,
    MAX_WORLD_ITEM_STACK_COUNT, MAX_WORLD_PARTICLES_PER_CALL, MAX_WORLD_PARTICLES_PER_TICK,
    MAX_WORLD_PARTICLE_SPEED, MAX_WORLD_PARTICLE_SPREAD, MAX_WORLD_REDSTONE_SIGNAL,
    MAX_WORLD_SPAWN_POINT_BYTES, MAX_WORLD_TAG_ID_BYTES, MAX_WORLD_TAG_MEMBERS,
    MAX_WORLD_TAG_MEMBERS_OUTPUT_BYTES, MAX_WORLD_TAG_MEMBERS_QUERY_BYTES,
    MAX_WORLD_TAG_MEMBERS_RESPONSE_BYTES, WORLD_BLOCK_UPDATE_ALLOWED_MASK,
};
pub use world_chunk_lifecycle::{
    WorldChunkLifecycle, MAX_WORLD_CHUNK_COORDINATE, MAX_WORLD_CHUNK_DIMENSION_ID_BYTES,
    MAX_WORLD_CHUNK_LIFECYCLE_CALLBACKS, MAX_WORLD_CHUNK_LIFECYCLE_PAYLOAD_BYTES,
    WORLD_CHUNK_LOADED_EVENT_NAME, WORLD_CHUNK_UNLOADED_EVENT_NAME,
};
pub use world_save_events::{
    WorldSaveObserved, MAX_WORLD_SAVE_CALLBACKS_PER_EVENT, MAX_WORLD_SAVE_DIMENSION_ID_BYTES,
    MAX_WORLD_SAVE_GAME_TIME_TICKS, MAX_WORLD_SAVE_OBSERVED_PAYLOAD_BYTES,
    WORLD_SAVE_OBSERVED_EVENT_NAME,
};

/// Common imports for an SJML guest module.
pub mod prelude {
    pub use crate::abi::{
        BlockEntityHandle, BlockHandle, EntityHandle, EventSubscriptionHandle, GlResourceHandle,
        ItemStackHandle, NetworkChannelHandle, PlayerHandle, RenderResourceHandle,
        StorageScopeHandle, TaskHandle, WorldHandle,
    };
    pub use crate::batch::{self, Batch, BatchValue, CallResult};
    pub use crate::block_entity::{
        self, BlockEntity, BlockEntitySnapshot, NbtValue, PersistentDataValue, SignSide,
        SignTextLines, MAX_PERSISTENT_DATA_KEY_BYTES, MAX_PERSISTENT_DATA_STRING_BYTES,
        MAX_SIGN_TEXT_LINE_BYTES, MAX_SIGN_TEXT_RESPONSE_BYTES,
    };
    pub use crate::chat_messages;
    pub use crate::chat_messages::{
        ChatMessage, ChatMessageAction, ChatMessageSource, ChatMessageSubscriptionHandle,
    };
    pub use crate::client_assets;
    pub use crate::client_assets::{
        list as list_client_assets, ClientAssetList, CLIENT_ASSETS_READ_CAPABILITY,
        MAX_CLIENT_ASSET_PATH_BYTES, MAX_CLIENT_ASSET_PREFIX_BYTES,
        MAX_CLIENT_ASSET_RESPONSE_BYTES, MAX_CLIENT_ASSET_RESULTS,
    };
    pub use crate::client_audio;
    pub use crate::client_audio::SoundCategory;
    pub use crate::client_camera;
    pub use crate::client_camera::{
        clear_fov_scale, set_fov_scale, CLIENT_CAMERA_FOV_CAPABILITY, MAX_FOV_SCALE, MIN_FOV_SCALE,
    };
    pub use crate::client_chat::{self, ChatLayout, ChatLayoutHandle, ChatScreenEvent, ChatTheme};
    pub use crate::client_chat_history;
    pub use crate::client_chat_history::{
        read_recent, ChatHistoryEntry, ChatHistorySnapshot, CLIENT_CHAT_HISTORY_READ_CAPABILITY,
        MAX_CLIENT_CHAT_HISTORY_MESSAGES, MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES,
        MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES,
    };
    pub use crate::client_input;
    pub use crate::client_input::{
        cursor_position, is_key_down, is_mouse_button_down, subscribe_cursor_motion,
        subscribe_cursor_moved, subscribe_global_keys, subscribe_scroll, unsubscribe_cursor_motion,
        unsubscribe_cursor_moved, unsubscribe_global_keys, unsubscribe_scroll,
        ClientCursorMotionEvent, ClientCursorMovedEvent, ClientInputAction, ClientInputEvent,
        ClientInputSubscriptionHandle, CursorMotionSubscriptionHandle,
        CursorMovedSubscriptionHandle, CursorPosition, GlobalKeyObservation,
        GlobalKeyObservationHandle, MouseScrollEvent, MouseScrollSubscriptionHandle,
        MAX_CLIENT_INPUT_SUBSCRIPTIONS, MAX_CURSOR_COORDINATE,
        MAX_CURSOR_MOTION_CALLBACKS_PER_MODULE_PER_TICK, MAX_CURSOR_MOTION_COORDINATE,
        MAX_CURSOR_MOTION_DELTA, MAX_CURSOR_MOTION_SUBSCRIPTIONS,
        MAX_CURSOR_MOTION_SUBSCRIPTIONS_PER_MODULE, MAX_CURSOR_MOVED_DELTA,
        MAX_CURSOR_MOVED_SUBSCRIPTIONS, MAX_CURSOR_MOVED_SUBSCRIPTIONS_PER_MODULE,
        MAX_GLFW_KEY_CODE, MAX_MOUSE_BUTTON,
    };
    pub use crate::client_keybindings;
    pub use crate::client_keybindings::{
        Context as KeybindingContext, Key as KeybindingKey, KeybindingDefinition,
        KeybindingSubscriptionHandle, Modifier as KeybindingModifier,
        MAX_CLIENT_KEYBINDING_SUBSCRIPTIONS,
    };
    pub use crate::client_localization;
    pub use crate::client_localization::{
        translate as translate_client_text, CLIENT_LOCALIZATION_READ_CAPABILITY,
        MAX_TRANSLATION_BYTES, MAX_TRANSLATION_KEY_BYTES,
    };
    pub use crate::client_music;
    pub use crate::client_music::{
        subscribe_music_selection, unsubscribe_music_selection, ClientMusicSelectionEvent,
        MusicSelectionDecision, CLIENT_MUSIC_CAPABILITY, MAX_MUSIC_DELAY_TICKS,
        MAX_MUSIC_SOUND_ID_BYTES,
    };
    pub use crate::client_notifications;
    pub use crate::client_notifications::NotificationKind;
    pub use crate::client_resources_reloaded;
    pub use crate::client_resources_reloaded::{
        ClientResourcesReloaded, EVENT_NAME as CLIENT_RESOURCES_RELOADED_EVENT_NAME,
        MAX_GENERATION as MAX_CLIENT_RESOURCE_RELOAD_GENERATION,
    };
    pub use crate::client_screen;
    pub use crate::client_screen::{
        ClientScreenButton, ClientScreenCheckbox, ClientScreenDefinition,
        ClientScreenFormDefinition, ClientScreenListFormDefinition,
        ClientScreenMultiSelectFormDefinition, ClientScreenMultiSelectionList,
        ClientScreenNumericSlider, ClientScreenProgressBar, ClientScreenSelectionList,
        ClientScreenSelectionOption, ClientScreenSettingsFormDefinition,
        ClientScreenSliderFormDefinition, ClientScreenTextField, ClientScreenWithProgressBars,
        MAX_CLIENT_SCREEN_PROGRESS_BARS, MAX_CLIENT_SCREEN_PROGRESS_BAR_ABS_VALUE,
    };
    pub use crate::client_sound_started;
    pub use crate::client_sound_started::{
        ClientSoundCategory, ClientSoundStarted, EVENT_NAME as CLIENT_SOUND_STARTED_EVENT_NAME,
        MAX_CALLBACKS_PER_TICK as MAX_CLIENT_SOUND_STARTED_CALLBACKS_PER_TICK,
        MAX_PAYLOAD_BYTES as MAX_CLIENT_SOUND_STARTED_PAYLOAD_BYTES,
        MAX_SOUND_ID_BYTES as MAX_CLIENT_SOUND_ID_BYTES,
        OBSERVE_CAPABILITY as CLIENT_AUDIO_OBSERVE_CAPABILITY,
    };
    pub use crate::command_suggestions::{
        ClientCommandSuggestionRequest, CommandSuggestionArgument, CommandSuggestionRequest,
        CommandSuggestions, CLIENT_COMMAND_SUGGEST_CAPABILITY,
        MAX_CLIENT_COMMAND_SUGGESTION_PARTIAL_BYTES, MAX_COMMAND_SUGGESTIONS,
        MAX_COMMAND_SUGGESTION_BYTES, MAX_COMMAND_SUGGESTION_PARTIAL_BYTES,
    };
    pub use crate::entity;
    pub use crate::entity::{
        add_villager_offer, clear_equipment, equipment_drop_chance, remove_villager_offer,
        set_equipment_drop_chance, set_villager_offer_special_price, set_villager_offer_uses,
        try_equip_default, update_villager_offer_demand, villager_offers, ActiveEffect,
        EntityAirSupply, EntityBounds, EntityEquipment, EntityEquipmentSlot, EntityEquipmentStack,
        EntityOrientation, EntitySnapshot, EntityStateSnapshot, EntityTargetSnapshot,
        EntityVelocity, EntityVitals, VillagerOffer, VillagerOfferItem,
        VillagerTradeOfferDefinition, AIR_SUPPLY_WRITE_CAPABILITY, AI_CONTROL_CAPABILITY,
        EQUIPMENT_DROP_CHANCE_WRITE_CAPABILITY, HEALTH_WRITE_CAPABILITY,
        INVULNERABILITY_WRITE_CAPABILITY, LOOT_PICKUP_WRITE_CAPABILITY, MAX_ACTIVE_ENTITY_EFFECTS,
        MAX_ACTIVE_ENTITY_EFFECTS_RESPONSE_BYTES, MAX_ENTITY_AIR_SUPPLY,
        MAX_ENTITY_AIR_SUPPLY_RESPONSE_BYTES, MAX_ENTITY_BOUNDS_COORDINATE,
        MAX_ENTITY_BOUNDS_RESPONSE_BYTES, MAX_ENTITY_EFFECT_ID_BYTES, MAX_ENTITY_EQUIPMENT_DAMAGE,
        MAX_ENTITY_EQUIPMENT_DROP_CHANCE_RESPONSE_BYTES, MAX_ENTITY_EQUIPMENT_ITEM_ID_BYTES,
        MAX_ENTITY_EQUIPMENT_RESPONSE_BYTES, MAX_ENTITY_EQUIPMENT_STACK_COUNT,
        MAX_ENTITY_HEAD_YAW_DEGREES, MAX_ENTITY_HEAL_AMOUNT, MAX_ENTITY_NAVIGATION_RANGE,
        MAX_ENTITY_NAVIGATION_SPEED, MAX_ENTITY_RAYCAST_DISTANCE, MAX_ENTITY_STATE_RESPONSE_BYTES,
        MAX_ENTITY_TARGET_COORDINATE, MAX_ENTITY_TARGET_ID_BYTES, MAX_ENTITY_TARGET_RANGE,
        MAX_ENTITY_TARGET_RESPONSE_BYTES, MAX_ENTITY_TARGET_SNAPSHOT_RESPONSE_BYTES,
        MAX_ENTITY_TARGET_UUID_BYTES, MAX_ENTITY_TELEPORT_COORDINATE,
        MAX_ENTITY_VELOCITY_COMPONENT, MAX_ENTITY_VITALS_VALUE, MAX_SPATIAL_QUERY_RADIUS,
        MAX_SPATIAL_QUERY_RESULTS, MAX_VILLAGER_RUNTIME_OFFERS_PER_MERCHANT,
        MAX_VILLAGER_RUNTIME_OFFERS_PER_MODULE, MAX_VILLAGER_RUNTIME_OFFER_DEMAND,
        MAX_VILLAGER_RUNTIME_OFFER_ITEM_COUNT, MAX_VILLAGER_RUNTIME_OFFER_PATH_BYTES,
        MAX_VILLAGER_RUNTIME_OFFER_PRICE_MULTIPLIER, MAX_VILLAGER_RUNTIME_OFFER_REQUEST_BYTES,
        MAX_VILLAGER_RUNTIME_OFFER_XP, MAX_VILLAGER_TRADE_DEMAND, MAX_VILLAGER_TRADE_ID_BYTES,
        MAX_VILLAGER_TRADE_ITEM_COUNT, MAX_VILLAGER_TRADE_OFFERS,
        MAX_VILLAGER_TRADE_RESPONSE_BYTES, MAX_VILLAGER_TRADE_SPECIAL_PRICE,
        MAX_VILLAGER_TRADE_USES, MIN_ENTITY_AIR_SUPPLY, NAVIGATION_WRITE_CAPABILITY,
        PERSISTENCE_WRITE_CAPABILITY, TARGET_WRITE_CAPABILITY, TELEPORT_CAPABILITY,
        VILLAGER_TRADE_WRITE_CAPABILITY,
    };
    pub use crate::entity_attributes;
    pub use crate::entity_attributes::{
        AttributeValues, LivingAttribute, MAX_ATTRIBUTE_MODIFIER_KEY_BYTES,
        MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT,
    };
    pub use crate::entity_data;
    pub use crate::entity_data::{
        ENTITY_DATA_MISSING, MAX_ENTITY_DATA_KEY_BYTES, MAX_ENTITY_DATA_VALUE_BYTES,
    };
    pub use crate::entity_left_level;
    pub use crate::entity_left_level::{
        EntityLeftLevel, ENTITY_LEFT_LEVEL_EVENT_NAME, MAX_ENTITY_LEFT_LEVEL_CALLBACKS,
        MAX_ENTITY_LEFT_LEVEL_PAYLOAD_BYTES, MAX_ENTITY_LEFT_LEVEL_STRING_BYTES,
    };
    pub use crate::entity_teleport_attempt::{
        EntityTeleportAttempt, EntityTeleportAttemptAction, EntityTeleportCause,
        ENTITY_TELEPORT_ATTEMPT_CANCEL_CAPABILITY,
    };
    pub use crate::error::{ErrorCode, SjmlError, SjmlResult};
    pub use crate::events;
    pub use crate::events::BlockGrowAttempt;
    pub use crate::events::EntityBreedAttempt;
    pub use crate::events::{
        subscribe, subscribe_with_interval, unsubscribe, BlockBreak, BlockBreakAttempt,
        BlockFarmlandTrampleAttempt, BlockFertilizeAttempt, BlockFireSpreadAttempt,
        BlockFireSpreadCause, BlockFluidPlaceAttempt, BlockPlace, BlockPlaceAttempt,
        BlockRandomTick, BlockTick, BlockTickProperty, BlockUseAction, ClientAction,
        ClientActionAttempt, ClientActionHand, ClientChatSendAttempt, ClientItemTooltip,
        ClientScreenAction, ClientScreenFormSubmit, ClientScreenFormValue,
        ClientScreenMultiSelectFormSubmit, ClientTick, EffectTickEvent, EntityDamageAttempt,
        EntityDamaged, EntityDeath, EntityDeathAttempt, EntityEffectApplicationAttempt,
        EntityFallAttempt, EntityHealAttempt, EntityJoinAttempt, EntityKnockbackAttempt,
        EntityMountAttempt, EntityMountTarget, EntitySpawned, EntityTargetAttempt, EntityTick,
        EventPriority, ExplosionAttempt, InventoryChanged, ItemTooltipLines, ItemUseAction,
        ModuleBlockUse, ModuleItemAttackAttempt, ModuleItemUse, ModuleItemUseOnBlock,
        ModuleItemUseOnEntity, PlayerAdvancementEarned, PlayerAdvancementProgressEvent,
        PlayerAttackAttempt, PlayerBreakSpeed, PlayerBreakSpeedAction, PlayerDropAttempt,
        PlayerEquipmentChanged, PlayerEquipmentItemSnapshot, PlayerEquipmentSlot,
        PlayerExperienceChangeAction, PlayerExperienceChangeAttempt,
        PlayerExperienceLevelChangeAction, PlayerExperienceLevelChangeAttempt, PlayerInteracted,
        PlayerInteractionAttempt, PlayerItemBroken, PlayerItemCrafted, PlayerItemPickupAttempt,
        PlayerItemUseAttempt, PlayerJoined, PlayerLeft, PlayerMoved, PlayerPotionUseAttempt,
        PlayerPotionUseAttemptAction, PlayerSleepAttempt, PlayerStatisticChanged, PlayerTick,
        ProjectileImpactAttempt, ResourcesReloaded, ServerTick, WorldLifecycle, WorldTick,
        BLOCK_FIRE_SPREAD_ATTEMPT_EVENT_NAME, CLIENT_ACTION_CANCEL_CAPABILITY,
        DEFAULT_SERVER_TICK_INTERVAL, MAX_BLOCK_DROPS_ATTEMPT_CALLBACKS,
        MAX_BLOCK_DROPS_ATTEMPT_DROPS, MAX_BLOCK_DROPS_ATTEMPT_DURABILITY,
        MAX_BLOCK_DROPS_ATTEMPT_EXPERIENCE, MAX_BLOCK_DROPS_ATTEMPT_ITEM_COUNT,
        MAX_BLOCK_DROPS_ATTEMPT_PAYLOAD_BYTES, MAX_BLOCK_FIRE_SPREAD_ATTEMPT_AGE,
        MAX_BLOCK_FIRE_SPREAD_ATTEMPT_CALLBACKS,
        MAX_BLOCK_FIRE_SPREAD_ATTEMPT_CALLBACKS_PER_SERVER_TICK,
        MAX_CLIENT_ACTION_ATTEMPT_CALLBACKS, MAX_CRAFTED_ITEM_COUNT,
        MAX_ENTITY_DROPS_ATTEMPT_CALLBACKS, MAX_ENTITY_DROPS_ATTEMPT_DROPS,
        MAX_ENTITY_DROPS_ATTEMPT_DURABILITY, MAX_ENTITY_DROPS_ATTEMPT_ITEM_COUNT,
        MAX_ENTITY_DROPS_ATTEMPT_PAYLOAD_BYTES, MAX_INVENTORY_CHANGE_COUNT,
        MAX_INVENTORY_CHANGE_SLOT, MAX_PLAYER_BREAK_SPEED, MAX_PLAYER_BREAK_SPEED_CALLBACKS,
        MAX_PLAYER_EQUIPMENT_ITEM_COUNT, MAX_PLAYER_EXPERIENCE_CHANGE_AMOUNT,
        MAX_PLAYER_EXPERIENCE_LEVEL, MAX_PLAYER_EXPERIENCE_LEVEL_DELTA,
        MAX_PLAYER_EXPERIENCE_LEVEL_SNAPSHOT, MAX_PLAYER_POTION_USE_ATTEMPT_CALLBACKS,
        MAX_PLAYER_POTION_USE_DURATION_TICKS, MAX_PLAYER_POTION_USE_ITEM_COUNT,
        MAX_PLAYER_STATISTIC_CHANGED_CALLBACKS, MAX_PLAYER_STATISTIC_ID_BYTES,
        MAX_SERVER_TICK_INTERVAL, MIN_PLAYER_EXPERIENCE_CHANGE_AMOUNT,
        MIN_PLAYER_EXPERIENCE_LEVEL_DELTA, PLAYER_ADVANCEMENT_OBSERVE_CAPABILITY,
        PLAYER_BREAK_SPEED_MODIFY_CAPABILITY, PLAYER_EQUIPMENT_OBSERVE_CAPABILITY,
        PLAYER_STATISTIC_OBSERVE_CAPABILITY,
    };
    pub use crate::events::{BlockDropsAttempt, BlockDropsBreakerSnapshot, BlockDropsItemSnapshot};
    pub use crate::events::{EntityDropsAttempt, EntityDropsItemSnapshot};
    pub use crate::events::{
        FishedDropSnapshot, PlayerFishedAttempt, MAX_FISHED_DROPS, MAX_FISHED_ITEM_COUNT,
    };
    pub use crate::events::{
        PlayerEnchanted, PlayerEnchantedEnchantment, MAX_PLAYER_ENCHANTED_CALLBACKS,
        MAX_PLAYER_ENCHANTED_ENCHANTMENTS, MAX_PLAYER_ENCHANTED_ID_BYTES,
        MAX_PLAYER_ENCHANTED_ITEM_VALUE, MAX_PLAYER_ENCHANTED_LEVEL,
        MAX_PLAYER_ENCHANTED_PAYLOAD_BYTES, PLAYER_ENCHANTED_EVENT_NAME,
        PLAYER_ENCHANTMENTS_OBSERVE_CAPABILITY,
    };
    pub use crate::gl;
    pub use crate::gl::{
        GlBlendFactor, GlBufferTarget, GlBufferUsage, GlCapabilities, GlCommand, GlCullFace,
        GlDepthFunction, GlExecutionReport, GlFramebufferAttachment, GlIndexType, GlPrimitiveMode,
        GlResourceDescriptor, GlScalarType,
    };
    pub use crate::host::{self, LogLevel};
    pub use crate::inventory;
    pub use crate::inventory::{
        read_float_component as read_block_inventory_float_component,
        write_float_component as write_block_inventory_float_component,
    };
    pub use crate::inventory::{
        BlockPos, Direction, InventoryEndpoint, InventorySnapshot, ItemStackDescriptor,
        ItemStackSnapshot, MAX_ITEM_TRANSFER_COUNT,
    };
    pub use crate::menu;
    pub use crate::menu::{
        EditableMenuDefinition, EditableMenuEndpoint, EditableMenuSlot, EditableMenuStack,
        MenuAction, MenuDefinition, MenuItem,
    };
    pub use crate::model_animation;
    pub use crate::model_animation::{
        register_block_entity_model_keyframe_track,
        register_block_entity_model_transform_keyframe_track, register_entity_model_keyframe_track,
        register_entity_model_transform_keyframe_track, ModelAnimationTarget, ModelKeyframe,
        ModelKeyframeTrack, ModelTransformKeyframe, ModelTransformKeyframeTrack,
        ModelTransformKind, MAX_MODEL_KEYFRAMES, MAX_MODEL_KEYFRAME_DURATION_TICKS,
        MAX_MODEL_SCALE, MIN_MODEL_SCALE,
    };
    pub use crate::network;
    pub use crate::network::{
        broadcast, send_to, send_to_server, subscribe as subscribe_network,
        unsubscribe as unsubscribe_network, ChannelRef, LogicalSide, NetworkContext,
        NetworkContextSnapshot, NetworkDirection, NetworkSubscriptionHandle, Player, Uuid,
    };
    pub use crate::player;
    pub use crate::player::{
        active_effects, air_supply as player_air_supply, clear_item_cooldown, clear_respawn_point,
        clear_team, grant_advancement, has_item_cooldown, movement_state as player_movement_state,
        reset_fall_distance as reset_player_fall_distance, respawn_point, revoke_advancement,
        selected_hotbar_slot, send_rich_system_message, send_system_message,
        set_air_supply as set_player_air_supply, set_custom_stat, set_flying, set_flying_speed,
        set_instabuild, set_invulnerable, set_item_cooldown, set_may_build, set_may_fly,
        set_respawn_point, set_rotation, set_selected_hotbar_slot, set_team, set_walking_speed,
        team, PlayerAbilities, PlayerAirSupply, PlayerEquipment, PlayerEquipmentStack,
        PlayerGameMode, PlayerMotion, PlayerMovementState, PlayerRespawnPoint, PlayerSnapshot,
        PlayerVelocity, PlayerVitals, RichText, RichTextClickAction, RichTextColor, RichTextNode,
        AIR_SUPPLY_WRITE_CAPABILITY as PLAYER_AIR_SUPPLY_WRITE_CAPABILITY,
        FALL_DISTANCE_WRITE_CAPABILITY as PLAYER_FALL_DISTANCE_WRITE_CAPABILITY,
        MAX_PLAYER_ABILITY_SPEED, MAX_PLAYER_ACTIVE_EFFECTS,
        MAX_PLAYER_ACTIVE_EFFECTS_RESPONSE_BYTES, MAX_PLAYER_ADVANCEMENT_ID_BYTES,
        MAX_PLAYER_COOLDOWN_ITEM_ID_BYTES, MAX_PLAYER_FALL_DISTANCE, MAX_PLAYER_HOTBAR_SLOT_INDEX,
        MAX_PLAYER_ITEM_COOLDOWN_TICKS, MAX_PLAYER_RESPAWN_ANGLE, MAX_PLAYER_RESPAWN_COORDINATE,
        MAX_PLAYER_RESPAWN_DIMENSION_ID_BYTES, MAX_PLAYER_RESPAWN_POINT_BYTES,
        MAX_PLAYER_RESPAWN_SNAPSHOT_BYTES, MAX_PLAYER_RICH_SYSTEM_MESSAGE_DEPTH,
        MAX_PLAYER_RICH_SYSTEM_MESSAGE_NODES, MAX_PLAYER_RICH_SYSTEM_MESSAGE_PAYLOAD_BYTES,
        MAX_PLAYER_RICH_SYSTEM_MESSAGE_STRING_BYTES,
        MAX_PLAYER_RICH_SYSTEM_MESSAGE_TOTAL_UTF8_BYTES, MAX_PLAYER_ROTATION_YAW,
        MAX_PLAYER_STATISTIC_VALUE, MAX_PLAYER_SYSTEM_MESSAGE_TEXT_BYTES,
        MAX_PLAYER_SYSTEM_MESSAGE_TEXT_CODE_POINTS, MAX_PLAYER_TEAM_NAME_BYTES,
        MAX_PLAYER_TEAM_RESPONSE_BYTES, MAX_PLAYER_VELOCITY_COMPONENT, MIN_PLAYER_ABILITY_SPEED,
    };
    pub use crate::player_advancement;
    pub use crate::player_advancement::{
        grant_criterion as grant_advancement_criterion, progress as player_advancement_progress,
        revoke_criterion as revoke_advancement_criterion, PlayerAdvancementCriterion,
        PlayerAdvancementProgress, MAX_PLAYER_ADVANCEMENT_CRITERIA,
        MAX_PLAYER_ADVANCEMENT_CRITERION_ID_BYTES, MAX_PLAYER_ADVANCEMENT_PROGRESS_BYTES,
    };
    pub use crate::player_inventory;
    pub use crate::player_inventory::{
        clear_item_glint_override as clear_player_inventory_item_glint_override,
        give_item as give_player_item, move_stack as move_player_inventory_stack,
        read_float_component as read_player_inventory_float_component,
        set_count as set_player_inventory_count,
        set_item_glint_override as set_player_inventory_item_glint_override,
        write_float_component as write_player_inventory_float_component, PlayerInventorySlotUpdate,
        MAX_PLAYER_INVENTORY_COMPONENT_PATH_BYTES, MAX_PLAYER_INVENTORY_GIVE_COUNT,
        MAX_PLAYER_INVENTORY_ITEM_ID_BYTES, MAX_PLAYER_INVENTORY_MOVE_COUNT,
        MAX_PLAYER_INVENTORY_SLOTS, MAX_PLAYER_INVENTORY_SLOT_COUNT,
        MAX_PLAYER_INVENTORY_SLOT_INDEX,
    };
    pub use crate::player_statistics;
    pub use crate::player_statistics::{
        increment as increment_player_statistic, read as player_statistic,
        set_value as set_player_statistic_value, PlayerStatistic, StatisticFamily,
        MAX_INCREMENT as MAX_PLAYER_REGISTRY_STATISTIC_INCREMENT, MAX_STATISTIC_ID_BYTES,
        MAX_VALUE as MAX_PLAYER_REGISTRY_STATISTIC_VALUE,
    };
    pub use crate::recipe::{CraftingSlot, RecipeEvaluation, RecipeMatchInput};
    pub use crate::registry;
    pub use crate::registry::{
        add_block_state_property, add_to_creative_tab, custom_registry_value,
        register_attribute_effect, register_block, register_block_entity,
        register_block_entity_model, register_block_with_behavior, register_block_with_shape,
        register_block_with_shape_boxes, register_boolean_component, register_brewing_recipe,
        register_cooking_recipe, register_creative_tab, register_custom_registry,
        register_custom_value, register_effect_tick, register_entity, register_entity_ai_goal,
        register_entity_model, register_entity_spawn_egg, register_float_component, register_fluid,
        register_fluid_with_type_properties, register_fluid_with_visuals,
        register_fluid_with_visuals_and_type_properties, register_integer_component, register_item,
        register_ore_generation, register_particle, register_potion, register_recipe_serializer,
        register_shaped_recipe, register_shapeless_recipe, register_smithing_transform_recipe,
        register_sound, register_string_component, register_villager_trade, set_block_behavior,
        set_block_random_ticking, set_block_shape, set_block_shape_boxes, set_block_state_behavior,
        set_block_state_light_level, set_block_state_shape, set_entity_attribute,
        set_item_attribute_modifier, set_item_boolean_component, set_item_fire_resistant,
        set_item_float_component, set_item_food, set_item_food_with_effects,
        set_item_glint_override, set_item_integer_component, set_item_no_repair, set_item_rarity,
        set_item_repair_tag, set_item_string_component, AttributeModifierOperation, BlockAabb,
        BlockBehavior, BlockDefinition, BlockEntityModelDefinition, BlockPushReaction, BlockShape,
        BlockShapeBoxes, BlockSound, BlockStateBehavior, BlockStateProperty,
        CookingRecipeDefinition, CookingRecipeKind, CreativeTabDefinition, EffectCategory,
        EntityAiGoal, EntityAttribute, EntityDefinition, EntityModelDefinition,
        FloatComponentDefinition, FluidDefinition, FluidTypeProperties, FluidVisualDefinition,
        IntegerComponentDefinition, ItemAttributeModifier, ItemAttributeSlot, ItemDefinition,
        ItemFood, ItemFoodEffect, ItemFoodEffects, ItemRarity, ModelAnimation, ModelAnimationAxis,
        ModelCuboid, OreGenerationDefinition, OreReplaceableBlocks, RecipeIngredient,
        ShapedRecipeDefinition, ShapedRecipeKey, ShapelessRecipeDefinition,
        SmithingTransformRecipeDefinition, StringComponentDefinition, VillagerTradeItem,
        VillagerTradeOffer, MAX_BLOCK_INTEGER_PROPERTY_VALUE, MAX_BLOCK_INTEGER_PROPERTY_VALUES,
        MAX_BLOCK_SHAPE_BOXES, MAX_BLOCK_STATE_PROPERTIES, MAX_BLOCK_STATE_VARIANTS,
        MAX_BOUND_MODEL_CUBOIDS, MAX_ENTITY_SPAWN_EGG_RGB_COLOR, MAX_ITEM_ATTRIBUTE_MODIFIERS,
        MAX_ITEM_ATTRIBUTE_MODIFIER_AMOUNT, MAX_ITEM_FOOD_EFFECTS,
        MAX_ITEM_FOOD_EFFECT_DURATION_TICKS, MAX_ITEM_FOOD_EFFECT_ID_BYTES,
        MAX_ITEM_REPAIR_TAG_ID_BYTES, MIN_BLOCK_INTEGER_PROPERTY_VALUE,
    };
    pub use crate::render;
    pub use crate::render::{
        suppress_gui_layer, unsuppress_gui_layer, GuiLayerSuppressionHandle, RenderContext,
        RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle, WorldModelBuilder,
        WorldRenderFrame, MAX_GUI_LAYER_ID_BYTES, MAX_RENDER_RESOURCE_PATH_BYTES,
        MAX_WORLD_MODEL_QUADS, MAX_WORLD_MODEL_VERTICES,
    };
    pub use crate::scoreboard;
    pub use crate::scoreboard::{
        clear_display_slot as scoreboard_clear_display_slot,
        create_objective as scoreboard_create_objective,
        objective_score as scoreboard_objective_score,
        remove_objective as scoreboard_remove_objective, score as scoreboard_score,
        set_display_slot as scoreboard_set_display_slot,
        set_objective_display_slot as scoreboard_set_objective_display_slot,
        set_objective_score as scoreboard_set_objective_score, set_score as scoreboard_set_score,
        DisplaySlot,
    };
    pub use crate::screen_input;
    pub use crate::screen_input::{
        subscribe_mouse_dragged, subscribe_mouse_released, unsubscribe_key_pressed,
        unsubscribe_mouse_dragged, unsubscribe_mouse_released, ScreenInputEvent,
        ScreenInputSubscriptionHandle, ScreenKeyPressedEvent, ScreenKeyPressedSubscriptionHandle,
        ScreenMouseDraggedEvent, ScreenMouseDraggedSubscriptionHandle, ScreenMouseReleasedEvent,
        ScreenMouseReleasedSubscriptionHandle, MAX_SCREEN_DRAG_DELTA, MAX_SCREEN_INPUT_COORDINATE,
        MAX_SCREEN_INPUT_SUBSCRIPTIONS, MAX_SCREEN_SCROLL_DELTA,
    };
    pub use crate::serialization::{
        from_bytes, to_bytes, MAX_COLLECTION_ITEMS, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
    };
    pub use crate::shield_block_attempt::{
        EntityShieldBlockAttempt, EntityShieldBlockAttemptAction,
    };
    pub use crate::storage;
    pub use crate::storage::{
        clear_migration_mode, migrate_record, migrate_scope, register_migration,
        register_migration_with_mode, set_migration_mode, MigrationFailure, MigrationMode,
        MigrationReport, StableRecord, StorageError, StorageResult, StorageScope,
    };
    pub use crate::task::{self, spawn_compute, GuestTaskId};
    pub use crate::widgets;
    pub use crate::widgets::{
        Button, ProgressBar, TextField, TextFieldAction, Tooltip, WidgetRect, MAX_TEXT_FIELD_BYTES,
    };
    pub use crate::world;
    pub use crate::world::{
        block_collision_boxes, block_destroy_speed, fluid_flow_at, spawn_particles,
        BiomeClimateSnapshot, BiomePrecipitation, BiomeTemperatureModifier, BlockCollisionBox,
        BlockProperty, BlockStateSnapshot, BlockUpdateOptions, ChunkPos, FluidFlowSnapshot,
        FluidStateSnapshot, HeightmapType, ItemSpawnVelocity, LightSnapshot, ParticleSpawnOptions,
        ProjectileVelocity, RaycastBlockMode, RaycastFluidMode, RaycastHit, RaycastPoint,
        TagRegistry, VanillaProjectileKind, WorldBorderSnapshot, WorldDifficulty, WorldGameRule,
        WorldIntegerGameRule, WorldSpawnPoint, MAX_WORLD_BIOME_CLIMATE_RESPONSE_BYTES,
        MAX_WORLD_BLOCK_COLLISION_BOXES, MAX_WORLD_BLOCK_COLLISION_BOXES_RESPONSE_BYTES,
        MAX_WORLD_BLOCK_COORDINATE, MAX_WORLD_BLOCK_DESTROY_SPEED,
        MAX_WORLD_BLOCK_DESTROY_SPEED_RESPONSE_BYTES, MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES,
        MAX_WORLD_BLOCK_PROPERTY_UPDATES, MAX_WORLD_BLOCK_STATES, MAX_WORLD_BLOCK_TICK_DELAY_TICKS,
        MAX_WORLD_BORDER_CENTER, MAX_WORLD_BORDER_DAMAGE_BUFFER, MAX_WORLD_BORDER_DAMAGE_PER_BLOCK,
        MAX_WORLD_BORDER_DIAMETER, MAX_WORLD_BORDER_LERP_MILLIS, MAX_WORLD_BORDER_WARNING_BLOCKS,
        MAX_WORLD_BORDER_WARNING_SECONDS, MAX_WORLD_FLUID_FLOW_COMPONENT,
        MAX_WORLD_FLUID_FLOW_LENGTH, MAX_WORLD_FLUID_FLOW_RESPONSE_BYTES,
        MAX_WORLD_FLUID_STATE_RESPONSE_BYTES, MAX_WORLD_ITEM_SPAWNS_PER_TICK,
        MAX_WORLD_ITEM_SPAWN_SPEED, MAX_WORLD_ITEM_STACK_COUNT, MAX_WORLD_PARTICLES_PER_CALL,
        MAX_WORLD_PARTICLES_PER_TICK, MAX_WORLD_PARTICLE_SPEED, MAX_WORLD_PARTICLE_SPREAD,
        MAX_WORLD_PROJECTILES_PER_TICK, MAX_WORLD_PROJECTILE_SPEED,
        MAX_WORLD_RAYCAST_RESPONSE_BYTES, MAX_WORLD_RAY_COORDINATE, MAX_WORLD_RAY_DISTANCE,
        MAX_WORLD_REDSTONE_SIGNAL, MAX_WORLD_SPAWN_POINT_BYTES, MIN_WORLD_PROJECTILE_SPEED,
        WORLD_BLOCK_UPDATE_ALLOWED_MASK,
    };
    pub use crate::world_chunk_lifecycle;
    pub use crate::world_chunk_lifecycle::{
        WorldChunkLifecycle, MAX_WORLD_CHUNK_COORDINATE, MAX_WORLD_CHUNK_DIMENSION_ID_BYTES,
        MAX_WORLD_CHUNK_LIFECYCLE_CALLBACKS, MAX_WORLD_CHUNK_LIFECYCLE_PAYLOAD_BYTES,
        WORLD_CHUNK_LOADED_EVENT_NAME, WORLD_CHUNK_UNLOADED_EVENT_NAME,
    };
    pub use crate::world_save_events;
    pub use crate::world_save_events::{
        WorldSaveObserved, MAX_WORLD_SAVE_CALLBACKS_PER_EVENT, MAX_WORLD_SAVE_DIMENSION_ID_BYTES,
        MAX_WORLD_SAVE_GAME_TIME_TICKS, MAX_WORLD_SAVE_OBSERVED_PAYLOAD_BYTES,
        WORLD_SAVE_OBSERVED_EVENT_NAME,
    };
    pub use crate::{
        client_command, client_command_suggestions, client_music_selection, command,
        command_suggestions, compute, event, event_async, mod_init, mod_register, mod_shutdown,
        recipe, service,
    };
    pub use crate::{StableDecode, StableEncode};
}

/// Internal helpers referenced by macro expansions.
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub mod __private {
    use crate::SjmlResult;

    /// # Safety
    ///
    /// For non-empty input, the pair must identify a live allocation from
    /// `abi::allocate_host_input` in this guest instance. Initialize every
    /// byte before calling and keep the buffer live and unmodified until return.
    pub unsafe fn dispatch_payload<I, O, F>(pointer: u32, length: u32, handler: F) -> u64
    where
        I: crate::StableDecode,
        O: crate::StableEncode,
        F: FnOnce(I) -> SjmlResult<O>,
    {
        // SAFETY: the caller guarantees the callback input allocation.
        unsafe { crate::abi::dispatch_payload(pointer, length, handler) }
    }
}
