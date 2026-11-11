//! Static content declarations captured before Minecraft registry freeze.
//!
//! These functions are valid only from `#[mod_register]`. They stage bounded
//! item, block, block-entity, generic entity and its attributes, creative-tab,
//! fluid, shaped/shapeless/cooking recipe, particle, sound, bounded attribute
//! effect and its optional tick callback, and string data component declarations; they do not mutate world
//! state or registries directly.

use crate::{ErrorCode, SjmlError, SjmlResult};

const MAX_CONTENT_PATH_BYTES: usize = 256;
const MAX_FLUID_PATH_BYTES: usize = MAX_CONTENT_PATH_BYTES - "_flowing".len();
const MAX_FLUID_TEXTURE_ID_BYTES: usize = 256;
const MAX_COMPONENT_VALUE_BYTES: usize = 4096;
const MAX_RECIPE_ITEM_ID_BYTES: usize = 256;
const MAX_RECIPE_INGREDIENTS: usize = 9;
const MAX_RECIPE_PATTERN_ROWS: usize = 3;
const MAX_RECIPE_PATTERN_WIDTH: usize = 3;
const MAX_RECIPE_KEY_LIST_BYTES: usize = 4096;
const MAX_RECIPE_COOKING_TIME_TICKS: u32 = 36_000;
const MAX_RECIPE_EXPERIENCE: f32 = 1_000.0;
const MAX_CUSTOM_REGISTRY_VALUE_BYTES: usize = 65_536;
const MAX_PARTICLE_TEXTURE_ID_BYTES: usize = 256;
const MIN_PARTICLE_SCALE: f32 = 0.05;
const MAX_PARTICLE_SCALE: f32 = 4.0;
/// Maximum colored cuboids in one static entity or block-entity model.
pub const MAX_BOUND_MODEL_CUBOIDS: usize = 32;
const MAX_MODEL_TEXTURE_ID_BYTES: usize = 256;
const BOUND_MODEL_SCHEMA_VERSION: u32 = 1;
const BOUND_MODEL_ANIMATION_SCHEMA_VERSION: u32 = 2;
const BOUND_MODEL_CUBOID_ANIMATION_SCHEMA_VERSION: u32 = 3;
const BOUND_MODEL_HEADER_BYTES: usize = 8;
const BOUND_MODEL_CUBOID_BYTES: usize = 28;
const BOUND_MODEL_ANIMATION_BYTES: usize = 32;
const MAX_MODEL_ANIMATION_PERIOD_TICKS: u32 = 12_000;
#[cfg(target_arch = "wasm32")]
const CUSTOM_REGISTRY_VALUE_FRAME_BYTES: u32 = (MAX_CUSTOM_REGISTRY_VALUE_BYTES + 9) as u32;

/// Properties for an item registered by this module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemDefinition {
    pub max_stack_size: u32,
    pub fire_resistant: bool,
}

impl Default for ItemDefinition {
    fn default() -> Self {
        Self {
            max_stack_size: 64,
            fire_resistant: false,
        }
    }
}

/// Properties for a basic block registered by this module.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockDefinition {
    pub strength: f32,
    pub resistance: f32,
    pub requires_correct_tool: bool,
    pub has_item_form: bool,
}

/// One bounded vanilla AI goal installed on a registered living entity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntityAiGoal {
    /// Swim when in fluid.
    Swim { priority: u32 },
    /// Walk to nearby positions at a bounded movement speed.
    RandomStroll { priority: u32, speed: f32 },
    /// Look at a nearby player within a bounded distance.
    LookAtPlayer { priority: u32, range: f32 },
    /// Attack a visible nearby target at a bounded movement speed.
    MeleeAttack {
        priority: u32,
        speed: f32,
        long_memory: bool,
    },
    /// Look around when no higher-priority goal is active.
    RandomLookAround { priority: u32 },
    /// Select nearby players as targets.
    TargetPlayer { priority: u32, must_see: bool },
    /// Target entities that previously hurt this entity.
    Retaliate { priority: u32 },
}

/// Dimensions and tracking policy for the host's generic living entity type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EntityDefinition {
    pub width: f32,
    pub height: f32,
    pub tracking_range: u32,
    pub update_interval: u32,
    pub fire_immune: bool,
}

/// One local-space colored cuboid used by an entity or block-entity model.
///
/// A model may apply one package-owned texture to all of its cuboids.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelCuboid {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub argb: u32,
}

/// Axis used by a deterministic sine-wave model rotation.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelAnimationAxis {
    X = 0,
    Y = 1,
    Z = 2,
}

/// A bounded client-side sine rotation evaluated from Minecraft tick time.
///
/// This animation is evaluated by the host renderer; it does not call guest code per frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelAnimation {
    pub axis: ModelAnimationAxis,
    pub pivot: [f32; 3],
    pub amplitude_degrees: f32,
    pub period_ticks: u32,
    pub phase_ticks: u32,
}

impl ModelAnimation {
    /// Creates a sine rotation with an amplitude up to 180 degrees and a period up to 10 minutes.
    pub fn sine_rotation(
        axis: ModelAnimationAxis,
        pivot: [f32; 3],
        amplitude_degrees: f32,
        period_ticks: u32,
        phase_ticks: u32,
    ) -> SjmlResult<Self> {
        if pivot.iter().any(|value| !value.is_finite())
            || !amplitude_degrees.is_finite()
            || !(-180.0..=180.0).contains(&amplitude_degrees)
            || !(1..=MAX_MODEL_ANIMATION_PERIOD_TICKS).contains(&period_ticks)
            || phase_ticks >= period_ticks
        {
            return Err(invalid_argument(
                "model animation parameters are outside their supported bounds",
            ));
        }
        Ok(Self {
            axis,
            pivot,
            amplitude_degrees,
            period_ticks,
            phase_ticks,
        })
    }
}

impl ModelCuboid {
    /// Creates a cuboid after checking finite ordered bounds and a local extent no larger than 8.
    pub fn new(min: [f32; 3], max: [f32; 3], argb: u32) -> SjmlResult<Self> {
        validate_model_cuboid(min, max, 4.0)?;
        Ok(Self { min, max, argb })
    }
}

/// Static geometry for an entity model, expressed in entity-local world units.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntityModelDefinition {
    cuboids: Vec<ModelCuboid>,
    cuboid_animations: Vec<Option<ModelAnimation>>,
    texture: Option<String>,
    animation: Option<ModelAnimation>,
}

impl EntityModelDefinition {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a bounded local-space cuboid. Coordinates are limited to +/-4 units.
    pub fn add_cuboid(&mut self, min: [f32; 3], max: [f32; 3], argb: u32) -> SjmlResult<&mut Self> {
        validate_model_cuboid(min, max, 4.0)?;
        push_model_cuboid(&mut self.cuboids, ModelCuboid { min, max, argb })?;
        self.cuboid_animations.push(None);
        Ok(self)
    }

    /// Uses a package-owned PNG texture at a Minecraft resource path such as
    /// `example:textures/entity/clockwork_mite.png` for every cuboid face.
    pub fn set_texture(&mut self, texture: impl Into<String>) -> SjmlResult<&mut Self> {
        let texture = texture.into();
        validate_model_texture(&texture)?;
        self.texture = Some(texture);
        Ok(self)
    }

    /// Adds a host-evaluated sine rotation around the entity-local pivot.
    pub fn set_animation(&mut self, animation: ModelAnimation) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, -4.0, 4.0)?;
        self.animation = Some(animation);
        Ok(self)
    }

    /// Animates one cuboid around an entity-local pivot, independently of sibling cuboids.
    /// The index is the zero-based order in which cuboids were added.
    pub fn set_cuboid_animation(
        &mut self,
        index: usize,
        animation: ModelAnimation,
    ) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, -4.0, 4.0)?;
        let track = self.cuboid_animations.get_mut(index).ok_or_else(|| {
            invalid_argument("cuboid animation index does not refer to a model cuboid")
        })?;
        *track = Some(animation);
        Ok(self)
    }

    /// Removes the independent animation from one cuboid.
    pub fn clear_cuboid_animation(&mut self, index: usize) -> SjmlResult<&mut Self> {
        let track = self.cuboid_animations.get_mut(index).ok_or_else(|| {
            invalid_argument("cuboid animation index does not refer to a model cuboid")
        })?;
        *track = None;
        Ok(self)
    }

    pub fn cuboid_count(&self) -> usize {
        self.cuboids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cuboids.is_empty()
    }
}

/// Static geometry for a block-entity model in block-local coordinates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlockEntityModelDefinition {
    cuboids: Vec<ModelCuboid>,
    cuboid_animations: Vec<Option<ModelAnimation>>,
    texture: Option<String>,
    animation: Option<ModelAnimation>,
}

impl BlockEntityModelDefinition {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a bounded cuboid inside the owning block's unit cube.
    pub fn add_cuboid(&mut self, min: [f32; 3], max: [f32; 3], argb: u32) -> SjmlResult<&mut Self> {
        validate_model_cuboid(min, max, 1.0)?;
        if min
            .into_iter()
            .chain(max)
            .any(|coordinate| coordinate < 0.0)
        {
            return Err(invalid_argument(
                "block-entity model cuboids must stay within [0, 1]",
            ));
        }
        push_model_cuboid(&mut self.cuboids, ModelCuboid { min, max, argb })?;
        self.cuboid_animations.push(None);
        Ok(self)
    }

    /// Uses a package-owned PNG texture at a Minecraft resource path such as
    /// `example:textures/block/clockwork_core.png` for every cuboid face.
    pub fn set_texture(&mut self, texture: impl Into<String>) -> SjmlResult<&mut Self> {
        let texture = texture.into();
        validate_model_texture(&texture)?;
        self.texture = Some(texture);
        Ok(self)
    }

    /// Adds a host-evaluated sine rotation around a block-local pivot.
    pub fn set_animation(&mut self, animation: ModelAnimation) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, 0.0, 1.0)?;
        self.animation = Some(animation);
        Ok(self)
    }

    /// Animates one cuboid around a block-local pivot, independently of sibling cuboids.
    /// The index is the zero-based order in which cuboids were added.
    pub fn set_cuboid_animation(
        &mut self,
        index: usize,
        animation: ModelAnimation,
    ) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, 0.0, 1.0)?;
        let track = self.cuboid_animations.get_mut(index).ok_or_else(|| {
            invalid_argument("cuboid animation index does not refer to a model cuboid")
        })?;
        *track = Some(animation);
        Ok(self)
    }

    /// Removes the independent animation from one cuboid.
    pub fn clear_cuboid_animation(&mut self, index: usize) -> SjmlResult<&mut Self> {
        let track = self.cuboid_animations.get_mut(index).ok_or_else(|| {
            invalid_argument("cuboid animation index does not refer to a model cuboid")
        })?;
        *track = None;
        Ok(self)
    }

    pub fn cuboid_count(&self) -> usize {
        self.cuboids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cuboids.is_empty()
    }
}

