//! Static content declarations captured before Minecraft registry freeze.
//!
//! These functions are valid only from `#[mod_register]`. They stage bounded
//! item, block, block-entity, generic entity and its attributes, creative-tab,
//! fluid, shaped/shapeless/cooking/stonecutting/smithing-transform recipe, bounded villager trade, particle,
//! sound, bounded attribute effect and its optional tick callback, and bounded string/integer/float data component declarations;
//! they do not mutate world state or registries directly.

use crate::{ErrorCode, SjmlError, SjmlResult};

const MAX_CONTENT_PATH_BYTES: usize = 256;
const MAX_BLOCK_STATE_PROPERTY_NAME_BYTES: usize = 32;
const MAX_BLOCK_STATE_VALUE_BYTES: usize = 5;
const BLOCK_STATE_SHAPE_SCHEMA_VERSION: u32 = 1;
const MAX_BLOCK_STATE_SHAPE_PAYLOAD_BYTES: usize = 512;
const BLOCK_SHAPE_BOXES_SCHEMA_VERSION: u32 = 1;
const BLOCK_SHAPE_BOXES_HEADER_BYTES: usize = 2 * core::mem::size_of::<u32>();
const BLOCK_SHAPE_BOX_BYTES: usize = 6 * core::mem::size_of::<f32>();
const MAX_BLOCK_SHAPE_BOXES_PAYLOAD_BYTES: usize =
    BLOCK_SHAPE_BOXES_HEADER_BYTES + MAX_BLOCK_SHAPE_BOXES * BLOCK_SHAPE_BOX_BYTES;
const BLOCK_STATE_LIGHT_LEVEL_SCHEMA_VERSION: u32 = 1;
const MAX_BLOCK_STATE_LIGHT_LEVEL_PAYLOAD_BYTES: usize = 512;
const BLOCK_STATE_BEHAVIOR_SCHEMA_VERSION: u32 = 2;
const MAX_BLOCK_STATE_BEHAVIOR_PAYLOAD_BYTES: usize = 512;
const BLOCK_STATE_BEHAVIOR_SOUND: u32 = 1;
const BLOCK_STATE_BEHAVIOR_REDSTONE_CONDUCTOR: u32 = 1 << 1;
const BLOCK_STATE_BEHAVIOR_SUFFOCATING: u32 = 1 << 2;
const BLOCK_STATE_BEHAVIOR_VIEW_BLOCKING: u32 = 1 << 3;
const BLOCK_STATE_BEHAVIOR_EMISSIVE_RENDERING: u32 = 1 << 4;
const BLOCK_STATE_BEHAVIOR_POST_PROCESS: u32 = 1 << 5;
const BLOCK_STATE_BEHAVIOR_FRICTION: u32 = 1 << 6;
/// Maximum number of vanilla status effects on one registered food item.
pub const MAX_ITEM_FOOD_EFFECTS: usize = 8;
/// Maximum UTF-8 size of one fully qualified vanilla or modded effect ID.
pub const MAX_ITEM_FOOD_EFFECT_ID_BYTES: usize = 256;
/// Maximum duration for one food-applied status effect, in ticks.
pub const MAX_ITEM_FOOD_EFFECT_DURATION_TICKS: u32 = 72_000;
/// Maximum static vanilla damage capacity for one registered item.
pub const MAX_ITEM_MAX_DAMAGE: u32 = 1_000_000;
/// Maximum static vanilla stack size for one registered item.
pub const MAX_ITEM_MAX_STACK_SIZE: u32 = 99;
/// Maximum static use duration for one registered item, in ticks.
pub const MAX_ITEM_USE_DURATION_TICKS: u32 = 72_000;
/// Maximum UTF-8 size of one fully qualified static item crafting remainder ID.
pub const MAX_ITEM_CRAFTING_REMAINDER_ID_BYTES: usize = 256;
/// Maximum number of vanilla attribute modifiers on one registered item.
pub const MAX_ITEM_ATTRIBUTE_MODIFIERS: usize = 16;
/// Maximum absolute static modifier amount on one registered item.
pub const MAX_ITEM_ATTRIBUTE_MODIFIER_AMOUNT: f32 = 1024.0;
/// Largest accepted RGB value for one vanilla spawn egg tint layer.
pub const MAX_ENTITY_SPAWN_EGG_RGB_COLOR: u32 = 0x00ff_ffff;
/// Maximum declared state properties on one registered block.
pub const MAX_BLOCK_STATE_PROPERTIES: usize = 7;
/// Maximum possible state combinations on one registered block.
pub const MAX_BLOCK_STATE_VARIANTS: usize = 128;
/// Maximum axis-aligned boxes in one registered block's bounded multi-box shape.
pub const MAX_BLOCK_SHAPE_BOXES: usize = 8;
/// Maximum values in a single bounded integer block-state property.
pub const MAX_BLOCK_INTEGER_PROPERTY_VALUES: usize = 16;
/// Minimum value in a bounded integer block-state property.
pub const MIN_BLOCK_INTEGER_PROPERTY_VALUE: i32 = -128;
/// Maximum value in a bounded integer block-state property.
pub const MAX_BLOCK_INTEGER_PROPERTY_VALUE: i32 = 127;
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
const MAX_VILLAGER_TRADE_ID_BYTES: usize = 256;
/// Maximum UTF-8 size of one fully qualified target entity type ID in a registered AI goal.
pub const MAX_ENTITY_AI_TARGET_ID_BYTES: usize = 256;
const MAX_VILLAGER_TRADE_USES: u32 = 1_000;
const MAX_VILLAGER_TRADE_XP: u32 = 1_000;
const MAX_VILLAGER_TRADE_PRICE_MULTIPLIER: f32 = 100.0;
const MAX_VILLAGER_TRADE_INITIAL_DEMAND: i32 = 1_000;
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
const BOUND_MODEL_AXIS_ANIMATION_SCHEMA_VERSION: u32 = 4;
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

/// Supported vanilla display rarity for a registered item.
///
/// The stable host-import values are `Common = 0`, `Uncommon = 1`, `Rare = 2`, and `Epic = 3`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemRarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
}

/// Vanilla item-use animation selected when a registered item is used.
///
/// The numeric values are stable host-import codes. `Custom` is excluded because it requires
/// a mod-owned animation implementation rather than one of Minecraft's static vanilla animations.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemUseAnimation {
    None = 0,
    Eat = 1,
    Drink = 2,
    Block = 3,
    Bow = 4,
    Spear = 5,
    Crossbow = 6,
    Spyglass = 7,
    TootHorn = 8,
    Brush = 9,
}

/// Static vanilla use animation and hold duration for a registered item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemUseBehavior {
    animation: ItemUseAnimation,
    duration_ticks: u32,
}

impl ItemUseBehavior {
    /// Creates a static use behavior with a vanilla animation and duration in `1..=72_000` ticks.
    pub fn new(animation: ItemUseAnimation, duration_ticks: u32) -> SjmlResult<Self> {
        if !(1..=MAX_ITEM_USE_DURATION_TICKS).contains(&duration_ticks) {
            return Err(invalid_argument(
                "item use duration must be between 1 and 72000 ticks",
            ));
        }
        Ok(Self {
            animation,
            duration_ticks,
        })
    }

    pub const fn animation(self) -> ItemUseAnimation {
        self.animation
    }

    pub const fn duration_ticks(self) -> u32 {
        self.duration_ticks
    }
}

/// Equipment-slot group for a static registered-item attribute modifier.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemAttributeSlot {
    Any = 0,
    Mainhand = 1,
    Offhand = 2,
    Hand = 3,
    Feet = 4,
    Legs = 5,
    Chest = 6,
    Head = 7,
    Armor = 8,
    Body = 9,
}

/// One static vanilla or modded attribute modifier for a registered item.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemAttributeModifier {
    attribute_id: String,
    modifier_path: String,
    amount: f32,
    operation: AttributeModifierOperation,
    slot: ItemAttributeSlot,
}

impl ItemAttributeModifier {
    /// Creates a modifier for a fully qualified attribute ID and a module-local modifier path.
    ///
    /// Amounts must be finite and within `-1024..=1024`. The modifier's final resource ID is
    /// namespaced to this package and module during static registration.
    pub fn new(
        attribute_id: impl Into<String>,
        modifier_path: impl Into<String>,
        amount: f32,
        operation: AttributeModifierOperation,
        slot: ItemAttributeSlot,
    ) -> SjmlResult<Self> {
        let attribute_id = attribute_id.into();
        let modifier_path = modifier_path.into();
        validate_villager_registry_id(&attribute_id, "item attribute ID")?;
        validate_path(&modifier_path)?;
        if !amount.is_finite()
            || !(-MAX_ITEM_ATTRIBUTE_MODIFIER_AMOUNT..=MAX_ITEM_ATTRIBUTE_MODIFIER_AMOUNT)
                .contains(&amount)
        {
            return Err(invalid_argument(
                "item attribute modifier amount must be finite and between -1024 and 1024",
            ));
        }
        Ok(Self {
            attribute_id,
            modifier_path,
            amount,
            operation,
            slot,
        })
    }

    pub fn attribute_id(&self) -> &str {
        &self.attribute_id
    }

    pub fn modifier_path(&self) -> &str {
        &self.modifier_path
    }

    pub const fn amount(&self) -> f32 {
        self.amount
    }

    pub const fn operation(&self) -> AttributeModifierOperation {
        self.operation
    }

    pub const fn slot(&self) -> ItemAttributeSlot {
        self.slot
    }
}

/// Bounded vanilla food behavior for a registered item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemFood {
    nutrition: u32,
    saturation_modifier: f32,
    fast: bool,
    always_eat: bool,
}

/// One bounded vanilla status effect applied when a registered food is consumed.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemFoodEffect {
    effect_id: String,
    duration_ticks: u32,
    amplifier: u32,
    probability: f32,
}

impl ItemFoodEffect {
    /// Creates an effect using a canonical, fully qualified registry ID such as `minecraft:poison`.
    pub fn new(
        effect_id: impl Into<String>,
        duration_ticks: u32,
        amplifier: u32,
        probability: f32,
    ) -> SjmlResult<Self> {
        let effect_id = effect_id.into();
        validate_villager_registry_id(&effect_id, "food effect ID")?;
        if effect_id.len() > MAX_ITEM_FOOD_EFFECT_ID_BYTES {
            return Err(invalid_argument(
                "food effect ID exceeds its supported byte limit",
            ));
        }
        if !(1..=MAX_ITEM_FOOD_EFFECT_DURATION_TICKS).contains(&duration_ticks)
            || amplifier > 255
            || !probability.is_finite()
            || !(0.0..=1.0).contains(&probability)
        {
            return Err(invalid_argument(
                "food effect values are outside their supported bounds",
            ));
        }
        Ok(Self {
            effect_id,
            duration_ticks,
            amplifier,
            probability,
        })
    }

    pub fn effect_id(&self) -> &str {
        &self.effect_id
    }

    pub const fn duration_ticks(&self) -> u32 {
        self.duration_ticks
    }

    pub const fn amplifier(&self) -> u32 {
        self.amplifier
    }

    pub const fn probability(&self) -> f32 {
        self.probability
    }
}

/// Bounded collection of vanilla status effects attached to one food declaration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemFoodEffects {
    effects: Vec<ItemFoodEffect>,
}

impl ItemFoodEffects {
    pub const fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Adds one effect, rejecting a ninth effect before any host import is called.
    pub fn with_effect(mut self, effect: ItemFoodEffect) -> SjmlResult<Self> {
        if self.effects.len() >= MAX_ITEM_FOOD_EFFECTS {
            return Err(invalid_argument(
                "a food item cannot declare more than 8 status effects",
            ));
        }
        self.effects.push(effect);
        Ok(self)
    }

    pub fn effects(&self) -> &[ItemFoodEffect] {
        &self.effects
    }
}

impl ItemFood {
    /// Creates food properties with nutrition `1..=20` and a saturation modifier `0..=10`.
    pub fn new(nutrition: u32, saturation_modifier: f32) -> SjmlResult<Self> {
        if !(1..=20).contains(&nutrition) {
            return Err(invalid_argument("food nutrition must be between 1 and 20"));
        }
        if !saturation_modifier.is_finite() || !(0.0..=10.0).contains(&saturation_modifier) {
            return Err(invalid_argument(
                "food saturation modifier must be finite and between 0 and 10",
            ));
        }
        Ok(Self {
            nutrition,
            saturation_modifier,
            fast: false,
            always_eat: false,
        })
    }

    /// Sets the vanilla fast-eating flag.
    pub const fn fast(mut self) -> Self {
        self.fast = true;
        self
    }

    /// Allows eating even when the hunger bar is full.
    pub const fn always_eat(mut self) -> Self {
        self.always_eat = true;
        self
    }

    pub const fn nutrition(self) -> u32 {
        self.nutrition
    }

    pub const fn saturation_modifier(self) -> f32 {
        self.saturation_modifier
    }

    pub const fn is_fast(self) -> bool {
        self.fast
    }

    pub const fn can_always_eat(self) -> bool {
        self.always_eat
    }
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

/// One custom Boolean or bounded integer property on a module-registered block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockStateProperty {
    name: String,
    property_type: i32,
    minimum: i32,
    maximum: i32,
    default_value: i32,
}

impl BlockStateProperty {
    /// Creates a Boolean property with the given default value.
    pub fn boolean(name: impl Into<String>, default_value: bool) -> SjmlResult<Self> {
        let name = name.into();
        validate_block_state_property_name(&name)?;
        Ok(Self {
            name,
            property_type: 0,
            minimum: 0,
            maximum: 1,
            default_value: i32::from(default_value),
        })
    }

    /// Creates an integer property with 2 to 16 allowed values in -128..=127.
    pub fn integer(
        name: impl Into<String>,
        minimum: i32,
        maximum: i32,
        default_value: i32,
    ) -> SjmlResult<Self> {
        let name = name.into();
        validate_block_state_property_name(&name)?;
        let value_count = i64::from(maximum) - i64::from(minimum) + 1;
        if minimum < MIN_BLOCK_INTEGER_PROPERTY_VALUE
            || maximum > MAX_BLOCK_INTEGER_PROPERTY_VALUE
            || !(2..=MAX_BLOCK_INTEGER_PROPERTY_VALUES as i64).contains(&value_count)
            || !(minimum..=maximum).contains(&default_value)
        {
            return Err(invalid_argument(
                "integer block-state properties need 2..=16 values in -128..=127 and a valid default",
            ));
        }
        Ok(Self {
            name,
            property_type: 1,
            minimum,
            maximum,
            default_value,
        })
    }

    /// Returns the canonical vanilla property name.
    pub fn name(&self) -> &str {
        &self.name
    }

    fn abi_values(&self) -> [i32; 4] {
        [
            self.property_type,
            self.minimum,
            self.maximum,
            self.default_value,
        ]
    }
}

/// A fixed selection and collision shape for a module-registered block.
///
/// Cuboid coordinates are fractions of one block, from 0.0 through 1.0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockShape {
    /// A full 1×1×1 block cube.
    FullCube,
    /// A block with no outline or collision shape.
    Empty,
    /// A single cuboid used for selection, collision, occlusion, and support.
    Cuboid {
        min_x: f32,
        min_y: f32,
        min_z: f32,
        max_x: f32,
        max_y: f32,
        max_z: f32,
    },
}

impl BlockShape {
    /// Creates a bounded cuboid shape from inclusive minimum and maximum corners.
    pub fn cuboid(min: [f32; 3], max: [f32; 3]) -> SjmlResult<Self> {
        let shape = Self::Cuboid {
            min_x: min[0],
            min_y: min[1],
            min_z: min[2],
            max_x: max[0],
            max_y: max[1],
            max_z: max[2],
        };
        shape.validate()?;
        Ok(shape)
    }

    fn validate(self) -> SjmlResult<()> {
        if let Self::Cuboid {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        } = self
        {
            let minimums = [min_x, min_y, min_z];
            let maximums = [max_x, max_y, max_z];
            if minimums
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
                || maximums
                    .iter()
                    .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
                || (0..3).any(|axis| minimums[axis] >= maximums[axis])
            {
                return Err(invalid_argument(
                    "block cuboid corners must be finite, inside 0..=1, and strictly ordered",
                ));
            }
        }
        Ok(())
    }

    fn abi_values(self) -> (i32, [f32; 6]) {
        match self {
            Self::FullCube => (0, [0.0; 6]),
            Self::Empty => (1, [0.0; 6]),
            Self::Cuboid {
                min_x,
                min_y,
                min_z,
                max_x,
                max_y,
                max_z,
            } => (2, [min_x, min_y, min_z, max_x, max_y, max_z]),
        }
    }
}

/// One validated axis-aligned cuboid in block-local normalized coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockAabb {
    min: [f32; 3],
    max: [f32; 3],
}

