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
pub mod client_input;
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
pub use client_input::{ClientInputAction, ClientInputEvent, ClientInputSubscriptionHandle};
pub use entity::EntitySnapshot;
pub use error::{ErrorCode, SjmlError, SjmlResult};
pub use gl::{
    GlBlendFactor, GlBufferTarget, GlBufferUsage, GlCapabilities, GlCommand, GlCullFace,
    GlDepthFunction, GlExecutionReport, GlFramebufferAttachment, GlIndexType, GlPrimitiveMode,
    GlResourceDescriptor, GlScalarType,
};
pub use inventory::{BlockPos, Direction, InventorySnapshot, ItemStackDescriptor};
pub use menu::{MenuAction, MenuDefinition, MenuItem};
pub use network::{
    ChannelRef, LogicalSide, NetworkContext, NetworkContextSnapshot, NetworkDirection,
    NetworkSubscriptionHandle, Player as NetworkPlayer, Uuid,
};
pub use player::PlayerSnapshot;
pub use recipe::{
    CraftingSlot, RecipeEvaluation, RecipeMatchInput, RECIPE_CALLBACK_SCHEMA_VERSION,
};
pub use registry::{
    BlockEntityModelDefinition, EntityModelDefinition, ModelAnimation, ModelAnimationAxis,
    ModelCuboid, MAX_BOUND_MODEL_CUBOIDS,
};
pub use render::{
    RenderContext, RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle,
    WorldModelBuilder, WorldRenderFrame, MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS,
    MAX_WORLD_MODEL_VERTICES,
};
pub use screen_input::{ScreenInputEvent, ScreenInputSubscriptionHandle};
pub use serialization::{
    from_bytes, to_bytes, Decoder, Encoder, StableDecode, StableEncode, MAX_COLLECTION_ITEMS,
    MAX_PAYLOAD_BYTES, MAX_STRING_BYTES,
};
pub use sjml_macros::{
    command, compute, event, event_async, mod_init, mod_register, mod_shutdown, recipe, service,
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
pub use world::{BlockStateSnapshot, MAX_WORLD_BLOCK_STATES};

/// Common imports for an SJML guest module.
pub mod prelude {
    pub use crate::abi::{
        BlockEntityHandle, BlockHandle, EntityHandle, EventSubscriptionHandle, GlResourceHandle,
        ItemStackHandle, NetworkChannelHandle, PlayerHandle, RenderResourceHandle,
        StorageScopeHandle, TaskHandle, WorldHandle,
    };
    pub use crate::batch::{self, Batch, BatchValue, CallResult};
    pub use crate::block_entity::{self, BlockEntity, BlockEntitySnapshot, NbtValue};
    pub use crate::client_input;
    pub use crate::client_input::{
        ClientInputAction, ClientInputEvent, ClientInputSubscriptionHandle,
        MAX_CLIENT_INPUT_SUBSCRIPTIONS,
    };
    pub use crate::entity;
    pub use crate::entity::EntitySnapshot;
    pub use crate::entity_data;
    pub use crate::entity_data::{
        ENTITY_DATA_MISSING, MAX_ENTITY_DATA_KEY_BYTES, MAX_ENTITY_DATA_VALUE_BYTES,
    };
    pub use crate::error::{ErrorCode, SjmlError, SjmlResult};
    pub use crate::events;
    pub use crate::events::{
        subscribe, subscribe_with_interval, unsubscribe, BlockBreak, BlockPlace, EffectTickEvent,
        EntityDamaged, EntityDeath, EntitySpawned, EventPriority, InventoryChanged,
        PlayerInteracted, PlayerJoined, PlayerLeft, ResourcesReloaded, ServerTick, WorldLifecycle,
        DEFAULT_SERVER_TICK_INTERVAL, MAX_INVENTORY_CHANGE_COUNT, MAX_INVENTORY_CHANGE_SLOT,
        MAX_SERVER_TICK_INTERVAL,
    };
    pub use crate::gl;
    pub use crate::gl::{
        GlBlendFactor, GlBufferTarget, GlBufferUsage, GlCapabilities, GlCommand, GlCullFace,
        GlDepthFunction, GlExecutionReport, GlFramebufferAttachment, GlIndexType, GlPrimitiveMode,
        GlResourceDescriptor, GlScalarType,
    };
    pub use crate::host::{self, LogLevel};
    pub use crate::inventory;
    pub use crate::inventory::{BlockPos, Direction, InventorySnapshot, ItemStackDescriptor};
    pub use crate::menu;
    pub use crate::menu::{MenuAction, MenuDefinition, MenuItem};
    pub use crate::network;
    pub use crate::network::{
        broadcast, send_to, send_to_server, subscribe as subscribe_network,
        unsubscribe as unsubscribe_network, ChannelRef, LogicalSide, NetworkContext,
        NetworkContextSnapshot, NetworkDirection, NetworkSubscriptionHandle, Player, Uuid,
    };
    pub use crate::player;
    pub use crate::player::PlayerSnapshot;
    pub use crate::recipe::{CraftingSlot, RecipeEvaluation, RecipeMatchInput};
    pub use crate::registry;
    pub use crate::registry::{
        add_to_creative_tab, custom_registry_value, register_attribute_effect, register_block,
        register_block_entity, register_block_entity_model, register_cooking_recipe,
        register_creative_tab, register_custom_registry, register_custom_value,
        register_effect_tick, register_entity, register_entity_ai_goal, register_entity_model,
        register_fluid, register_fluid_with_visuals, register_item, register_particle,
        register_recipe_serializer, register_shaped_recipe, register_shapeless_recipe,
        register_sound, register_string_component, set_entity_attribute, set_item_string_component,
        AttributeModifierOperation, BlockDefinition, BlockEntityModelDefinition,
        CookingRecipeDefinition, CookingRecipeKind, CreativeTabDefinition, EffectCategory,
        EntityAiGoal, EntityAttribute, EntityDefinition, EntityModelDefinition, FluidDefinition,
        FluidVisualDefinition, ItemDefinition, ModelAnimation, ModelAnimationAxis, ModelCuboid,
        RecipeIngredient, ShapedRecipeDefinition, ShapedRecipeKey, ShapelessRecipeDefinition,
        StringComponentDefinition, MAX_BOUND_MODEL_CUBOIDS,
    };
    pub use crate::render;
    pub use crate::render::{
        RenderContext, RenderFont, RenderFrame, RenderLayer, RenderSubscriptionHandle,
        WorldModelBuilder, WorldRenderFrame, MAX_RENDER_RESOURCE_PATH_BYTES, MAX_WORLD_MODEL_QUADS,
        MAX_WORLD_MODEL_VERTICES,
    };
    pub use crate::screen_input;
    pub use crate::screen_input::{
        ScreenInputEvent, ScreenInputSubscriptionHandle, MAX_SCREEN_INPUT_COORDINATE,
        MAX_SCREEN_INPUT_SUBSCRIPTIONS,
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
        BlockStateSnapshot, RaycastBlockMode, RaycastFluidMode, RaycastHit, RaycastPoint,
        MAX_WORLD_BLOCK_STATES, MAX_WORLD_RAYCAST_RESPONSE_BYTES, MAX_WORLD_RAY_COORDINATE,
        MAX_WORLD_RAY_DISTANCE,
    };
    pub use crate::{
        command, compute, event, event_async, mod_init, mod_register, mod_shutdown, recipe, service,
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