/// Bounded flow behavior for a simple source/flowing fluid using vanilla water visuals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidDefinition {
    pub tick_rate: u32,
    pub level_decrease_per_block: u32,
    pub slope_find_distance: u32,
    pub explosion_resistance: f32,
}

/// Package-owned still and flowing texture IDs for a registered fluid.
///
/// IDs use Minecraft's texture resource convention, for example
/// `example:block/ruby_still` refers to `assets/example/textures/block/ruby_still.png`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FluidVisualDefinition {
    still_texture: String,
    flowing_texture: String,
}

impl FluidVisualDefinition {
    /// Creates a fluid skin after validating both namespaced texture IDs.
    pub fn new(
        still_texture: impl Into<String>,
        flowing_texture: impl Into<String>,
    ) -> SjmlResult<Self> {
        let definition = Self {
            still_texture: still_texture.into(),
            flowing_texture: flowing_texture.into(),
        };
        validate_fluid_texture(&definition.still_texture)?;
        validate_fluid_texture(&definition.flowing_texture)?;
        Ok(definition)
    }

    pub fn still_texture(&self) -> &str {
        &self.still_texture
    }

    pub fn flowing_texture(&self) -> &str {
        &self.flowing_texture
    }
}

/// Package-owned atlas texture and bounded render scale for a simple SJML particle.
///
/// Particle motion, lifetime, and blend behavior stay on the host's safe vanilla defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleDefinition {
    texture: String,
    scale: f32,
}

impl ParticleDefinition {
    /// Uses an atlas texture such as `example:particle/ruby_spark`.
    /// The package must include it at `assets/example/textures/particle/ruby_spark.png`.
    pub fn new(texture: impl Into<String>) -> SjmlResult<Self> {
        let texture = texture.into();
        validate_particle_texture(&texture)?;
        Ok(Self {
            texture,
            scale: 0.25,
        })
    }

    /// Sets the particle's base render scale, bounded to 0.05–4.0 times the vanilla quad size.
    pub fn with_scale(mut self, scale: f32) -> SjmlResult<Self> {
        validate_particle_scale(scale)?;
        self.scale = scale;
        Ok(self)
    }

    /// Returns the package-owned particle atlas texture ID.
    pub fn texture(&self) -> &str {
        &self.texture
    }

    /// Returns the bounded base render scale.
    pub fn scale(&self) -> f32 {
        self.scale
    }
}

/// A single item ID or item-tag ingredient for a vanilla recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecipeIngredient {
    /// Matches only this item.
    Item(String),
    /// Matches every item listed by this item tag.
    Tag(String),
}

impl RecipeIngredient {
    /// Creates an ingredient that matches one item ID.
    pub fn item(id: impl Into<String>) -> Self {
        Self::Item(id.into())
    }

    /// Creates an ingredient that matches an item tag ID.
    pub fn tag(id: impl Into<String>) -> Self {
        Self::Tag(id.into())
    }
}

impl From<String> for RecipeIngredient {
    fn from(id: String) -> Self {
        Self::Item(id)
    }
}

impl From<&str> for RecipeIngredient {
    fn from(id: &str) -> Self {
        Self::Item(id.to_owned())
    }
}

/// Output and ingredients for a vanilla shapeless crafting recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapelessRecipeDefinition {
    pub output_item: String,
    pub output_count: u32,
    pub ingredients: Vec<RecipeIngredient>,
}

/// One uppercase symbol in a shaped recipe pattern and its required ingredient.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapedRecipeKey {
    pub symbol: char,
    pub ingredient: RecipeIngredient,
}

/// Output, 1–3 row pattern, and ingredient bindings for a vanilla shaped recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapedRecipeDefinition {
    pub output_item: String,
    pub output_count: u32,
    pub pattern: Vec<String>,
    pub key: Vec<ShapedRecipeKey>,
}

/// Vanilla recipe serializer used by a single-item cooking recipe.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CookingRecipeKind {
    Smelting = 0,
    Blasting = 1,
    Smoking = 2,
    CampfireCooking = 3,
}

/// Input, output, and bounded timing data for a vanilla cooking recipe.
#[derive(Clone, Debug, PartialEq)]
pub struct CookingRecipeDefinition {
    pub kind: CookingRecipeKind,
    pub ingredient: RecipeIngredient,
    pub output_item: String,
    pub output_count: u32,
    pub experience: f32,
    pub cooking_time: u32,
}

/// A bounded vanilla base attribute supported by a generic SJML living entity.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityAttribute {
    MaxHealth = 0,
    MovementSpeed = 1,
    AttackDamage = 2,
    Armor = 3,
    FollowRange = 4,
    KnockbackResistance = 5,
}

/// Display category for a guest-declared status effect.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectCategory {
    Beneficial = 0,
    Harmful = 1,
    Neutral = 2,
}

/// Vanilla operation used by a custom status effect's one attribute modifier.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributeModifierOperation {
    AddValue = 0,
    AddMultipliedBase = 1,
    AddMultipliedTotal = 2,
}

/// Title and module-owned icon item for a registered creative tab.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreativeTabDefinition {
    pub title: String,
    pub icon_path: String,
}

/// Maximum Unicode scalar count for a persistent/networked string data component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StringComponentDefinition {
    pub max_characters: u32,
}

impl Default for StringComponentDefinition {
    fn default() -> Self {
        Self {
            max_characters: 256,
        }
    }
}

impl CreativeTabDefinition {
    /// Creates a tab definition after validating its plain-text title and icon path.
    pub fn new(title: impl Into<String>, icon_path: impl Into<String>) -> SjmlResult<Self> {
        let definition = Self {
            title: title.into(),
            icon_path: icon_path.into(),
        };
        validate_title(&definition.title)?;
        validate_path(&definition.icon_path)?;
        Ok(definition)
    }
}

impl Default for EntityDefinition {
    fn default() -> Self {
        Self {
            width: 0.6,
            height: 1.8,
            tracking_range: 8,
            update_interval: 3,
            fire_immune: false,
        }
    }
}

impl Default for FluidDefinition {
    fn default() -> Self {
        Self {
            tick_rate: 5,
            level_decrease_per_block: 1,
            slope_find_distance: 4,
            explosion_resistance: 100.0,
        }
    }
}