impl BlockAabb {
    /// Creates a cuboid whose corners are finite, within 0.0 through 1.0, and strictly ordered.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> SjmlResult<Self> {
        let value = Self { min, max };
        value.validate()?;
        Ok(value)
    }

    /// Returns the minimum x/y/z corner.
    pub const fn min(&self) -> [f32; 3] {
        self.min
    }

    /// Returns the maximum x/y/z corner.
    pub const fn max(&self) -> [f32; 3] {
        self.max
    }

    fn validate(&self) -> SjmlResult<()> {
        if self
            .min
            .iter()
            .chain(self.max.iter())
            .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
            || (0..3).any(|axis| self.min[axis] >= self.max[axis])
        {
            return Err(invalid_argument(
                "block box corners must be finite, inside 0..=1, and strictly ordered",
            ));
        }
        Ok(())
    }
}

/// A validated union of one through eight block-local cuboids.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockShapeBoxes {
    boxes: Vec<BlockAabb>,
}

impl BlockShapeBoxes {
    /// Creates a multi-box shape, rejecting empty lists and lists above [`MAX_BLOCK_SHAPE_BOXES`].
    pub fn new(boxes: impl IntoIterator<Item = BlockAabb>) -> SjmlResult<Self> {
        let boxes: Vec<_> = boxes.into_iter().collect();
        if boxes.is_empty() || boxes.len() > MAX_BLOCK_SHAPE_BOXES {
            return Err(invalid_argument(
                "block multi-box shape must contain 1 through 8 boxes",
            ));
        }
        for cuboid in &boxes {
            cuboid.validate()?;
        }
        Ok(Self { boxes })
    }

    /// Returns the immutable cuboids in this shape.
    pub fn boxes(&self) -> &[BlockAabb] {
        &self.boxes
    }
}

/// Bounded movement, light, ignition, piston, and sound properties for a registered block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockBehavior {
    friction: f32,
    speed_factor: f32,
    jump_factor: f32,
    light_level: u32,
    ignited_by_lava: bool,
    push_reaction: BlockPushReaction,
    sound: BlockSound,
}

/// State-specific properties handled by Minecraft's BlockState hooks.
///
/// Fields left unset retain the registered block's existing behavior. These overrides are static
/// and are selected from the immutable state properties; they never call guest code at runtime.
/// Friction uses NeoForge's state-aware block extension. Speed and jump factors are only
/// block-wide in Minecraft 1.21.1 and belong on [`BlockBehavior`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlockStateBehavior {
    friction: Option<f32>,
    sound: Option<BlockSound>,
    redstone_conductor: Option<bool>,
    suffocating: Option<bool>,
    view_blocking: Option<bool>,
    emissive_rendering: Option<bool>,
    post_process: Option<bool>,
}

impl BlockStateBehavior {
    /// Creates an empty override. Set at least one field before staging it.
    pub const fn new() -> Self {
        Self {
            friction: None,
            sound: None,
            redstone_conductor: None,
            suffocating: None,
            view_blocking: None,
            emissive_rendering: None,
            post_process: None,
        }
    }

    /// Sets the friction for this exact block state from 0.0 through 2.0.
    ///
    /// NeoForge's state-aware friction hook is used. Minecraft 1.21.1 does not expose equivalent
    /// state-aware hooks for speed and jump factors, so those remain block-wide settings.
    pub fn friction(mut self, value: f32) -> SjmlResult<Self> {
        if !value.is_finite() || !(0.0..=2.0).contains(&value) {
            return Err(invalid_argument(
                "block-state friction must be finite and between 0 and 2",
            ));
        }
        self.friction = Some(value);
        Ok(self)
    }

    /// Sets the vanilla sound group used by this exact block state.
    pub const fn sound(mut self, sound: BlockSound) -> Self {
        self.sound = Some(sound);
        self
    }

    /// Overrides whether this state conducts redstone signals.
    pub const fn redstone_conductor(mut self, value: bool) -> Self {
        self.redstone_conductor = Some(value);
        self
    }

    /// Overrides whether this state blocks entity suffocation checks.
    pub const fn suffocating(mut self, value: bool) -> Self {
        self.suffocating = Some(value);
        self
    }

    /// Overrides whether this state blocks the camera view.
    pub const fn view_blocking(mut self, value: bool) -> Self {
        self.view_blocking = Some(value);
        self
    }

    /// Overrides whether this state renders as emissive.
    pub const fn emissive_rendering(mut self, value: bool) -> Self {
        self.emissive_rendering = Some(value);
        self
    }

    /// Overrides whether this state needs a post-process render pass.
    pub const fn post_process(mut self, value: bool) -> Self {
        self.post_process = Some(value);
        self
    }

    fn validate(self) -> SjmlResult<()> {
        if self.friction.is_none()
            && self.sound.is_none()
            && self.redstone_conductor.is_none()
            && self.suffocating.is_none()
            && self.view_blocking.is_none()
            && self.emissive_rendering.is_none()
            && self.post_process.is_none()
        {
            return Err(invalid_argument(
                "block-state behavior must set at least one property",
            ));
        }
        Ok(())
    }

    fn abi_values(self) -> (u32, i32, u32, u32) {
        let mut mask = 0;
        let mut values = 0;
        if self.friction.is_some() {
            mask |= BLOCK_STATE_BEHAVIOR_FRICTION;
        }
        if self.sound.is_some() {
            mask |= BLOCK_STATE_BEHAVIOR_SOUND;
        }
        for (field, value) in [
            (
                BLOCK_STATE_BEHAVIOR_REDSTONE_CONDUCTOR,
                self.redstone_conductor,
            ),
            (BLOCK_STATE_BEHAVIOR_SUFFOCATING, self.suffocating),
            (BLOCK_STATE_BEHAVIOR_VIEW_BLOCKING, self.view_blocking),
            (
                BLOCK_STATE_BEHAVIOR_EMISSIVE_RENDERING,
                self.emissive_rendering,
            ),
            (BLOCK_STATE_BEHAVIOR_POST_PROCESS, self.post_process),
        ] {
            if let Some(value) = value {
                mask |= field;
                if value {
                    values |= field;
                }
            }
        }
        (
            mask,
            self.sound.map_or(0, |sound| sound as i32),
            values,
            self.friction.map_or(0, f32::to_bits),
        )
    }
}

impl BlockBehavior {
    /// Creates vanilla-like movement and sound properties with no emitted light.
    pub const fn new() -> Self {
        Self {
            friction: 0.6,
            speed_factor: 1.0,
            jump_factor: 1.0,
            light_level: 0,
            ignited_by_lava: false,
            push_reaction: BlockPushReaction::Normal,
            sound: BlockSound::Stone,
        }
    }

    /// Sets friction from 0.0 through 2.0; vanilla blocks commonly use 0.4 through 0.98.
    pub fn friction(mut self, value: f32) -> SjmlResult<Self> {
        if !value.is_finite() || !(0.0..=2.0).contains(&value) {
            return Err(invalid_argument(
                "block friction must be finite and between 0 and 2",
            ));
        }
        self.friction = value;
        Ok(self)
    }

    /// Sets the walking-speed multiplier from 0.0 through 4.0.
    pub fn speed_factor(mut self, value: f32) -> SjmlResult<Self> {
        if !value.is_finite() || !(0.0..=4.0).contains(&value) {
            return Err(invalid_argument(
                "block speed factor must be finite and between 0 and 4",
            ));
        }
        self.speed_factor = value;
        Ok(self)
    }

    /// Sets the jump-height multiplier from 0.0 through 4.0.
    pub fn jump_factor(mut self, value: f32) -> SjmlResult<Self> {
        if !value.is_finite() || !(0.0..=4.0).contains(&value) {
            return Err(invalid_argument(
                "block jump factor must be finite and between 0 and 4",
            ));
        }
        self.jump_factor = value;
        Ok(self)
    }

    /// Sets emitted block light from 0 through 15.
    pub fn light_level(mut self, value: u32) -> SjmlResult<Self> {
        if value > 15 {
            return Err(invalid_argument(
                "block light level must be between 0 and 15",
            ));
        }
        self.light_level = value;
        Ok(self)
    }

    /// Makes the block catch fire when lava ignites it.
    pub const fn ignited_by_lava(mut self) -> Self {
        self.ignited_by_lava = true;
        self
    }

    /// Selects the vanilla piston reaction.
    pub const fn push_reaction(mut self, reaction: BlockPushReaction) -> Self {
        self.push_reaction = reaction;
        self
    }

    /// Selects a bounded vanilla block sound group.
    pub const fn sound(mut self, sound: BlockSound) -> Self {
        self.sound = sound;
        self
    }

    fn validate(self) -> SjmlResult<()> {
        if !self.friction.is_finite()
            || !(0.0..=2.0).contains(&self.friction)
            || !self.speed_factor.is_finite()
            || !(0.0..=4.0).contains(&self.speed_factor)
            || !self.jump_factor.is_finite()
            || !(0.0..=4.0).contains(&self.jump_factor)
            || self.light_level > 15
        {
            return Err(invalid_argument(
                "block behavior values are outside their supported bounds",
            ));
        }
        Ok(())
    }

    fn abi_values(self) -> [i32; 7] {
        [
            self.friction.to_bits() as i32,
            self.speed_factor.to_bits() as i32,
            self.jump_factor.to_bits() as i32,
            self.light_level as i32,
            if self.ignited_by_lava { 1 } else { 0 },
            self.push_reaction as i32,
            self.sound as i32,
        ]
    }
}

impl Default for BlockBehavior {
    fn default() -> Self {
        Self::new()
    }
}

/// Stable piston-reaction codes used by registered block behaviors.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlockPushReaction {
    #[default]
    Normal = 0,
    Destroy = 1,
    Block = 2,
    Ignore = 3,
    PushOnly = 4,
}

/// Stable, allowlisted vanilla sound groups for registered blocks.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlockSound {
    #[default]
    Stone = 0,
    Wood = 1,
    Gravel = 2,
    Grass = 3,
    Metal = 4,
    Glass = 5,
    Sand = 6,
    Wool = 7,
    Snow = 8,
    Slime = 9,
    Honey = 10,
    Netherite = 11,
    Amethyst = 12,
}

/// One bounded vanilla AI goal installed on a registered living entity.
#[derive(Clone, Debug, PartialEq)]
pub enum EntityAiGoal {
    /// Swim when in fluid.
    Swim { priority: u32 },
    /// Walk to nearby positions at a bounded movement speed.
    RandomStroll { priority: u32, speed: f32 },
    /// Wander only along paths that avoid water at a bounded movement speed.
    WaterAvoidingRandomStroll { priority: u32, speed: f32 },
    /// Flee at a bounded movement speed after taking damage or catching fire.
    Panic { priority: u32, speed: f32 },
    /// Swim to a random position while in water at a bounded movement speed.
    RandomSwimming { priority: u32, speed: f32 },
    /// Avoid nearby players at bounded range with separate walk and sprint speeds.
    AvoidPlayer {
        priority: u32,
        range: f32,
        walk_speed: f32,
        sprint_speed: f32,
    },
    /// Avoid nearby living entities of exactly this registered type.
    AvoidEntityType {
        priority: u32,
        entity_type_id: String,
        range: f32,
        walk_speed: f32,
        sprint_speed: f32,
    },
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
    /// Select nearby living entities of exactly this registered type as targets.
    TargetEntityType {
        priority: u32,
        entity_type_id: String,
        must_see: bool,
    },
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

/// One bounded item stack used as a villager trade input or result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VillagerTradeItem {
    item_id: String,
    count: u32,
}

impl VillagerTradeItem {
    /// Creates an item stack from a fully qualified item registry ID and a count from 1 to 64.
    pub fn new(item_id: impl Into<String>, count: u32) -> SjmlResult<Self> {
        let item_id = item_id.into();
        validate_villager_registry_id(&item_id, "trade item ID")?;
        if !(1..=64).contains(&count) {
            return Err(invalid_argument(
                "villager trade item counts must be between 1 and 64",
            ));
        }
        Ok(Self { item_id, count })
    }

    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}

/// One static offer added to a vanilla villager profession and level.
#[derive(Clone, Debug, PartialEq)]
pub struct VillagerTradeOffer {
    profession_id: String,
    level: u32,
    cost_a: VillagerTradeItem,
    cost_b: Option<VillagerTradeItem>,
    result: VillagerTradeItem,
    max_uses: u32,
    villager_xp: u32,
    price_multiplier: f32,
    initial_demand: i32,
}

impl VillagerTradeOffer {
    /// Creates a trade with one cost, 12 uses, one villager XP, and vanilla's 0.05 price multiplier.
    pub fn new(
        profession_id: impl Into<String>,
        level: u32,
        cost_a: VillagerTradeItem,
        result: VillagerTradeItem,
    ) -> SjmlResult<Self> {
        let profession_id = profession_id.into();
        validate_villager_registry_id(&profession_id, "villager profession ID")?;
        if !(1..=5).contains(&level) {
            return Err(invalid_argument(
                "villager trade levels must be between 1 and 5",
            ));
        }
        Ok(Self {
            profession_id,
            level,
            cost_a,
            cost_b: None,
            result,
            max_uses: 12,
            villager_xp: 1,
            price_multiplier: 0.05,
            initial_demand: 0,
        })
    }

    /// Adds an optional second cost stack, such as a book alongside emeralds.
    pub fn with_second_cost(mut self, cost_b: VillagerTradeItem) -> Self {
        self.cost_b = Some(cost_b);
        self
    }

    /// Sets bounded offer stock and pricing values.
    pub fn with_economy(
        mut self,
        max_uses: u32,
        villager_xp: u32,
        price_multiplier: f32,
    ) -> SjmlResult<Self> {
        validate_villager_economy(max_uses, villager_xp, price_multiplier)?;
        self.max_uses = max_uses;
        self.villager_xp = villager_xp;
        self.price_multiplier = price_multiplier;
        Ok(self)
    }

    /// Sets the offer's bounded initial vanilla demand value.
    ///
    /// Positive demand increases the first input's displayed price according to the
    /// price multiplier. Minecraft updates demand after trades; negative values do
    /// not discount the base price because vanilla clamps the demand surcharge at 0.
    pub fn with_initial_demand(mut self, initial_demand: i32) -> SjmlResult<Self> {
        validate_villager_initial_demand(initial_demand)?;
        self.initial_demand = initial_demand;
        Ok(self)
    }
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
    cuboid_animation_tracks: Vec<[Option<ModelAnimation>; 3]>,
    texture: Option<String>,
    animation: Option<ModelAnimation>,
    animation_tracks: [Option<ModelAnimation>; 3],
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
        self.cuboid_animation_tracks.push([None; 3]);
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

