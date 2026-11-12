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
pub mod client_audio;
pub mod client_chat;
pub mod client_input;
pub mod client_keybindings;
pub mod client_notifications;
pub mod entity;
pub mod entity_attributes;
pub mod entity_data;
mod error;
pub mod events;
pub mod gl;
pub mod host;
pub mod inventory;
pub mod menu;
pub mod network;
pub mod player;
pub mod player_inventory;
pub mod recipe;
pub mod registry;
pub mod render;
pub mod screen_input;
mod serialization;
pub mod storage;
pub mod task;
pub mod widgets;
pub mod world;

pub use abi::{
    BlockEntityHandle, BlockEntityTag, BlockHandle, BlockTag, EntityHandle, EntityTag,
    EventSubscriptionHandle, EventSubscriptionTag, GlResourceHandle, GlResourceTag, Handle,
    HandleKind, ItemStackHandle, ItemStackTag, NetworkChannelHandle, NetworkChannelTag,
    PlayerHandle, PlayerTag, RenderResourceHandle, RenderResourceTag, StorageScopeHandle,
    StorageScopeTag, TaskHandle, TaskTag, WorldHandle, WorldTag,
};
pub use batch::{Batch, BatchValue, CallResult};
pub use block_entity::{BlockEntity, BlockEntitySnapshot, NbtValue};
pub use chat_messages::{
    ChatClickAction, ChatMessage, ChatMessageAction, ChatMessageSource,
    ChatMessageSubscriptionHandle, ChatSpan, ChatText, MAX_CHAT_INTERACTION_BYTES,
    MAX_CHAT_INTERACTION_VALUE_BYTES, MAX_CHAT_TEXT_SPANS,
};
pub use client_audio::SoundCategory;
pub use client_chat::{ChatLayout, ChatLayoutHandle, ChatScreenEvent, ChatTheme};
pub use client_input::{
    cursor_position, subscribe_global_keys, unsubscribe_global_keys, ClientInputAction,
    ClientInputEvent, ClientInputSubscriptionHandle, CursorPosition, GlobalKeyObservation,
    GlobalKeyObservationHandle, MouseScrollEvent, MouseScrollSubscriptionHandle,
    MAX_CURSOR_COORDINATE,
};
pub use client_keybindings::{
    Context as KeybindingContext, Key as KeybindingKey, KeybindingDefinition,
    KeybindingSubscriptionHandle, Modifier as KeybindingModifier,
    MAX_CLIENT_KEYBINDING_SUBSCRIPTIONS,
};
pub use client_notifications::NotificationKind;
pub use entity::{
    EntitySnapshot, EntityVelocity, MAX_ENTITY_VELOCITY_COMPONENT, MAX_SPATIAL_QUERY_RADIUS,
    MAX_SPATIAL_QUERY_RESULTS,
};
pub use entity_attributes::{AttributeValues, LivingAttribute};
pub use error::{ErrorCode, SjmlError, SjmlResult};
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
pub use network::{
    ChannelRef, LogicalSide, NetworkContext, NetworkContextSnapshot, NetworkDirection,
    NetworkSubscriptionHandle, Player as NetworkPlayer, Uuid,
};
pub use player::{
    PlayerAbilities, PlayerEquipment, PlayerEquipmentStack, PlayerGameMode, PlayerMotion,
    PlayerSnapshot, PlayerTitleTimings, PlayerVelocity, PlayerVitals, MAX_PLAYER_TITLE_TEXT_BYTES,
    MAX_PLAYER_TITLE_TEXT_CODE_POINTS, MAX_PLAYER_TITLE_TIMING_TICKS,
    MAX_PLAYER_VELOCITY_COMPONENT,
};
pub use player_inventory::{
    PlayerInventorySlotUpdate, MAX_PLAYER_INVENTORY_ITEM_ID_BYTES, MAX_PLAYER_INVENTORY_SLOTS,
    MAX_PLAYER_INVENTORY_SLOT_COUNT, MAX_PLAYER_INVENTORY_SLOT_INDEX,
};
pub use recipe::{
    CraftingSlot, RecipeEvaluation, RecipeMatchInput, RECIPE_CALLBACK_SCHEMA_VERSION,
};
pub use registry::{
    BlockEntityModelDefinition, EntityModelDefinition, IntegerComponentDefinition, ItemFood,
    ItemRarity, ModelAnimation, ModelAnimationAxis, ModelCuboid, VillagerTradeItem,
    VillagerTradeOffer, MAX_BOUND_MODEL_CUBOIDS,
};
pub use render::{
    RenderContext, RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle,
    WorldModelBuilder, WorldRenderFrame, MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS,
    MAX_WORLD_MODEL_VERTICES,
};
pub use screen_input::{
    unsubscribe_key_pressed, ScreenInputEvent, ScreenInputSubscriptionHandle,
    ScreenKeyPressedEvent, ScreenKeyPressedSubscriptionHandle, MAX_SCREEN_SCROLL_DELTA,
};
pub use serialization::{
    from_bytes, to_bytes, Decoder, Encoder, StableDecode, StableEncode, MAX_COLLECTION_ITEMS,
    MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
};
pub use sjml_macros::{
    client_command, command, compute, event, event_async, mod_init, mod_register, mod_shutdown,
    recipe, service,
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
    BlockProperty, BlockStateSnapshot, ChunkPos, HeightmapType, LightSnapshot, WorldBorderSnapshot,
    WorldGameRule, WorldIntegerGameRule, MAX_WORLD_BIOME_IDS, MAX_WORLD_BIOME_IDS_RESPONSE_BYTES,
    MAX_WORLD_BIOME_ID_BYTES, MAX_WORLD_BLOCK_COORDINATE, MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES,
    MAX_WORLD_BLOCK_PROPERTY_UPDATES, MAX_WORLD_BLOCK_STATES, MAX_WORLD_BLOCK_TICK_DELAY_TICKS,
    MAX_WORLD_BORDER_CENTER, MAX_WORLD_BORDER_DAMAGE_BUFFER, MAX_WORLD_BORDER_DAMAGE_PER_BLOCK,
    MAX_WORLD_BORDER_DIAMETER, MAX_WORLD_BORDER_LERP_MILLIS, MAX_WORLD_BORDER_WARNING_BLOCKS,
    MAX_WORLD_BORDER_WARNING_SECONDS, MAX_WORLD_CHUNKS, MAX_WORLD_CHUNKS_RESPONSE_BYTES,
    MAX_WORLD_CHUNK_RADIUS, MAX_WORLD_GAME_TIME_TICKS,
};

