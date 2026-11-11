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
pub mod client_chat;
pub mod client_input;
pub mod chat_messages;
pub mod client_notifications;
pub mod client_audio;
pub mod entity;
pub mod entity_data;
mod error;
pub mod events;
pub mod gl;
pub mod host;
pub mod inventory;
pub mod menu;
pub mod network;
pub mod player;
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
pub use client_chat::{ChatLayout, ChatLayoutHandle, ChatScreenEvent, ChatTheme};
pub use client_input::{ClientInputAction, ClientInputEvent, ClientInputSubscriptionHandle};
pub use client_notifications::NotificationKind;
pub use client_audio::SoundCategory;
pub use chat_messages::{
    ChatMessage, ChatMessageAction, ChatMessageSource, ChatMessageSubscriptionHandle,
};
pub use entity::{EntitySnapshot, EntityVelocity, MAX_ENTITY_VELOCITY_COMPONENT};
pub use error::{ErrorCode, SjmlError, SjmlResult};
pub use gl::{
    GlBlendFactor, GlBufferTarget, GlBufferUsage, GlCapabilities, GlCommand, GlCullFace,
    GlDepthFunction, GlExecutionReport, GlFramebufferAttachment, GlIndexType, GlPrimitiveMode,
    GlResourceDescriptor, GlScalarType,
};
pub use inventory::{
    BlockPos, Direction, InventoryEndpoint, InventorySnapshot, ItemStackDescriptor,
    MAX_ITEM_TRANSFER_COUNT,
};
pub use menu::{MenuAction, MenuDefinition, MenuItem};
pub use network::{
    ChannelRef, LogicalSide, NetworkContext, NetworkContextSnapshot, NetworkDirection,
    NetworkSubscriptionHandle, Player as NetworkPlayer, Uuid,
};
pub use player::{
    MAX_PLAYER_VELOCITY_COMPONENT, PlayerAbilities, PlayerEquipment, PlayerEquipmentStack,
    PlayerGameMode, PlayerMotion, PlayerSnapshot, PlayerVelocity, PlayerVitals,
};
pub use recipe::{
    CraftingSlot, RECIPE_CALLBACK_SCHEMA_VERSION, RecipeEvaluation, RecipeMatchInput,
};
pub use registry::{
    BlockEntityModelDefinition, EntityModelDefinition, MAX_BOUND_MODEL_CUBOIDS, ModelAnimation,
    ModelAnimationAxis, ModelCuboid,
};
pub use render::{
    MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS, MAX_WORLD_MODEL_VERTICES, RenderContext,
    RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle, WorldModelBuilder,
    WorldRenderFrame,
};
pub use screen_input::{ScreenInputEvent, ScreenInputSubscriptionHandle};
pub use serialization::{
    Decoder, Encoder, MAX_COLLECTION_ITEMS, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES, StableDecode,
    StableEncode, from_bytes, to_bytes,
};
pub use sjml_macros::{StableDecode, StableEncode};
pub use sjml_macros::{
    command, compute, event, event_async, mod_init, mod_register, mod_shutdown, recipe, service,
};
pub use storage::{
    MigrationFailure, MigrationMode, MigrationReport, StableRecord, StorageError, StorageResult,
    StorageScope,
};
pub use task::GuestTaskId;
pub use widgets::{
    Button, MAX_TEXT_FIELD_BYTES, ProgressBar, TextField, TextFieldAction, Tooltip, WidgetRect,
};
pub use world::{
    BlockStateSnapshot, HeightmapType, LightSnapshot, WorldBorderSnapshot,
    MAX_WORLD_BLOCK_COORDINATE, MAX_WORLD_BLOCK_STATES, MAX_WORLD_BORDER_CENTER,
    MAX_WORLD_BORDER_DAMAGE_BUFFER, MAX_WORLD_BORDER_DAMAGE_PER_BLOCK,
    MAX_WORLD_BORDER_DIAMETER, MAX_WORLD_BORDER_LERP_MILLIS,
    MAX_WORLD_BORDER_WARNING_BLOCKS, MAX_WORLD_BORDER_WARNING_SECONDS,
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
    pub use crate::client_chat::{self, ChatLayout, ChatLayoutHandle, ChatScreenEvent, ChatTheme};
    pub use crate::client_input;
    pub use crate::client_input::{
        ClientInputAction, ClientInputEvent, ClientInputSubscriptionHandle,
        MAX_CLIENT_INPUT_SUBSCRIPTIONS,
    };
    pub use crate::chat_messages;
    pub use crate::chat_messages::{
        ChatMessage, ChatMessageAction, ChatMessageSource, ChatMessageSubscriptionHandle,
    };
    pub use crate::client_notifications;
    pub use crate::client_notifications::NotificationKind;
    pub use crate::client_audio;
    pub use crate::client_audio::SoundCategory;
    pub use crate::entity;
    pub use crate::entity::{EntitySnapshot, EntityVelocity, MAX_ENTITY_VELOCITY_COMPONENT};
    pub use crate::entity_data;
    pub use crate::entity_data::{
        ENTITY_DATA_MISSING, MAX_ENTITY_DATA_KEY_BYTES, MAX_ENTITY_DATA_VALUE_BYTES,
    };
    pub use crate::error::{ErrorCode, SjmlError, SjmlResult};
    pub use crate::events;
    pub use crate::events::{
        BlockBreak, BlockPlace, DEFAULT_SERVER_TICK_INTERVAL, EffectTickEvent, EntityDamaged,
        EntityDeath, EntitySpawned, EventPriority, InventoryChanged, MAX_INVENTORY_CHANGE_COUNT,
        MAX_INVENTORY_CHANGE_SLOT, MAX_SERVER_TICK_INTERVAL, PlayerInteracted, PlayerJoined,
        PlayerLeft, PlayerMoved, ResourcesReloaded, ServerTick, WorldLifecycle, subscribe,
        subscribe_with_interval, unsubscribe,
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
        MAX_ITEM_TRANSFER_COUNT,
    };
    pub use crate::menu;
    pub use crate::menu::{MenuAction, MenuDefinition, MenuItem};
    pub use crate::network;
    pub use crate::network::{
        ChannelRef, LogicalSide, NetworkContext, NetworkContextSnapshot, NetworkDirection,
        NetworkSubscriptionHandle, Player, Uuid, broadcast, send_to, send_to_server,
        subscribe as subscribe_network, unsubscribe as unsubscribe_network,
    };
    pub use crate::player;
    pub use crate::player::{
        MAX_PLAYER_VELOCITY_COMPONENT, PlayerAbilities, PlayerEquipment, PlayerEquipmentStack,
        PlayerGameMode, PlayerMotion, PlayerSnapshot, PlayerVelocity, PlayerVitals,
    };
    pub use crate::recipe::{CraftingSlot, RecipeEvaluation, RecipeMatchInput};
    pub use crate::registry;
    pub use crate::registry::{
        AttributeModifierOperation, BlockDefinition, BlockEntityModelDefinition,
        CookingRecipeDefinition, CookingRecipeKind, CreativeTabDefinition, EffectCategory,
        EntityAiGoal, EntityAttribute, EntityDefinition, EntityModelDefinition, FluidDefinition,
        FluidVisualDefinition, ItemDefinition, MAX_BOUND_MODEL_CUBOIDS, ModelAnimation,
        ModelAnimationAxis, ModelCuboid, RecipeIngredient, ShapedRecipeDefinition, ShapedRecipeKey,
        ShapelessRecipeDefinition, SmithingTransformRecipeDefinition, StringComponentDefinition,
        add_to_creative_tab,
        custom_registry_value, register_attribute_effect, register_block, register_block_entity,
        register_block_entity_model, register_cooking_recipe, register_creative_tab,
        register_custom_registry, register_custom_value, register_effect_tick, register_entity,
        register_entity_ai_goal, register_entity_model, register_fluid,
        register_fluid_with_visuals, register_item, register_particle, register_recipe_serializer,
        register_shaped_recipe, register_shapeless_recipe, register_smithing_transform_recipe,
        register_sound,
        register_string_component, set_entity_attribute, set_item_string_component,
    };
    pub use crate::render;
    pub use crate::render::{
        MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS, MAX_WORLD_MODEL_VERTICES,
        RenderContext, RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle,
        WorldModelBuilder, WorldRenderFrame,
    };
    pub use crate::screen_input;
    pub use crate::screen_input::{
        MAX_SCREEN_INPUT_COORDINATE, MAX_SCREEN_INPUT_SUBSCRIPTIONS, ScreenInputEvent,
        ScreenInputSubscriptionHandle,
    };
    pub use crate::serialization::{
        MAX_COLLECTION_ITEMS, MAX_PAYLOAD_BYTES, MAX_STRING_BYTES, from_bytes, to_bytes,
    };
    pub use crate::storage;
    pub use crate::storage::{
        MigrationFailure, MigrationMode, MigrationReport, StableRecord, StorageError,
        StorageResult, StorageScope, clear_migration_mode, migrate_record, migrate_scope,
        register_migration, register_migration_with_mode, set_migration_mode,
    };
    pub use crate::task::{self, GuestTaskId, spawn_compute};
    pub use crate::widgets;
    pub use crate::widgets::{
        Button, MAX_TEXT_FIELD_BYTES, ProgressBar, TextField, TextFieldAction, Tooltip, WidgetRect,
    };
    pub use crate::world;
    pub use crate::world::{
        BlockStateSnapshot, HeightmapType, LightSnapshot, WorldBorderSnapshot,
        MAX_WORLD_BLOCK_COORDINATE, MAX_WORLD_BLOCK_STATES, MAX_WORLD_BORDER_CENTER,
        MAX_WORLD_BORDER_DAMAGE_BUFFER, MAX_WORLD_BORDER_DAMAGE_PER_BLOCK,
        MAX_WORLD_BORDER_DIAMETER, MAX_WORLD_BORDER_LERP_MILLIS,
        MAX_WORLD_BORDER_WARNING_BLOCKS, MAX_WORLD_BORDER_WARNING_SECONDS,
        MAX_WORLD_RAY_COORDINATE, MAX_WORLD_RAY_DISTANCE, MAX_WORLD_RAYCAST_RESPONSE_BYTES,
        RaycastBlockMode, RaycastFluidMode, RaycastHit, RaycastPoint,
    };
    pub use crate::{StableDecode, StableEncode};
    pub use crate::{
        command, compute, event, event_async, mod_init, mod_register, mod_shutdown, recipe, service,
    };
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