    /// Replaces model-wide rotation tracks with a sine rotation around the entity-local pivot.
    pub fn set_animation(&mut self, animation: ModelAnimation) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, -4.0, 4.0)?;
        self.animation = Some(animation);
        self.animation_tracks = [None; 3];
        Ok(self)
    }

    /// Adds or replaces one axis rotation while retaining the other model rotations.
    /// Each model can carry at most one bounded track for each of X, Y, and Z.
    pub fn add_animation(&mut self, animation: ModelAnimation) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, -4.0, 4.0)?;
        if let Some(legacy) = self.animation.take() {
            self.animation_tracks[legacy.axis as usize] = Some(legacy);
        }
        self.animation_tracks[animation.axis as usize] = Some(animation);
        Ok(self)
    }

    /// Removes one model-wide axis rotation.
    pub fn clear_animation(&mut self, axis: ModelAnimationAxis) -> &mut Self {
        if self
            .animation
            .is_some_and(|animation| animation.axis == axis)
        {
            self.animation = None;
        }
        self.animation_tracks[axis as usize] = None;
        self
    }

    /// Replaces this cuboid's axis tracks with a rotation around an entity-local pivot.
    /// The animation is independent of sibling cuboids.
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
        self.cuboid_animation_tracks[index] = [None; 3];
        Ok(self)
    }

    /// Adds or replaces one axis rotation on a cuboid without changing its other axes.
    pub fn add_cuboid_animation(
        &mut self,
        index: usize,
        animation: ModelAnimation,
    ) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, -4.0, 4.0)?;
        if index >= self.cuboid_animation_tracks.len() {
            return Err(invalid_argument(
                "cuboid animation index does not refer to a model cuboid",
            ));
        }
        let legacy = self.cuboid_animations[index].take();
        if let Some(legacy) = legacy {
            self.cuboid_animation_tracks[index][legacy.axis as usize] = Some(legacy);
        }
        self.cuboid_animation_tracks[index][animation.axis as usize] = Some(animation);
        Ok(self)
    }

    /// Removes the independent animation from one cuboid.
    pub fn clear_cuboid_animation(&mut self, index: usize) -> SjmlResult<&mut Self> {
        let track = self.cuboid_animations.get_mut(index).ok_or_else(|| {
            invalid_argument("cuboid animation index does not refer to a model cuboid")
        })?;
        *track = None;
        self.cuboid_animation_tracks[index] = [None; 3];
        Ok(self)
    }

    /// Removes one axis rotation from a cuboid and leaves its other axes intact.
    pub fn clear_cuboid_animation_axis(
        &mut self,
        index: usize,
        axis: ModelAnimationAxis,
    ) -> SjmlResult<&mut Self> {
        if index >= self.cuboid_animation_tracks.len() {
            return Err(invalid_argument(
                "cuboid animation index does not refer to a model cuboid",
            ));
        }
        if self.cuboid_animations[index].is_some_and(|animation| animation.axis == axis) {
            self.cuboid_animations[index] = None;
        }
        self.cuboid_animation_tracks[index][axis as usize] = None;
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
    cuboid_animation_tracks: Vec<[Option<ModelAnimation>; 3]>,
    texture: Option<String>,
    animation: Option<ModelAnimation>,
    animation_tracks: [Option<ModelAnimation>; 3],
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
        self.cuboid_animation_tracks.push([None; 3]);
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

    /// Replaces model-wide rotation tracks with a sine rotation around the block-local pivot.
    pub fn set_animation(&mut self, animation: ModelAnimation) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, 0.0, 1.0)?;
        self.animation = Some(animation);
        self.animation_tracks = [None; 3];
        Ok(self)
    }

    /// Adds or replaces one axis rotation while retaining the other model rotations.
    /// Each model can carry at most one bounded track for each of X, Y, and Z.
    pub fn add_animation(&mut self, animation: ModelAnimation) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, 0.0, 1.0)?;
        if let Some(legacy) = self.animation.take() {
            self.animation_tracks[legacy.axis as usize] = Some(legacy);
        }
        self.animation_tracks[animation.axis as usize] = Some(animation);
        Ok(self)
    }

    /// Removes one model-wide axis rotation.
    pub fn clear_animation(&mut self, axis: ModelAnimationAxis) -> &mut Self {
        if self
            .animation
            .is_some_and(|animation| animation.axis == axis)
        {
            self.animation = None;
        }
        self.animation_tracks[axis as usize] = None;
        self
    }

    /// Replaces this cuboid's axis tracks with a rotation around a block-local pivot.
    /// The animation is independent of sibling cuboids.
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
        self.cuboid_animation_tracks[index] = [None; 3];
        Ok(self)
    }

    /// Adds or replaces one axis rotation on a cuboid without changing its other axes.
    pub fn add_cuboid_animation(
        &mut self,
        index: usize,
        animation: ModelAnimation,
    ) -> SjmlResult<&mut Self> {
        validate_model_animation(animation, 0.0, 1.0)?;
        if index >= self.cuboid_animation_tracks.len() {
            return Err(invalid_argument(
                "cuboid animation index does not refer to a model cuboid",
            ));
        }
        let legacy = self.cuboid_animations[index].take();
        if let Some(legacy) = legacy {
            self.cuboid_animation_tracks[index][legacy.axis as usize] = Some(legacy);
        }
        self.cuboid_animation_tracks[index][animation.axis as usize] = Some(animation);
        Ok(self)
    }

    /// Removes the independent animation from one cuboid.
    pub fn clear_cuboid_animation(&mut self, index: usize) -> SjmlResult<&mut Self> {
        let track = self.cuboid_animations.get_mut(index).ok_or_else(|| {
            invalid_argument("cuboid animation index does not refer to a model cuboid")
        })?;
        *track = None;
        self.cuboid_animation_tracks[index] = [None; 3];
        Ok(self)
    }

    /// Removes one axis rotation from a cuboid and leaves its other axes intact.
    pub fn clear_cuboid_animation_axis(
        &mut self,
        index: usize,
        axis: ModelAnimationAxis,
    ) -> SjmlResult<&mut Self> {
        if index >= self.cuboid_animation_tracks.len() {
            return Err(invalid_argument(
                "cuboid animation index does not refer to a model cuboid",
            ));
        }
        if self.cuboid_animations[index].is_some_and(|animation| animation.axis == axis) {
            self.cuboid_animations[index] = None;
        }
        self.cuboid_animation_tracks[index][axis as usize] = None;
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

/// Input and output for one vanilla stonecutting recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StonecuttingRecipeDefinition {
    pub ingredient: RecipeIngredient,
    pub output_item: String,
    pub output_count: u32,
}

/// Template, base, addition, and output for one vanilla smithing transform recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmithingTransformRecipeDefinition {
    pub template: RecipeIngredient,
    pub base: RecipeIngredient,
    pub addition: RecipeIngredient,
    pub output_item: String,
    pub output_count: u32,
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

/// Inclusive integer value bounds for a persistent, network-synchronized data component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegerComponentDefinition {
    pub minimum: i32,
    pub maximum: i32,
}

/// Inclusive finite `f32` value bounds for a persistent, network-synchronized data component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatComponentDefinition {
    pub minimum: f32,
    pub maximum: f32,
}

impl IntegerComponentDefinition {
    /// Creates a component definition after checking that its inclusive range is ordered.
    pub fn new(minimum: i32, maximum: i32) -> SjmlResult<Self> {
        if minimum > maximum {
            return Err(invalid_argument(
                "integer component minimum must not exceed maximum",
            ));
        }
        Ok(Self { minimum, maximum })
    }
}

impl FloatComponentDefinition {
    /// Creates a component definition after checking that both bounds are finite and ordered.
    pub fn new(minimum: f32, maximum: f32) -> SjmlResult<Self> {
        if !minimum.is_finite() || !maximum.is_finite() {
            return Err(invalid_argument("float component bounds must be finite"));
        }
        if minimum > maximum {
            return Err(invalid_argument(
                "float component minimum must not exceed maximum",
            ));
        }
        Ok(Self { minimum, maximum })
    }