/// Common imports for an SJML guest module.
pub mod prelude {
    pub use crate::abi::{
        BlockEntityHandle, BlockHandle, EntityHandle, EventSubscriptionHandle, GlResourceHandle,
        ItemStackHandle, NetworkChannelHandle, PlayerHandle, RenderResourceHandle,
        StorageScopeHandle, TaskHandle, WorldHandle,
    };
    pub use crate::batch::{self, Batch, BatchValue, CallResult};
    pub use crate::block_entity::{self, BlockEntity, BlockEntitySnapshot, NbtValue};
    pub use crate::chat_messages;
    pub use crate::chat_messages::{
        ChatMessage, ChatMessageAction, ChatMessageSource, ChatMessageSubscriptionHandle,
    };
    pub use crate::client_audio;
    pub use crate::client_audio::SoundCategory;
    pub use crate::client_chat::{self, ChatLayout, ChatLayoutHandle, ChatScreenEvent, ChatTheme};
    pub use crate::client_input;
    pub use crate::client_input::{
        cursor_position, subscribe_global_keys, subscribe_scroll, unsubscribe_global_keys,
        unsubscribe_scroll, ClientInputAction, ClientInputEvent, ClientInputSubscriptionHandle,
        CursorPosition, GlobalKeyObservation, GlobalKeyObservationHandle, MouseScrollEvent,
        MouseScrollSubscriptionHandle, MAX_CLIENT_INPUT_SUBSCRIPTIONS, MAX_CURSOR_COORDINATE,
    };
    pub use crate::client_keybindings;
    pub use crate::client_keybindings::{
        Context as KeybindingContext, Key as KeybindingKey, KeybindingDefinition,
        KeybindingSubscriptionHandle, Modifier as KeybindingModifier,
        MAX_CLIENT_KEYBINDING_SUBSCRIPTIONS,
    };
    pub use crate::client_notifications;
    pub use crate::client_notifications::NotificationKind;
    pub use crate::entity;
    pub use crate::entity::{
        EntitySnapshot, EntityVelocity, MAX_ENTITY_RAYCAST_DISTANCE, MAX_ENTITY_VELOCITY_COMPONENT,
        MAX_SPATIAL_QUERY_RADIUS, MAX_SPATIAL_QUERY_RESULTS,
    };
    pub use crate::entity_attributes;
    pub use crate::entity_attributes::{AttributeValues, LivingAttribute};
    pub use crate::entity_data;
    pub use crate::entity_data::{
        ENTITY_DATA_MISSING, MAX_ENTITY_DATA_KEY_BYTES, MAX_ENTITY_DATA_VALUE_BYTES,
    };
    pub use crate::error::{ErrorCode, SjmlError, SjmlResult};
    pub use crate::events;
    pub use crate::events::{
        subscribe, subscribe_with_interval, unsubscribe, BlockBreak, BlockBreakAttempt, BlockPlace,
        BlockPlaceAttempt, EffectTickEvent, EntityDamageAttempt, EntityDamaged, EntityDeath,
        EntityDeathAttempt, EntitySpawned, EntityTargetAttempt, EventPriority, ExplosionAttempt,
        InventoryChanged, PlayerAdvancementEarned, PlayerDropAttempt, PlayerInteracted,
        PlayerInteractionAttempt, PlayerJoined, PlayerLeft, PlayerMoved, ProjectileImpactAttempt,
        ResourcesReloaded, ServerTick, WorldLifecycle, DEFAULT_SERVER_TICK_INTERVAL,
        MAX_INVENTORY_CHANGE_COUNT, MAX_INVENTORY_CHANGE_SLOT, MAX_SERVER_TICK_INTERVAL,
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
        BlockPos, Direction, InventoryEndpoint, InventorySnapshot, ItemStackDescriptor,
        ItemStackSnapshot, MAX_ITEM_TRANSFER_COUNT,
    };
    pub use crate::menu;
    pub use crate::menu::{
        EditableMenuDefinition, EditableMenuEndpoint, EditableMenuSlot, EditableMenuStack,
        MenuAction, MenuDefinition, MenuItem,
    };
    pub use crate::network;
    pub use crate::network::{
        broadcast, send_to, send_to_server, subscribe as subscribe_network,
        unsubscribe as unsubscribe_network, ChannelRef, LogicalSide, NetworkContext,
        NetworkContextSnapshot, NetworkDirection, NetworkSubscriptionHandle, Player, Uuid,
    };
    pub use crate::player;
    pub use crate::player::{
        clear_item_cooldown, has_item_cooldown, set_item_cooldown, PlayerAbilities,
        PlayerEquipment, PlayerEquipmentStack, PlayerGameMode, PlayerMotion, PlayerSnapshot,
        PlayerVelocity, PlayerVitals, MAX_PLAYER_COOLDOWN_ITEM_ID_BYTES,
        MAX_PLAYER_ITEM_COOLDOWN_TICKS, MAX_PLAYER_VELOCITY_COMPONENT,
    };
    pub use crate::player_inventory;
    pub use crate::player_inventory::{
        PlayerInventorySlotUpdate, MAX_PLAYER_INVENTORY_COMPONENT_PATH_BYTES,
        MAX_PLAYER_INVENTORY_ITEM_ID_BYTES, MAX_PLAYER_INVENTORY_SLOTS,
        MAX_PLAYER_INVENTORY_SLOT_COUNT, MAX_PLAYER_INVENTORY_SLOT_INDEX,
    };
    pub use crate::recipe::{CraftingSlot, RecipeEvaluation, RecipeMatchInput};
    pub use crate::registry;
    pub use crate::registry::{
        add_to_creative_tab, custom_registry_value, register_attribute_effect, register_block,
        register_block_entity, register_block_entity_model, register_cooking_recipe,
        register_creative_tab, register_custom_registry, register_custom_value,
        register_effect_tick, register_entity, register_entity_ai_goal, register_entity_model,
        register_fluid, register_fluid_with_visuals, register_integer_component, register_item,
        register_particle, register_recipe_serializer, register_shaped_recipe,
        register_shapeless_recipe, register_smithing_transform_recipe, register_sound,
        register_string_component, register_villager_trade, set_entity_attribute, set_item_food,
        set_item_integer_component, set_item_rarity, set_item_string_component,
        AttributeModifierOperation, BlockDefinition, BlockEntityModelDefinition,
        CookingRecipeDefinition, CookingRecipeKind, CreativeTabDefinition, EffectCategory,
        EntityAiGoal, EntityAttribute, EntityDefinition, EntityModelDefinition, FluidDefinition,
        FluidVisualDefinition, IntegerComponentDefinition, ItemDefinition, ItemFood, ItemRarity,
        ModelAnimation, ModelAnimationAxis, ModelCuboid, RecipeIngredient, ShapedRecipeDefinition,
        ShapedRecipeKey, ShapelessRecipeDefinition, SmithingTransformRecipeDefinition,
        StringComponentDefinition, VillagerTradeItem, VillagerTradeOffer, MAX_BOUND_MODEL_CUBOIDS,
    };
    pub use crate::render;
    pub use crate::render::{
        RenderContext, RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle,
        WorldModelBuilder, WorldRenderFrame, MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS,
        MAX_WORLD_MODEL_VERTICES,
    };
    pub use crate::screen_input;
    pub use crate::screen_input::{
        unsubscribe_key_pressed, ScreenInputEvent, ScreenInputSubscriptionHandle,
        ScreenKeyPressedEvent, ScreenKeyPressedSubscriptionHandle, MAX_SCREEN_INPUT_COORDINATE,
        MAX_SCREEN_INPUT_SUBSCRIPTIONS, MAX_SCREEN_SCROLL_DELTA,
    };
    pub use crate::serialization::{
        from_bytes, to_bytes, MAX_COLLECTION_ITEMS, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
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
        BlockProperty, BlockStateSnapshot, ChunkPos, HeightmapType, LightSnapshot,
        RaycastBlockMode, RaycastFluidMode, RaycastHit, RaycastPoint, WorldBorderSnapshot,
        WorldGameRule, WorldIntegerGameRule, MAX_WORLD_BLOCK_COORDINATE,
        MAX_WORLD_BLOCK_PROPERTY_MUTATION_BYTES, MAX_WORLD_BLOCK_PROPERTY_UPDATES,
        MAX_WORLD_BLOCK_STATES, MAX_WORLD_BLOCK_TICK_DELAY_TICKS, MAX_WORLD_BORDER_CENTER,
        MAX_WORLD_BORDER_DAMAGE_BUFFER, MAX_WORLD_BORDER_DAMAGE_PER_BLOCK,
        MAX_WORLD_BORDER_DIAMETER, MAX_WORLD_BORDER_LERP_MILLIS, MAX_WORLD_BORDER_WARNING_BLOCKS,
        MAX_WORLD_BORDER_WARNING_SECONDS, MAX_WORLD_RAYCAST_RESPONSE_BYTES,
        MAX_WORLD_RAY_COORDINATE, MAX_WORLD_RAY_DISTANCE,
    };
    pub use crate::{
        client_command, command, compute, event, event_async, mod_init, mod_register, mod_shutdown,
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