impl Default for BlockDefinition {
    fn default() -> Self {
        Self {
            strength: 1.0,
            resistance: 1.0,
            requires_correct_tool: false,
            has_item_form: true,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_registry_v1")]
extern "C" {
    #[link_name = "register_item"]
    fn imported_register_item(
        path_pointer: i32,
        path_length: i32,
        max_stack_size: i32,
        fire_resistant: i32,
    ) -> i32;
    #[link_name = "register_block"]
    fn imported_register_block(
        path_pointer: i32,
        path_length: i32,
        strength_bits: i32,
        resistance_bits: i32,
        requires_correct_tool: i32,
        has_item_form: i32,
    ) -> i32;
    #[link_name = "register_block_entity"]
    fn imported_register_block_entity(
        path_pointer: i32,
        path_length: i32,
        block_path_pointer: i32,
        block_path_length: i32,
    ) -> i32;
    #[link_name = "register_fluid"]
    fn imported_register_fluid(
        path_pointer: i32,
        path_length: i32,
        tick_rate: i32,
        level_decrease_per_block: i32,
        slope_find_distance: i32,
        explosion_resistance_bits: i32,
    ) -> i32;
    #[link_name = "register_fluid_with_visuals"]
    fn imported_register_fluid_with_visuals(
        path_pointer: i32,
        path_length: i32,
        tick_rate: i32,
        level_decrease_per_block: i32,
        slope_find_distance: i32,
        explosion_resistance_bits: i32,
        still_texture_pointer: i32,
        still_texture_length: i32,
        flowing_texture_pointer: i32,
        flowing_texture_length: i32,
    ) -> i32;
    #[link_name = "register_particle"]
    fn imported_register_particle(path_pointer: i32, path_length: i32) -> i32;
    #[link_name = "register_particle_visual"]
    fn imported_register_particle_visual(
        path_pointer: i32,
        path_length: i32,
        texture_pointer: i32,
        texture_length: i32,
        scale_bits: i32,
    ) -> i32;
    #[link_name = "register_shapeless_recipe"]
    fn imported_register_shapeless_recipe(
        path_pointer: i32,
        path_length: i32,
        output_item_pointer: i32,
        output_item_length: i32,
        output_count: i32,
        ingredient_items_pointer: i32,
        ingredient_items_length: i32,
    ) -> i32;
    #[link_name = "register_shaped_recipe"]
    fn imported_register_shaped_recipe(
        path_pointer: i32,
        path_length: i32,
        output_item_pointer: i32,
        output_item_length: i32,
        output_count: i32,
        pattern_pointer: i32,
        pattern_length: i32,
        key_items_pointer: i32,
        key_items_length: i32,
    ) -> i32;
    #[link_name = "register_cooking_recipe"]
    fn imported_register_cooking_recipe(
        path_pointer: i32,
        path_length: i32,
        output_item_pointer: i32,
        output_item_length: i32,
        output_count: i32,
        ingredient_item_pointer: i32,
        ingredient_item_length: i32,
        kind: i32,
        experience_bits: i32,
        cooking_time: i32,
    ) -> i32;
    #[link_name = "register_entity"]
    fn imported_register_entity(
        path_pointer: i32,
        path_length: i32,
        width_bits: i32,
        height_bits: i32,
        tracking_range: i32,
        update_interval: i32,
        fire_immune: i32,
    ) -> i32;
    #[link_name = "register_entity_ai_goal"]
    fn imported_register_entity_ai_goal(
        path_pointer: i32,
        path_length: i32,
        goal: i32,
        priority: i32,
        value_bits: i32,
        flags: i32,
    ) -> i32;
    #[link_name = "register_entity_model"]
    fn imported_register_entity_model(
        path_pointer: i32,
        path_length: i32,
        model_pointer: i32,
        model_length: i32,
    ) -> i32;
    #[link_name = "register_entity_textured_model"]
    fn imported_register_entity_textured_model(
        path_pointer: i32,
        path_length: i32,
        texture_pointer: i32,
        texture_length: i32,
        model_pointer: i32,
        model_length: i32,
    ) -> i32;
    #[link_name = "register_block_entity_model"]
    fn imported_register_block_entity_model(
        path_pointer: i32,
        path_length: i32,
        model_pointer: i32,
        model_length: i32,
    ) -> i32;
    #[link_name = "register_block_entity_textured_model"]
    fn imported_register_block_entity_textured_model(
        path_pointer: i32,
        path_length: i32,
        texture_pointer: i32,
        texture_length: i32,
        model_pointer: i32,
        model_length: i32,
    ) -> i32;
    #[link_name = "set_entity_attribute"]
    fn imported_set_entity_attribute(
        path_pointer: i32,
        path_length: i32,
        attribute: i32,
        value_bits: i32,
    ) -> i32;
    #[link_name = "register_sound"]
    fn imported_register_sound(path_pointer: i32, path_length: i32) -> i32;
    #[link_name = "register_attribute_effect"]
    fn imported_register_attribute_effect(
        path_pointer: i32,
        path_length: i32,
        category: i32,
        color: i32,
        attribute: i32,
        amount_bits: i32,
        operation: i32,
    ) -> i32;
    #[link_name = "register_effect_tick"]
    fn imported_register_effect_tick(
        path_pointer: i32,
        path_length: i32,
        interval_ticks: i32,
    ) -> i32;
    #[link_name = "register_custom_registry"]
    fn imported_register_custom_registry(
        path_pointer: i32,
        path_length: i32,
        sync_to_clients: i32,
    ) -> i32;
    #[link_name = "register_recipe_serializer"]
    fn imported_register_recipe_serializer(path_pointer: i32, path_length: i32) -> i32;
    #[link_name = "register_custom_value"]
    fn imported_register_custom_value(
        registry_path_pointer: i32,
        registry_path_length: i32,
        value_path_pointer: i32,
        value_path_length: i32,
        value_pointer: i32,
        value_length: i32,
    ) -> i32;
    #[link_name = "get_custom_registry_value"]
    fn imported_get_custom_registry_value(
        registry_path_pointer: i32,
        registry_path_length: i32,
        value_path_pointer: i32,
        value_path_length: i32,
        output_pointer: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "register_creative_tab"]
    fn imported_register_creative_tab(
        path_pointer: i32,
        path_length: i32,
        title_pointer: i32,
        title_length: i32,
        icon_path_pointer: i32,
        icon_path_length: i32,
    ) -> i32;
    #[link_name = "add_to_creative_tab"]
    fn imported_add_to_creative_tab(
        tab_path_pointer: i32,
        tab_path_length: i32,
        item_path_pointer: i32,
        item_path_length: i32,
    ) -> i32;
    #[link_name = "register_string_component"]
    fn imported_register_string_component(
        path_pointer: i32,
        path_length: i32,
        max_characters: i32,
    ) -> i32;
    #[link_name = "set_item_string_component"]
    fn imported_set_item_string_component(
        item_path_pointer: i32,
        item_path_length: i32,
        component_path_pointer: i32,
        component_path_length: i32,
        value_pointer: i32,
        value_length: i32,
    ) -> i32;
}

/// Stages an item. The path is relative to this module's resource namespace.
pub fn register_item(path: &str, definition: ItemDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if !(1..=99).contains(&definition.max_stack_size) {
        return Err(invalid_argument("item stack size must be between 1 and 99"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host call copies this bounded UTF-8 slice before returning.
        let status = unsafe {
            imported_register_item(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.max_stack_size as i32,
                i32::from(definition.fire_resistant),
            )
        };
        return status_result(status, "host rejected the item declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Stages a block. Its default item form is registered in the later item phase.
pub fn register_block(path: &str, definition: BlockDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if !definition.strength.is_finite()
        || definition.strength < 0.0
        || !definition.resistance.is_finite()
        || definition.resistance < 0.0
    {
        return Err(invalid_argument(
            "block strength and resistance must be finite and non-negative",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host call copies this bounded UTF-8 slice before returning.
        let status = unsafe {
            imported_register_block(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.strength.to_bits() as i32,
                definition.resistance.to_bits() as i32,
                i32::from(definition.requires_correct_tool),
                i32::from(definition.has_item_form),
            )
        };
        return status_result(status, "host rejected the block declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Stages a block entity backed by another block declared by this same module.
pub fn register_block_entity(path: &str, block_path: &str) -> SjmlResult<()> {
    validate_path(path)?;
    validate_path(block_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_register_block_entity(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                block_path.as_ptr() as usize as u32 as i32,
                block_path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the block entity declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Stages a flowing fluid, its source/flowing states, liquid block, and bucket item.
/// Fluids use vanilla water visuals unless [`register_fluid_with_visuals`] is used.
pub fn register_fluid(path: &str, definition: FluidDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > MAX_FLUID_PATH_BYTES {
        return Err(invalid_argument(
            "fluid path must leave room for generated registry suffixes",
        ));
    }
    validate_fluid_definition(definition)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and decodes the fixed-width flow settings.
        let status = unsafe {
            imported_register_fluid(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.tick_rate as i32,
                definition.level_decrease_per_block as i32,
                definition.slope_find_distance as i32,
                definition.explosion_resistance.to_bits() as i32,
            )
        };
        return status_result(status, "host rejected the fluid declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Stages a fluid with package-owned still and flowing textures.
///
/// Each texture ID must be in this package's namespace and use the `block/` path,
/// such as `ruby_pack:block/ruby_still`. The package archive must contain the
/// matching PNG at `assets/ruby_pack/textures/block/ruby_still.png`.
pub fn register_fluid_with_visuals(
    path: &str,
    definition: FluidDefinition,
    visuals: FluidVisualDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > MAX_FLUID_PATH_BYTES {
        return Err(invalid_argument(
            "fluid path must leave room for generated registry suffixes",
        ));
    }
    validate_fluid_definition(definition)?;
    validate_fluid_texture(&visuals.still_texture)?;
    validate_fluid_texture(&visuals.flowing_texture)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies these bounded strings and decodes the fixed-width flow settings.
        let status = unsafe {
            imported_register_fluid_with_visuals(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.tick_rate as i32,
                definition.level_decrease_per_block as i32,
                definition.slope_find_distance as i32,
                definition.explosion_resistance.to_bits() as i32,
                visuals.still_texture.as_ptr() as usize as u32 as i32,
                visuals.still_texture.len() as u32 as i32,
                visuals.flowing_texture.as_ptr() as usize as u32 as i32,
                visuals.flowing_texture.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the fluid visual declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (definition, visuals);
        Err(unsupported_target())
    }
}

fn validate_fluid_definition(definition: FluidDefinition) -> SjmlResult<()> {
    if !(1..=1200).contains(&definition.tick_rate)
        || !(1..=8).contains(&definition.level_decrease_per_block)
        || !(1..=16).contains(&definition.slope_find_distance)
        || !definition.explosion_resistance.is_finite()
        || !(0.0..=1_000_000.0).contains(&definition.explosion_resistance)
    {
        return Err(invalid_argument(
            "fluid flow settings are outside their supported bounds",
        ));
    }
    Ok(())
}

/// Stages a simple particle type with SJML's shared `minecraft:generic_0` sprite renderer.
/// Use [`register_particle_with_definition`] to select a package-owned texture and base scale.
pub fn register_particle(path: &str) -> SjmlResult<()> {
    validate_path(path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded UTF-8 path before returning.
        let status = unsafe {
            imported_register_particle(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the particle declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Stages a simple particle using a package-owned PNG sprite and bounded base scale.
///
/// The host keeps vanilla motion, lifetime, and translucent rendering defaults. The particle
/// texture ID must be in the package namespace and use the `particle/` atlas path.
pub fn register_particle_with_definition(
    path: &str,
    definition: ParticleDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    validate_particle_texture(&definition.texture)?;
    validate_particle_scale(definition.scale)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded UTF-8 strings before the import returns.
        let status = unsafe {
            imported_register_particle_visual(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.texture.as_ptr() as usize as u32 as i32,
                definition.texture.len() as u32 as i32,
                definition.scale.to_bits() as i32,
            )
        };
        status_result(status, "host rejected the particle visual declaration")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Stages a single-output shapeless recipe parsed by Minecraft's vanilla serializer.
///
/// Unqualified item and tag paths refer to this package and module; namespaced IDs can
/// refer to vanilla data or content registered by another mod.
pub fn register_shapeless_recipe(
    path: &str,
    definition: ShapelessRecipeDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 || path == "." || path == ".." {
        return Err(invalid_argument(
            "recipe path must be at most 128 bytes and cannot be . or ..",
        ));
    }
    if !(1..=64).contains(&definition.output_count) {
        return Err(invalid_argument(
            "recipe output count must be between 1 and 64",
        ));
    }
    if definition.ingredients.is_empty() || definition.ingredients.len() > MAX_RECIPE_INGREDIENTS {
        return Err(invalid_argument(
            "shapeless recipes must have between 1 and 9 ingredients",
        ));
    }
    validate_recipe_item_id(&definition.output_item)?;
    let mut ingredient_tokens = Vec::with_capacity(definition.ingredients.len());
    for ingredient in &definition.ingredients {
        ingredient_tokens.push(encode_recipe_ingredient(ingredient)?);
    }
    let ingredient_list = ingredient_tokens.join("\n");
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies all bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_register_shapeless_recipe(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.output_item.as_ptr() as usize as u32 as i32,
                definition.output_item.len() as u32 as i32,
                definition.output_count as i32,
                ingredient_list.as_ptr() as usize as u32 as i32,
                ingredient_list.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the shapeless recipe declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ingredient_list;
        Err(unsupported_target())
    }
}

/// Stages a shaped recipe for Minecraft's vanilla serializer.
///
/// Pattern rows are one to three uppercase ASCII letters wide. A space marks
/// an unused cell. Every used letter must appear exactly once in `key`, and
/// every key letter must occur in the pattern. IDs without a namespace refer
/// to this package and module. Bindings may name an item or an item tag.
pub fn register_shaped_recipe(path: &str, definition: ShapedRecipeDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 || path == "." || path == ".." {
        return Err(invalid_argument(
            "recipe path must be at most 128 bytes and cannot be . or ..",
        ));
    }
    if !(1..=64).contains(&definition.output_count) {
        return Err(invalid_argument(
            "recipe output count must be between 1 and 64",
        ));
    }
    validate_shaped_recipe(&definition)?;
    validate_recipe_item_id(&definition.output_item)?;

    let pattern = definition.pattern.join("\n");
    let mut keys = definition.key;
    keys.sort_by_key(|key| key.symbol);
    let key_items = keys
        .iter()
        .map(|key| {
            encode_recipe_ingredient(&key.ingredient)
                .map(|ingredient| format!("{}\n{ingredient}", key.symbol))
        })
        .collect::<SjmlResult<Vec<_>>>()?
        .join("\n");
    if pattern.len() > 11 || key_items.len() > MAX_RECIPE_KEY_LIST_BYTES {
        return Err(invalid_argument(
            "shaped recipe pattern or key bindings exceed their supported limits",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies these bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_register_shaped_recipe(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.output_item.as_ptr() as usize as u32 as i32,
                definition.output_item.len() as u32 as i32,
                definition.output_count as i32,
                pattern.as_ptr() as usize as u32 as i32,
                pattern.len() as u32 as i32,
                key_items.as_ptr() as usize as u32 as i32,
                key_items.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the shaped recipe declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (pattern, key_items);
        Err(unsupported_target())
    }
}

fn validate_shaped_recipe(definition: &ShapedRecipeDefinition) -> SjmlResult<()> {
    if definition.pattern.is_empty() || definition.pattern.len() > MAX_RECIPE_PATTERN_ROWS {
        return Err(invalid_argument(
            "shaped recipes must have between 1 and 3 pattern rows",
        ));
    }
    let width = definition.pattern[0].len();
    if !(1..=MAX_RECIPE_PATTERN_WIDTH).contains(&width) {
        return Err(invalid_argument(
            "shaped recipe rows must be between 1 and 3 ASCII cells wide",
        ));
    }
    let mut used_symbols = std::collections::BTreeSet::new();
    for row in &definition.pattern {
        if row.len() != width || !row.is_ascii() {
            return Err(invalid_argument(
                "shaped recipe rows must have the same ASCII width",
            ));
        }
        for symbol in row.chars().filter(|symbol| *symbol != ' ') {
            if !symbol.is_ascii_uppercase() {
                return Err(invalid_argument(
                    "shaped recipe cells must use uppercase ASCII letters or spaces",
                ));
            }
            used_symbols.insert(symbol);
        }
    }
    if used_symbols.is_empty() {
        return Err(invalid_argument(
            "shaped recipe patterns must use at least one ingredient symbol",
        ));
    }
    if definition.key.is_empty() || definition.key.len() > MAX_RECIPE_INGREDIENTS {
        return Err(invalid_argument(
            "shaped recipes must bind between 1 and 9 ingredient symbols",
        ));
    }
    let mut bound_symbols = std::collections::BTreeSet::new();
    for key in &definition.key {
        if !key.symbol.is_ascii_uppercase() || !bound_symbols.insert(key.symbol) {
            return Err(invalid_argument(
                "shaped recipe keys must use unique uppercase ASCII letters",
            ));
        }
        encode_recipe_ingredient(&key.ingredient)?;
    }
    if used_symbols != bound_symbols {
        return Err(invalid_argument(
            "shaped recipe keys must match the pattern symbols exactly",
        ));
    }
    Ok(())
}

/// Stages a one-item vanilla smelting, blasting, smoking, or campfire recipe.
///
/// The input is one item ID or item tag, experience is finite and between 0 and 1000,
/// and cooking time is between 1 and 36,000 ticks. Unqualified item paths refer
/// to this package and module.
pub fn register_cooking_recipe(path: &str, definition: CookingRecipeDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 || path == "." || path == ".." {
        return Err(invalid_argument(
            "recipe path must be at most 128 bytes and cannot be . or ..",
        ));
    }
    if !(1..=64).contains(&definition.output_count) {
        return Err(invalid_argument(
            "recipe output count must be between 1 and 64",
        ));
    }
    if !definition.experience.is_finite()
        || !(0.0..=MAX_RECIPE_EXPERIENCE).contains(&definition.experience)
    {
        return Err(invalid_argument(
            "recipe experience must be finite and between 0 and 1000",
        ));
    }
    if !(1..=MAX_RECIPE_COOKING_TIME_TICKS).contains(&definition.cooking_time) {
        return Err(invalid_argument(
            "recipe cooking time must be between 1 and 36000 ticks",
        ));
    }
    let ingredient = encode_recipe_ingredient(&definition.ingredient)?;
    validate_recipe_item_id(&definition.output_item)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded UTF-8 IDs before returning.
        let status = unsafe {
            imported_register_cooking_recipe(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.output_item.as_ptr() as usize as u32 as i32,
                definition.output_item.len() as u32 as i32,
                definition.output_count as i32,
                ingredient.as_ptr() as usize as u32 as i32,
                ingredient.len() as u32 as i32,
                definition.kind as i32,
                definition.experience.to_bits() as i32,
                definition.cooking_time as i32,
            )
        };
        return status_result(status, "host rejected the cooking recipe declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ingredient;
        Err(unsupported_target())
    }
}

fn validate_recipe_item_id(item_id: &str) -> SjmlResult<()> {
    if item_id.is_empty()
        || item_id.len() > MAX_RECIPE_ITEM_ID_BYTES
        || !item_id.is_ascii()
        || item_id.bytes().any(|byte| matches!(byte, b'\n' | b'\r'))
    {
        return Err(invalid_argument(
            "recipe item ID is empty or exceeds its supported bounds",
        ));
    }
    let (namespace, path) = match item_id.split_once(':') {
        Some((namespace, path)) if !path.contains(':') => (Some(namespace), path),
        Some(_) => {
            return Err(invalid_argument(
                "recipe item ID must contain at most one namespace separator",
            ))
        }
        None => (None, item_id),
    };
    let valid_namespace = namespace.is_none_or(|value| {
        !value.is_empty()
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'-' | b'.')
            })
    });
    let valid_path = !path.is_empty()
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.' | b'/')
        });
    if !valid_namespace || !valid_path {
        return Err(invalid_argument(
            "recipe item ID contains characters not allowed in a resource ID",
        ));
    }
    Ok(())
}

fn encode_recipe_ingredient(ingredient: &RecipeIngredient) -> SjmlResult<String> {
    match ingredient {
        RecipeIngredient::Item(item_id) => {
            validate_recipe_item_id(item_id)?;
            Ok(item_id.clone())
        }
        RecipeIngredient::Tag(tag_id) => {
            validate_recipe_item_id(tag_id)?;
            let encoded = format!("#{tag_id}");
            if encoded.len() > MAX_RECIPE_ITEM_ID_BYTES {
                return Err(invalid_argument(
                    "recipe ingredient tag ID exceeds its supported bounds",
                ));
            }
            Ok(encoded)
        }
    }
}

/// Stages a generic AI-free living entity type with explicit size and tracking bounds.
pub fn register_entity(path: &str, definition: EntityDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if !definition.width.is_finite()
        || !(0.01..=16.0).contains(&definition.width)
        || !definition.height.is_finite()
        || !(0.01..=16.0).contains(&definition.height)
        || !(1..=128).contains(&definition.tracking_range)
        || !(1..=1200).contains(&definition.update_interval)
    {
        return Err(invalid_argument(
            "entity dimensions or tracking settings are out of range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded UTF-8 path before returning.
        let status = unsafe {
            imported_register_entity(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.width.to_bits() as i32,
                definition.height.to_bits() as i32,
                definition.tracking_range as i32,
                definition.update_interval as i32,
                i32::from(definition.fire_immune),
            )
        };
        return status_result(status, "host rejected the entity declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Adds one bounded vanilla AI goal to an entity previously declared by this module.
///
/// Call this from `#[mod_register]`, after [`register_entity`]. A maximum of
/// sixteen goals may be added to one entity type.
pub fn register_entity_ai_goal(path: &str, goal: EntityAiGoal) -> SjmlResult<()> {
    validate_path(path)?;
    if !valid_entity_ai_goal(goal) {
        return Err(invalid_argument(
            "entity AI goal priority or parameter is outside its supported range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let (kind, priority, value, flags) = entity_ai_goal_wire(goal);
        // SAFETY: the host copies this bounded path and scalar goal descriptor before returning.
        let status = unsafe {
            imported_register_entity_ai_goal(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                kind,
                priority as i32,
                value.to_bits() as i32,
                flags,
            )
        };
        return status_result(status, "host rejected an entity AI goal");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = goal;
        Err(unsupported_target())
    }
}

/// Registers a colored or textured cuboid model, optionally with a host-side sine rotation.
///
/// Call from `#[mod_register]` after [`register_entity`]. Rendering uses Minecraft's buffered
/// vertex pipeline and does not require `client.render.opengl`. Models contain at most 32
/// cuboids and use entity-local world units.
pub fn register_entity_model(path: &str, definition: EntityModelDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    if definition.cuboids.is_empty() {
        return Err(invalid_argument(
            "entity model must contain at least one cuboid",
        ));
    }
    let encoded = encode_bound_model(
        &definition.cuboids,
        definition.animation,
        &definition.cuboid_animations,
    )?;
    let texture = definition.texture.as_deref();
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded slices before this import returns.
        let status = unsafe {
            if let Some(texture) = texture {
                imported_register_entity_textured_model(
                    path.as_ptr() as usize as u32 as i32,
                    path.len() as u32 as i32,
                    texture.as_ptr() as usize as u32 as i32,
                    texture.len() as u32 as i32,
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                )
            } else {
                imported_register_entity_model(
                    path.as_ptr() as usize as u32 as i32,
                    path.len() as u32 as i32,
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                )
            }
        };
        return status_result(status, "host rejected the entity model");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (encoded, texture);
        Err(unsupported_target())
    }
}

/// Registers a colored or textured cuboid model, optionally with a host-side sine rotation.
///
/// Call from `#[mod_register]` after [`register_block_entity`]. All geometry must fit in the
/// owning block's unit cube and the model is rendered with Minecraft's safe vertex pipeline.
pub fn register_block_entity_model(
    path: &str,
    definition: BlockEntityModelDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    if definition.cuboids.is_empty() {
        return Err(invalid_argument(
            "block-entity model must contain at least one cuboid",
        ));
    }
    let encoded = encode_bound_model(
        &definition.cuboids,
        definition.animation,
        &definition.cuboid_animations,
    )?;
    let texture = definition.texture.as_deref();
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded slices before this import returns.
        let status = unsafe {
            if let Some(texture) = texture {
                imported_register_block_entity_textured_model(
                    path.as_ptr() as usize as u32 as i32,
                    path.len() as u32 as i32,
                    texture.as_ptr() as usize as u32 as i32,
                    texture.len() as u32 as i32,
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                )
            } else {
                imported_register_block_entity_model(
                    path.as_ptr() as usize as u32 as i32,
                    path.len() as u32 as i32,
                    encoded.as_ptr() as usize as u32 as i32,
                    encoded.len() as u32 as i32,
                )
            }
        };
        return status_result(status, "host rejected the block-entity model");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (encoded, texture);
        Err(unsupported_target())
    }
}

fn validate_model_cuboid(min: [f32; 3], max: [f32; 3], coordinate_limit: f32) -> SjmlResult<()> {
    for axis in 0..3 {
        if !min[axis].is_finite()
            || !max[axis].is_finite()
            || min[axis].abs() > coordinate_limit
            || max[axis].abs() > coordinate_limit
            || min[axis] >= max[axis]
        {
            return Err(invalid_argument(
                "model cuboid bounds are invalid or out of range",
            ));
        }
    }
    Ok(())
}

fn validate_model_animation(
    animation: ModelAnimation,
    minimum: f32,
    maximum: f32,
) -> SjmlResult<()> {
    ModelAnimation::sine_rotation(
        animation.axis,
        animation.pivot,
        animation.amplitude_degrees,
        animation.period_ticks,
        animation.phase_ticks,
    )?;
    if animation
        .pivot
        .into_iter()
        .any(|coordinate| coordinate < minimum || coordinate > maximum)
    {
        return Err(invalid_argument(
            "model animation pivot is outside the model's local coordinate bounds",
        ));
    }
    Ok(())
}

fn validate_model_texture(texture: &str) -> SjmlResult<()> {
    let invalid =
        || invalid_argument("model texture must be a package-owned PNG resource location");
    if texture.is_empty() || texture.len() > MAX_MODEL_TEXTURE_ID_BYTES || !texture.is_ascii() {
        return Err(invalid());
    }
    let Some((namespace, path)) = texture.split_once(':') else {
        return Err(invalid());
    };
    if namespace.is_empty()
        || namespace == "."
        || namespace == ".."
        || path.is_empty()
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        || !path.starts_with("textures/")
        || !path.ends_with(".png")
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'/' | b'-')
        })
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_fluid_texture(texture: &str) -> SjmlResult<()> {
    let invalid = || {
        invalid_argument(
            "fluid texture must be a namespaced block/ resource ID of at most 256 ASCII bytes",
        )
    };
    if texture.is_empty() || texture.len() > MAX_FLUID_TEXTURE_ID_BYTES || !texture.is_ascii() {
        return Err(invalid());
    }
    let Some((namespace, path)) = texture.split_once(':') else {
        return Err(invalid());
    };
    if namespace.is_empty()
        || namespace == "."
        || namespace == ".."
        || path.is_empty()
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        || !path.starts_with("block/")
        || path.ends_with(".png")
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'/' | b'-')
        })
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_particle_texture(texture: &str) -> SjmlResult<()> {
    let invalid = || {
        invalid_argument(
            "particle texture must be a lowercase namespace:particle/path ID of at most 256 bytes",
        )
    };
    if texture.len() < 11 || texture.len() > MAX_PARTICLE_TEXTURE_ID_BYTES {
        return Err(invalid());
    }
    let Some((namespace, path)) = texture.split_once(':') else {
        return Err(invalid());
    };
    if namespace.is_empty()
        || path.is_empty()
        || path.contains(':')
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
        })
        || !path.starts_with("particle/")
        || path.ends_with(".png")
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'.' | b'-' | b'/')
        })
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(invalid());
    }
    Ok(())
}

fn validate_particle_scale(scale: f32) -> SjmlResult<()> {
    if !scale.is_finite() || !(MIN_PARTICLE_SCALE..=MAX_PARTICLE_SCALE).contains(&scale) {
        return Err(invalid_argument(
            "particle scale must be finite and between 0.05 and 4.0",
        ));
    }
    Ok(())
}

fn push_model_cuboid(cuboids: &mut Vec<ModelCuboid>, cuboid: ModelCuboid) -> SjmlResult<()> {
    if cuboids.len() >= MAX_BOUND_MODEL_CUBOIDS {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "model cuboid limit reached",
        ));
    }
    cuboids.push(cuboid);
    Ok(())
}

fn encode_bound_model(
    cuboids: &[ModelCuboid],
    animation: Option<ModelAnimation>,
    cuboid_animations: &[Option<ModelAnimation>],
) -> SjmlResult<Vec<u8>> {
    if cuboids.is_empty() || cuboids.len() > MAX_BOUND_MODEL_CUBOIDS {
        return Err(invalid_argument(
            "model cuboid count is outside its supported range",
        ));
    }
    if cuboid_animations.len() != cuboids.len() {
        return Err(invalid_argument(
            "model cuboid animations do not match the cuboid count",
        ));
    }
    let has_cuboid_animation = cuboid_animations.iter().any(Option::is_some);
    let animation_bytes = if has_cuboid_animation {
        (cuboids.len() + 1) * BOUND_MODEL_ANIMATION_BYTES
    } else {
        animation.map_or(0, |_| BOUND_MODEL_ANIMATION_BYTES)
    };
    let mut encoded = Vec::with_capacity(
        BOUND_MODEL_HEADER_BYTES + cuboids.len() * BOUND_MODEL_CUBOID_BYTES + animation_bytes,
    );
    let schema = if has_cuboid_animation {
        BOUND_MODEL_CUBOID_ANIMATION_SCHEMA_VERSION
    } else if animation.is_some() {
        BOUND_MODEL_ANIMATION_SCHEMA_VERSION
    } else {
        BOUND_MODEL_SCHEMA_VERSION
    };
    encoded.extend_from_slice(&schema.to_le_bytes());
    encoded.extend_from_slice(&(cuboids.len() as u32).to_le_bytes());
    for cuboid in cuboids {
        for coordinate in cuboid.min.into_iter().chain(cuboid.max) {
            encoded.extend_from_slice(&coordinate.to_bits().to_le_bytes());
        }
        encoded.extend_from_slice(&cuboid.argb.to_le_bytes());
    }
    if has_cuboid_animation {
        encode_model_animation_slot(&mut encoded, animation);
        for animation in cuboid_animations {
            encode_model_animation_slot(&mut encoded, *animation);
        }
    } else if let Some(animation) = animation {
        encoded.extend_from_slice(&1_u32.to_le_bytes());
        encode_model_animation_fields(&mut encoded, animation);
    }
    Ok(encoded)
}

fn encode_model_animation_slot(encoded: &mut Vec<u8>, animation: Option<ModelAnimation>) {
    if let Some(animation) = animation {
        encoded.extend_from_slice(&1_u32.to_le_bytes());
        encode_model_animation_fields(encoded, animation);
    } else {
        encoded.extend_from_slice(&[0; BOUND_MODEL_ANIMATION_BYTES]);
    }
}

fn encode_model_animation_fields(encoded: &mut Vec<u8>, animation: ModelAnimation) {
    encoded.extend_from_slice(&(animation.axis as u32).to_le_bytes());
    for coordinate in animation.pivot {
        encoded.extend_from_slice(&coordinate.to_bits().to_le_bytes());
    }
    encoded.extend_from_slice(&animation.amplitude_degrees.to_bits().to_le_bytes());
    encoded.extend_from_slice(&animation.period_ticks.to_le_bytes());
    encoded.extend_from_slice(&animation.phase_ticks.to_le_bytes());
}

fn valid_entity_ai_goal(goal: EntityAiGoal) -> bool {
    let valid_priority = |priority: u32| priority <= 31;
    match goal {
        EntityAiGoal::Swim { priority }
        | EntityAiGoal::RandomLookAround { priority }
        | EntityAiGoal::Retaliate { priority } => valid_priority(priority),
        EntityAiGoal::RandomStroll { priority, speed }
        | EntityAiGoal::MeleeAttack {
            priority, speed, ..
        } => valid_priority(priority) && speed.is_finite() && (0.01..=8.0).contains(&speed),
        EntityAiGoal::LookAtPlayer { priority, range } => {
            valid_priority(priority) && range.is_finite() && (1.0..=64.0).contains(&range)
        }
        EntityAiGoal::TargetPlayer { priority, .. } => valid_priority(priority),
    }
}

#[cfg(target_arch = "wasm32")]
fn entity_ai_goal_wire(goal: EntityAiGoal) -> (i32, u32, f32, i32) {
    match goal {
        EntityAiGoal::Swim { priority } => (0, priority, 0.0, 0),
        EntityAiGoal::RandomStroll { priority, speed } => (1, priority, speed, 0),
        EntityAiGoal::LookAtPlayer { priority, range } => (2, priority, range, 0),
        EntityAiGoal::MeleeAttack {
            priority,
            speed,
            long_memory,
        } => (3, priority, speed, i32::from(long_memory)),
        EntityAiGoal::RandomLookAround { priority } => (4, priority, 0.0, 0),
        EntityAiGoal::TargetPlayer { priority, must_see } => {
            (5, priority, 0.0, i32::from(must_see))
        }
        EntityAiGoal::Retaliate { priority } => (6, priority, 0.0, 0),
    }
}

/// Sets a base attribute on an entity type declared by this same module.
pub fn set_entity_attribute(path: &str, attribute: EntityAttribute, value: f32) -> SjmlResult<()> {
    validate_path(path)?;
    if !valid_entity_attribute_value(attribute, value) {
        return Err(invalid_argument(
            "entity attribute value is outside its supported range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and decodes the scalar before returning.
        let status = unsafe {
            imported_set_entity_attribute(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                attribute as i32,
                value.to_bits() as i32,
            )
        };
        return status_result(status, "host rejected the entity attribute");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (attribute, value);
        Err(unsupported_target())
    }
}

fn valid_entity_attribute_value(attribute: EntityAttribute, value: f32) -> bool {
    if !value.is_finite() {
        return false;
    }
    match attribute {
        EntityAttribute::MaxHealth => (1.0..=1024.0).contains(&value),
        EntityAttribute::MovementSpeed => (0.0..=1.0).contains(&value),
        EntityAttribute::AttackDamage => (0.0..=1024.0).contains(&value),
        EntityAttribute::Armor => (0.0..=30.0).contains(&value),
        EntityAttribute::FollowRange => (0.0..=1024.0).contains(&value),
        EntityAttribute::KnockbackResistance => (0.0..=1.0).contains(&value),
    }
}

/// Stages a variable-range sound event under this module's resource namespace.
pub fn register_sound(path: &str) -> SjmlResult<()> {
    validate_path(path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded UTF-8 path before returning.
        let status = unsafe {
            imported_register_sound(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the sound event declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Stages a vanilla status effect with one bounded attribute modifier.
pub fn register_attribute_effect(
    path: &str,
    category: EffectCategory,
    color: u32,
    attribute: EntityAttribute,
    amount: f32,
    operation: AttributeModifierOperation,
) -> SjmlResult<()> {
    validate_path(path)?;
    if color > 0x00FF_FFFF || !amount.is_finite() || !(-16.0..=16.0).contains(&amount) {
        return Err(invalid_argument(
            "mob effect color or modifier amount is outside its supported range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and decodes fixed-width scalar values.
        let status = unsafe {
            imported_register_attribute_effect(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                category as i32,
                color as i32,
                attribute as i32,
                amount.to_bits() as i32,
                operation as i32,
            )
        };
        return status_result(status, "host rejected the mob effect declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (category, attribute, amount, operation);
        Err(unsupported_target())
    }
}

/// Enables a subscribed `entity.effect.tick` callback for a previously registered status effect.
///
/// The callback is considered for active instances every `interval_ticks` entity ticks, where the
/// interval must be between 1 and 1,200. The host invokes the callback synchronously on the logical
/// server thread; this declaration does not implicitly subscribe the module to the event.
pub fn register_effect_tick(path: &str, interval_ticks: u32) -> SjmlResult<()> {
    validate_path(path)?;
    if !(1..=1200).contains(&interval_ticks) {
        return Err(invalid_argument(
            "mob effect tick interval must be between 1 and 1200 ticks",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the interval before staging it.
        let status = unsafe {
            imported_register_effect_tick(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                interval_ticks as i32,
            )
        };
        return status_result(status, "host rejected the mob effect tick declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = interval_ticks;
        Err(unsupported_target())
    }
}

/// Declares a module-owned custom registry. `sync_to_clients` asks NeoForge to synchronize its entry IDs.
pub fn register_custom_registry(path: &str, sync_to_clients: bool) -> SjmlResult<()> {
    validate_path(path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path before creating the registry declaration.
        let status = unsafe {
            imported_register_custom_registry(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                i32::from(sync_to_clients),
            )
        };
        return status_result(status, "host rejected the custom registry declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = sync_to_clients;
        Err(unsupported_target())
    }
}

/// Stages a guest-defined crafting recipe serializer. Implement its callback with
/// `#[recipe(name = "<same-path>")]`; the callback name must match this path.
/// Package recipe JSON then uses the type ID
/// `<package>:<module>/<path>` and supplies an optional opaque `payload` string.
/// The module must be declared with `side = "both"` because recipe matching runs
/// on both logical client and server threads.
pub fn register_recipe_serializer(path: &str) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 {
        return Err(invalid_argument(
            "recipe serializer path must be at most 128 ASCII bytes",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies and validates this bounded UTF-8 registry path.
        let status = unsafe {
            imported_register_recipe_serializer(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the recipe serializer declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = path;
        Err(unsupported_target())
    }
}

/// Stages a bounded UTF-8 string value in a custom registry declared by this module.
pub fn register_custom_value(registry_path: &str, value_path: &str, value: &str) -> SjmlResult<()> {
    validate_path(registry_path)?;
    validate_path(value_path)?;
    if value.len() > MAX_CUSTOM_REGISTRY_VALUE_BYTES {
        return Err(invalid_argument(
            "custom registry value exceeds the 64 KiB limit",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies all bounded UTF-8 slices during this synchronous registration call.
        let status = unsafe {
            imported_register_custom_value(
                registry_path.as_ptr() as usize as u32 as i32,
                registry_path.len() as u32 as i32,
                value_path.as_ptr() as usize as u32 as i32,
                value_path.len() as u32 as i32,
                value.as_ptr() as usize as u32 as i32,
                value.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the custom registry value");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = value;
        Err(unsupported_target())
    }
}

/// Reads a value from a custom registry declared by this module after registry setup completes.
pub fn custom_registry_value(registry_path: &str, value_path: &str) -> SjmlResult<Option<String>> {
    validate_path(registry_path)?;
    validate_path(value_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        let output_pointer = crate::abi::allocate_host_input(CUSTOM_REGISTRY_VALUE_FRAME_BYTES);
        if output_pointer == 0 {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate the custom registry value output buffer",
            ));
        }
        // SAFETY: this allocation is exclusively owned by the guest until it is deallocated below.
        unsafe {
            core::ptr::write_bytes(
                output_pointer as *mut u8,
                0,
                CUSTOM_REGISTRY_VALUE_FRAME_BYTES as usize,
            );
        }
        let result = (|| -> SjmlResult<Option<String>> {
            let status = unsafe {
                imported_get_custom_registry_value(
                    registry_path.as_ptr() as usize as u32 as i32,
                    registry_path.len() as u32 as i32,
                    value_path.as_ptr() as usize as u32 as i32,
                    value_path.len() as u32 as i32,
                    output_pointer as i32,
                    CUSTOM_REGISTRY_VALUE_FRAME_BYTES as i32,
                )
            };
            if status != 0 {
                return Err(SjmlError::from_abi_code(
                    status as u32,
                    "host rejected the custom registry value lookup",
                ));
            }
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    output_pointer as *const u8,
                    CUSTOM_REGISTRY_VALUE_FRAME_BYTES as usize,
                )
            };
            decode_custom_registry_value_response(bytes)
        })();
        unsafe {
            crate::abi::deallocate_host_input(output_pointer, CUSTOM_REGISTRY_VALUE_FRAME_BYTES);
        }
        return result;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

#[derive(crate::StableDecode)]
#[sjml(version = 1)]
#[cfg(any(target_arch = "wasm32", test))]
struct CustomRegistryValueSnapshot {
    value: Option<String>,
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_custom_registry_value_response(bytes: &[u8]) -> SjmlResult<Option<String>> {
    if bytes.len() < 5 || u32::from_le_bytes(bytes[0..4].try_into().unwrap()) != 1 {
        return Err(invalid_argument(
            "custom registry value response has an unsupported schema",
        ));
    }
    let initialized_length = match bytes[4] {
        0 => 5,
        1 => {
            if bytes.len() < 9 {
                return Err(invalid_argument(
                    "custom registry value response is truncated",
                ));
            }
            let value_length = u32::from_le_bytes(bytes[5..9].try_into().unwrap()) as usize;
            if value_length > MAX_CUSTOM_REGISTRY_VALUE_BYTES {
                return Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "custom registry value response exceeds the 64 KiB limit",
                ));
            }
            9 + value_length
        }
        _ => {
            return Err(invalid_argument(
                "custom registry value response has an invalid option tag",
            ))
        }
    };
    if initialized_length > bytes.len() {
        return Err(invalid_argument(
            "custom registry value response is truncated",
        ));
    }
    let snapshot: CustomRegistryValueSnapshot = crate::from_bytes(&bytes[..initialized_length])?;
    Ok(snapshot.value)
}

/// Stages a creative tab with a previously declared module-owned icon item.
pub fn register_creative_tab(path: &str, definition: CreativeTabDefinition) -> SjmlResult<()> {
    validate_path(path)?;
    validate_title(&definition.title)?;
    validate_path(&definition.icon_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies all bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_register_creative_tab(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.title.as_ptr() as usize as u32 as i32,
                definition.title.len() as u32 as i32,
                definition.icon_path.as_ptr() as usize as u32 as i32,
                definition.icon_path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the creative tab declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Adds a previously declared module-owned item or block item to a creative tab.
pub fn add_to_creative_tab(tab_path: &str, item_path: &str) -> SjmlResult<()> {
    validate_path(tab_path)?;
    validate_path(item_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_add_to_creative_tab(
                tab_path.as_ptr() as usize as u32 as i32,
                tab_path.len() as u32 as i32,
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the creative tab item");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Registers a persistent, network-synchronized UTF-8 string data component type.
pub fn register_string_component(
    path: &str,
    definition: StringComponentDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    if !(1..=1024).contains(&definition.max_characters) {
        return Err(invalid_argument(
            "string component character limit must be between 1 and 1024",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded UTF-8 path before returning.
        let status = unsafe {
            imported_register_string_component(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.max_characters as i32,
            )
        };
        return status_result(status, "host rejected the string component declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Sets a persistent string component default on a previously declared item.
pub fn set_item_string_component(
    item_path: &str,
    component_path: &str,
    value: &str,
) -> SjmlResult<()> {
    validate_path(item_path)?;
    validate_path(component_path)?;
    if value.len() > MAX_COMPONENT_VALUE_BYTES {
        return Err(invalid_argument(
            "string component value exceeds 4096 UTF-8 bytes",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies all bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_set_item_string_component(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as u32 as i32,
                value.as_ptr() as usize as u32 as i32,
                value.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the item component default");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

fn validate_path(path: &str) -> SjmlResult<()> {
    if path.is_empty()
        || path.len() > MAX_CONTENT_PATH_BYTES
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(invalid_argument(
            "content path must contain 1..=256 lowercase ASCII letters, digits, '.', '_' or '-'",
        ));
    }
    Ok(())
}

fn validate_title(title: &str) -> SjmlResult<()> {
    if title.trim().is_empty()
        || title.len() > MAX_CONTENT_PATH_BYTES
        || title.chars().any(char::is_control)
    {
        return Err(invalid_argument(
            "creative tab title must be 1-256 UTF-8 bytes without control characters",
        ));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn status_result(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

fn invalid_argument(message: &str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "SJML registry imports require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn definitions_have_safe_defaults() {
        assert_eq!(ItemDefinition::default().max_stack_size, 64);
        assert!(!ItemDefinition::default().fire_resistant);
        assert_eq!(BlockDefinition::default().strength, 1.0);
        assert!(BlockDefinition::default().has_item_form);
        assert_eq!(EntityDefinition::default().width, 0.6);
        assert_eq!(EntityDefinition::default().height, 1.8);
        assert_eq!(FluidDefinition::default().tick_rate, 5);
        assert_eq!(FluidDefinition::default().slope_find_distance, 4);
    }

    #[test]
    fn bound_model_builders_validate_geometry_and_enforce_cuboid_limits() {
        let mut entity = EntityModelDefinition::new();
        entity
            .add_cuboid([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5], 0xFFFF_0080)
            .unwrap();
        assert_eq!(entity.cuboid_count(), 1);
        assert!(entity
            .add_cuboid([0.0; 3], [f32::INFINITY, 1.0, 1.0], 0)
            .is_err());
        assert!(entity
            .add_cuboid([0.5, 0.0, 0.0], [-0.5, 1.0, 1.0], 0)
            .is_err());
        assert!(entity
            .add_cuboid([-4.1, 0.0, 0.0], [0.0, 1.0, 1.0], 0)
            .is_err());

        let mut block_entity = BlockEntityModelDefinition::new();
        block_entity
            .add_cuboid([0.0; 3], [1.0; 3], 0xFF20_C0FF)
            .unwrap();
        assert!(block_entity
            .add_cuboid([-0.01, 0.0, 0.0], [0.5; 3], 0)
            .is_err());
        assert!(block_entity
            .add_cuboid([0.0; 3], [1.01, 1.0, 1.0], 0)
            .is_err());

        let mut full = EntityModelDefinition::new();
        for _ in 0..MAX_BOUND_MODEL_CUBOIDS {
            full.add_cuboid([0.0; 3], [1.0; 3], 0).unwrap();
        }
        assert_eq!(
            full.add_cuboid([0.0; 3], [1.0; 3], 0).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn model_texture_paths_are_bounded_package_resource_locations() {
        let mut entity = EntityModelDefinition::new();
        entity
            .set_texture("example:textures/entity/clockwork_mite.png")
            .unwrap();
        assert_eq!(
            entity.texture.as_deref(),
            Some("example:textures/entity/clockwork_mite.png")
        );
        let mut block = BlockEntityModelDefinition::new();
        block
            .set_texture("example:textures/block/clockwork_core.png")
            .unwrap();
        assert_eq!(
            block.texture.as_deref(),
            Some("example:textures/block/clockwork_core.png")
        );

        for invalid in [
            "textures/entity/missing_namespace.png",
            "example:entity/missing_textures_prefix.png",
            "example:textures/../outside.png",
            "example:textures/entity/not_png.jpg",
            "Example:textures/entity/uppercase.png",
            "example:textures//empty_segment.png",
        ] {
            assert!(
                EntityModelDefinition::new().set_texture(invalid).is_err(),
                "{invalid}"
            );
        }
        let too_long = format!("example:textures/entity/{}.png", "a".repeat(256));
        assert!(EntityModelDefinition::new().set_texture(too_long).is_err());
    }

    #[test]
    fn model_wire_encoding_is_versioned_little_endian_and_bounded() {
        let cuboid = ModelCuboid::new([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5], 0xFF12_3456).unwrap();
        let encoded = encode_bound_model(&[cuboid], None, &[None]).unwrap();
        assert_eq!(
            encoded.len(),
            BOUND_MODEL_HEADER_BYTES + BOUND_MODEL_CUBOID_BYTES
        );
        assert_eq!(&encoded[..8], &[1, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(&encoded[8..12], &(-0.5_f32).to_bits().to_le_bytes());
        assert_eq!(&encoded[32..36], &0xFF12_3456_u32.to_le_bytes());
    }

    #[test]
    fn animated_model_wire_encoding_is_bounded_and_versioned() {
        let cuboid = ModelCuboid::new([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5], 0xFF12_3456).unwrap();
        let animation =
            ModelAnimation::sine_rotation(ModelAnimationAxis::Y, [0.0, 0.5, 0.0], 30.0, 80, 20)
                .unwrap();
        let encoded = encode_bound_model(&[cuboid], Some(animation), &[None]).unwrap();
        assert_eq!(
            encoded.len(),
            BOUND_MODEL_HEADER_BYTES + BOUND_MODEL_CUBOID_BYTES + BOUND_MODEL_ANIMATION_BYTES
        );
        assert_eq!(&encoded[..8], &[2, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(&encoded[36..40], &[1, 0, 0, 0]);
        assert_eq!(&encoded[40..44], &[1, 0, 0, 0]);
        assert_eq!(&encoded[44..48], &0.0_f32.to_bits().to_le_bytes());
        assert_eq!(&encoded[48..52], &0.5_f32.to_bits().to_le_bytes());
        assert_eq!(&encoded[56..60], &30.0_f32.to_bits().to_le_bytes());
        assert_eq!(&encoded[60..64], &80_u32.to_le_bytes());
        assert_eq!(&encoded[64..68], &20_u32.to_le_bytes());
    }

    #[test]
    fn cuboid_animations_use_schema_three_without_changing_legacy_payloads() {
        let first = ModelCuboid::new([-0.5, 0.0, -0.5], [0.5, 0.5, 0.5], 0xFF12_3456).unwrap();
        let second = ModelCuboid::new([-0.25, 0.5, -0.25], [0.25, 1.0, 0.25], 0xFF65_4321).unwrap();
        let global =
            ModelAnimation::sine_rotation(ModelAnimationAxis::Y, [0.0, 0.0, 0.0], 15.0, 80, 0)
                .unwrap();
        let child =
            ModelAnimation::sine_rotation(ModelAnimationAxis::X, [0.0, 0.5, 0.0], 45.0, 40, 10)
                .unwrap();
        let encoded =
            encode_bound_model(&[first, second], Some(global), &[None, Some(child)]).unwrap();
        assert_eq!(encoded.len(), 8 + 2 * 28 + 3 * 32);
        assert_eq!(&encoded[..8], &[3, 0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(
            &encoded[64..68],
            &[1, 0, 0, 0],
            "global parent transform slot"
        );
        assert_eq!(&encoded[96..128], &[0; 32], "first cuboid is unanimated");
        assert_eq!(
            &encoded[128..132],
            &[1, 0, 0, 0],
            "second cuboid has a child transform"
        );
        assert_eq!(&encoded[132..136], &[0, 0, 0, 0], "child rotates around X");
        assert_eq!(&encoded[136..140], &0.0_f32.to_bits().to_le_bytes());
        assert_eq!(&encoded[140..144], &0.5_f32.to_bits().to_le_bytes());
        assert_eq!(&encoded[148..152], &45.0_f32.to_bits().to_le_bytes());
        assert_eq!(&encoded[152..156], &40_u32.to_le_bytes());
        assert_eq!(&encoded[156..160], &10_u32.to_le_bytes());

        let mut model = EntityModelDefinition::new();
        model
            .add_cuboid(first.min, first.max, first.argb)
            .unwrap()
            .add_cuboid(second.min, second.max, second.argb)
            .unwrap()
            .set_cuboid_animation(1, child)
            .unwrap();
        assert_eq!(model.cuboid_animations, [None, Some(child)]);
        assert!(model.set_cuboid_animation(2, child).is_err());
        assert!(model.clear_cuboid_animation(2).is_err());
        assert!(model
            .set_cuboid_animation(
                0,
                ModelAnimation {
                    pivot: [4.01, 0.0, 0.0],
                    ..child
                }
            )
            .is_err());
        model.clear_cuboid_animation(1).unwrap();
        assert_eq!(model.cuboid_animations, [None, None]);

        let mut block_model = BlockEntityModelDefinition::new();
        block_model
            .add_cuboid([0.0, 0.0, 0.0], [0.5, 0.5, 0.5], 0)
            .unwrap();
        assert!(block_model
            .set_cuboid_animation(
                0,
                ModelAnimation {
                    pivot: [-0.01, 0.5, 0.5],
                    ..child
                }
            )
            .is_err());
    }

    #[test]
    fn model_animation_validation_enforces_time_rotation_and_pivot_bounds() {
        let valid =
            ModelAnimation::sine_rotation(ModelAnimationAxis::X, [0.0, 0.5, 0.0], 12.0, 20, 19)
                .unwrap();
        assert!(EntityModelDefinition::new().set_animation(valid).is_ok());
        assert!(BlockEntityModelDefinition::new()
            .set_animation(valid)
            .is_ok());
        assert!(BlockEntityModelDefinition::new()
            .set_animation(ModelAnimation {
                pivot: [-0.01, 0.5, 0.5],
                ..valid
            })
            .is_err());
        assert!(EntityModelDefinition::new()
            .set_animation(ModelAnimation {
                pivot: [4.01, 0.0, 0.0],
                ..valid
            })
            .is_err());
        for invalid in [
            ModelAnimation {
                amplitude_degrees: f32::NAN,
                ..valid
            },
            ModelAnimation {
                amplitude_degrees: 181.0,
                ..valid
            },
            ModelAnimation {
                period_ticks: 0,
                ..valid
            },
            ModelAnimation {
                period_ticks: MAX_MODEL_ANIMATION_PERIOD_TICKS + 1,
                ..valid
            },
            ModelAnimation {
                period_ticks: 10,
                phase_ticks: 10,
                ..valid
            },
        ] {
            assert!(EntityModelDefinition::new().set_animation(invalid).is_err());
        }
    }

    #[test]
    fn native_registration_is_reported_as_unsupported() {
        assert_eq!(
            register_item("ruby", ItemDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_block("ruby_block", BlockDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_block_entity("ruby_machine", "ruby_block")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_particle("ruby_spark").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        let particle = ParticleDefinition::new("ruby_pack:particle/ruby_spark").unwrap();
        assert_eq!(particle.scale(), 0.25);
        assert_eq!(
            register_particle_with_definition("ruby_spark", particle)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_fluid("ruby_fluid", FluidDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_entity("ruby_mite", EntityDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        let mut model = EntityModelDefinition::new();
        model
            .add_cuboid([-0.5, 0.0, -0.5], [0.5, 1.0, 0.5], 0xFFFFFFFF)
            .unwrap();
        assert_eq!(
            register_entity_model("ruby_mite", model)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        let mut model = BlockEntityModelDefinition::new();
        model.add_cuboid([0.0; 3], [1.0; 3], 0xFFFFFFFF).unwrap();
        assert_eq!(
            register_block_entity_model("ruby_machine", model)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_entity_attribute("ruby_mite", EntityAttribute::MaxHealth, 40.0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_sound("ruby_mite_idle").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_attribute_effect(
                "ruby_mite_power",
                EffectCategory::Beneficial,
                0xD02090,
                EntityAttribute::AttackDamage,
                2.0,
                AttributeModifierOperation::AddValue,
            )
            .unwrap_err()
            .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_effect_tick("ruby_mite_power", 20)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_recipe_serializer("ruby_smelting")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_custom_registry("ruby_types", true)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_custom_value("ruby_types", "ore", "ruby")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            custom_registry_value("ruby_types", "ore")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        let tab = CreativeTabDefinition::new("Ruby", "ruby").unwrap();
        assert_eq!(
            register_creative_tab("ruby_tab", tab).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            add_to_creative_tab("ruby_tab", "ruby").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            register_string_component("ruby_origin", StringComponentDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_string_component("ruby", "ruby_origin", "mined")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_string_component("ruby", "ruby_origin", &"x".repeat(4097))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn particle_definitions_validate_atlas_ids_and_bounded_scale() {
        let particle = ParticleDefinition::new("ruby_pack:particle/effects/spark")
            .unwrap()
            .with_scale(1.5)
            .unwrap();
        assert_eq!(particle.texture(), "ruby_pack:particle/effects/spark");
        assert_eq!(particle.scale(), 1.5);

        for texture in [
            "ruby_pack:textures/particle/spark.png",
            "ruby_pack:particle/../spark",
            "ruby_pack:particle//spark",
            "ruby_pack:particle/Spark",
            "ruby_pack:particle/spark.png",
            "ruby_pack:particles/spark",
        ] {
            assert_eq!(
                ParticleDefinition::new(texture).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "texture should be rejected: {texture}"
            );
        }

        let particle = ParticleDefinition::new("ruby_pack:particle/spark").unwrap();
        for scale in [f32::NAN, f32::INFINITY, 0.0, 0.049, 4.01] {
            assert_eq!(
                particle.clone().with_scale(scale).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn entity_definitions_are_validated_before_import() {
        assert_eq!(
            register_entity(
                "ruby_mite",
                EntityDefinition {
                    width: f32::NAN,
                    ..EntityDefinition::default()
                },
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_entity(
                "ruby_mite",
                EntityDefinition {
                    tracking_range: 129,
                    ..EntityDefinition::default()
                },
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_block_entity("machine", "../block")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_entity_attribute("ruby_mite", EntityAttribute::MaxHealth, f32::INFINITY)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_entity_attribute("ruby_mite", EntityAttribute::Armor, 31.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_attribute_effect(
                "ruby_mite_power",
                EffectCategory::Beneficial,
                0x01000000,
                EntityAttribute::AttackDamage,
                2.0,
                AttributeModifierOperation::AddValue,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_effect_tick("ruby_mite_power", 0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_recipe_serializer("../serializer")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_recipe_serializer(&"s".repeat(129))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_effect_tick("ruby_mite_power", 1201)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_custom_registry("../ruby_types", true)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_custom_value("ruby_types", "ore", &"x".repeat(65_537))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            custom_registry_value("ruby_types", "../ore")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_attribute_effect(
                "ruby_mite_power",
                EffectCategory::Beneficial,
                0xD02090,
                EntityAttribute::AttackDamage,
                f32::INFINITY,
                AttributeModifierOperation::AddValue,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            CreativeTabDefinition::new("\n", "ruby").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            CreativeTabDefinition::new("Ruby", "../ruby")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_string_component(
                "ruby_origin",
                StringComponentDefinition {
                    max_characters: 1025
                },
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn entity_ai_goals_validate_before_host_access() {
        for goal in [
            EntityAiGoal::Swim { priority: 32 },
            EntityAiGoal::RandomStroll {
                priority: 1,
                speed: f32::NAN,
            },
            EntityAiGoal::LookAtPlayer {
                priority: 1,
                range: 64.1,
            },
            EntityAiGoal::MeleeAttack {
                priority: 1,
                speed: 0.0,
                long_memory: false,
            },
        ] {
            assert_eq!(
                register_entity_ai_goal("ruby_mite", goal)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            register_entity_ai_goal("ruby_mite", EntityAiGoal::RandomLookAround { priority: 0 })
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn fluid_settings_are_validated_before_host_access() {
        for definition in [
            FluidDefinition {
                tick_rate: 0,
                ..FluidDefinition::default()
            },
            FluidDefinition {
                level_decrease_per_block: 9,
                ..FluidDefinition::default()
            },
            FluidDefinition {
                slope_find_distance: 17,
                ..FluidDefinition::default()
            },
            FluidDefinition {
                explosion_resistance: f32::NAN,
                ..FluidDefinition::default()
            },
            FluidDefinition {
                explosion_resistance: 1_000_001.0,
                ..FluidDefinition::default()
            },
        ] {
            assert_eq!(
                register_fluid("ruby_fluid", definition).unwrap_err().code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_fluid("../fluid", FluidDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
        );
        assert_eq!(
            register_fluid(
                &"f".repeat(MAX_FLUID_PATH_BYTES + 1),
                FluidDefinition::default()
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn fluid_visuals_require_safe_namespaced_block_texture_ids() {
        let valid =
            FluidVisualDefinition::new("ruby_pack:block/ruby_still", "ruby_pack:block/ruby_flow")
                .unwrap();
        assert_eq!(valid.still_texture(), "ruby_pack:block/ruby_still");
        for texture in [
            "block/ruby_still",
            "ruby_pack:textures/block/ruby_still.png",
            "ruby_pack:block/../outside",
            "ruby_pack:Block/ruby_still",
            "OtherPack:block/ruby_still",
        ] {
            assert_eq!(
                FluidVisualDefinition::new(texture, "ruby_pack:block/ruby_flow")
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
    }

    #[test]
    fn shapeless_recipe_bounds_and_item_ids_are_validated_before_host_access() {
        let valid = ShapelessRecipeDefinition {
            output_item: "minecraft:diamond".into(),
            output_count: 1,
            ingredients: vec![
                RecipeIngredient::tag("forge:coal"),
                RecipeIngredient::item("ruby"),
            ],
        };
        assert_eq!(
            register_shapeless_recipe("ruby_recipe", valid.clone())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );

        for definition in [
            ShapelessRecipeDefinition {
                output_count: 0,
                ..valid.clone()
            },
            ShapelessRecipeDefinition {
                output_count: 65,
                ..valid.clone()
            },
            ShapelessRecipeDefinition {
                ingredients: vec![],
                ..valid.clone()
            },
            ShapelessRecipeDefinition {
                ingredients: vec!["ruby".into(); MAX_RECIPE_INGREDIENTS + 1],
                ..valid.clone()
            },
            ShapelessRecipeDefinition {
                output_item: "minecraft:bad id".into(),
                ..valid.clone()
            },
            ShapelessRecipeDefinition {
                ingredients: vec![RecipeIngredient::tag("forge:")],
                ..valid.clone()
            },
        ] {
            assert_eq!(
                register_shapeless_recipe("ruby_recipe", definition)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_shapeless_recipe("..", valid).unwrap_err().code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn shaped_recipe_bounds_pattern_and_keys_are_validated_before_host_access() {
        let valid = ShapedRecipeDefinition {
            output_item: "ruby_block".into(),
            output_count: 1,
            pattern: vec!["RR".into(), "RR".into()],
            key: vec![ShapedRecipeKey {
                symbol: 'R',
                ingredient: RecipeIngredient::tag("c:storage_blocks/ruby"),
            }],
        };
        assert_eq!(
            register_shaped_recipe("ruby_block", valid.clone())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );

        for definition in [
            ShapedRecipeDefinition {
                output_count: 0,
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                pattern: vec![],
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                pattern: vec!["RRRR".into()],
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                pattern: vec!["RR".into(), "R".into()],
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                pattern: vec!["Rr".into(), "RR".into()],
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                key: vec![
                    ShapedRecipeKey {
                        symbol: 'R',
                        ingredient: "ruby".into(),
                    },
                    ShapedRecipeKey {
                        symbol: 'R',
                        ingredient: "minecraft:coal".into(),
                    },
                ],
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                key: vec![ShapedRecipeKey {
                    symbol: 'A',
                    ingredient: "ruby".into(),
                }],
                ..valid.clone()
            },
            ShapedRecipeDefinition {
                key: vec![ShapedRecipeKey {
                    symbol: 'R',
                    ingredient: RecipeIngredient::tag("bad tag id"),
                }],
                ..valid.clone()
            },
        ] {
            assert_eq!(
                register_shaped_recipe("ruby_recipe", definition)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_shaped_recipe("..", valid).unwrap_err().code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn cooking_recipe_bounds_are_validated_before_host_access() {
        let valid = CookingRecipeDefinition {
            kind: CookingRecipeKind::Smelting,
            ingredient: RecipeIngredient::tag("c:iron_ores"),
            output_item: "minecraft:iron_ingot".into(),
            output_count: 1,
            experience: 0.7,
            cooking_time: 200,
        };
        assert_eq!(
            register_cooking_recipe("iron_ingot", valid.clone())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );

        for definition in [
            CookingRecipeDefinition {
                output_count: 0,
                ..valid.clone()
            },
            CookingRecipeDefinition {
                experience: -0.1,
                ..valid.clone()
            },
            CookingRecipeDefinition {
                experience: f32::NAN,
                ..valid.clone()
            },
            CookingRecipeDefinition {
                experience: 1000.1,
                ..valid.clone()
            },
            CookingRecipeDefinition {
                cooking_time: 0,
                ..valid.clone()
            },
            CookingRecipeDefinition {
                cooking_time: MAX_RECIPE_COOKING_TIME_TICKS + 1,
                ..valid.clone()
            },
            CookingRecipeDefinition {
                ingredient: RecipeIngredient::tag("bad tag id"),
                ..valid.clone()
            },
        ] {
            assert_eq!(
                register_cooking_recipe("iron_ingot", definition)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_cooking_recipe("..", valid).unwrap_err().code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn custom_registry_value_response_decodes_bounded_optional_utf8() {
        assert_eq!(
            decode_custom_registry_value_response(&[1, 0, 0, 0, 0]).unwrap(),
            None
        );

        let value = "ruby: 多面体".as_bytes();
        let mut response = Vec::with_capacity(9 + value.len());
        response.extend_from_slice(&1_u32.to_le_bytes());
        response.push(1);
        response.extend_from_slice(&(value.len() as u32).to_le_bytes());
        response.extend_from_slice(value);
        assert_eq!(
            decode_custom_registry_value_response(&response).unwrap(),
            Some("ruby: 多面体".into())
        );

        assert_eq!(
            decode_custom_registry_value_response(&[1, 0, 0, 0, 2])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            decode_custom_registry_value_response(&[1, 0, 0, 0, 1, 3, 0, 0, 0])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let mut oversized = vec![1, 0, 0, 0, 1];
        oversized.extend_from_slice(&((MAX_CUSTOM_REGISTRY_VALUE_BYTES as u32) + 1).to_le_bytes());
        assert_eq!(
            decode_custom_registry_value_response(&oversized)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn item_stack_sizes_and_content_paths_are_validated_before_import() {
        assert_eq!(
            register_item(
                "ruby",
                ItemDefinition {
                    max_stack_size: 100,
                    fire_resistant: false,
                },
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_item("../ruby", ItemDefinition::default())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