    /// Returns whether a value is finite and inside this definition's inclusive bounds.
    pub fn accepts(self, value: f32) -> bool {
        value.is_finite() && value >= self.minimum && value <= self.maximum
    }
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
    #[link_name = "set_item_rarity"]
    fn imported_set_item_rarity(path_pointer: i32, path_length: i32, rarity: i32) -> i32;
    #[link_name = "set_item_enchantment_value"]
    fn imported_set_item_enchantment_value(path_pointer: i32, path_length: i32, value: i32) -> i32;
    #[link_name = "set_item_attribute_modifier"]
    fn imported_set_item_attribute_modifier(
        item_path_pointer: i32,
        item_path_length: i32,
        attribute_id_pointer: i32,
        attribute_id_length: i32,
        modifier_path_pointer: i32,
        modifier_path_length: i32,
        amount_bits: i32,
        operation: i32,
        slot: i32,
    ) -> i32;
    #[link_name = "set_item_max_damage"]
    fn imported_set_item_max_damage(path_pointer: i32, path_length: i32, max_damage: i32) -> i32;
    #[link_name = "set_item_max_stack_size"]
    fn imported_set_item_max_stack_size(
        path_pointer: i32,
        path_length: i32,
        max_stack_size: i32,
    ) -> i32;
    #[link_name = "set_item_use_behavior"]
    fn imported_set_item_use_behavior(
        path_pointer: i32,
        path_length: i32,
        animation: i32,
        duration_ticks: i32,
    ) -> i32;
    #[link_name = "set_item_crafting_remainder"]
    fn imported_set_item_crafting_remainder(
        item_path_pointer: i32,
        item_path_length: i32,
        remainder_item_id_pointer: i32,
        remainder_item_id_length: i32,
    ) -> i32;
    #[link_name = "set_item_food"]
    fn imported_set_item_food(
        path_pointer: i32,
        path_length: i32,
        nutrition: i32,
        saturation_modifier_bits: i32,
        flags: i32,
    ) -> i32;
    #[link_name = "add_item_food_effect"]
    fn imported_add_item_food_effect(
        item_path_pointer: i32,
        item_path_length: i32,
        effect_id_pointer: i32,
        effect_id_length: i32,
        duration_ticks: i32,
        amplifier: i32,
        probability_bits: i32,
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
    #[link_name = "set_block_random_ticking"]
    fn imported_set_block_random_ticking(path_pointer: i32, path_length: i32, enabled: i32) -> i32;
    #[link_name = "set_block_shape"]
    fn imported_set_block_shape(
        path_pointer: i32,
        path_length: i32,
        shape_kind: i32,
        min_x_bits: i32,
        min_y_bits: i32,
        min_z_bits: i32,
        max_x_bits: i32,
        max_y_bits: i32,
        max_z_bits: i32,
    ) -> i32;
    #[link_name = "set_block_shape_boxes"]
    fn imported_set_block_shape_boxes(
        path_pointer: i32,
        path_length: i32,
        payload_pointer: i32,
        payload_length: i32,
    ) -> i32;
    #[link_name = "set_block_behavior"]
    fn imported_set_block_behavior(
        path_pointer: i32,
        path_length: i32,
        friction_bits: i32,
        speed_factor_bits: i32,
        jump_factor_bits: i32,
        light_level: i32,
        flags: i32,
        push_reaction: i32,
        sound: i32,
    ) -> i32;
    #[link_name = "add_block_state_property"]
    fn imported_add_block_state_property(
        path_pointer: i32,
        path_length: i32,
        property_name_pointer: i32,
        property_name_length: i32,
        property_type: i32,
        minimum: i32,
        maximum: i32,
        default_value: i32,
    ) -> i32;
    #[link_name = "set_block_state_shape"]
    fn imported_set_block_state_shape(
        path_pointer: i32,
        path_length: i32,
        payload_pointer: i32,
        payload_length: i32,
    ) -> i32;
    #[link_name = "set_block_state_light_level"]
    fn imported_set_block_state_light_level(
        path_pointer: i32,
        path_length: i32,
        payload_pointer: i32,
        payload_length: i32,
    ) -> i32;
    #[link_name = "set_block_state_behavior"]
    fn imported_set_block_state_behavior(
        path_pointer: i32,
        path_length: i32,
        payload_pointer: i32,
        payload_length: i32,
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
    #[link_name = "register_stonecutting_recipe"]
    fn imported_register_stonecutting_recipe(
        path_pointer: i32,
        path_length: i32,
        output_item_pointer: i32,
        output_item_length: i32,
        output_count: i32,
        ingredient_item_pointer: i32,
        ingredient_item_length: i32,
    ) -> i32;
    #[link_name = "register_smithing_transform_recipe"]
    fn imported_register_smithing_transform_recipe(
        path_pointer: i32,
        path_length: i32,
        template_pointer: i32,
        template_length: i32,
        base_pointer: i32,
        base_length: i32,
        addition_pointer: i32,
        addition_length: i32,
        output_item_pointer: i32,
        output_item_length: i32,
        output_count: i32,
    ) -> i32;
    #[link_name = "register_brewing_recipe"]
    fn imported_register_brewing_recipe(
        path_pointer: i32,
        path_length: i32,
        input_potion_pointer: i32,
        input_potion_length: i32,
        ingredient_item_pointer: i32,
        ingredient_item_length: i32,
        output_potion_pointer: i32,
        output_potion_length: i32,
    ) -> i32;
    #[link_name = "register_potion"]
    fn imported_register_potion(
        path_pointer: i32,
        path_length: i32,
        effect_id_pointer: i32,
        effect_id_length: i32,
        duration_ticks: i32,
        amplifier: i32,
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
    #[link_name = "register_entity_spawn_egg"]
    fn imported_register_entity_spawn_egg(
        entity_path_pointer: i32,
        entity_path_length: i32,
        item_path_pointer: i32,
        item_path_length: i32,
        primary_color: i32,
        secondary_color: i32,
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
    #[link_name = "register_entity_ai_goal_v2"]
    fn imported_register_entity_ai_goal_v2(
        path_pointer: i32,
        path_length: i32,
        goal: i32,
        priority: i32,
        range_bits: i32,
        walk_speed_bits: i32,
        sprint_speed_bits: i32,
        flags: i32,
    ) -> i32;
    #[link_name = "register_entity_ai_goal_v3"]
    fn imported_register_entity_ai_goal_v3(
        path_pointer: i32,
        path_length: i32,
        goal: i32,
        priority: i32,
        range_bits: i32,
        walk_speed_bits: i32,
        sprint_speed_bits: i32,
        flags: i32,
        entity_type_pointer: i32,
        entity_type_length: i32,
    ) -> i32;
    #[link_name = "register_villager_trade_v2"]
    fn imported_register_villager_trade(
        path_pointer: i32,
        path_length: i32,
        profession_pointer: i32,
        profession_length: i32,
        level: i32,
        cost_a_pointer: i32,
        cost_a_length: i32,
        cost_a_count: i32,
        cost_b_pointer: i32,
        cost_b_length: i32,
        cost_b_count: i32,
        result_pointer: i32,
        result_length: i32,
        result_count: i32,
        max_uses: i32,
        villager_xp: i32,
        price_multiplier_bits: i32,
        initial_demand: i32,
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
    #[link_name = "register_integer_component"]
    fn imported_register_integer_component(
        path_pointer: i32,
        path_length: i32,
        minimum: i32,
        maximum: i32,
    ) -> i32;
    #[link_name = "register_float_component"]
    fn imported_register_float_component(
        path_pointer: i32,
        path_length: i32,
        minimum_bits: i32,
        maximum_bits: i32,
    ) -> i32;
    #[link_name = "set_item_float_component"]
    fn imported_set_item_float_component(
        item_path_pointer: i32,
        item_path_length: i32,
        component_path_pointer: i32,
        component_path_length: i32,
        value_bits: i32,
    ) -> i32;
    #[link_name = "set_item_integer_component"]
    fn imported_set_item_integer_component(
        item_path_pointer: i32,
        item_path_length: i32,
        component_path_pointer: i32,
        component_path_length: i32,
        value: i32,
    ) -> i32;
    #[link_name = "register_boolean_component"]
    fn imported_register_boolean_component(path_pointer: i32, path_length: i32) -> i32;
    #[link_name = "set_item_boolean_component"]
    fn imported_set_item_boolean_component(
        item_path_pointer: i32,
        item_path_length: i32,
        component_path_pointer: i32,
        component_path_length: i32,
        value: i32,
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

/// Selects a vanilla rarity for an item declared earlier by this module.
///
/// Call this from `#[mod_register]`, after [`register_item`] or the registration of a block with
/// an item form. The host accepts only items owned by the same package and module; the default is
/// [`ItemRarity::Common`].
pub fn set_item_rarity(item_path: &str, rarity: ItemRarity) -> SjmlResult<()> {
    validate_path(item_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the stable rarity code.
        let status = unsafe {
            imported_set_item_rarity(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                rarity as i32,
            )
        };
        return status_result(status, "host rejected the item rarity");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = rarity;
        Err(unsupported_target())
    }
}

/// Sets vanilla enchantability for an item declared earlier by this module.
///
/// Use a value from `0..=255` during `#[mod_register]`, after `register_item` or registration of a
/// block with an item form. Zero keeps the vanilla non-enchantable behavior. The host accepts only
/// items owned by the same package and module, and each item accepts one declaration.
pub fn set_item_enchantment_value(item_path: &str, value: u32) -> SjmlResult<()> {
    validate_path(item_path)?;
    if value > 255 {
        return Err(invalid_argument(
            "item enchantment value must be between 0 and 255",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the integer value.
        let status = unsafe {
            imported_set_item_enchantment_value(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                value as i32,
            )
        };
        return status_result(status, "host rejected the item enchantment value");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = value;
        Err(unsupported_target())
    }
}

/// Adds one static vanilla attribute modifier to an item declared by this module.
///
/// Call this from `#[mod_register]`, after `register_item` or after registering a block with an
/// item form. The attribute ID must be fully qualified, for example
/// `minecraft:generic.attack_damage`; modifier paths are scoped to the declaring package/module.
/// Each item accepts at most [`MAX_ITEM_ATTRIBUTE_MODIFIERS`] entries. The host resolves the
/// attribute against Minecraft's active registry during item registration.
pub fn set_item_attribute_modifier(
    item_path: &str,
    modifier: &ItemAttributeModifier,
) -> SjmlResult<()> {
    validate_path(item_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies all bounded strings and validates the stable scalar codes.
        let status = unsafe {
            imported_set_item_attribute_modifier(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                modifier.attribute_id.as_ptr() as usize as u32 as i32,
                modifier.attribute_id.len() as u32 as i32,
                modifier.modifier_path.as_ptr() as usize as u32 as i32,
                modifier.modifier_path.len() as u32 as i32,
                modifier.amount.to_bits() as i32,
                modifier.operation as i32,
                modifier.slot as i32,
            )
        };
        return status_result(status, "host rejected the item attribute modifier");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = modifier;
        Err(unsupported_target())
    }
}

/// Sets a registered item's vanilla durability capacity.
///
/// Call this from `#[mod_register]` after declaring a standalone item with
/// [`ItemDefinition::max_stack_size`] set to `1`. Durability is stored in the
/// normal vanilla `MAX_DAMAGE` component, so Minecraft handles stack damage,
/// repair, and break behavior. `max_damage` must be in `1..=1_000_000`; the
/// host rejects block items, buckets, stackable items, duplicate declarations,
/// and items that belong to another module.
pub fn set_item_max_damage(item_path: &str, max_damage: u32) -> SjmlResult<()> {
    validate_path(item_path)?;
    if !(1..=MAX_ITEM_MAX_DAMAGE).contains(&max_damage) {
        return Err(invalid_argument(
            "item max damage must be between 1 and 1000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the damage capacity and owner.
        let status = unsafe {
            imported_set_item_max_damage(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                max_damage as i32,
            )
        };
        return status_result(status, "host rejected the item max-damage declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = max_damage;
        Err(unsupported_target())
    }
}

/// Sets a registered item's vanilla maximum stack size.
///
/// Call this from `#[mod_register]` after declaring a standalone item with
/// [`register_item`]. `max_stack_size` must be in `1..=99`. The host rejects
/// block items, items registered by another module, duplicate declarations,
/// and stack sizes above `1` when vanilla durability is also configured.
pub fn set_item_max_stack_size(item_path: &str, max_stack_size: u32) -> SjmlResult<()> {
    validate_path(item_path)?;
    if !(1..=MAX_ITEM_MAX_STACK_SIZE).contains(&max_stack_size) {
        return Err(invalid_argument(
            "item max stack size must be between 1 and 99",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the size and item owner.
        let status = unsafe {
            imported_set_item_max_stack_size(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                max_stack_size as i32,
            )
        };
        return status_result(status, "host rejected the item max-stack-size declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = max_stack_size;
        Err(unsupported_target())
    }
}

/// Sets a registered item's static vanilla use animation and use duration.
///
/// Call this from `#[mod_register]` after declaring the item. The host preserves the selected
/// vanilla animation and duration as immutable item properties; it does not invoke guest code
/// while the item is being used. The duration is bounded to `1..=72_000` ticks. `ItemUseAnimation`
/// exposes the vanilla animations supported by the host, excluding custom arm animations.
pub fn set_item_use_behavior(item_path: &str, behavior: ItemUseBehavior) -> SjmlResult<()> {
    validate_path(item_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the static animation and duration.
        let status = unsafe {
            imported_set_item_use_behavior(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                behavior.animation as i32,
                behavior.duration_ticks as i32,
            )
        };
        return status_result(status, "host rejected the item use behavior");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = behavior;
        Err(unsupported_target())
    }
}

/// Assigns a static vanilla crafting remainder to a standalone item declared by this module.
///
/// Call this from `#[mod_register]` after [`register_item`]. The remainder must be a canonical,
/// fully qualified item ID of at most 256 UTF-8 bytes that is already present in Minecraft's item
/// registry before SJML's registry declarations run, such as `minecraft:bucket`. This permits safe
/// lookup before the host creates the registered item and avoids dependencies on registry-event
/// ordering. The API does not construct item stacks or set components, and cannot reference another
/// item being staged by SJML. Each standalone item accepts one remainder declaration.
pub fn set_item_crafting_remainder(item_path: &str, remainder_item_id: &str) -> SjmlResult<()> {
    validate_path(item_path)?;
    validate_registry_id(
        remainder_item_id,
        "crafting remainder item ID",
        MAX_ITEM_CRAFTING_REMAINDER_ID_BYTES,
    )?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_set_item_crafting_remainder(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                remainder_item_id.as_ptr() as usize as u32 as i32,
                remainder_item_id.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the item crafting remainder");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Declares an already-registered item as a usable vanilla food item.
///
/// Call this during `#[mod_register]`, after `register_item` or registration of a block with an
/// item form. The declared item must belong to the same package and module. Each item can receive
/// one food declaration; unspecified items keep their existing non-food behavior. This call keeps
/// the original `set_item_food` host-import schema; use [`set_item_food_with_effects`] for effects.
pub fn set_item_food(item_path: &str, food: ItemFood) -> SjmlResult<()> {
    validate_path(item_path)?;
    let flags = i32::from(food.fast) | (i32::from(food.always_eat) << 1);
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and validates the food values and flag mask.
        let status = unsafe {
            imported_set_item_food(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                food.nutrition as i32,
                food.saturation_modifier.to_bits() as i32,
                flags,
            )
        };
        return status_result(status, "host rejected the item food declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (food, flags);
        Err(unsupported_target())
    }
}

/// Declares a vanilla food item with a bounded list of status effects.
///
/// This first calls [`set_item_food`] with its unchanged ABI, then stages each effect using the
/// separate bounded effect import. The package registration callback is rolled back by the host
/// if any effect declaration fails.
pub fn set_item_food_with_effects(
    item_path: &str,
    food: ItemFood,
    effects: ItemFoodEffects,
) -> SjmlResult<()> {
    validate_path(item_path)?;
    set_item_food(item_path, food)?;
    for effect in &effects.effects {
        stage_item_food_effect(item_path, effect)?;
    }
    Ok(())
}

fn stage_item_food_effect(item_path: &str, effect: &ItemFoodEffect) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: both strings are valid for the duration of the call and host-side bounds are checked.
        let status = unsafe {
            imported_add_item_food_effect(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                effect.effect_id.as_ptr() as usize as u32 as i32,
                effect.effect_id.len() as u32 as i32,
                effect.duration_ticks as i32,
                effect.amplifier as i32,
                effect.probability.to_bits() as i32,
            )
        };
        return status_result(status, "host rejected the item food effect");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (item_path, effect);
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

/// Enables or disables vanilla random ticks for a block registered by this module.
///
/// Call during `#[mod_register]`, after [`register_block`] and before registry freeze. Random
/// ticks use Minecraft's normal random-tick selection and are delivered as the distinct
/// synchronous `block.random_tick` event, only to the module that owns the registered block.
/// Calls outside registration, for unknown blocks, or for another module's blocks are rejected
/// by the host. The host runs callbacks on the logical server thread and includes them in the
/// shared per-server-tick callback budget with scheduled `block.tick` events.
pub fn set_block_random_ticking(path: &str, enabled: bool) -> SjmlResult<()> {
    validate_path(path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded UTF-8 path and validates the registration phase.
        let status = unsafe {
            imported_set_block_random_ticking(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                i32::from(enabled),
            )
        };
        return status_result(status, "host rejected the block random-ticking setting");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = enabled;
        Err(unsupported_target())
    }
}

/// Adds one Boolean or bounded integer property to a block already staged by this module.
///
/// Declare properties during `#[mod_register]`, after `register_block` and before registry freeze.
/// A block may declare at most seven properties and expose at most 128 possible states. These
/// properties can then be read through `world::block_properties` and changed through
/// `world::set_block_properties` while the block is in a loaded chunk.
pub fn add_block_state_property(path: &str, property: &BlockStateProperty) -> SjmlResult<()> {
    validate_path(path)?;
    let [property_type, minimum, maximum, default_value] = property.abi_values();
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded UTF-8 values and validates the scalar declaration.
        let status = unsafe {
            imported_add_block_state_property(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                property.name.as_ptr() as usize as u32 as i32,
                property.name.len() as u32 as i32,
                property_type,
                minimum,
                maximum,
                default_value,
            )
        };
        return status_result(status, "host rejected the block state property");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (property_type, minimum, maximum, default_value);
        Err(unsupported_target())
    }
}

/// Overrides a registered block's selection, collision, occlusion, and support shape for one exact state.
///
/// Call this from `#[mod_register]` after registering the block and all of its state properties. The
/// state slice must name every declared property exactly once. States without an override keep the
/// block's default shape. For example, a closed door state can keep its cuboid while the open state
/// uses a rotated or empty collision approximation.
pub fn set_block_state_shape(
    path: &str,
    state: &[(&str, &str)],
    shape: BlockShape,
) -> SjmlResult<()> {
    validate_path(path)?;
    shape.validate()?;
    let payload = encode_block_state_shape(state, shape)?;
    if payload.len() > MAX_BLOCK_STATE_SHAPE_PAYLOAD_BYTES {
        return Err(invalid_argument(
            "block state shape payload exceeds its supported byte limit",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and versioned state/shape payload before returning.
        let status = unsafe {
            imported_set_block_state_shape(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the block-state shape");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = payload;
        Err(unsupported_target())
    }
}

/// Sets emitted light for one exact state of a module-registered block.
///
/// Call this from `#[mod_register]` after the block and all of its state properties have been
/// declared. The state slice must name every declared property exactly once with a canonical
/// value accepted by that property; duplicate, missing, unknown, or out-of-range values are
/// rejected by the host. The level is limited to `0..=15`. Minecraft stores the resolved light
/// value in each immutable `BlockState`, so the host resolves this table during static registration;
/// it never calls guest code while the game queries block states.
pub fn set_block_state_light_level(
    path: &str,
    state: &[(&str, &str)],
    level: u32,
) -> SjmlResult<()> {
    validate_path(path)?;
    if level > 15 {
        return Err(invalid_argument(
            "block state light level must be between 0 and 15",
        ));
    }
    let payload = encode_block_state_light_level(state, level)?;
    if payload.len() > MAX_BLOCK_STATE_LIGHT_LEVEL_PAYLOAD_BYTES {
        return Err(invalid_argument(
            "block state light-level payload exceeds its supported byte limit",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and exact-state payload before returning.
        let status = unsafe {
            imported_set_block_state_light_level(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the block-state light level");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = payload;
        Err(unsupported_target())
    }
}

/// Applies selected state-aware physics, sound, and rendering behavior to one exact block state.
///
/// Call this from `#[mod_register]` after declaring the block and all of its state properties.
/// The state slice must name every declared property exactly once using canonical values. Fields
/// left unset in [`BlockStateBehavior`] retain the block's existing state behavior. The host
/// resolves these values into bounded immutable lookup tables and never invokes guest code from
/// Minecraft's friction, sound, redstone, collision, or render queries.
pub fn set_block_state_behavior(
    path: &str,
    state: &[(&str, &str)],
    behavior: BlockStateBehavior,
) -> SjmlResult<()> {
    validate_path(path)?;
    behavior.validate()?;
    let payload = encode_block_state_behavior(state, behavior)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and exact-state behavior payload before returning.
        let status = unsafe {
            imported_set_block_state_behavior(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the block-state behavior");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = payload;
        Err(unsupported_target())
    }
}

fn encode_block_state_light_level(state: &[(&str, &str)], level: u32) -> SjmlResult<Vec<u8>> {
    if state.is_empty() || state.len() > MAX_BLOCK_STATE_PROPERTIES {
        return Err(invalid_argument(
            "block-state light override must name between 1 and 7 properties",
        ));
    }
    let mut payload = Vec::with_capacity(MAX_BLOCK_STATE_LIGHT_LEVEL_PAYLOAD_BYTES);
    payload.extend_from_slice(&BLOCK_STATE_LIGHT_LEVEL_SCHEMA_VERSION.to_le_bytes());
    payload.extend_from_slice(&(state.len() as u32).to_le_bytes());
    let mut names = Vec::with_capacity(state.len());
    for (name, value) in state {
        validate_block_state_property_name(name)?;
        if value.is_empty()
            || value.len() > MAX_BLOCK_STATE_VALUE_BYTES
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
        {
            return Err(invalid_argument(
                "block-state values must be 1..=5 lowercase ASCII letters, digits, '_' or '-'; the host validates the declared property type and range",
            ));
        }
        if names.iter().any(|existing| existing == name) {
            return Err(invalid_argument(
                "block-state light override repeats a property name",
            ));
        }
        names.push(*name);
        payload.extend_from_slice(&(name.len() as u32).to_le_bytes());
        payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
        payload.extend_from_slice(name.as_bytes());
        payload.extend_from_slice(value.as_bytes());
    }
    payload.extend_from_slice(&level.to_le_bytes());
    Ok(payload)
}

fn encode_block_state_behavior(
    state: &[(&str, &str)],
    behavior: BlockStateBehavior,
) -> SjmlResult<Vec<u8>> {
    behavior.validate()?;
    if state.is_empty() || state.len() > MAX_BLOCK_STATE_PROPERTIES {
        return Err(invalid_argument(
            "block-state behavior must name between 1 and 7 properties",
        ));
    }
    let mut payload = Vec::with_capacity(MAX_BLOCK_STATE_BEHAVIOR_PAYLOAD_BYTES);
    payload.extend_from_slice(&BLOCK_STATE_BEHAVIOR_SCHEMA_VERSION.to_le_bytes());
    payload.extend_from_slice(&(state.len() as u32).to_le_bytes());
    let mut names = Vec::with_capacity(state.len());
    for (name, value) in state {
        validate_block_state_property_name(name)?;
        if value.is_empty()
            || value.len() > MAX_BLOCK_STATE_VALUE_BYTES
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
        {
            return Err(invalid_argument(
                "block-state behavior values must be 1..=5 lowercase ASCII letters, digits, '_' or '-'; the host validates the declared property type and range",
            ));
        }
        if names.iter().any(|existing| existing == name) {
            return Err(invalid_argument(
                "block-state behavior repeats a property name",
            ));
        }
        names.push(*name);
        payload.extend_from_slice(&(name.len() as u32).to_le_bytes());
        payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
        payload.extend_from_slice(name.as_bytes());
        payload.extend_from_slice(value.as_bytes());
    }
    let (mask, sound, boolean_values, friction_bits) = behavior.abi_values();
    payload.extend_from_slice(&mask.to_le_bytes());
    payload.extend_from_slice(&sound.to_le_bytes());
    payload.extend_from_slice(&boolean_values.to_le_bytes());
    payload.extend_from_slice(&friction_bits.to_le_bytes());
    if payload.len() > MAX_BLOCK_STATE_BEHAVIOR_PAYLOAD_BYTES {
        return Err(invalid_argument(
            "block-state behavior payload exceeds its supported byte limit",
        ));
    }
    Ok(payload)
}

fn encode_block_state_shape(state: &[(&str, &str)], shape: BlockShape) -> SjmlResult<Vec<u8>> {
    if state.is_empty() || state.len() > MAX_BLOCK_STATE_PROPERTIES {
        return Err(invalid_argument(
            "block-state shape must name between 1 and 7 properties",
        ));
    }
    let mut payload = Vec::with_capacity(MAX_BLOCK_STATE_SHAPE_PAYLOAD_BYTES);
    payload.extend_from_slice(&BLOCK_STATE_SHAPE_SCHEMA_VERSION.to_le_bytes());
    let (shape_kind, shape_values) = shape.abi_values();
    payload.extend_from_slice(&(shape_kind as u32).to_le_bytes());
    payload.extend_from_slice(&(state.len() as u32).to_le_bytes());
    let mut names = Vec::with_capacity(state.len());
    for (name, value) in state {
        validate_block_state_property_name(name)?;
        if value.is_empty()
            || value.len() > MAX_BLOCK_STATE_VALUE_BYTES
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
        {
            return Err(invalid_argument(
                "block-state shape values must be 1..=5 lowercase ASCII letters, digits, '_' or '-'; the host validates the declared property type and range",
            ));
        }
        if names.iter().any(|existing| existing == name) {
            return Err(invalid_argument(
                "block-state shape repeats a property name",
            ));
        }
        names.push(*name);
        payload.extend_from_slice(&(name.len() as u32).to_le_bytes());
        payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
        payload.extend_from_slice(name.as_bytes());
        payload.extend_from_slice(value.as_bytes());
    }
    for value in shape_values {
        payload.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    Ok(payload)
}

fn encode_block_shape_boxes(shape: &BlockShapeBoxes) -> SjmlResult<Vec<u8>> {
    if shape.boxes.is_empty() || shape.boxes.len() > MAX_BLOCK_SHAPE_BOXES {
        return Err(invalid_argument(
            "block multi-box shape must contain 1 through 8 boxes",
        ));
    }
    let mut payload = Vec::with_capacity(MAX_BLOCK_SHAPE_BOXES_PAYLOAD_BYTES);
    payload.extend_from_slice(&BLOCK_SHAPE_BOXES_SCHEMA_VERSION.to_le_bytes());
    payload.extend_from_slice(&(shape.boxes.len() as u32).to_le_bytes());
    for cuboid in &shape.boxes {
        cuboid.validate()?;
        for value in cuboid.min.into_iter().chain(cuboid.max) {
            payload.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    Ok(payload)
}

/// Applies one fixed selection and collision shape to a block already staged by this module.
///
/// The shape is used for selection, collision, occlusion, and support checks. Call this during
/// `#[mod_register]`, after [`register_block`]. Cuboid coordinates are local block fractions from
/// 0.0 through 1.0. The legacy block-registration import stays unchanged; shaped blocks use a
/// separate bounded import.
pub fn set_block_shape(path: &str, shape: BlockShape) -> SjmlResult<()> {
    validate_path(path)?;
    shape.validate()?;
    let (kind, values) = shape.abi_values();
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and validates the fixed-size shape values.
        let status = unsafe {
            imported_set_block_shape(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                kind,
                values[0].to_bits() as i32,
                values[1].to_bits() as i32,
                values[2].to_bits() as i32,
                values[3].to_bits() as i32,
                values[4].to_bits() as i32,
                values[5].to_bits() as i32,
            )
        };
        return status_result(status, "host rejected the block shape");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (kind, values);
        Err(unsupported_target())
    }
}

/// Applies a union of one through eight AABBs to a block already staged by this module.
///
/// Call this during `#[mod_register]`, after [`register_block`]. Each box uses block-local
/// coordinates from 0.0 through 1.0 and must have finite, strictly ordered corners. The union is
/// used for selection, collision, occlusion, and support geometry. This uses its own bounded ABI
/// import and leaves the legacy single-box [`set_block_shape`] import unchanged.
pub fn set_block_shape_boxes(path: &str, shape: &BlockShapeBoxes) -> SjmlResult<()> {
    validate_path(path)?;
    let payload = encode_block_shape_boxes(shape)?;
    if payload.len() > MAX_BLOCK_SHAPE_BOXES_PAYLOAD_BYTES {
        return Err(invalid_argument(
            "block multi-box shape payload exceeds its supported byte limit",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and versioned shape payload before returning.
        let status = unsafe {
            imported_set_block_shape_boxes(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                payload.as_ptr() as usize as u32 as i32,
                payload.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the block multi-box shape");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = payload;
        Err(unsupported_target())
    }
}

/// Stages a block and assigns its selection and collision shape in one call.
pub fn register_block_with_shape(
    path: &str,
    definition: BlockDefinition,
    shape: BlockShape,
) -> SjmlResult<()> {
    validate_path(path)?;
    shape.validate()?;
    register_block(path, definition)?;
    set_block_shape(path, shape)
}

/// Stages a block and assigns its union-of-boxes selection and collision shape in one call.
pub fn register_block_with_shape_boxes(
    path: &str,
    definition: BlockDefinition,
    shape: &BlockShapeBoxes,
) -> SjmlResult<()> {
    validate_path(path)?;
    encode_block_shape_boxes(shape)?;
    register_block(path, definition)?;
    set_block_shape_boxes(path, shape)
}

/// Applies bounded vanilla movement and physical properties to a block already staged by this module.
///
/// Call this during `#[mod_register]`, after [`register_block`]. The host uses the selected values
/// for movement speed, friction, jump response, emitted light, lava ignition, piston behavior, and
/// one allowlisted vanilla sound group.
pub fn set_block_behavior(path: &str, behavior: BlockBehavior) -> SjmlResult<()> {
    validate_path(path)?;
    behavior.validate()?;
    let values = behavior.abi_values();
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and validates the fixed-size behavior values.
        let status = unsafe {
            imported_set_block_behavior(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                values[0],
                values[1],
                values[2],
                values[3],
                values[4],
                values[5],
                values[6],
            )
        };
        return status_result(status, "host rejected the block behavior");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = values;
        Err(unsupported_target())
    }
}

/// Stages a block and assigns its bounded physical behavior in one registration call.
pub fn register_block_with_behavior(
    path: &str,
    definition: BlockDefinition,
    behavior: BlockBehavior,
) -> SjmlResult<()> {
    validate_path(path)?;
    behavior.validate()?;
    register_block(path, definition)?;
    set_block_behavior(path, behavior)
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

/// Stages a single-input vanilla stonecutting recipe.
///
/// The input may be an item or item tag, the result count is between 1 and 64,
/// and unqualified IDs refer to this package and module. Registration uses
/// Minecraft's vanilla stonecutting serializer during server data reload.
pub fn register_stonecutting_recipe(
    path: &str,
    definition: StonecuttingRecipeDefinition,
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
    validate_recipe_item_id(&definition.output_item)?;
    let ingredient = encode_recipe_ingredient(&definition.ingredient)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies these bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_register_stonecutting_recipe(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.output_item.as_ptr() as usize as u32 as i32,
                definition.output_item.len() as u32 as i32,
                definition.output_count as i32,
                ingredient.as_ptr() as usize as u32 as i32,
                ingredient.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the stonecutting recipe declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ingredient;
        Err(unsupported_target())
    }
}

/// Stages a vanilla smithing transform recipe using one item or item tag per slot.
///
/// Result counts are limited to 1–64. The recipe is written to SJML's generated
/// server data pack and loaded by Minecraft's normal recipe reload flow.
pub fn register_smithing_transform_recipe(
    path: &str,
    definition: SmithingTransformRecipeDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 || path == "." || path == ".." {
        return Err(invalid_argument(
            "smithing transform recipe path must be at most 128 bytes and cannot be . or ..",
        ));
    }
    if !(1..=64).contains(&definition.output_count) {
        return Err(invalid_argument(
            "smithing transform output count must be between 1 and 64",
        ));
    }
    validate_recipe_item_id(&definition.output_item)?;
    let template = encode_recipe_ingredient(&definition.template)?;
    let base = encode_recipe_ingredient(&definition.base)?;
    let addition = encode_recipe_ingredient(&definition.addition)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies all bounded UTF-8 slices before returning.
        let status = unsafe {
            imported_register_smithing_transform_recipe(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                template.as_ptr() as usize as u32 as i32,
                template.len() as u32 as i32,
                base.as_ptr() as usize as u32 as i32,
                base.len() as u32 as i32,
                addition.as_ptr() as usize as u32 as i32,
                addition.len() as u32 as i32,
                definition.output_item.as_ptr() as usize as u32 as i32,
                definition.output_item.len() as u32 as i32,
                definition.output_count as i32,
            )
        };
        return status_result(
            status,
            "host rejected the smithing transform recipe declaration",
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (template, base, addition);
        Err(unsupported_target())
    }
}

/// Registers one bounded vanilla brewing mix from an existing potion and item to an existing potion.
///
/// This declaration is applied by NeoForge on both physical sides, but the module must use
/// `side = "server"` or `side = "both"` because it changes server gameplay. IDs must be fully
/// qualified registry IDs such as `minecraft:water`, `minecraft:emerald`, and `minecraft:awkward`.
/// Call this from `#[mod_register]` with `content.register` granted.
pub fn register_brewing_recipe(
    path: &str,
    input_potion_id: &str,
    ingredient_item_id: &str,
    output_potion_id: &str,
) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 || path == "." || path == ".." {
        return Err(invalid_argument(
            "brewing recipe path must be at most 128 bytes and cannot be . or ..",
        ));
    }
    validate_villager_registry_id(input_potion_id, "input potion ID")?;
    validate_villager_registry_id(ingredient_item_id, "brewing ingredient item ID")?;
    validate_villager_registry_id(output_potion_id, "output potion ID")?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies these bounded canonical IDs before returning.
        let status = unsafe {
            imported_register_brewing_recipe(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                input_potion_id.as_ptr() as usize as u32 as i32,
                input_potion_id.len() as u32 as i32,
                ingredient_item_id.as_ptr() as usize as u32 as i32,
                ingredient_item_id.len() as u32 as i32,
                output_potion_id.as_ptr() as usize as u32 as i32,
                output_potion_id.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the brewing recipe declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Registers a package-owned potion containing one bounded instance of an existing mob effect.
///
/// The potion ID is formed as `package:module/path`. Use it as an existing potion ID in
/// [`register_brewing_recipe`]. Duration is `1..=72_000` ticks and amplifier is `0..=255`.
/// The module must request `content.register` and use `side = "server"` or `side = "both"`.
pub fn register_potion(
    path: &str,
    effect_id: &str,
    duration_ticks: u32,
    amplifier: u32,
) -> SjmlResult<()> {
    validate_path(path)?;
    if path.len() > 128 || path == "." || path == ".." {
        return Err(invalid_argument(
            "potion path must be at most 128 bytes and cannot be . or ..",
        ));
    }
    validate_villager_registry_id(effect_id, "potion effect ID")?;
    if !(1..=72_000).contains(&duration_ticks) || amplifier > 255 {
        return Err(invalid_argument(
            "potion effect duration must be 1–72000 ticks and amplifier 0–255",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies this bounded path and canonical effect ID before returning.
        let status = unsafe {
            imported_register_potion(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                effect_id.as_ptr() as usize as u32 as i32,
                effect_id.len() as u32 as i32,
                duration_ticks as i32,
                amplifier as i32,
            )
        };
        return status_result(status, "host rejected the potion declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
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
            ));
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

/// Registers a vanilla-usable spawn egg for a generic entity declared by this module.
///
/// `entity_path` must have been passed to [`register_entity`] earlier in the same
/// `#[mod_register]` callback. `item_path` is the egg's separate item registry path.
/// Colors are 24-bit RGB values (`0x000000..=0xFFFFFF`); primary is the base tint
/// and secondary is the spot tint. The host registers a `SpawnEggItem` only after
/// the entity type is registered and adds the item to Minecraft's Spawn Eggs tab.
pub fn register_entity_spawn_egg(
    entity_path: &str,
    item_path: &str,
    primary_color: u32,
    secondary_color: u32,
) -> SjmlResult<()> {
    validate_entity_spawn_egg(entity_path, item_path, primary_color, secondary_color)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both validated paths and bounded colors before returning.
        let status = unsafe {
            imported_register_entity_spawn_egg(
                entity_path.as_ptr() as usize as u32 as i32,
                entity_path.len() as u32 as i32,
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                primary_color as i32,
                secondary_color as i32,
            )
        };
        return status_result(status, "host rejected the entity spawn-egg declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (entity_path, item_path, primary_color, secondary_color);
        Err(unsupported_target())
    }
}

fn validate_entity_spawn_egg(
    entity_path: &str,
    item_path: &str,
    primary_color: u32,
    secondary_color: u32,
) -> SjmlResult<()> {
    validate_path(entity_path)?;
    validate_path(item_path)?;
    if primary_color > MAX_ENTITY_SPAWN_EGG_RGB_COLOR
        || secondary_color > MAX_ENTITY_SPAWN_EGG_RGB_COLOR
    {
        return Err(invalid_argument(
            "spawn-egg colors must be 24-bit RGB values from 0x000000 to 0xFFFFFF",
        ));
    }
    Ok(())
}

/// Adds one bounded vanilla AI goal to an entity previously declared by this module.
///
/// Call this from `#[mod_register]`, after [`register_entity`]. A maximum of
/// sixteen goals may be added to one entity type.
pub fn register_entity_ai_goal(path: &str, goal: EntityAiGoal) -> SjmlResult<()> {
    validate_path(path)?;
    if !valid_entity_ai_goal(&goal) {
        return Err(invalid_argument(
            "entity AI goal priority or parameter is outside its supported range",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Some((kind, priority, range, walk_speed, sprint_speed, flags, entity_type_id)) =
            entity_ai_goal_v3_wire(&goal)
        {
            // SAFETY: the host copies both bounded UTF-8 strings and scalar goal parameters before returning.
            let status = unsafe {
                imported_register_entity_ai_goal_v3(
                    path.as_ptr() as usize as u32 as i32,
                    path.len() as u32 as i32,
                    kind,
                    priority as i32,
                    range.to_bits() as i32,
                    walk_speed.to_bits() as i32,
                    sprint_speed.to_bits() as i32,
                    flags,
                    entity_type_id.as_ptr() as usize as u32 as i32,
                    entity_type_id.len() as u32 as i32,
                )
            };
            return status_result(status, "host rejected an entity AI goal");
        }
        if let Some((kind, priority, range, walk_speed, sprint_speed, flags)) =
            entity_ai_goal_v2_wire(&goal)
        {
            // SAFETY: the host copies this bounded path and scalar goal descriptor before returning.
            let status = unsafe {
                imported_register_entity_ai_goal_v2(
                    path.as_ptr() as usize as u32 as i32,
                    path.len() as u32 as i32,
                    kind,
                    priority as i32,
                    range.to_bits() as i32,
                    walk_speed.to_bits() as i32,
                    sprint_speed.to_bits() as i32,
                    flags,
                )
            };
            return status_result(status, "host rejected an entity AI goal");
        }
        let (kind, priority, value, flags) = entity_ai_goal_wire(&goal);
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

/// Adds a bounded offer for a registered villager profession during `#[mod_register]`.
///
/// The host installs the offer when NeoForge fires `VillagerTradesEvent` during server data
/// loading. The declaring module must request `content.register` and use `side = "server"`
/// or `side = "both"`.
pub fn register_villager_trade(path: &str, offer: VillagerTradeOffer) -> SjmlResult<()> {
    validate_path(path)?;
    validate_villager_registry_id(&offer.profession_id, "villager profession ID")?;
    validate_villager_trade_item(&offer.cost_a)?;
    if let Some(cost_b) = &offer.cost_b {
        validate_villager_trade_item(cost_b)?;
    }
    validate_villager_trade_item(&offer.result)?;
    if !(1..=5).contains(&offer.level) {
        return Err(invalid_argument(
            "villager trade levels must be between 1 and 5",
        ));
    }
    validate_villager_economy(offer.max_uses, offer.villager_xp, offer.price_multiplier)?;
    validate_villager_initial_demand(offer.initial_demand)?;
    #[cfg(target_arch = "wasm32")]
    {
        let (cost_b_pointer, cost_b_length, cost_b_count) =
            offer.cost_b.as_ref().map_or((0, 0, 0), |item| {
                (
                    item.item_id.as_ptr() as usize as u32 as i32,
                    item.item_id.len() as u32 as i32,
                    item.count as i32,
                )
            });
        // SAFETY: the host copies these bounded strings and scalar values during mod_register.
        let status = unsafe {
            imported_register_villager_trade(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                offer.profession_id.as_ptr() as usize as u32 as i32,
                offer.profession_id.len() as u32 as i32,
                offer.level as i32,
                offer.cost_a.item_id.as_ptr() as usize as u32 as i32,
                offer.cost_a.item_id.len() as u32 as i32,
                offer.cost_a.count as i32,
                cost_b_pointer,
                cost_b_length,
                cost_b_count,
                offer.result.item_id.as_ptr() as usize as u32 as i32,
                offer.result.item_id.len() as u32 as i32,
                offer.result.count as i32,
                offer.max_uses as i32,
                offer.villager_xp as i32,
                offer.price_multiplier.to_bits() as i32,
                offer.initial_demand,
            )
        };
        return status_result(status, "host rejected a villager trade declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = offer;
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
        &definition.animation_tracks,
        &definition.cuboid_animation_tracks,
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
        &definition.animation_tracks,
        &definition.cuboid_animation_tracks,
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
    animation_tracks: &[Option<ModelAnimation>; 3],
    cuboid_animation_tracks: &[[Option<ModelAnimation>; 3]],
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
    if cuboid_animation_tracks.len() != cuboids.len() {
        return Err(invalid_argument(
            "model cuboid animation tracks do not match the cuboid count",
        ));
    }
    let mut all_tracks = cuboid_animation_tracks.to_vec();
    for (index, legacy) in cuboid_animations.iter().enumerate() {
        if let Some(animation) = legacy {
            all_tracks[index][animation.axis as usize] = Some(*animation);
        }
    }
    let mut root_tracks = *animation_tracks;
    if let Some(animation) = animation {
        root_tracks[animation.axis as usize] = Some(animation);
    }
    let has_compound_animation = root_tracks.iter().flatten().count() > 1
        || all_tracks
            .iter()
            .any(|tracks| tracks.iter().flatten().count() > 1);
    let has_cuboid_animation = all_tracks
        .iter()
        .any(|tracks| tracks.iter().any(Option::is_some));
    let animation_bytes = if has_compound_animation {
        (cuboids.len() + 1) * 3 * BOUND_MODEL_ANIMATION_BYTES
    } else if has_cuboid_animation {
        (cuboids.len() + 1) * BOUND_MODEL_ANIMATION_BYTES
    } else {
        if root_tracks.iter().any(Option::is_some) {
            BOUND_MODEL_ANIMATION_BYTES
        } else {
            0
        }
    };
    let mut encoded = Vec::with_capacity(
        BOUND_MODEL_HEADER_BYTES + cuboids.len() * BOUND_MODEL_CUBOID_BYTES + animation_bytes,
    );
    let schema = if has_compound_animation {
        BOUND_MODEL_AXIS_ANIMATION_SCHEMA_VERSION
    } else if has_cuboid_animation {
        BOUND_MODEL_CUBOID_ANIMATION_SCHEMA_VERSION
    } else if root_tracks.iter().any(Option::is_some) {
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
    if has_compound_animation {
        for track in root_tracks.iter().copied() {
            encode_model_animation_slot(&mut encoded, track);
        }
        for tracks in &all_tracks {
            for animation in tracks {
                encode_model_animation_slot(&mut encoded, *animation);
            }
        }
    } else if has_cuboid_animation {
        encode_model_animation_slot(&mut encoded, root_tracks.iter().flatten().next().copied());
        for tracks in &all_tracks {
            encode_model_animation_slot(&mut encoded, tracks.iter().flatten().next().copied());
        }
    } else if let Some(animation) = root_tracks.iter().flatten().next().copied() {
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

fn valid_entity_ai_goal(goal: &EntityAiGoal) -> bool {
    let valid_priority = |priority: u32| priority <= 31;
    match goal {
        EntityAiGoal::Swim { priority }
        | EntityAiGoal::RandomLookAround { priority }
        | EntityAiGoal::Retaliate { priority } => valid_priority(*priority),
        EntityAiGoal::RandomStroll { priority, speed }
        | EntityAiGoal::WaterAvoidingRandomStroll { priority, speed }
        | EntityAiGoal::Panic { priority, speed }
        | EntityAiGoal::RandomSwimming { priority, speed }
        | EntityAiGoal::MeleeAttack {
            priority, speed, ..
        } => valid_priority(*priority) && speed.is_finite() && (0.01..=8.0).contains(speed),
        EntityAiGoal::AvoidPlayer {
            priority,
            range,
            walk_speed,
            sprint_speed,
        }
        | EntityAiGoal::AvoidEntityType {
            priority,
            range,
            walk_speed,
            sprint_speed,
            ..
        } => {
            valid_priority(*priority)
                && range.is_finite()
                && (1.0..=32.0).contains(range)
                && walk_speed.is_finite()
                && (0.01..=8.0).contains(walk_speed)
                && sprint_speed.is_finite()
                && (*walk_speed..=8.0).contains(sprint_speed)
                && match goal {
                    EntityAiGoal::AvoidEntityType { entity_type_id, .. } => {
                        valid_ai_entity_type_id(entity_type_id)
                    }
                    _ => true,
                }
        }
        EntityAiGoal::LookAtPlayer { priority, range } => {
            valid_priority(*priority) && range.is_finite() && (1.0..=64.0).contains(range)
        }
        EntityAiGoal::TargetPlayer { priority, .. } => valid_priority(*priority),
        EntityAiGoal::TargetEntityType {
            priority,
            entity_type_id,
            ..
        } => valid_priority(*priority) && valid_ai_entity_type_id(entity_type_id),
    }
}

fn valid_ai_entity_type_id(value: &str) -> bool {
    value.len() <= MAX_ENTITY_AI_TARGET_ID_BYTES
        && validate_registry_id(
            value,
            "entity AI target type ID",
            MAX_ENTITY_AI_TARGET_ID_BYTES,
        )
        .is_ok()
}

#[cfg(any(target_arch = "wasm32", test))]
fn entity_ai_goal_wire(goal: &EntityAiGoal) -> (i32, u32, f32, i32) {
    match goal {
        EntityAiGoal::Swim { priority } => (0, *priority, 0.0, 0),
        EntityAiGoal::RandomStroll { priority, speed } => (1, *priority, *speed, 0),
        EntityAiGoal::LookAtPlayer { priority, range } => (2, *priority, *range, 0),
        EntityAiGoal::MeleeAttack {
            priority,
            speed,
            long_memory,
        } => (3, *priority, *speed, i32::from(*long_memory)),
        EntityAiGoal::RandomLookAround { priority } => (4, *priority, 0.0, 0),
        EntityAiGoal::TargetPlayer { priority, must_see } => {
            (5, *priority, 0.0, i32::from(*must_see))
        }
        EntityAiGoal::Retaliate { priority } => (6, *priority, 0.0, 0),
        EntityAiGoal::WaterAvoidingRandomStroll { priority, speed } => (7, *priority, *speed, 0),
        EntityAiGoal::Panic { priority, speed } => (8, *priority, *speed, 0),
        EntityAiGoal::RandomSwimming { priority, speed } => (9, *priority, *speed, 0),
        EntityAiGoal::AvoidPlayer { .. } => unreachable!("AvoidPlayer uses the v2 host import"),
        EntityAiGoal::AvoidEntityType { .. } | EntityAiGoal::TargetEntityType { .. } => {
            unreachable!("type-specific goals use the v3 host import")
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn entity_ai_goal_v2_wire(goal: &EntityAiGoal) -> Option<(i32, u32, f32, f32, f32, i32)> {
    match goal {
        EntityAiGoal::AvoidPlayer {
            priority,
            range,
            walk_speed,
            sprint_speed,
        } => Some((10, *priority, *range, *walk_speed, *sprint_speed, 0)),
        _ => None,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn entity_ai_goal_v3_wire(goal: &EntityAiGoal) -> Option<(i32, u32, f32, f32, f32, i32, &str)> {
    match goal {
        EntityAiGoal::AvoidEntityType {
            priority,
            entity_type_id,
            range,
            walk_speed,
            sprint_speed,
        } => Some((
            11,
            *priority,
            *range,
            *walk_speed,
            *sprint_speed,
            0,
            entity_type_id,
        )),
        EntityAiGoal::TargetEntityType {
            priority,
            entity_type_id,
            must_see,
        } => Some((
            12,
            *priority,
            0.0,
            0.0,
            0.0,
            i32::from(*must_see),
            entity_type_id,
        )),
        _ => None,
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
            ));
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

/// Registers a persistent, network-synchronized bounded integer data component type.
pub fn register_integer_component(
    path: &str,
    definition: IntegerComponentDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    if definition.minimum > definition.maximum {
        return Err(invalid_argument(
            "integer component minimum must not exceed maximum",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and scalar range before returning.
        let status = unsafe {
            imported_register_integer_component(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.minimum,
                definition.maximum,
            )
        };
        return status_result(status, "host rejected the integer component declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Registers a persistent, network-synchronized finite `f32` component with inclusive bounds.
pub fn register_float_component(
    path: &str,
    definition: FloatComponentDefinition,
) -> SjmlResult<()> {
    validate_path(path)?;
    let definition = FloatComponentDefinition::new(definition.minimum, definition.maximum)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded path and scalar bounds before returning.
        let status = unsafe {
            imported_register_float_component(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
                definition.minimum.to_bits() as i32,
                definition.maximum.to_bits() as i32,
            )
        };
        return status_result(status, "host rejected the float component declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = definition;
        Err(unsupported_target())
    }
}

/// Sets a finite float component default on an item declared by this module.
///
/// The host confirms that the component and item are module-owned and that the value is inside
/// the component's inclusive registered bounds.
pub fn set_item_float_component(
    item_path: &str,
    component_path: &str,
    value: f32,
) -> SjmlResult<()> {
    validate_path(item_path)?;
    validate_path(component_path)?;
    if !value.is_finite() {
        return Err(invalid_argument("float component value must be finite"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both paths and the scalar value before returning.
        let status = unsafe {
            imported_set_item_float_component(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as u32 as i32,
                value.to_bits() as i32,
            )
        };
        return status_result(status, "host rejected the item float component default");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Sets an integer component default on an item registered by this module.
///
/// The host checks that the component belongs to this module and that the value falls within the
/// inclusive range supplied to [`register_integer_component`].
pub fn set_item_integer_component(
    item_path: &str,
    component_path: &str,
    value: i32,
) -> SjmlResult<()> {
    validate_path(item_path)?;
    validate_path(component_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded paths and the scalar value before returning.
        let status = unsafe {
            imported_set_item_integer_component(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as u32 as i32,
                value,
            )
        };
        return status_result(status, "host rejected the integer component default");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = value;
        Err(unsupported_target())
    }
}

/// Registers a persistent, network-synchronized boolean data component type.
pub fn register_boolean_component(path: &str) -> SjmlResult<()> {
    validate_path(path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies the bounded UTF-8 path before returning.
        let status = unsafe {
            imported_register_boolean_component(
                path.as_ptr() as usize as u32 as i32,
                path.len() as u32 as i32,
            )
        };
        return status_result(status, "host rejected the boolean component declaration");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Sets a boolean component default on an item registered by this module.
pub fn set_item_boolean_component(
    item_path: &str,
    component_path: &str,
    value: bool,
) -> SjmlResult<()> {
    validate_path(item_path)?;
    validate_path(component_path)?;
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host copies both bounded paths and the scalar value before returning.
        let status = unsafe {
            imported_set_item_boolean_component(
                item_path.as_ptr() as usize as u32 as i32,
                item_path.len() as u32 as i32,
                component_path.as_ptr() as usize as u32 as i32,
                component_path.len() as u32 as i32,
                i32::from(value),
            )
        };
        return status_result(status, "host rejected the boolean component default");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = value;
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

fn validate_block_state_property_name(name: &str) -> SjmlResult<()> {
    if name.is_empty()
        || name.len() > MAX_BLOCK_STATE_PROPERTY_NAME_BYTES
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(invalid_argument(
            "block state property name must contain 1..=32 lowercase ASCII letters, digits, or '_'",
        ));
    }
    Ok(())
}

fn validate_villager_registry_id(value: &str, label: &str) -> SjmlResult<()> {
    validate_registry_id(value, label, MAX_VILLAGER_TRADE_ID_BYTES)
}

fn validate_registry_id(value: &str, label: &str, max_bytes: usize) -> SjmlResult<()> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(invalid_argument(&format!(
            "{label} must be a fully qualified registry ID"
        )));
    };
    let valid_namespace = !namespace.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        });
    let valid_path = !path.is_empty()
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_./-".contains(&byte)
        });
    if value.len() > max_bytes || !valid_namespace || !valid_path {
        return Err(invalid_argument(&format!(
            "{label} must be a valid fully qualified registry ID of at most {max_bytes} bytes"
        )));
    }
    Ok(())
}

fn validate_villager_trade_item(item: &VillagerTradeItem) -> SjmlResult<()> {
    validate_villager_registry_id(&item.item_id, "trade item ID")?;
    if !(1..=64).contains(&item.count) {
        return Err(invalid_argument(
            "villager trade item counts must be between 1 and 64",
        ));
    }
    Ok(())
}

fn validate_villager_economy(
    max_uses: u32,
    villager_xp: u32,
    price_multiplier: f32,
) -> SjmlResult<()> {
    if !(1..=MAX_VILLAGER_TRADE_USES).contains(&max_uses)
        || villager_xp > MAX_VILLAGER_TRADE_XP
        || !price_multiplier.is_finite()
        || !(0.0..=MAX_VILLAGER_TRADE_PRICE_MULTIPLIER).contains(&price_multiplier)
    {
        return Err(invalid_argument(
            "villager trade stock or pricing values are outside their supported bounds",
        ));
    }
    Ok(())
}

fn validate_villager_initial_demand(initial_demand: i32) -> SjmlResult<()> {
    if !(-MAX_VILLAGER_TRADE_INITIAL_DEMAND..=MAX_VILLAGER_TRADE_INITIAL_DEMAND)
        .contains(&initial_demand)
    {
        return Err(invalid_argument(
            "villager trade initial demand must be between -1000 and 1000",
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
    fn block_shapes_validate_normalized_cuboids_before_host_access() {
        assert_eq!(
            BlockShape::cuboid([0.25, 0.0, 0.25], [0.75, 0.5, 0.75]).unwrap(),
            BlockShape::Cuboid {
                min_x: 0.25,
                min_y: 0.0,
                min_z: 0.25,
                max_x: 0.75,
                max_y: 0.5,
                max_z: 0.75,
            }
        );
        for (minimum, maximum) in [
            ([0.5, 0.0, 0.0], [0.5, 1.0, 1.0]),
            ([f32::NAN, 0.0, 0.0], [1.0, 1.0, 1.0]),
            ([-0.01, 0.0, 0.0], [1.0, 1.0, 1.0]),
            ([0.0, 0.0, 0.0], [1.01, 1.0, 1.0]),
        ] {
            assert_eq!(
                BlockShape::cuboid(minimum, maximum).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_block_shape("block", BlockShape::Empty)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_block_behavior("block", BlockBehavior::default())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn multi_box_block_shapes_are_bounded_and_encode_exactly() {
        let left = BlockAabb::new([0.0, 0.0, 0.0], [0.25, 1.0, 1.0]).unwrap();
        let right = BlockAabb::new([0.75, 0.0, 0.0], [1.0, 1.0, 1.0]).unwrap();
        assert_eq!(left.min(), [0.0, 0.0, 0.0]);
        assert_eq!(right.max(), [1.0, 1.0, 1.0]);
        let shape = BlockShapeBoxes::new([left, right]).unwrap();
        assert_eq!(shape.boxes(), &[left, right]);

        let encoded = encode_block_shape_boxes(&shape).unwrap();
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 2);
        assert_eq!(encoded.len(), 8 + 2 * 24);
        assert_eq!(
            u32::from_le_bytes(encoded[8..12].try_into().unwrap()),
            0.0_f32.to_bits()
        );
        assert_eq!(
            u32::from_le_bytes(encoded[20..24].try_into().unwrap()),
            0.25_f32.to_bits()
        );
        assert!(encoded.len() <= MAX_BLOCK_SHAPE_BOXES_PAYLOAD_BYTES);
        let maximum =
            BlockShapeBoxes::new(std::iter::repeat_n(left, MAX_BLOCK_SHAPE_BOXES)).unwrap();
        assert_eq!(
            encode_block_shape_boxes(&maximum).unwrap().len(),
            MAX_BLOCK_SHAPE_BOXES_PAYLOAD_BYTES
        );

        for (minimum, maximum) in [
            ([0.5, 0.0, 0.0], [0.5, 1.0, 1.0]),
            ([f32::NAN, 0.0, 0.0], [1.0, 1.0, 1.0]),
            ([-0.01, 0.0, 0.0], [1.0, 1.0, 1.0]),
            ([0.0, 0.0, 0.0], [1.01, 1.0, 1.0]),
        ] {
            assert_eq!(
                BlockAabb::new(minimum, maximum).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            BlockShapeBoxes::new([]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            BlockShapeBoxes::new(std::iter::repeat_n(left, MAX_BLOCK_SHAPE_BOXES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_block_shape_boxes("block", &shape).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_block_shape_boxes("../bad", &shape).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn random_tick_registration_validates_paths_before_host_access() {
        assert_eq!(
            set_block_random_ticking("valid_block", true)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_block_random_ticking("../other", true)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn block_behavior_builders_reject_unbounded_values_and_keep_stable_codes() {
        assert!(BlockBehavior::new().friction(f32::NAN).is_err());
        assert!(BlockBehavior::new().friction(2.01).is_err());
        assert!(BlockBehavior::new().speed_factor(f32::INFINITY).is_err());
        assert!(BlockBehavior::new().speed_factor(4.01).is_err());
        assert!(BlockBehavior::new().jump_factor(-0.01).is_err());
        assert!(BlockBehavior::new().light_level(16).is_err());

        let behavior = BlockBehavior::new()
            .friction(0.91)
            .unwrap()
            .speed_factor(0.4)
            .unwrap()
            .jump_factor(1.5)
            .unwrap()
            .light_level(13)
            .unwrap()
            .ignited_by_lava()
            .push_reaction(BlockPushReaction::PushOnly)
            .sound(BlockSound::Amethyst);
        assert_eq!(
            behavior.abi_values(),
            [
                0.91_f32.to_bits() as i32,
                0.4_f32.to_bits() as i32,
                1.5_f32.to_bits() as i32,
                13,
                1,
                4,
                12
            ]
        );
    }

    #[test]
    fn block_state_property_builders_keep_codes_bounds_and_defaults() {
        assert_eq!(MAX_BLOCK_STATE_PROPERTIES, 7);
        assert_eq!(MAX_BLOCK_STATE_VARIANTS, 128);
        assert_eq!(MAX_BLOCK_INTEGER_PROPERTY_VALUES, 16);
        let powered = BlockStateProperty::boolean("powered", true).unwrap();
        assert_eq!(powered.name(), "powered");
        assert_eq!(powered.abi_values(), [0, 0, 1, 1]);
        let charge = BlockStateProperty::integer("charge", 0, 3, 1).unwrap();
        assert_eq!(charge.name(), "charge");
        assert_eq!(charge.abi_values(), [1, 0, 3, 1]);

        for invalid in [
            BlockStateProperty::boolean("Upper", false),
            BlockStateProperty::boolean("bad.name", false),
            BlockStateProperty::boolean("", false),
            BlockStateProperty::boolean("x".repeat(33), false),
            BlockStateProperty::integer("charge", 0, 16, 0),
            BlockStateProperty::integer("charge", 10, 9, 9),
            BlockStateProperty::integer("charge", 1, 1, 1),
            BlockStateProperty::integer("charge", 0, 3, 4),
            BlockStateProperty::integer("charge", -129, -128, -128),
            BlockStateProperty::integer("charge", 127, 128, 127),
        ] {
            assert_eq!(invalid.unwrap_err().code(), ErrorCode::InvalidArgument);
        }
    }

    #[test]
    fn block_state_shape_payloads_are_stable_bounded_and_canonical() {
        let shape = BlockShape::cuboid([0.0, 0.0, 0.0], [1.0, 0.5, 1.0]).unwrap();
        let encoded =
            encode_block_state_shape(&[("powered", "true"), ("charge", "3")], shape).unwrap();
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(encoded[8..12].try_into().unwrap()), 2);
        assert!(encoded.len() <= MAX_BLOCK_STATE_SHAPE_PAYLOAD_BYTES);
        assert!(encoded
            .windows(b"powered".len())
            .any(|bytes| bytes == b"powered"));
        assert!(encoded
            .windows(b"charge".len())
            .any(|bytes| bytes == b"charge"));
        assert_eq!(
            set_block_state_shape("block", &[], BlockShape::Empty)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            encode_block_state_shape(
                &[("powered", "true"), ("powered", "false")],
                BlockShape::Empty
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            encode_block_state_shape(&[("powered", "perhaps")], BlockShape::Empty)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            encode_block_state_shape(&[("Upper", "true")], BlockShape::Empty)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_block_state_shape("block", &[("powered", "true")], BlockShape::Empty)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn block_state_light_payloads_are_stable_exact_and_bounded() {
        let encoded =
            encode_block_state_light_level(&[("powered", "true"), ("charge", "3")], 12).unwrap();
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 2);
        assert!(encoded.len() <= MAX_BLOCK_STATE_LIGHT_LEVEL_PAYLOAD_BYTES);
        assert_eq!(&encoded[encoded.len() - 4..], &12_u32.to_le_bytes());

        assert_eq!(
            set_block_state_light_level("block", &[("powered", "true")], 16)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        for state in [
            &[][..],
            &[("powered", "true"), ("powered", "false")][..],
            &[("Upper", "true")][..],
            &[("powered", "perhaps")][..],
            &[("powered", "True")][..],
        ] {
            assert_eq!(
                encode_block_state_light_level(state, 1).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_block_state_light_level("block", &[("powered", "true")], 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn block_state_behavior_payloads_preserve_exact_optional_fields() {
        let behavior = BlockStateBehavior::new()
            .friction(0.95)
            .unwrap()
            .sound(BlockSound::Amethyst)
            .redstone_conductor(false)
            .suffocating(false)
            .view_blocking(true)
            .emissive_rendering(true)
            .post_process(false);
        let encoded = encode_block_state_behavior(&[("powered", "true")], behavior).unwrap();
        assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 1);
        assert_eq!(
            u32::from_le_bytes(encoded[27..31].try_into().unwrap()),
            BLOCK_STATE_BEHAVIOR_SOUND
                | BLOCK_STATE_BEHAVIOR_REDSTONE_CONDUCTOR
                | BLOCK_STATE_BEHAVIOR_SUFFOCATING
                | BLOCK_STATE_BEHAVIOR_VIEW_BLOCKING
                | BLOCK_STATE_BEHAVIOR_EMISSIVE_RENDERING
                | BLOCK_STATE_BEHAVIOR_POST_PROCESS
                | BLOCK_STATE_BEHAVIOR_FRICTION
        );
        assert_eq!(i32::from_le_bytes(encoded[31..35].try_into().unwrap()), 12);
        assert_eq!(
            u32::from_le_bytes(encoded[35..39].try_into().unwrap()),
            BLOCK_STATE_BEHAVIOR_VIEW_BLOCKING | BLOCK_STATE_BEHAVIOR_EMISSIVE_RENDERING
        );
        assert_eq!(
            u32::from_le_bytes(encoded[39..43].try_into().unwrap()),
            0.95_f32.to_bits()
        );
        assert!(encoded.len() <= MAX_BLOCK_STATE_BEHAVIOR_PAYLOAD_BYTES);

        let friction_only = encode_block_state_behavior(
            &[("powered", "true")],
            BlockStateBehavior::new().friction(1.2).unwrap(),
        )
        .unwrap();
        assert_eq!(
            u32::from_le_bytes(friction_only[27..31].try_into().unwrap()),
            BLOCK_STATE_BEHAVIOR_FRICTION
        );
        assert_eq!(
            u32::from_le_bytes(friction_only[39..43].try_into().unwrap()),
            1.2_f32.to_bits()
        );
        for invalid in [f32::NAN, f32::INFINITY, -0.01, 2.01] {
            assert_eq!(
                BlockStateBehavior::new()
                    .friction(invalid)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }

        assert_eq!(
            encode_block_state_behavior(&[("powered", "true")], BlockStateBehavior::new())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        for state in [
            &[][..],
            &[("powered", "true"), ("powered", "false")][..],
            &[("Upper", "true")][..],
            &[("powered", "perhaps")][..],
        ] {
            assert_eq!(
                encode_block_state_behavior(
                    state,
                    BlockStateBehavior::new().sound(BlockSound::Stone)
                )
                .unwrap_err()
                .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_block_state_behavior("block", &[("powered", "true")], behavior)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn integer_component_definitions_validate_ranges_before_host_access() {
        let definition = IntegerComponentDefinition::new(-5, 10).unwrap();
        assert_eq!(definition.minimum, -5);
        assert_eq!(definition.maximum, 10);
        assert_eq!(
            IntegerComponentDefinition::new(1, 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_integer_component(
                "charge",
                IntegerComponentDefinition {
                    minimum: 2,
                    maximum: 1
                }
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_integer_component("charge", definition)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_integer_component("ruby", "charge", 5)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_integer_component("../ruby", "charge", 5)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn float_component_definitions_require_finite_ordered_bounds_and_values() {
        let definition = FloatComponentDefinition::new(-0.5, 8.25).unwrap();
        assert_eq!(definition.minimum, -0.5);
        assert_eq!(definition.maximum, 8.25);
        assert!(definition.accepts(-0.5));
        assert!(definition.accepts(8.25));
        assert!(definition.accepts(2.0));
        for (minimum, maximum) in [
            (f32::NAN, 1.0),
            (0.0, f32::INFINITY),
            (f32::NEG_INFINITY, f32::INFINITY),
            (1.0, 0.0),
        ] {
            assert_eq!(
                FloatComponentDefinition::new(minimum, maximum)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert!(!definition.accepts(f32::NAN));
        assert!(!definition.accepts(f32::INFINITY));
        assert!(!definition.accepts(-0.51));
        assert!(!definition.accepts(8.26));
        assert_eq!(
            register_float_component(
                "heat",
                FloatComponentDefinition {
                    minimum: f32::NEG_INFINITY,
                    maximum: 1.0,
                }
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_float_component("heat", definition)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_float_component("ruby", "heat", f32::NAN)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_float_component("ruby", "heat", 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn boolean_components_validate_paths_before_host_access() {
        assert_eq!(
            register_boolean_component("../enabled").unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            register_boolean_component("enabled").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_boolean_component("ruby", "enabled", true)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_boolean_component("../ruby", "enabled", false)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
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
        let encoded = encode_legacy_bound_model(&[cuboid], None, &[None]).unwrap();
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
        let encoded = encode_legacy_bound_model(&[cuboid], Some(animation), &[None]).unwrap();
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
    fn villager_trade_builders_enforce_registry_and_economy_bounds() {
        let emeralds = VillagerTradeItem::new("minecraft:emerald", 18).unwrap();
        let book = VillagerTradeItem::new("minecraft:book", 1).unwrap();
        let enchanted_book = VillagerTradeItem::new("minecraft:enchanted_book", 1).unwrap();
        let offer = VillagerTradeOffer::new("minecraft:librarian", 3, emeralds, enchanted_book)
            .unwrap()
            .with_second_cost(book)
            .with_economy(8, 12, 0.2)
            .unwrap()
            .with_initial_demand(20)
            .unwrap();
        assert_eq!(offer.profession_id, "minecraft:librarian");
        assert_eq!(offer.level, 3);
        assert_eq!(offer.cost_a.count(), 18);
        assert_eq!(offer.cost_b.as_ref().unwrap().item_id(), "minecraft:book");
        assert_eq!(offer.result.item_id(), "minecraft:enchanted_book");
        assert_eq!(offer.max_uses, 8);
        assert_eq!(offer.villager_xp, 12);
        assert_eq!(offer.price_multiplier, 0.2);
        assert_eq!(offer.initial_demand, 20);

        assert_eq!(
            VillagerTradeItem::new("minecraft:emerald", 65)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            VillagerTradeItem::new("emerald", 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            VillagerTradeOffer::new(
                "minecraft:librarian",
                6,
                VillagerTradeItem::new("minecraft:emerald", 1).unwrap(),
                VillagerTradeItem::new("minecraft:book", 1).unwrap(),
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            VillagerTradeOffer::new(
                "minecraft:librarian",
                1,
                VillagerTradeItem::new("minecraft:emerald", 1).unwrap(),
                VillagerTradeItem::new("minecraft:book", 1).unwrap(),
            )
            .unwrap()
            .with_economy(0, 0, 0.05)
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            VillagerTradeOffer::new(
                "minecraft:librarian",
                1,
                VillagerTradeItem::new("minecraft:emerald", 1).unwrap(),
                VillagerTradeItem::new("minecraft:book", 1).unwrap(),
            )
            .unwrap()
            .with_economy(12, 1, f32::NAN)
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            VillagerTradeOffer::new(
                "minecraft:librarian",
                1,
                VillagerTradeItem::new("minecraft:emerald", 1).unwrap(),
                VillagerTradeItem::new("minecraft:book", 1).unwrap(),
            )
            .unwrap()
            .with_initial_demand(1_001)
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
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
            encode_legacy_bound_model(&[first, second], Some(global), &[None, Some(child)])
                .unwrap();
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
    fn multi_axis_model_animations_use_schema_four_and_keep_each_axis_track() {
        let first = ModelCuboid::new([-0.5, 0.0, -0.5], [0.5, 0.5, 0.5], 0xFF12_3456).unwrap();
        let second = ModelCuboid::new([-0.25, 0.5, -0.25], [0.25, 1.0, 0.25], 0xFF65_4321).unwrap();
        let x = ModelAnimation::sine_rotation(ModelAnimationAxis::X, [0.0, 0.5, 0.0], 12.0, 40, 0)
            .unwrap();
        let y = ModelAnimation::sine_rotation(ModelAnimationAxis::Y, [0.0, 0.5, 0.0], 24.0, 60, 10)
            .unwrap();
        let z = ModelAnimation::sine_rotation(ModelAnimationAxis::Z, [0.0, 0.5, 0.0], 36.0, 80, 20)
            .unwrap();
        let cuboids = [first, second];
        let root_tracks = [Some(x), Some(y), None];
        let cuboid_tracks = [[None, None, None], [Some(x), None, Some(z)]];
        let encoded =
            encode_bound_model(&cuboids, None, &[None, None], &root_tracks, &cuboid_tracks)
                .unwrap();

        assert_eq!(
            encoded.len(),
            BOUND_MODEL_HEADER_BYTES
                + 2 * BOUND_MODEL_CUBOID_BYTES
                + 3 * 3 * BOUND_MODEL_ANIMATION_BYTES
        );
        assert_eq!(&encoded[..8], &[4, 0, 0, 0, 2, 0, 0, 0]);
        let slots = &encoded[BOUND_MODEL_HEADER_BYTES + 2 * BOUND_MODEL_CUBOID_BYTES..];
        assert_eq!(&slots[..4], &[1, 0, 0, 0], "the model has a root X track");
        assert_eq!(
            &slots[BOUND_MODEL_ANIMATION_BYTES..BOUND_MODEL_ANIMATION_BYTES + 4],
            &[1, 0, 0, 0],
            "the model has a root Y track"
        );
        assert_eq!(
            &slots[2 * BOUND_MODEL_ANIMATION_BYTES..3 * BOUND_MODEL_ANIMATION_BYTES],
            &[0; BOUND_MODEL_ANIMATION_BYTES],
            "the model has no root Z track"
        );
        let second_cuboid_slots =
            &slots[3 * BOUND_MODEL_ANIMATION_BYTES + 3 * BOUND_MODEL_ANIMATION_BYTES..];
        assert_eq!(&second_cuboid_slots[..4], &[1, 0, 0, 0], "cuboid X track");
        assert_eq!(
            &second_cuboid_slots[BOUND_MODEL_ANIMATION_BYTES..2 * BOUND_MODEL_ANIMATION_BYTES],
            &[0; BOUND_MODEL_ANIMATION_BYTES],
            "cuboid Y track is empty"
        );
        assert_eq!(
            &second_cuboid_slots
                [2 * BOUND_MODEL_ANIMATION_BYTES..2 * BOUND_MODEL_ANIMATION_BYTES + 4],
            &[1, 0, 0, 0],
            "cuboid Z track"
        );

        let mut model = EntityModelDefinition::new();
        model
            .add_cuboid(first.min, first.max, first.argb)
            .unwrap()
            .add_cuboid(second.min, second.max, second.argb)
            .unwrap()
            .set_animation(x)
            .unwrap()
            .add_animation(y)
            .unwrap()
            .set_cuboid_animation(1, x)
            .unwrap()
            .add_cuboid_animation(1, z)
            .unwrap();
        assert_eq!(model.animation_tracks, [Some(x), Some(y), None]);
        assert_eq!(model.cuboid_animation_tracks[1], [Some(x), None, Some(z)]);
        model.clear_animation(ModelAnimationAxis::X);
        model
            .clear_cuboid_animation_axis(1, ModelAnimationAxis::Z)
            .unwrap();
        assert_eq!(model.animation_tracks, [None, Some(y), None]);
        assert_eq!(model.cuboid_animation_tracks[1], [Some(x), None, None]);

        let mut block_model = BlockEntityModelDefinition::new();
        block_model
            .add_cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 0)
            .unwrap()
            .set_animation(y)
            .unwrap()
            .add_animation(z)
            .unwrap()
            .set_cuboid_animation(0, x)
            .unwrap()
            .add_cuboid_animation(0, z)
            .unwrap();
        assert_eq!(block_model.animation_tracks, [None, Some(y), Some(z)]);
        assert_eq!(
            block_model.cuboid_animation_tracks[0],
            [Some(x), None, Some(z)]
        );
        block_model.clear_animation(ModelAnimationAxis::Y);
        block_model
            .clear_cuboid_animation_axis(0, ModelAnimationAxis::X)
            .unwrap();
        assert_eq!(block_model.animation_tracks, [None, None, Some(z)]);
        assert_eq!(
            block_model.cuboid_animation_tracks[0],
            [None, None, Some(z)]
        );
    }

    fn encode_legacy_bound_model(
        cuboids: &[ModelCuboid],
        animation: Option<ModelAnimation>,
        cuboid_animations: &[Option<ModelAnimation>],
    ) -> SjmlResult<Vec<u8>> {
        encode_bound_model(
            cuboids,
            animation,
            cuboid_animations,
            &[None; 3],
            &vec![[None; 3]; cuboids.len()],
        )
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
        let powered = BlockStateProperty::boolean("powered", false).unwrap();
        assert_eq!(
            add_block_state_property("ruby_block", &powered)
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
        assert_eq!(
            register_entity_spawn_egg("ruby_mite", "ruby_mite_egg", 0x00ff_ffff, 0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        let trade = VillagerTradeOffer::new(
            "minecraft:farmer",
            1,
            VillagerTradeItem::new("minecraft:emerald", 1).unwrap(),
            VillagerTradeItem::new("minecraft:bread", 1).unwrap(),
        )
        .unwrap();
        assert_eq!(
            register_villager_trade("emerald_for_bread", trade)
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
            EntityAiGoal::WaterAvoidingRandomStroll {
                priority: 1,
                speed: 8.1,
            },
            EntityAiGoal::Panic {
                priority: 32,
                speed: 1.0,
            },
            EntityAiGoal::RandomSwimming {
                priority: 1,
                speed: 0.0,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 32,
                range: 16.0,
                walk_speed: 0.5,
                sprint_speed: 1.0,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 1,
                range: 0.99,
                walk_speed: 0.5,
                sprint_speed: 1.0,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 1,
                range: 32.01,
                walk_speed: 0.5,
                sprint_speed: 1.0,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 1,
                range: 16.0,
                walk_speed: f32::NAN,
                sprint_speed: 1.0,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 1,
                range: 16.0,
                walk_speed: 0.5,
                sprint_speed: 0.49,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 1,
                range: 16.0,
                walk_speed: 8.01,
                sprint_speed: 8.01,
            },
            EntityAiGoal::AvoidPlayer {
                priority: 1,
                range: 16.0,
                walk_speed: 0.5,
                sprint_speed: f32::INFINITY,
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
    fn added_entity_ai_goals_keep_stable_wire_tags_and_speed_bounds() {
        assert_eq!(
            entity_ai_goal_wire(&EntityAiGoal::WaterAvoidingRandomStroll {
                priority: 5,
                speed: 0.75,
            }),
            (7, 5, 0.75, 0)
        );
        assert_eq!(
            entity_ai_goal_wire(&EntityAiGoal::Panic {
                priority: 2,
                speed: 1.25,
            }),
            (8, 2, 1.25, 0)
        );
        assert_eq!(
            entity_ai_goal_wire(&EntityAiGoal::RandomSwimming {
                priority: 6,
                speed: 0.5,
            }),
            (9, 6, 0.5, 0)
        );
        assert_eq!(
            entity_ai_goal_wire(&EntityAiGoal::Retaliate { priority: 0 }).0,
            6
        );
        assert_eq!(
            entity_ai_goal_v2_wire(&EntityAiGoal::AvoidPlayer {
                priority: 3,
                range: 16.0,
                walk_speed: 0.75,
                sprint_speed: 1.5,
            }),
            Some((10, 3, 16.0, 0.75, 1.5, 0))
        );
        assert!(valid_entity_ai_goal(&EntityAiGoal::AvoidPlayer {
            priority: 0,
            range: 1.0,
            walk_speed: 0.01,
            sprint_speed: 0.01,
        }));
        assert!(valid_entity_ai_goal(&EntityAiGoal::AvoidPlayer {
            priority: 31,
            range: 32.0,
            walk_speed: 8.0,
            sprint_speed: 8.0,
        }));
        let avoid_cows = EntityAiGoal::AvoidEntityType {
            priority: 3,
            entity_type_id: "minecraft:cow".to_owned(),
            range: 12.0,
            walk_speed: 0.8,
            sprint_speed: 1.4,
        };
        assert!(valid_entity_ai_goal(&avoid_cows));
        assert_eq!(
            entity_ai_goal_v3_wire(&avoid_cows),
            Some((11, 3, 12.0, 0.8, 1.4, 0, "minecraft:cow"))
        );
        let target_cows = EntityAiGoal::TargetEntityType {
            priority: 1,
            entity_type_id: "minecraft:cow".to_owned(),
            must_see: true,
        };
        assert!(valid_entity_ai_goal(&target_cows));
        assert_eq!(
            entity_ai_goal_v3_wire(&target_cows),
            Some((12, 1, 0.0, 0.0, 0.0, 1, "minecraft:cow"))
        );
        assert!(!valid_entity_ai_goal(&EntityAiGoal::TargetEntityType {
            priority: 1,
            entity_type_id: "minecraft:unknown type".to_owned(),
            must_see: false,
        }));
        assert!(!valid_entity_ai_goal(&EntityAiGoal::AvoidEntityType {
            priority: 1,
            entity_type_id: "minecraft:pig".to_owned(),
            range: 1.0,
            walk_speed: 1.0,
            sprint_speed: 0.99,
        }));
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
    fn stonecutting_recipe_bounds_are_validated_before_host_access() {
        let valid = StonecuttingRecipeDefinition {
            ingredient: RecipeIngredient::tag("c:stone"),
            output_item: "minecraft:stone_slab".into(),
            output_count: 2,
        };
        assert_eq!(
            register_stonecutting_recipe("stone_slab", valid.clone())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );
        for definition in [
            StonecuttingRecipeDefinition {
                output_count: 0,
                ..valid.clone()
            },
            StonecuttingRecipeDefinition {
                output_count: 65,
                ..valid.clone()
            },
            StonecuttingRecipeDefinition {
                ingredient: RecipeIngredient::tag("invalid tag"),
                ..valid.clone()
            },
            StonecuttingRecipeDefinition {
                output_item: "bad id".into(),
                ..valid.clone()
            },
        ] {
            assert_eq!(
                register_stonecutting_recipe("stone_slab", definition)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_stonecutting_recipe("..", valid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn smithing_transform_recipe_bounds_and_ingredients_are_validated_before_host_access() {
        let valid = SmithingTransformRecipeDefinition {
            template: RecipeIngredient::tag("minecraft:smithing_templates"),
            base: RecipeIngredient::item("minecraft:diamond_sword"),
            addition: RecipeIngredient::item("minecraft:netherite_ingot"),
            output_item: "minecraft:netherite_sword".into(),
            output_count: 1,
        };
        assert_eq!(
            register_smithing_transform_recipe("netherite_sword", valid.clone())
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );
        for definition in [
            SmithingTransformRecipeDefinition {
                output_count: 0,
                ..valid.clone()
            },
            SmithingTransformRecipeDefinition {
                output_count: 65,
                ..valid.clone()
            },
            SmithingTransformRecipeDefinition {
                template: RecipeIngredient::tag("invalid tag"),
                ..valid.clone()
            },
            SmithingTransformRecipeDefinition {
                base: RecipeIngredient::item("bad item id"),
                ..valid.clone()
            },
            SmithingTransformRecipeDefinition {
                addition: RecipeIngredient::tag("bad tag id"),
                ..valid.clone()
            },
            SmithingTransformRecipeDefinition {
                output_item: "bad item id".into(),
                ..valid.clone()
            },
        ] {
            assert_eq!(
                register_smithing_transform_recipe("netherite_sword", definition)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_smithing_transform_recipe("..", valid)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn brewing_recipe_ids_and_paths_are_validated_before_host_access() {
        let valid = ("minecraft:water", "minecraft:emerald", "minecraft:awkward");
        assert_eq!(
            register_brewing_recipe("water_to_awkward", valid.0, valid.1, valid.2)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );
        for (input, ingredient, output) in [
            ("water", valid.1, valid.2),
            (valid.0, "minecraft:", valid.2),
            (valid.0, valid.1, "minecraft:bad id"),
        ] {
            assert_eq!(
                register_brewing_recipe("water_to_awkward", input, ingredient, output)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
        assert_eq!(
            register_brewing_recipe("..", valid.0, valid.1, valid.2)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
        );
    }

    #[test]
    fn potion_effect_and_path_bounds_are_validated_before_host_access() {
        assert_eq!(
            register_potion("ruby_tonic", "minecraft:jump_boost", 1200, 1)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
        );
        for (path, effect, duration, amplifier) in [
            ("ruby_tonic", "jump_boost", 1200, 1),
            ("ruby_tonic", "minecraft:jump_boost", 0, 1),
            ("ruby_tonic", "minecraft:jump_boost", 72_001, 1),
            ("ruby_tonic", "minecraft:jump_boost", 1200, 256),
            ("..", "minecraft:jump_boost", 1200, 1),
        ] {
            assert_eq!(
                register_potion(path, effect, duration, amplifier)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
            );
        }
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

    #[test]
    fn entity_spawn_egg_paths_and_rgb_colors_are_bounded() {
        assert!(validate_entity_spawn_egg("ruby_mite", "ruby_mite_egg", 0, 0x00ff_ffff).is_ok());
        for (entity_path, item_path, primary_color, secondary_color) in [
            ("../ruby_mite", "ruby_mite_egg", 0, 0),
            ("ruby_mite", "../egg", 0, 0),
            ("ruby_mite", "ruby_mite_egg", 0x0100_0000, 0),
            ("ruby_mite", "ruby_mite_egg", 0, 0x0100_0000),
        ] {
            assert_eq!(
                validate_entity_spawn_egg(entity_path, item_path, primary_color, secondary_color)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn item_rarity_values_have_stable_wire_codes_and_paths_validate_before_import() {
        assert_eq!(ItemRarity::Common as i32, 0);
        assert_eq!(ItemRarity::Uncommon as i32, 1);
        assert_eq!(ItemRarity::Rare as i32, 2);
        assert_eq!(ItemRarity::Epic as i32, 3);
        assert_eq!(
            set_item_rarity("ruby", ItemRarity::Epic)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        assert_eq!(
            set_item_rarity("../ruby", ItemRarity::Epic)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_enchantment_value_is_bounded_and_paths_validate_before_import() {
        assert_eq!(
            set_item_enchantment_value("ruby", 255).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        assert_eq!(
            set_item_enchantment_value("ruby", 256).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_enchantment_value("../ruby", 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_attribute_modifier_codes_bounds_and_ids_are_stable() {
        assert_eq!(MAX_ITEM_ATTRIBUTE_MODIFIERS, 16);
        assert_eq!(ItemAttributeSlot::Any as i32, 0);
        assert_eq!(ItemAttributeSlot::Mainhand as i32, 1);
        assert_eq!(ItemAttributeSlot::Offhand as i32, 2);
        assert_eq!(ItemAttributeSlot::Hand as i32, 3);
        assert_eq!(ItemAttributeSlot::Feet as i32, 4);
        assert_eq!(ItemAttributeSlot::Legs as i32, 5);
        assert_eq!(ItemAttributeSlot::Chest as i32, 6);
        assert_eq!(ItemAttributeSlot::Head as i32, 7);
        assert_eq!(ItemAttributeSlot::Armor as i32, 8);
        assert_eq!(ItemAttributeSlot::Body as i32, 9);

        let modifier = ItemAttributeModifier::new(
            "minecraft:generic.attack_damage",
            "attack_bonus",
            3.5,
            AttributeModifierOperation::AddValue,
            ItemAttributeSlot::Mainhand,
        )
        .unwrap();
        assert_eq!(modifier.attribute_id(), "minecraft:generic.attack_damage");
        assert_eq!(modifier.modifier_path(), "attack_bonus");
        assert_eq!(modifier.amount(), 3.5);
        assert_eq!(modifier.operation(), AttributeModifierOperation::AddValue);
        assert_eq!(modifier.slot(), ItemAttributeSlot::Mainhand);
        assert_eq!(
            set_item_attribute_modifier("ruby", &modifier)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        assert_eq!(
            set_item_attribute_modifier("../ruby", &modifier)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ItemAttributeModifier::new(
                "generic.attack_damage",
                "attack_bonus",
                1.0,
                AttributeModifierOperation::AddValue,
                ItemAttributeSlot::Any,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        for amount in [f32::NAN, f32::INFINITY, 1024.01, -1024.01] {
            assert_eq!(
                ItemAttributeModifier::new(
                    "minecraft:generic.attack_damage",
                    "attack_bonus",
                    amount,
                    AttributeModifierOperation::AddValue,
                    ItemAttributeSlot::Any,
                )
                .unwrap_err()
                .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            ItemAttributeModifier::new(
                "minecraft:generic.attack_damage",
                "../attack_bonus",
                1.0,
                AttributeModifierOperation::AddValue,
                ItemAttributeSlot::Any,
            )
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_max_damage_is_bounded_and_paths_validate_before_import() {
        assert_eq!(MAX_ITEM_MAX_DAMAGE, 1_000_000);
        assert_eq!(
            set_item_max_damage("tool", 1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        assert_eq!(
            set_item_max_damage("tool", MAX_ITEM_MAX_DAMAGE)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_max_damage("tool", 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_max_damage("tool", MAX_ITEM_MAX_DAMAGE + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_max_damage("../tool", 20).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_max_stack_size_is_bounded_and_paths_validate_before_import() {
        assert_eq!(MAX_ITEM_MAX_STACK_SIZE, 99);
        assert_eq!(
            set_item_max_stack_size("bundle", 1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        assert_eq!(
            set_item_max_stack_size("bundle", MAX_ITEM_MAX_STACK_SIZE)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set_item_max_stack_size("bundle", 0).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_max_stack_size("bundle", MAX_ITEM_MAX_STACK_SIZE + 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_max_stack_size("../bundle", 16).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_use_behavior_is_bounded_and_paths_validate_before_import() {
        assert_eq!(MAX_ITEM_USE_DURATION_TICKS, 72_000);
        assert_eq!(
            [
                ItemUseAnimation::None as i32,
                ItemUseAnimation::Eat as i32,
                ItemUseAnimation::Drink as i32,
                ItemUseAnimation::Block as i32,
                ItemUseAnimation::Bow as i32,
                ItemUseAnimation::Spear as i32,
                ItemUseAnimation::Crossbow as i32,
                ItemUseAnimation::Spyglass as i32,
                ItemUseAnimation::TootHorn as i32,
                ItemUseAnimation::Brush as i32,
            ],
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        let minimum = ItemUseBehavior::new(ItemUseAnimation::Eat, 1).unwrap();
        assert_eq!(minimum.animation(), ItemUseAnimation::Eat);
        assert_eq!(minimum.duration_ticks(), 1);
        let maximum =
            ItemUseBehavior::new(ItemUseAnimation::Brush, MAX_ITEM_USE_DURATION_TICKS).unwrap();
        assert_eq!(maximum.duration_ticks(), MAX_ITEM_USE_DURATION_TICKS);
        assert_eq!(
            set_item_use_behavior("canteen", minimum)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        for invalid_duration in [0, MAX_ITEM_USE_DURATION_TICKS + 1] {
            assert_eq!(
                ItemUseBehavior::new(ItemUseAnimation::Drink, invalid_duration)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set_item_use_behavior("../canteen", maximum)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_crafting_remainder_requires_a_canonical_item_id_and_valid_item_path() {
        assert_eq!(MAX_ITEM_CRAFTING_REMAINDER_ID_BYTES, 256);
        assert_eq!(
            set_item_crafting_remainder("portable_container", "minecraft:bucket")
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        for invalid_id in [
            "bucket",
            "Minecraft:bucket",
            "minecraft:Bucket",
            "minecraft:bucket/../milk_bucket",
            "minecraft:",
        ] {
            assert_eq!(
                set_item_crafting_remainder("portable_container", invalid_id)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
                "invalid registry IDs must be rejected before the host import"
            );
        }
        assert_eq!(
            set_item_crafting_remainder("../container", "minecraft:bucket")
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let too_long_id = format!(
            "minecraft:{}",
            "a".repeat(MAX_ITEM_CRAFTING_REMAINDER_ID_BYTES)
        );
        assert_eq!(
            set_item_crafting_remainder("container", &too_long_id)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_food_values_and_flags_validate_before_host_access() {
        let food = ItemFood::new(6, 0.6).unwrap().fast().always_eat();
        assert_eq!(food.nutrition(), 6);
        assert_eq!(food.saturation_modifier(), 0.6);
        assert!(food.is_fast());
        assert!(food.can_always_eat());
        assert_eq!(
            ItemFood::new(0, 0.6).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ItemFood::new(21, 0.6).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ItemFood::new(6, f32::NAN).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ItemFood::new(6, -0.1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            ItemFood::new(6, 10.1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            set_item_food("ruby", food).unwrap_err().code(),
            ErrorCode::UnknownErrorCode,
            "valid native calls reach the unsupported-target result"
        );
        assert_eq!(
            set_item_food("../ruby", food).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn item_food_effects_validate_registry_ids_and_vanilla_bounds() {
        let effect = ItemFoodEffect::new("minecraft:poison", 72_000, 255, 1.0).unwrap();
        assert_eq!(effect.effect_id(), "minecraft:poison");
        assert_eq!(effect.duration_ticks(), MAX_ITEM_FOOD_EFFECT_DURATION_TICKS);
        assert_eq!(effect.amplifier(), 255);
        assert_eq!(effect.probability(), 1.0);
        assert!(ItemFoodEffect::new("modid:effect/path", 1, 0, 0.0).is_ok());

        for invalid_id in [
            "poison",
            "Minecraft:poison",
            "minecraft:",
            "minecraft:../poison",
        ] {
            assert_eq!(
                ItemFoodEffect::new(invalid_id, 1, 0, 1.0)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument,
                "effect IDs must be canonical and fully qualified: {invalid_id}"
            );
        }
        assert_eq!(
            ItemFoodEffect::new(format!("minecraft:{}", "x".repeat(248)), 1, 0, 1.0)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        for duration in [0, MAX_ITEM_FOOD_EFFECT_DURATION_TICKS + 1] {
            assert_eq!(
                ItemFoodEffect::new("minecraft:poison", duration, 0, 1.0)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for amplifier in [256, u32::MAX] {
            assert_eq!(
                ItemFoodEffect::new("minecraft:poison", 1, amplifier, 1.0)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
        for probability in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
            assert_eq!(
                ItemFoodEffect::new("minecraft:poison", 1, 0, probability)
                    .unwrap_err()
                    .code(),
                ErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn item_food_allows_at_most_eight_effects() {
        let mut effects = ItemFoodEffects::new();
        for _ in 0..MAX_ITEM_FOOD_EFFECTS {
            effects = effects
                .with_effect(ItemFoodEffect::new("minecraft:poison", 1, 0, 1.0).unwrap())
                .unwrap();
        }
        assert_eq!(
            effects
                .with_effect(ItemFoodEffect::new("minecraft:poison", 1, 0, 1.0).unwrap())
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
