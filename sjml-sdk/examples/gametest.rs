#![cfg_attr(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    allow(dead_code)
)]

use sjml::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

#[path = "gametest/block_fertilize_attempt.rs"]
mod block_fertilize_attempt;
#[path = "gametest/block_grow_attempt.rs"]
mod block_grow_attempt;
#[path = "gametest/block_inventory_custom_name.rs"]
mod block_inventory_custom_name;
#[path = "gametest/block_inventory_move.rs"]
mod block_inventory_move;
#[path = "gametest/block_use.rs"]
mod block_use;
#[path = "gametest/entity_ai_control.rs"]
mod entity_ai_control;
#[path = "gametest/entity_breed_attempt.rs"]
mod entity_breed_attempt;
#[path = "gametest/entity_effect_application.rs"]
mod entity_effect_application;
#[path = "gametest/entity_heal_attempt.rs"]
mod entity_heal_attempt;
#[path = "gametest/entity_join_attempt.rs"]
mod entity_join_attempt;
#[path = "gametest/entity_riding.rs"]
mod entity_riding;
#[path = "gametest/entity_tick.rs"]
mod entity_tick;
#[path = "gametest/gameplay_events.rs"]
mod gameplay_events;
#[path = "gametest/player_advancements.rs"]
mod player_advancements;
#[path = "gametest/player_fished_attempt.rs"]
mod player_fished_attempt;
#[path = "gametest/player_inventory_custom_name.rs"]
mod player_inventory_custom_name;
#[path = "gametest/player_inventory_enchantment.rs"]
mod player_inventory_enchantment;
#[path = "gametest/player_item_use_attempt.rs"]
mod player_item_use_attempt;
#[path = "gametest/player_respawn_point.rs"]
mod player_respawn_point;
#[path = "gametest/player_statistics.rs"]
mod player_statistics;
#[path = "gametest/player_tick.rs"]
mod player_tick;
#[path = "gametest/scoreboard.rs"]
mod scoreboard;
#[path = "gametest/scoreboard_teams.rs"]
mod scoreboard_teams;
#[path = "gametest/world_block_tag.rs"]
mod world_block_tag;
#[path = "gametest/world_difficulty.rs"]
mod world_difficulty;
#[path = "gametest/world_fluid_state.rs"]
mod world_fluid_state;
#[path = "gametest/world_item_tag.rs"]
mod world_item_tag;
#[path = "gametest/world_light_at.rs"]
mod world_light_at;
#[path = "gametest/world_neighbor_update.rs"]
mod world_neighbor_update;
#[path = "gametest/world_redstone_signal.rs"]
mod world_redstone_signal;
#[path = "gametest/world_spawn_point.rs"]
mod world_spawn_point;
#[path = "gametest/world_tag_members.rs"]
mod world_tag_members;

static SAW_PIG_DEATH: AtomicBool = AtomicBool::new(false);
static SAW_PIG_DEATH_ATTEMPT: AtomicBool = AtomicBool::new(false);
static ARM_ENTITY_TARGET_ATTEMPT: AtomicBool = AtomicBool::new(false);
static SAW_ENTITY_TARGET_ATTEMPT: AtomicBool = AtomicBool::new(false);
static SAW_PROJECTILE_IMPACT_ATTEMPT: AtomicBool = AtomicBool::new(false);
static SAW_EFFECT_TICK: AtomicBool = AtomicBool::new(false);
static SAW_BLOCK_PLACE: AtomicBool = AtomicBool::new(false);
static SAW_BLOCK_PLACE_VETO: AtomicBool = AtomicBool::new(false);
static SAW_ENTITY_SPAWNED: AtomicBool = AtomicBool::new(false);
static SAW_PLAYER_INTERACTED: AtomicBool = AtomicBool::new(false);
static SAW_PLAYER_INTERACTION_VETO: AtomicBool = AtomicBool::new(false);
static SAW_ENTITY_DAMAGED: AtomicBool = AtomicBool::new(false);
static SAW_ENTITY_DAMAGE_ATTEMPT: AtomicBool = AtomicBool::new(false);
static SAW_WORLD_LOADED: AtomicBool = AtomicBool::new(false);
static SAW_SERVER_TICK: AtomicBool = AtomicBool::new(false);
static SAW_INVENTORY_PICKUP: AtomicBool = AtomicBool::new(false);
static SAW_MENU_ACTION: AtomicBool = AtomicBool::new(false);
static SAW_PLAYER_DROP_ATTEMPT: AtomicBool = AtomicBool::new(false);

#[mod_register]
fn register_test_content() -> SjmlResult<()> {
    registry::register_block("gametest_block", BlockDefinition::default())?;
    registry::register_block_entity("gametest_machine", "gametest_block")?;
    registry::register_block_with_shape(
        "gametest_shaped_block",
        BlockDefinition::default(),
        BlockShape::cuboid([0.25, 0.0, 0.25], [0.75, 0.5, 0.75])?,
    )?;
    registry::register_block_with_behavior(
        "gametest_behavior_block",
        BlockDefinition::default(),
        BlockBehavior::new()
            .friction(0.91)?
            .speed_factor(0.4)?
            .jump_factor(1.5)?
            .light_level(13)?
            .ignited_by_lava()
            .push_reaction(BlockPushReaction::PushOnly)
            .sound(BlockSound::Amethyst),
    )?;
    registry::register_block("gametest_interactive_block", BlockDefinition::default())?;
    let mut block_model = BlockEntityModelDefinition::new();
    block_model.add_cuboid([0.125, 0.0, 0.125], [0.875, 0.75, 0.875], 0xFF20_C0FF)?;
    block_model.set_texture("sjml_game_test:textures/block/gametest_model.png")?;
    block_model.set_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::X,
        [0.5, 0.375, 0.5],
        12.0,
        60,
        15,
    )?)?;
    block_model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Y,
        [0.5, 0.375, 0.5],
        18.0,
        80,
        0,
    )?)?;
    block_model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Z,
        [0.5, 0.375, 0.5],
        6.0,
        40,
        10,
    )?)?;
    block_model.add_cuboid_animation(
        0,
        ModelAnimation::sine_rotation(ModelAnimationAxis::X, [0.5, 0.375, 0.5], 10.0, 60, 15)?,
    )?;
    block_model.add_cuboid_animation(
        0,
        ModelAnimation::sine_rotation(ModelAnimationAxis::Y, [0.5, 0.375, 0.5], 8.0, 80, 20)?,
    )?;
    block_model.add_cuboid_animation(
        0,
        ModelAnimation::sine_rotation(ModelAnimationAxis::Z, [0.5, 0.375, 0.5], 4.0, 100, 25)?,
    )?;
    registry::register_block_entity_model("gametest_machine", block_model)?;
    registry::register_item("gametest_ruby", ItemDefinition::default())?;
    registry::register_item("gametest_epic_rarity", ItemDefinition::default())?;
    registry::set_item_rarity("gametest_epic_rarity", ItemRarity::Epic)?;
    registry::register_item("gametest_enchantable", ItemDefinition::default())?;
    registry::set_item_enchantment_value("gametest_enchantable", 31)?;
    registry::register_item(
        "gametest_attribute_item",
        ItemDefinition {
            max_stack_size: 1,
            fire_resistant: false,
        },
    )?;
    registry::set_item_attribute_modifier(
        "gametest_attribute_item",
        &ItemAttributeModifier::new(
            "minecraft:generic.attack_damage",
            "attack_bonus",
            3.5,
            AttributeModifierOperation::AddValue,
            ItemAttributeSlot::Mainhand,
        )?,
    )?;
    registry::register_item("gametest_food", ItemDefinition::default())?;
    let food_effects = registry::ItemFoodEffects::new().with_effect(
        registry::ItemFoodEffect::new("minecraft:poison", 1200, 1, 1.0)?,
    )?;
    registry::set_item_food_with_effects(
        "gametest_food",
        ItemFood::new(6, 0.6)?.fast().always_eat(),
        food_effects,
    )?;
    registry::register_item(
        "gametest_durable",
        ItemDefinition {
            max_stack_size: 1,
            fire_resistant: false,
        },
    )?;
    registry::set_item_max_damage("gametest_durable", 256)?;
    registry::register_integer_component(
        "player_charge",
        IntegerComponentDefinition::new(-10, 10)?,
    )?;
    registry::register_boolean_component("player_enabled")?;
    registry::register_entity("gametest_ai", EntityDefinition::default())?;
    let mut entity_model = EntityModelDefinition::new();
    entity_model.add_cuboid([-0.25, 0.0, -0.25], [0.25, 0.5, 0.25], 0xFFFF_2080)?;
    entity_model.add_cuboid([-0.125, 0.5, -0.125], [0.125, 0.75, 0.125], 0xFF80_FF80)?;
    entity_model.set_texture("sjml_game_test:textures/entity/gametest_model.png")?;
    entity_model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Y,
        [0.0, 0.25, 0.0],
        30.0,
        40,
        0,
    )?)?;
    entity_model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::X,
        [0.0, 0.25, 0.0],
        12.0,
        60,
        15,
    )?)?;
    entity_model.add_animation(ModelAnimation::sine_rotation(
        ModelAnimationAxis::Z,
        [0.0, 0.25, 0.0],
        8.0,
        80,
        20,
    )?)?;
    entity_model.add_cuboid_animation(
        0,
        ModelAnimation::sine_rotation(ModelAnimationAxis::X, [0.0, 0.25, 0.0], 6.0, 40, 0)?,
    )?;
    entity_model.add_cuboid_animation(
        0,
        ModelAnimation::sine_rotation(ModelAnimationAxis::Y, [0.0, 0.25, 0.0], 6.0, 40, 0)?,
    )?;
    entity_model.add_cuboid_animation(
        0,
        ModelAnimation::sine_rotation(ModelAnimationAxis::Z, [0.0, 0.25, 0.0], 6.0, 40, 0)?,
    )?;
    registry::register_entity_model("gametest_ai", entity_model)?;
    let keyframe_track = ModelKeyframeTrack::new(
        ModelAnimationAxis::X,
        [0.0, 0.625, 0.0],
        40,
        vec![ModelKeyframe::new(0, 0.0)?, ModelKeyframe::new(20, 35.0)?],
    )?;
    model_animation::register_entity_model_keyframe_track(
        "gametest_ai",
        ModelAnimationTarget::Cuboid(1),
        keyframe_track,
    )?;
    registry::register_entity_ai_goal("gametest_ai", EntityAiGoal::Swim { priority: 0 })?;
    registry::register_entity_ai_goal(
        "gametest_ai",
        EntityAiGoal::RandomLookAround { priority: 7 },
    )?;
    registry::register_entity_ai_goal(
        "gametest_ai",
        EntityAiGoal::TargetPlayer {
            priority: 1,
            must_see: true,
        },
    )?;
    registry::register_fluid("gametest_fluid", FluidDefinition::default())?;
    registry::register_particle("gametest_particle")?;
    registry::register_sound("gametest_sound")?;
    registry::register_attribute_effect(
        "gametest_tick",
        EffectCategory::Beneficial,
        0x40D080,
        EntityAttribute::MaxHealth,
        1.0,
        AttributeModifierOperation::AddValue,
    )?;
    registry::register_effect_tick("gametest_tick", 1)?;
    registry::register_custom_registry("gametest_types", true)?;
    registry::register_custom_value(
        "gametest_types",
        "ruby",
        "{\"hardness\":3,\"variant\":\"ruby\"}",
    )?;
    registry::register_recipe_serializer("gametest_threshold")?;
    registry::register_shapeless_recipe(
        "gametest_block",
        ShapelessRecipeDefinition {
            output_item: "gametest_block".into(),
            output_count: 1,
            ingredients: vec!["gametest_ruby".into(); 9],
        },
    )?;
    registry::register_shaped_recipe(
        "gametest_ruby_block",
        ShapedRecipeDefinition {
            output_item: "gametest_block".into(),
            output_count: 1,
            pattern: vec!["RR".into(), "RR".into()],
            key: vec![ShapedRecipeKey {
                symbol: 'R',
                ingredient: RecipeIngredient::tag("minecraft:planks"),
            }],
        },
    )?;
    registry::register_stonecutting_recipe(
        "gametest_stonecutting",
        registry::StonecuttingRecipeDefinition {
            ingredient: RecipeIngredient::item("minecraft:stone"),
            output_item: "gametest_ruby".into(),
            output_count: 2,
        },
    )?;
    registry::register_smithing_transform_recipe(
        "gametest_smithing_transform",
        SmithingTransformRecipeDefinition {
            template: RecipeIngredient::item("minecraft:netherite_upgrade_smithing_template"),
            base: RecipeIngredient::item("minecraft:diamond_sword"),
            addition: RecipeIngredient::item("minecraft:netherite_ingot"),
            output_item: "minecraft:netherite_sword".into(),
            output_count: 1,
        },
    )?;
    registry::register_brewing_recipe(
        "gametest_brewing",
        "minecraft:water",
        "minecraft:emerald",
        "sjml_game_test:content/gametest_potion",
    )?;
    registry::register_potion(
        "gametest_potion",
        "sjml_game_test:content/gametest_tick",
        1200,
        1,
    )?;
    for (path, kind) in [
        ("gametest_smelting", CookingRecipeKind::Smelting),
        ("gametest_blasting", CookingRecipeKind::Blasting),
        ("gametest_smoking", CookingRecipeKind::Smoking),
        ("gametest_campfire", CookingRecipeKind::CampfireCooking),
    ] {
        registry::register_cooking_recipe(
            path,
            CookingRecipeDefinition {
                kind,
                ingredient: RecipeIngredient::tag("minecraft:iron_ores"),
                output_item: "gametest_ruby".into(),
                output_count: 1,
                experience: 0.7,
                cooking_time: 200,
            },
        )?;
    }
    registry::register_villager_trade(
        "gametest_librarian_trade",
        VillagerTradeOffer::new(
            "minecraft:librarian",
            1,
            VillagerTradeItem::new("minecraft:emerald", 2)?,
            VillagerTradeItem::new("minecraft:diamond", 1)?,
        )?,
    )?;
    Ok(())
}

#[recipe(name = "gametest_threshold")]
fn evaluate_threshold_recipe(input: RecipeMatchInput) -> SjmlResult<RecipeEvaluation> {
    if input.payload == "minimum_ruby=2"
        && input
            .slots
            .iter()
            .flatten()
            .any(|slot| slot.item_id == "sjml_game_test:content/gametest_ruby" && slot.count >= 2)
    {
        RecipeEvaluation::matched(2)
    } else {
        Ok(RecipeEvaluation::no_match())
    }
}

#[mod_init]
fn subscribe_to_server_events() -> SjmlResult<()> {
    // Dedicated activation runs before server binding; these must be safe early-init reads.
    let _ = world::get("minecraft:overworld")?;
    let _ = world::loaded()?;
    let _ = player::online()?;
    events::subscribe("block.break", EventPriority::High)?;
    events::subscribe("block.break_attempt", EventPriority::High)?;
    events::subscribe("entity.death", EventPriority::Normal)?;
    events::subscribe("entity.death_attempt", EventPriority::Normal)?;
    events::subscribe("entity.target_attempt", EventPriority::Normal)?;
    events::subscribe("entity.projectile_impact_attempt", EventPriority::Normal)?;
    events::subscribe("entity.spawned", EventPriority::Normal)?;
    events::subscribe("entity.effect.tick", EventPriority::Normal)?;
    events::subscribe("block.place", EventPriority::Normal)?;
    events::subscribe("block.place_attempt", EventPriority::Normal)?;
    events::subscribe("player.interacted", EventPriority::Normal)?;
    events::subscribe("player.interaction", EventPriority::Normal)?;
    events::subscribe("player.drop_attempt", EventPriority::Normal)?;
    events::subscribe("entity.damaged", EventPriority::Normal)?;
    events::subscribe("entity.damage_attempt", EventPriority::Normal)?;
    events::subscribe("world.loaded", EventPriority::Normal)?;
    events::subscribe("world.unloaded", EventPriority::Normal)?;
    events::subscribe_with_interval("server.tick", EventPriority::Normal, 20)?;
    events::subscribe("inventory.changed", EventPriority::Normal)?;
    events::subscribe("resources.reloaded", EventPriority::Normal)?;
    events::subscribe("menu.action", EventPriority::Normal)?;
    gameplay_events::initialize()?;
    player_tick::initialize()?;
    player_item_use_attempt::initialize()?;
    block_use::initialize()?;
    player_fished_attempt::initialize()?;
    entity_heal_attempt::initialize()?;
    entity_effect_application::initialize()?;
    entity_join_attempt::initialize()?;
    entity_breed_attempt::initialize()?;
    block_grow_attempt::initialize()?;
    block_fertilize_attempt::initialize()?;
    entity_tick::initialize()?;
    Ok(())
}

#[event(name = "block.break")]
fn veto_test_block_break(event: BlockBreak) -> SjmlResult<bool> {
    Ok(event.block_id == "sjml_game_test:content/gametest_block")
}

#[event(name = "block.break_attempt")]
fn veto_diamond_block_break_attempt(event: BlockBreakAttempt) -> SjmlResult<bool> {
    Ok(event.block_id == "minecraft:diamond_block")
}

#[event(name = "entity.death")]
fn observe_pig_death(event: EntityDeath) -> SjmlResult<()> {
    if event.entity_type_id == "minecraft:pig" {
        SAW_PIG_DEATH.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "entity.death_attempt")]
fn veto_cow_death_attempt(event: EntityDeathAttempt) -> SjmlResult<bool> {
    // Keep this test entity separate from the pig used by the notification test:
    // a successful veto correctly suppresses the later entity.death notification.
    let veto = event.entity_type_id == "minecraft:cow";
    if veto {
        SAW_PIG_DEATH_ATTEMPT.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "entity.target_attempt")]
fn veto_armed_entity_target_attempt(event: EntityTargetAttempt) -> SjmlResult<bool> {
    let veto = event.target_change_type.ends_with("#MOB_TARGET")
        && event.requested_target_id.is_some()
        && ARM_ENTITY_TARGET_ATTEMPT.swap(false, Ordering::Relaxed);
    if veto {
        SAW_ENTITY_TARGET_ATTEMPT.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "entity.projectile_impact_attempt")]
fn cancel_snowball_impact(event: ProjectileImpactAttempt) -> SjmlResult<bool> {
    let veto = event.projectile_type_id == "minecraft:snowball" && event.hit_kind == "block";
    if veto {
        SAW_PROJECTILE_IMPACT_ATTEMPT.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "entity.damage_attempt")]
fn veto_goat_damage_attempt(event: EntityDamageAttempt) -> SjmlResult<bool> {
    let veto = event.entity_type_id == "minecraft:goat"
        && event.damage_type_id == "minecraft:generic"
        && event.original_damage >= 4.0
        && event.amount >= 4.0;
    if veto {
        SAW_ENTITY_DAMAGE_ATTEMPT.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "block.place")]
fn observe_block_place(event: BlockPlace) -> SjmlResult<()> {
    if event.block_id == "minecraft:diamond_block" && event.placer_id.is_some() {
        SAW_BLOCK_PLACE.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "block.place_attempt")]
fn veto_gold_block_placement(event: BlockPlaceAttempt) -> SjmlResult<bool> {
    let veto = event.block_id == "minecraft:gold_block";
    if veto {
        SAW_BLOCK_PLACE_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "entity.spawned")]
fn observe_entity_spawned(event: EntitySpawned) -> SjmlResult<()> {
    if event.entity_type_id == "minecraft:pig" && !event.loaded_from_disk {
        SAW_ENTITY_SPAWNED.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "entity.effect.tick")]
fn observe_effect_tick(event: EffectTickEvent) -> SjmlResult<()> {
    if event.effect_id == "sjml_game_test:content/gametest_tick"
        && event.tick_interval == 1
        && event.entity_type_id == "minecraft:pig"
    {
        SAW_EFFECT_TICK.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "menu.action")]
fn observe_menu_action(event: MenuAction) -> SjmlResult<()> {
    event.validate()?;
    if event.slot_index == 4 && event.action_id == "claim" {
        SAW_MENU_ACTION.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "player.interacted")]
fn observe_player_interaction(event: PlayerInteracted) -> SjmlResult<()> {
    if event.action == "right_click_block"
        && event.hand == "main_hand"
        && event.target_id.as_deref() == Some("minecraft:stone")
        && event.block_position.is_some()
    {
        SAW_PLAYER_INTERACTED.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "player.interaction")]
fn veto_chest_open_interaction(event: PlayerInteractionAttempt) -> SjmlResult<bool> {
    let veto = event.action == "right_click_block"
        && event.target_id.as_deref() == Some("minecraft:chest");
    if veto {
        SAW_PLAYER_INTERACTION_VETO.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "player.drop_attempt")]
fn veto_diamond_drop_attempt(event: PlayerDropAttempt) -> SjmlResult<bool> {
    let veto = event.item_id == "minecraft:diamond" && event.can_cancel;
    if veto {
        SAW_PLAYER_DROP_ATTEMPT.store(true, Ordering::Relaxed);
    }
    Ok(veto)
}

#[event(name = "entity.damaged")]
fn observe_entity_damage(event: EntityDamaged) -> SjmlResult<()> {
    if event.entity_type_id == "minecraft:pig"
        && event.applied_damage > 0.0
        && event.original_damage >= event.applied_damage
    {
        SAW_ENTITY_DAMAGED.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "world.loaded")]
fn observe_world_loaded(event: WorldLifecycle) -> SjmlResult<()> {
    if event.dimension_id == "minecraft:overworld" {
        SAW_WORLD_LOADED.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "world.unloaded")]
fn observe_world_unloaded(_: WorldLifecycle) -> SjmlResult<()> {
    Ok(())
}

#[event(name = "server.tick")]
fn observe_server_tick(event: ServerTick) -> SjmlResult<()> {
    if event.tick_count >= 0 {
        SAW_SERVER_TICK.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "inventory.changed")]
fn observe_inventory_changed(event: InventoryChanged) -> SjmlResult<()> {
    if event.change_type == "picked_up"
        && event.item_id == "minecraft:diamond"
        && event.count == 2
        && event.slot.is_none()
    {
        SAW_INVENTORY_PICKUP.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[event(name = "resources.reloaded")]
fn mark_successful_resource_reload(event: ResourcesReloaded) -> SjmlResult<()> {
    if event.tick_count < 0 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "resources.reloaded tick count must be non-negative",
        ));
    }
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    world::set_block(
        world,
        BlockPos { x: 0, y: 64, z: 0 },
        "minecraft:gold_block",
    )
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityDeathCheckArguments {}

#[command(name = "entity.death.check")]
fn check_pig_death(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_PIG_DEATH.swap(false, Ordering::Relaxed) {
        Ok("living entity death callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.death callback did not observe a pig",
        ))
    }
}

#[command(name = "entity.death_attempt.check")]
fn check_cow_death_attempt(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_PIG_DEATH_ATTEMPT.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed the cow death attempt".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.death_attempt callback did not veto the cow",
        ))
    }
}

#[command(name = "entity.target_attempt.arm")]
fn arm_entity_target_attempt(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    SAW_ENTITY_TARGET_ATTEMPT.store(false, Ordering::Relaxed);
    ARM_ENTITY_TARGET_ATTEMPT.store(true, Ordering::Relaxed);
    Ok("armed entity target veto".to_owned())
}

#[command(name = "entity.target_attempt.check")]
fn check_entity_target_attempt(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_ENTITY_TARGET_ATTEMPT.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed the entity target attempt".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.target_attempt callback did not veto the target change",
        ))
    }
}

#[command(name = "entity.projectile_impact.check")]
fn check_projectile_impact_attempt(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_PROJECTILE_IMPACT_ATTEMPT.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed a snowball block impact".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.projectile_impact_attempt callback did not veto the snowball",
        ))
    }
}

#[command(name = "entity.damage_attempt.check")]
fn check_entity_damage_attempt(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_ENTITY_DAMAGE_ATTEMPT.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed the goat damage attempt".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.damage_attempt callback did not veto the goat",
        ))
    }
}

#[command(name = "block.place.check")]
fn check_block_place(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_BLOCK_PLACE.swap(false, Ordering::Relaxed) {
        Ok("block.place callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block.place callback did not observe a placed diamond block",
        ))
    }
}

#[command(name = "block.place_attempt.check")]
fn check_block_place_veto(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_BLOCK_PLACE_VETO.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed gold-block placement".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block.place_attempt callback did not veto the gold block",
        ))
    }
}

#[command(name = "entity.spawned.check")]
fn check_entity_spawned(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_ENTITY_SPAWNED.swap(false, Ordering::Relaxed) {
        Ok("entity.spawned callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.spawned callback did not observe a fresh pig",
        ))
    }
}

#[command(name = "entity.effect.tick.check")]
fn check_effect_tick(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_EFFECT_TICK.swap(false, Ordering::Relaxed) {
        Ok("registered status effect tick callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.effect.tick callback did not observe the registered effect",
        ))
    }
}

#[command(name = "player.interacted.check")]
fn check_player_interacted(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_PLAYER_INTERACTED.swap(false, Ordering::Relaxed) {
        Ok("player.interacted callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player.interacted callback did not observe a stone block interaction",
        ))
    }
}

#[command(name = "player.interaction.check")]
fn check_player_interaction_veto(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_PLAYER_INTERACTION_VETO.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed the chest interaction".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player.interaction callback did not veto the chest action",
        ))
    }
}

#[command(name = "player.drop_attempt.check")]
fn check_player_drop_attempt(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_PLAYER_DROP_ATTEMPT.swap(false, Ordering::Relaxed) {
        Ok("guest vetoed the diamond drop and the stack was restored".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player.drop_attempt callback did not veto the diamond drop",
        ))
    }
}

#[command(name = "entity.damaged.check")]
fn check_entity_damaged(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_ENTITY_DAMAGED.swap(false, Ordering::Relaxed) {
        Ok("entity.damaged callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity.damaged callback did not observe applied pig damage",
        ))
    }
}

#[command(name = "world.loaded.check")]
fn check_world_loaded(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_WORLD_LOADED.load(Ordering::Relaxed) {
        Ok("world.loaded callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world.loaded callback did not observe the overworld",
        ))
    }
}

#[command(name = "server.tick.check")]
fn check_server_tick(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_SERVER_TICK.swap(false, Ordering::Relaxed) {
        Ok("scheduled server.tick callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "server.tick callback was not delivered at the subscribed interval",
        ))
    }
}

#[command(name = "inventory.changed.check")]
fn check_inventory_changed(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    if SAW_INVENTORY_PICKUP.swap(false, Ordering::Relaxed) {
        Ok("successful inventory pickup callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "inventory.changed callback did not observe the successful diamond pickup",
        ))
    }
}

#[command(name = "registry.custom.check")]
fn check_custom_registry(_: EntityDeathCheckArguments) -> SjmlResult<String> {
    match registry::custom_registry_value("gametest_types", "ruby")? {
        Some(value) if value == "{\"hardness\":3,\"variant\":\"ruby\"}" => {
            Ok("custom registry value is available after registry setup".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "custom registry did not return the registered value",
        )),
    }
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EchoArguments {
    message: String,
    repetitions: i32,
    uppercase: bool,
}

#[command(name = "chat.echo")]
fn echo(arguments: EchoArguments) -> SjmlResult<String> {
    if !(1..=4).contains(&arguments.repetitions) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "repetitions must be between 1 and 4",
        ));
    }
    let message = if arguments.uppercase {
        arguments.message.to_uppercase()
    } else {
        arguments.message
    };
    let repetitions = arguments.repetitions as usize;
    if message.len().saturating_mul(repetitions) > MAX_STRING_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "echo response exceeds the stable string limit",
        ));
    }
    Ok(message.repeat(repetitions))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct WorldArguments {}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct LoadedChunksArguments {
    x: i32,
    z: i32,
}

#[command(name = "world.loaded_chunks.check")]
fn check_loaded_chunks(arguments: LoadedChunksArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the test overworld must be loaded",
        )
    })?;
    let chunks = world::loaded_chunks(
        world,
        BlockPos {
            x: arguments.x,
            y: 0,
            z: arguments.z,
        },
        0,
    )?;
    let expected = ChunkPos {
        x: arguments.x.div_euclid(16),
        z: arguments.z.div_euclid(16),
    };
    if chunks.len() != 1 || chunks[0] != expected {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "radius-zero loaded-chunk query did not return exactly the loaded center chunk",
        ));
    }
    Ok("loaded chunk query returned the loaded center chunk without loading neighbors".to_owned())
}

#[command(name = "world.inspect")]
fn inspect_world(_: WorldArguments) -> SjmlResult<String> {
    let overworld = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    let loaded = world::loaded()?;
    let ids = loaded
        .iter()
        .copied()
        .map(world::dimension_id)
        .collect::<SjmlResult<Vec<_>>>()?;
    let original_day_time = world::day_time(overworld)?;
    let original_daylight_cycle = world::daylight_cycle_enabled(overworld)?;
    let original_border = world::border(overworld)?;
    let first_player = player::online()?.into_iter().next().ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest player must be online",
        )
    })?;
    let player_snapshot = player::snapshot(first_player)?;
    let player_world = world::get(&player_snapshot.dimension_id)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest player's world must be loaded",
        )
    })?;
    let player_position = BlockPos {
        x: player_snapshot.x.floor() as i32,
        y: player_snapshot.y.floor() as i32,
        z: player_snapshot.z.floor() as i32,
    };
    let biome_id = world::biome_at(player_world, player_position)?;
    let heightmap = world::heightmap(
        player_world,
        HeightmapType::MotionBlocking,
        player_position.x,
        player_position.z,
    )?;
    let light = world::light_at(player_world, player_position)?;
    let original_game_time = world::game_time(overworld)?;
    if !(0..=world::MAX_WORLD_GAME_TIME_TICKS).contains(&original_game_time) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld game time was outside the supported range",
        ));
    }
    let changed_game_time = if original_game_time > world::MAX_WORLD_GAME_TIME_TICKS - 1200 {
        original_game_time - 1200
    } else {
        original_game_time + 1200
    };
    world::set_game_time(overworld, changed_game_time)?;
    let observed_game_time = world::game_time(overworld);
    let restored_game_time = world::set_game_time(overworld, original_game_time);
    let observed_game_time = observed_game_time?;
    restored_game_time?;
    if !(0..=world::MAX_WORLD_DAY_TIME_TICKS).contains(&original_day_time) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld day time was outside the supported range",
        ));
    }
    let changed_day_time = if original_day_time > world::MAX_WORLD_DAY_TIME_TICKS - 1200 {
        original_day_time - 1200
    } else {
        original_day_time + 1200
    };
    world::set_time(overworld, changed_day_time)?;
    let observed_day_time = world::day_time(overworld);
    let restored_day_time = world::set_time(overworld, original_day_time);
    let observed_day_time = observed_day_time?;
    restored_day_time?;
    world::set_daylight_cycle_enabled(overworld, !original_daylight_cycle)?;
    let observed_daylight_cycle = world::daylight_cycle_enabled(overworld);
    let restored_daylight_cycle =
        world::set_daylight_cycle_enabled(overworld, original_daylight_cycle);
    let observed_daylight_cycle = observed_daylight_cycle?;
    restored_daylight_cycle?;
    let original_keep_inventory = world::game_rule(overworld, world::WorldGameRule::KeepInventory)?;
    let changed_keep_inventory = world::set_game_rule(
        overworld,
        world::WorldGameRule::KeepInventory,
        !original_keep_inventory,
    )
    .and_then(|()| world::game_rule(overworld, world::WorldGameRule::KeepInventory));
    let restored_keep_inventory = world::set_game_rule(
        overworld,
        world::WorldGameRule::KeepInventory,
        original_keep_inventory,
    );
    let changed_keep_inventory = changed_keep_inventory?;
    restored_keep_inventory?;
    let original_random_tick_speed =
        world::integer_game_rule(overworld, world::WorldIntegerGameRule::RandomTickSpeed)?;
    let changed_random_tick_speed = if original_random_tick_speed < 4096 {
        original_random_tick_speed + 1
    } else {
        original_random_tick_speed - 1
    };
    let observed_random_tick_speed = world::set_integer_game_rule(
        overworld,
        world::WorldIntegerGameRule::RandomTickSpeed,
        changed_random_tick_speed,
    )
    .and_then(|()| {
        world::integer_game_rule(overworld, world::WorldIntegerGameRule::RandomTickSpeed)
    });
    let restored_random_tick_speed = world::set_integer_game_rule(
        overworld,
        world::WorldIntegerGameRule::RandomTickSpeed,
        original_random_tick_speed,
    );
    let observed_random_tick_speed = observed_random_tick_speed?;
    restored_random_tick_speed?;
    let border_mutation_observed = if original_border.remaining_lerp_millis == 0 {
        let changed_center_x = if original_border.center_x > world::MAX_WORLD_BORDER_CENTER - 1.0 {
            original_border.center_x - 1.0
        } else {
            original_border.center_x + 1.0
        };
        let changed_diameter =
            if original_border.current_diameter < world::MAX_WORLD_BORDER_DIAMETER - 1.0 {
                original_border.current_diameter + 1.0
            } else {
                original_border.current_diameter - 1.0
            };
        let changed_warning_time = if original_border.warning_time_seconds
            < world::MAX_WORLD_BORDER_WARNING_SECONDS as i32
        {
            original_border.warning_time_seconds + 1
        } else {
            original_border.warning_time_seconds - 1
        };
        let changed_warning_distance = if original_border.warning_distance_blocks
            < world::MAX_WORLD_BORDER_WARNING_BLOCKS as i32
        {
            original_border.warning_distance_blocks + 1
        } else {
            original_border.warning_distance_blocks - 1
        };
        let changed_damage_buffer =
            if original_border.damage_buffer_blocks < world::MAX_WORLD_BORDER_DAMAGE_BUFFER {
                original_border.damage_buffer_blocks + 0.25
            } else {
                original_border.damage_buffer_blocks - 0.25
            };
        let changed_damage =
            if original_border.damage_per_block < world::MAX_WORLD_BORDER_DAMAGE_PER_BLOCK {
                original_border.damage_per_block + 0.25
            } else {
                original_border.damage_per_block - 0.25
            };
        let mutation: SjmlResult<bool> = (|| {
            world::set_border_center(overworld, changed_center_x, original_border.center_z)?;
            world::set_border_size(overworld, changed_diameter, 0)?;
            world::set_border_warnings(
                overworld,
                changed_warning_time as u32,
                changed_warning_distance as u32,
            )?;
            world::set_border_damage(overworld, changed_damage_buffer, changed_damage)?;
            let changed = world::border(overworld)?;
            Ok(changed.center_x == changed_center_x
                && changed.center_z == original_border.center_z
                && changed.current_diameter == changed_diameter
                && changed.target_diameter == changed_diameter
                && changed.remaining_lerp_millis == 0
                && changed.warning_time_seconds == changed_warning_time
                && changed.warning_distance_blocks == changed_warning_distance
                && changed.damage_buffer_blocks == changed_damage_buffer
                && changed.damage_per_block == changed_damage)
        })();
        let restore = world::set_border_center(
            overworld,
            original_border.center_x,
            original_border.center_z,
        )
        .and_then(|()| world::set_border_size(overworld, original_border.current_diameter, 0))
        .and_then(|()| {
            world::set_border_warnings(
                overworld,
                original_border.warning_time_seconds as u32,
                original_border.warning_distance_blocks as u32,
            )
        })
        .and_then(|()| {
            world::set_border_damage(
                overworld,
                original_border.damage_buffer_blocks,
                original_border.damage_per_block,
            )
        });
        let mutation_observed = mutation?;
        restore?;
        let restored = world::border(overworld)?;
        mutation_observed
            && restored.center_x == original_border.center_x
            && restored.center_z == original_border.center_z
            && restored.current_diameter == original_border.current_diameter
            && restored.warning_time_seconds == original_border.warning_time_seconds
            && restored.warning_distance_blocks == original_border.warning_distance_blocks
            && restored.damage_buffer_blocks == original_border.damage_buffer_blocks
            && restored.damage_per_block == original_border.damage_per_block
    } else {
        true
    };
    if !loaded.contains(&overworld)
        || world::get("minecraft:overworld")? != Some(overworld)
        || world::dimension_id(overworld)? != "minecraft:overworld"
        || !ids.windows(2).all(|pair| pair[0] < pair[1])
        || world::game_time(overworld)? < 0
        || observed_game_time != changed_game_time
        || world::game_time(overworld)? != original_game_time
        || observed_day_time != changed_day_time
        || observed_daylight_cycle == original_daylight_cycle
        || world::daylight_cycle_enabled(overworld)? != original_daylight_cycle
        || changed_keep_inventory != !original_keep_inventory
        || world::game_rule(overworld, world::WorldGameRule::KeepInventory)?
            != original_keep_inventory
        || observed_random_tick_speed != changed_random_tick_speed
        || world::integer_game_rule(overworld, world::WorldIntegerGameRule::RandomTickSpeed)?
            != original_random_tick_speed
        || !border_mutation_observed
        || biome_id
            .as_deref()
            .and_then(|id| id.split_once(':'))
            .is_none()
        || heightmap.is_none_or(|height| !(-64..=320).contains(&height))
        || light.is_none_or(|light| light.sky > 15 || light.block > 15)
        || world::get("sjml_game_test:unloaded")?.is_some()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world read contract failed",
        ));
    }
    Ok("loaded overworld read succeeded".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockBatchArguments {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "world.block.inspect")]
fn inspect_block_batch(arguments: BlockBatchArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    let position = BlockPos {
        x: arguments.x,
        y: arguments.y,
        z: arguments.z,
    };
    let states = world::block_states(
        world,
        &[
            position,
            BlockPos {
                x: arguments.x,
                y: i32::MIN,
                z: arguments.z,
            },
            position,
        ],
    )?;
    let properties = world::block_properties(world, position)?;
    let changed_properties = world::set_block_properties(
        world,
        position,
        "minecraft:oak_log",
        &[world::BlockProperty {
            name: "axis".to_owned(),
            value: "x".to_owned(),
        }],
    )
    .and_then(|()| world::block_properties(world, position));
    let restore_properties = world::set_block_properties(
        world,
        position,
        "minecraft:oak_log",
        &[world::BlockProperty {
            name: "axis".to_owned(),
            value: "y".to_owned(),
        }],
    );
    let changed_properties = changed_properties?;
    restore_properties?;
    let restored_properties = world::block_properties(world, position)?;
    if states.len() != 3
        || states[0]
            .as_ref()
            .is_none_or(|state| state.block_id != "minecraft:oak_log")
        || states[1].is_some()
        || states[0] != states[2]
        || properties.as_ref().is_none_or(|properties| {
            !properties
                .iter()
                .any(|property| property.name == "axis" && property.value == "y")
        })
        || changed_properties.as_ref().is_none_or(|properties| {
            !properties
                .iter()
                .any(|property| property.name == "axis" && property.value == "x")
        })
        || restored_properties.as_ref().is_none_or(|properties| {
            !properties
                .iter()
                .any(|property| property.name == "axis" && property.value == "y")
        })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "ordered block-state or property snapshot contract failed",
        ));
    }
    Ok("ordered block-state batch and property snapshot succeeded".to_owned())
}

#[command(name = "world.raycast.check")]
fn inspect_world_raycast(arguments: BlockBatchArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    let hit = world::raycast(
        world,
        RaycastPoint::new(
            f64::from(arguments.x) + 1.5,
            f64::from(arguments.y) + 0.5,
            f64::from(arguments.z) + 0.5,
        ),
        RaycastPoint::new(
            f64::from(arguments.x) + 0.5,
            f64::from(arguments.y) + 0.5,
            f64::from(arguments.z) + 0.5,
        ),
        RaycastBlockMode::Collider,
        RaycastFluidMode::None,
    )?
    .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "raycast missed the test block"))?;
    if hit.position
        != (BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        })
        || hit.block.block_id != "minecraft:stone"
        || !hit.location.x.is_finite()
        || !hit.location.y.is_finite()
        || !hit.location.z.is_finite()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world raycast returned the wrong block or hit point",
        ));
    }
    Ok("loaded block raycast returned its stone hit".to_owned())
}

#[command(name = "world.collision.check")]
fn inspect_world_collision(arguments: BlockBatchArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    let bounds_at = |x: i32| {
        world::Aabb::new(
            RaycastPoint::new(
                f64::from(x) + 0.1,
                f64::from(arguments.y) + 0.1,
                f64::from(arguments.z) + 0.1,
            ),
            RaycastPoint::new(
                f64::from(x) + 0.9,
                f64::from(arguments.y) + 0.9,
                f64::from(arguments.z) + 0.9,
            ),
        )
    };
    if world::collides(world, bounds_at(arguments.x))? != Some(true)
        || world::collides(world, bounds_at(arguments.x + 1))? != Some(false)
        || world::collides(world, bounds_at(arguments.x + 256))?.is_some()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "world collision query did not distinguish solid, empty, and unloaded bounds",
        ));
    }
    Ok("loaded block collision and unloaded-chunk query succeeded".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockMutationArguments {
    x: i32,
    y: i32,
    z: i32,
}

#[command(name = "world.block.set")]
fn set_test_block(arguments: BlockMutationArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    world::set_block(
        world,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        "minecraft:gold_block",
    )?;
    world::spawn_particle(
        world,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        "sjml_game_test:content/gametest_particle",
    )?;
    world::play_sound(
        world,
        BlockPos {
            x: arguments.x,
            y: arguments.y,
            z: arguments.z,
        },
        "sjml_game_test:content/gametest_sound",
        1.0,
        1.0,
    )?;
    Ok("loaded block update, particle, and sound succeeded".to_owned())
}

#[command(name = "world.entity.spawn")]
fn spawn_test_entity(arguments: BlockMutationArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the overworld must already be loaded",
        )
    })?;
    world::spawn_entity(
        world,
        "sjml_game_test:content/gametest_ai",
        RaycastPoint::new(
            f64::from(arguments.x) + 0.5,
            f64::from(arguments.y),
            f64::from(arguments.z) + 0.5,
        ),
    )?;
    Ok("registered guest entity spawned".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct PlayerArguments {
    player_id: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct MenuActionOpenArguments {
    player_id: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct MenuUpdateArguments {
    player_id: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct MenuActionCheckArguments {}

#[command(name = "menu.action.open")]
fn open_menu_action(arguments: MenuActionOpenArguments) -> SjmlResult<String> {
    let handle = player::online()?
        .into_iter()
        .find(|handle| {
            player::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.player_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "requested menu player is not online",
            )
        })?;
    let mut menu = MenuDefinition::new("Action test", 1)?;
    menu.slot(
        4,
        MenuItem::new("minecraft:emerald")?
            .label("Claim")?
            .action("claim")?,
    )?;
    menu.open(handle)?;
    Ok("action menu opened".to_owned())
}

#[command(name = "menu.update.check")]
fn update_test_menu(arguments: MenuUpdateArguments) -> SjmlResult<String> {
    let handle = player::online()?
        .into_iter()
        .find(|handle| {
            player::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.player_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "requested menu player is not online",
            )
        })?;
    let initial = MenuDefinition::new("Original test", 1)?;
    initial.open(handle)?;
    let mut replacement = MenuDefinition::new("Updated test", 2)?;
    replacement.slot(9, MenuItem::new("minecraft:diamond")?.count(2)?)?;
    replacement.update(handle)?;
    Ok("the module-owned menu was replaced".to_owned())
}

#[command(name = "menu.editable.check")]
fn exercise_editable_menu(arguments: MenuUpdateArguments) -> SjmlResult<String> {
    let handle = player::online()?
        .into_iter()
        .find(|handle| {
            player::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.player_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "requested editable-menu player is not online",
            )
        })?;

    let mut menu = sjml::menu::EditableMenuDefinition::new("Stone exchange", 1)?;
    menu.slot(sjml::menu::EditableMenuSlot::input(0, ["minecraft:stone"])?)?;
    menu.slot(sjml::menu::EditableMenuSlot::output(1)?)?;
    menu.open(handle)?;

    menu.transfer(
        handle,
        sjml::menu::EditableMenuEndpoint::PlayerInventory(0),
        sjml::menu::EditableMenuEndpoint::Menu(0),
        3,
    )?;
    if menu
        .read_slot(handle, 0)?
        .as_ref()
        .is_none_or(|stack| stack.item_id != "minecraft:stone" || stack.count != 3)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "editable input slot did not receive the requested existing items",
        ));
    }

    menu.transfer(
        handle,
        sjml::menu::EditableMenuEndpoint::Menu(0),
        sjml::menu::EditableMenuEndpoint::Menu(1),
        2,
    )?;
    if menu
        .read_slot(handle, 0)?
        .as_ref()
        .is_none_or(|stack| stack.count != 1)
        || menu
            .read_slot(handle, 1)?
            .as_ref()
            .is_none_or(|stack| stack.count != 2)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "editable menu transfer did not conserve the expected item counts",
        ));
    }

    menu.transfer(
        handle,
        sjml::menu::EditableMenuEndpoint::Menu(1),
        sjml::menu::EditableMenuEndpoint::PlayerInventory(2),
        1,
    )?;
    if menu
        .read_slot(handle, 1)?
        .as_ref()
        .is_none_or(|stack| stack.count != 1)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "editable menu did not transfer one existing output item back to inventory",
        ));
    }

    if menu
        .transfer(
            handle,
            sjml::menu::EditableMenuEndpoint::PlayerInventory(1),
            sjml::menu::EditableMenuEndpoint::Menu(0),
            1,
        )
        .is_ok()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "editable menu accepted an item outside the input allowlist",
        ));
    }

    Ok("editable menu transfers preserve item identity and count".to_owned())
}

#[command(name = "menu.action.check")]
fn check_menu_action(_: MenuActionCheckArguments) -> SjmlResult<String> {
    if SAW_MENU_ACTION.swap(false, Ordering::Relaxed) {
        Ok("menu.action callback delivered".to_owned())
    } else {
        Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "menu.action callback did not observe the clicked action slot",
        ))
    }
}

#[command(name = "player.inspect")]
fn inspect_players(arguments: PlayerArguments) -> SjmlResult<String> {
    let players = player::online()?;
    let mut target_player = None;
    let mut target_snapshot = None;
    for handle in players.iter().copied() {
        let snapshot = player::snapshot(handle)?;
        if snapshot.uuid == arguments.player_id {
            target_player = Some(handle);
            target_snapshot = Some(snapshot);
        }
    }
    let target_player = target_player.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest player must be online",
        )
    })?;
    let target_snapshot = target_snapshot.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest player snapshot must be available",
        )
    })?;
    let cooldown_item_id = "minecraft:diamond";
    if player::has_item_cooldown(target_player, cooldown_item_id)? {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player cooldown GameTest expects its fixture item to start without a cooldown",
        ));
    }
    let cooldown_set_and_read = player::set_item_cooldown(target_player, cooldown_item_id, 20)
        .and_then(|()| player::has_item_cooldown(target_player, cooldown_item_id));
    let cooldown_cleanup = player::clear_item_cooldown(target_player, cooldown_item_id);
    let cooldown_active = cooldown_set_and_read?;
    cooldown_cleanup?;
    let cooldown_cleared = player::has_item_cooldown(target_player, cooldown_item_id)?;
    player::send_action_bar(target_player, "SJML GameTest action-bar request")?;
    let original_vitals = player::vitals(target_player)?;
    let original_game_mode = player::game_mode(target_player)?;
    let equipment = player::equipment(target_player)?;
    if equipment
        .main_hand
        .as_ref()
        .map(|stack| stack.item_id.as_str())
        != Some("minecraft:diamond_sword")
        || equipment
            .off_hand
            .as_ref()
            .map(|stack| stack.item_id.as_str())
            != Some("minecraft:shield")
        || equipment.feet.as_ref().map(|stack| stack.item_id.as_str())
            != Some("minecraft:iron_boots")
        || equipment.legs.as_ref().map(|stack| stack.item_id.as_str())
            != Some("minecraft:iron_leggings")
        || equipment.chest.as_ref().map(|stack| stack.item_id.as_str())
            != Some("minecraft:iron_chestplate")
        || equipment.head.as_ref().map(|stack| stack.item_id.as_str())
            != Some("minecraft:iron_helmet")
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player equipment snapshot did not match the GameTest fixture",
        ));
    }
    let inventory = sjml::player_inventory::read(target_player)?;
    if inventory.slots.len() != 41
        || inventory.slots[0]
            .as_ref()
            .is_none_or(|stack| stack.item_id != "minecraft:diamond_sword")
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory snapshot did not match the GameTest hotbar fixture",
        ));
    }
    let original_hotbar_slot = player::selected_hotbar_slot(target_player)?;
    let temporary_hotbar_slot = (original_hotbar_slot + 1) % 9;
    let changed_hotbar_slot = (|| {
        player::set_selected_hotbar_slot(target_player, temporary_hotbar_slot)?;
        player::selected_hotbar_slot(target_player)
    })();
    let restore_hotbar_slot = player::set_selected_hotbar_slot(target_player, original_hotbar_slot);
    let changed_hotbar_slot = changed_hotbar_slot?;
    restore_hotbar_slot?;
    let restored_hotbar_slot = player::selected_hotbar_slot(target_player)?;
    let original_attack_damage = sjml::entity_attributes::read_player(
        target_player,
        sjml::entity_attributes::LivingAttribute::AttackDamage,
    )?;
    let temporary_attack_damage = if original_attack_damage.base_value < 1024.0 {
        original_attack_damage.base_value + 0.25
    } else {
        original_attack_damage.base_value - 0.25
    };
    let changed_attack_damage = (|| {
        sjml::entity_attributes::set_player_base_value(
            target_player,
            sjml::entity_attributes::LivingAttribute::AttackDamage,
            temporary_attack_damage,
        )?;
        sjml::entity_attributes::read_player(
            target_player,
            sjml::entity_attributes::LivingAttribute::AttackDamage,
        )
    })();
    let restore_attack_damage = sjml::entity_attributes::set_player_base_value(
        target_player,
        sjml::entity_attributes::LivingAttribute::AttackDamage,
        original_attack_damage.base_value,
    );
    let changed_attack_damage = changed_attack_damage?;
    restore_attack_damage?;
    let restored_attack_damage = sjml::entity_attributes::read_player(
        target_player,
        sjml::entity_attributes::LivingAttribute::AttackDamage,
    )?;
    let original_interaction_range = sjml::entity_attributes::read_player(
        target_player,
        sjml::entity_attributes::LivingAttribute::BlockInteractionRange,
    )?;
    let temporary_interaction_range = if original_interaction_range.base_value < 64.0 {
        original_interaction_range.base_value + 0.25
    } else {
        original_interaction_range.base_value - 0.25
    };
    let changed_interaction_range = (|| {
        sjml::entity_attributes::set_player_base_value(
            target_player,
            sjml::entity_attributes::LivingAttribute::BlockInteractionRange,
            temporary_interaction_range,
        )?;
        sjml::entity_attributes::read_player(
            target_player,
            sjml::entity_attributes::LivingAttribute::BlockInteractionRange,
        )
    })();
    let restore_interaction_range = sjml::entity_attributes::set_player_base_value(
        target_player,
        sjml::entity_attributes::LivingAttribute::BlockInteractionRange,
        original_interaction_range.base_value,
    );
    let changed_interaction_range = changed_interaction_range?;
    restore_interaction_range?;
    let restored_interaction_range = sjml::entity_attributes::read_player(
        target_player,
        sjml::entity_attributes::LivingAttribute::BlockInteractionRange,
    )?;
    let original_abilities = player::abilities(target_player)?;
    if !original_abilities.may_fly {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player abilities GameTest must grant may_fly before the guest callback",
        ));
    }
    let temporary_flying = !original_abilities.flying;
    let temporary_abilities = (|| {
        player::set_flying(target_player, temporary_flying)?;
        player::abilities(target_player)
    })();
    let restore_flying = player::set_flying(target_player, original_abilities.flying);
    let temporary_abilities = temporary_abilities?;
    restore_flying?;
    let restored_abilities = player::abilities(target_player)?;
    let temporary_game_mode = if original_game_mode == sjml::PlayerGameMode::Creative {
        sjml::PlayerGameMode::Adventure
    } else {
        sjml::PlayerGameMode::Creative
    };
    player::set_game_mode(target_player, temporary_game_mode)?;
    let observed_game_mode = player::game_mode(target_player);
    let restored_game_mode = player::set_game_mode(target_player, original_game_mode);
    let observed_game_mode = observed_game_mode?;
    restored_game_mode?;
    let original_motion = player::motion(target_player)?;
    let temporary_velocity = player::PlayerVelocity::new(0.125, -0.25, 0.5)?;
    let temporary_motion = (|| {
        player::set_velocity(target_player, temporary_velocity)?;
        player::motion(target_player)
    })();
    let restore_motion = player::set_velocity(
        target_player,
        player::PlayerVelocity::new(
            original_motion.velocity_x,
            original_motion.velocity_y,
            original_motion.velocity_z,
        )?,
    );
    let temporary_motion = temporary_motion?;
    restore_motion?;
    let restored_motion = player::motion(target_player)?;
    let temporary_health = if original_vitals.health < original_vitals.max_health {
        (original_vitals.health + 0.5).min(original_vitals.max_health)
    } else {
        (original_vitals.health - 0.5).max(0.0)
    };
    let temporary_food = if original_vitals.food_level == 20 {
        19
    } else {
        original_vitals.food_level + 1
    };
    let temporary_saturation = temporary_food as f32 * 0.25;
    let temporary_experience_level = if original_vitals.experience_level == 20_000 {
        19_999
    } else {
        original_vitals.experience_level + 1
    };
    let temporary_vitals = (|| {
        player::set_health(target_player, temporary_health)?;
        player::set_food(target_player, temporary_food, temporary_saturation)?;
        player::set_experience(target_player, temporary_experience_level, 0.0)?;
        player::vitals(target_player)
    })();
    let restore_vitals = (|| {
        player::set_health(target_player, original_vitals.health)?;
        player::set_food(
            target_player,
            original_vitals.food_level,
            original_vitals.saturation,
        )?;
        player::set_experience(
            target_player,
            original_vitals.experience_level,
            original_vitals.experience_progress,
        )
    })();
    let temporary_vitals = temporary_vitals?;
    restore_vitals?;
    let restored_vitals = player::vitals(target_player)?;
    let valid_snapshot = target_snapshot.uuid.len() == 36
        && target_snapshot.dimension_id.split_once(':').is_some()
        && target_snapshot.x.is_finite()
        && target_snapshot.y.is_finite()
        && target_snapshot.z.is_finite();
    let valid_original_vitals = original_vitals.max_health > 0.0
        && original_vitals.health <= original_vitals.max_health
        && original_vitals.food_level <= 20
        && (0.0..=20.0).contains(&original_vitals.saturation)
        && original_vitals.experience_progress <= 1.0;
    if players.is_empty()
        || !valid_snapshot
        || !valid_original_vitals
        || temporary_abilities.flying != temporary_flying
        || temporary_abilities.may_fly != original_abilities.may_fly
        || temporary_abilities.invulnerable != original_abilities.invulnerable
        || temporary_abilities.may_build != original_abilities.may_build
        || restored_abilities != original_abilities
        || (changed_attack_damage.base_value - temporary_attack_damage).abs() > 0.0001
        || (restored_attack_damage.base_value - original_attack_damage.base_value).abs() > 0.0001
        || (changed_interaction_range.base_value - temporary_interaction_range).abs() > 0.0001
        || (restored_interaction_range.base_value - original_interaction_range.base_value).abs()
            > 0.0001
        || observed_game_mode != temporary_game_mode
        || (temporary_motion.velocity_x - temporary_velocity.x).abs() > 0.0001
        || (temporary_motion.velocity_y - temporary_velocity.y).abs() > 0.0001
        || (temporary_motion.velocity_z - temporary_velocity.z).abs() > 0.0001
        || (restored_motion.velocity_x - original_motion.velocity_x).abs() > 0.0001
        || (restored_motion.velocity_y - original_motion.velocity_y).abs() > 0.0001
        || (restored_motion.velocity_z - original_motion.velocity_z).abs() > 0.0001
        || restored_motion.yaw != original_motion.yaw
        || restored_motion.pitch != original_motion.pitch
        || (temporary_vitals.health - temporary_health).abs() > 0.01
        || temporary_vitals.food_level != temporary_food
        || (temporary_vitals.saturation - temporary_saturation).abs() > 0.01
        || temporary_vitals.experience_level != temporary_experience_level
        || temporary_vitals.experience_progress != 0.0
        || !cooldown_active
        || cooldown_cleared
        || changed_hotbar_slot != temporary_hotbar_slot
        || restored_hotbar_slot != original_hotbar_slot
        || (restored_vitals.health - original_vitals.health).abs() > 0.01
        || restored_vitals.food_level != original_vitals.food_level
        || (restored_vitals.saturation - original_vitals.saturation).abs() > 0.01
        || restored_vitals.experience_level != original_vitals.experience_level
        || (restored_vitals.experience_progress - original_vitals.experience_progress).abs() > 0.02
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player read contract failed",
        ));
    }
    Ok(format!("read {} online player snapshot(s)", players.len()))
}

#[command(name = "player.teleport.dimension")]
fn teleport_player_to_nether(arguments: PlayerArguments) -> SjmlResult<String> {
    let target_player = player::online()?
        .into_iter()
        .find(|handle| {
            player::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.player_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the requested player is not online",
            )
        })?;
    let target_world = world::get("minecraft:the_nether")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the Nether world must already be loaded",
        )
    })?;
    player::teleport_to_world(
        target_player,
        target_world,
        player::PlayerTeleport::new(0.5, 64.0, 0.5, 0.0, 0.0)?,
    )?;
    Ok("player transferred to the already-loaded Nether chunk".to_owned())
}

#[command(name = "player.inventory.write")]
fn write_player_inventory(arguments: PlayerArguments) -> SjmlResult<String> {
    let target_player = player::online()?
        .into_iter()
        .find(|handle| {
            player::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.player_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the requested player is not online",
            )
        })?;
    let before = sjml::player_inventory::read(target_player)?;
    if before.slots.first().is_none_or(Option::is_some)
        || before.slots.get(1).is_none_or(Option::is_some)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory write fixture expects two empty hotbar slots",
        ));
    }
    let write_and_read = (|| {
        sjml::player_inventory::set_slot(
            target_player,
            1,
            PlayerInventorySlotUpdate::Set {
                item_id: "minecraft:emerald",
                count: 3,
            },
        )?;
        let written = sjml::player_inventory::read(target_player)?;
        Ok::<_, SjmlError>(
            written
                .slots
                .get(1)
                .and_then(Option::as_ref)
                .is_some_and(|stack| stack.item_id == "minecraft:emerald" && stack.count == 3),
        )
    })();
    let clear_result =
        sjml::player_inventory::set_slot(target_player, 1, PlayerInventorySlotUpdate::Clear);
    let cleared = clear_result.and_then(|()| sjml::player_inventory::read(target_player));
    let damage_and_read =
        (|| {
            sjml::player_inventory::set_slot(
                target_player,
                0,
                PlayerInventorySlotUpdate::Set {
                    item_id: "minecraft:diamond_sword",
                    count: 1,
                },
            )?;
            sjml::player_inventory::set_damage(target_player, 0, 7)?;
            let equipment = player::equipment(target_player)?;
            Ok::<_, SjmlError>(equipment.main_hand.as_ref().is_some_and(|stack| {
                stack.item_id == "minecraft:diamond_sword" && stack.damage == 7
            }))
        })();
    let clear_damaged_result =
        sjml::player_inventory::set_slot(target_player, 0, PlayerInventorySlotUpdate::Clear);
    let damage_slot_cleared =
        clear_damaged_result.and_then(|()| sjml::player_inventory::read(target_player));
    let wrote_expected = write_and_read?;
    let cleared = cleared?;
    let damage_checked = damage_and_read?;
    let damage_slot_cleared = damage_slot_cleared?;
    if !wrote_expected
        || cleared.slots.get(1).is_none_or(Option::is_some)
        || !damage_checked
        || damage_slot_cleared
            .slots
            .first()
            .is_none_or(Option::is_some)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player inventory slot or durability write did not match or restore the requested values",
        ));
    }
    Ok("player inventory slot write/clear and item durability update succeeded".to_owned())
}

#[command(name = "player.inventory.integer_component")]
fn player_inventory_integer_component(arguments: PlayerArguments) -> SjmlResult<String> {
    let target_player = player::online()?
        .into_iter()
        .find(|handle| {
            player::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.player_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the requested player is not online",
            )
        })?;
    let path = "player_charge";
    if sjml::player_inventory::read_integer_component(target_player, 1, path)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the player inventory component fixture must start without the module component",
        ));
    }
    sjml::player_inventory::write_integer_component(target_player, 1, path, 7)?;
    if sjml::player_inventory::read_integer_component(target_player, 1, path)? != Some(7) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the player inventory component did not round-trip through the guest API",
        ));
    }
    let boolean_path = "player_enabled";
    if sjml::player_inventory::read_boolean_component(target_player, 1, boolean_path)?.is_some() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the player inventory boolean fixture must start without the module component",
        ));
    }
    sjml::player_inventory::write_boolean_component(target_player, 1, boolean_path, true)?;
    if sjml::player_inventory::read_boolean_component(target_player, 1, boolean_path)? != Some(true)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the player inventory boolean component did not round-trip through the guest API",
        ));
    }
    Ok("player inventory integer and boolean component read/write succeeded".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityArguments {
    entity_id: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityRaycastArguments {
    entity_id: String,
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ClientEchoArguments {
    message: String,
}

#[client_command(name = "fixture.echo")]
fn client_echo(arguments: ClientEchoArguments) -> SjmlResult<String> {
    Ok(format!("local echo: {}", arguments.message))
}

#[command(name = "entity.raycast.check")]
fn check_entity_raycast(arguments: EntityRaycastArguments) -> SjmlResult<String> {
    let target = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.entity_id)
        })
        .ok_or_else(|| {
            SjmlError::new(ErrorCode::InvalidArgument, "raycast test pig is not loaded")
        })?;
    let snapshot = entity::snapshot(target)?;
    let world = world::get(&snapshot.dimension_id)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "raycast test world is not loaded",
        )
    })?;
    let start = RaycastPoint::new(snapshot.x - 2.0, snapshot.y + 0.5, snapshot.z);
    let end = RaycastPoint::new(snapshot.x + 2.0, snapshot.y + 0.5, snapshot.z);
    let hit =
        entity::raycast(world, start, end, Some(&snapshot.entity_type_id))?.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity raycast missed the test pig",
            )
        })?;
    if hit != target {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity raycast returned a different pig handle",
        ));
    }
    Ok("entity raycast returned the nearest loaded pig".to_owned())
}

#[command(name = "entity.fire_ticks.check")]
fn check_entity_fire_ticks(arguments: EntityRaycastArguments) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.entity_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "fire-tick test entity is not loaded",
            )
        })?;
    let original_ticks = entity::fire_ticks(handle)?;
    let check = (|| {
        entity::set_fire_ticks(handle, 120)?;
        if entity::fire_ticks(handle)? != 120 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity fire ticks did not round-trip to 120",
            ));
        }
        entity::set_fire_ticks(handle, 0)?;
        if entity::fire_ticks(handle)? != 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity fire ticks did not clear",
            ));
        }
        Ok(())
    })();
    let restore = entity::set_fire_ticks(handle, original_ticks);
    check?;
    restore?;
    Ok("entity fire ticks were set, read, and cleared".to_owned())
}

#[command(name = "entity.state_flags.check")]
fn check_entity_state_flags(arguments: EntityRaycastArguments) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.entity_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity-state test target is not loaded",
            )
        })?;
    let original_glowing = entity::is_glowing(handle)?;
    let original_silent = entity::is_silent(handle)?;
    let check = (|| {
        entity::set_glowing(handle, !original_glowing)?;
        if entity::is_glowing(handle)? == original_glowing {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity glowing state did not change",
            ));
        }
        entity::set_silent(handle, !original_silent)?;
        if entity::is_silent(handle)? == original_silent {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity silent state did not change",
            ));
        }
        Ok(())
    })();
    let restore_glowing = entity::set_glowing(handle, original_glowing);
    let restore_silent = entity::set_silent(handle, original_silent);
    check?;
    restore_glowing?;
    restore_silent?;
    Ok("entity glowing and silent flags were read, changed, and restored".to_owned())
}

#[command(name = "entity.custom_name.check")]
fn check_entity_custom_name(arguments: EntityRaycastArguments) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.entity_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "custom-name test target is not loaded",
            )
        })?;
    let original_name = entity::custom_name(handle)?;
    let check = (|| {
        entity::set_custom_name(handle, "SJML literal entity name")?;
        if entity::custom_name(handle)?.as_deref() != Some("SJML literal entity name") {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity custom name did not round-trip as literal text",
            ));
        }
        entity::clear_custom_name(handle)?;
        if entity::custom_name(handle)?.is_some() {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "entity custom name did not clear",
            ));
        }
        Ok(())
    })();
    let restore = match original_name.as_deref() {
        Some(name) => entity::set_custom_name(handle, name),
        None => entity::clear_custom_name(handle),
    };
    check?;
    restore?;
    Ok("entity custom name was read, set, cleared, and restored".to_owned())
}

#[command(name = "entity.inspect")]
fn inspect_entities(arguments: EntityArguments) -> SjmlResult<String> {
    let handles = entity::loaded()?;
    let snapshots = handles
        .iter()
        .copied()
        .map(entity::snapshot)
        .collect::<SjmlResult<Vec<_>>>()?;
    let (pig_handle, pig_snapshot) = handles
        .iter()
        .copied()
        .zip(&snapshots)
        .find(|(_, snapshot)| {
            snapshot.uuid == arguments.entity_id && snapshot.entity_type_id == "minecraft:pig"
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "the entity velocity GameTest pig is missing",
            )
        })?;
    let pig_world = world::get(&pig_snapshot.dimension_id)?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the GameTest pig's loaded world is missing",
        )
    })?;
    let pig_position = BlockPos {
        x: pig_snapshot.x.floor() as i32,
        y: pig_snapshot.y.floor() as i32,
        z: pig_snapshot.z.floor() as i32,
    };
    let nearby_pigs = entity::within(pig_world, pig_position, 4.0, 4.0, Some("minecraft:pig"))?;
    if !nearby_pigs.contains(&pig_handle) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "loaded-entity spatial query omitted its center pig",
        ));
    }
    let active_effects = entity::active_effects(pig_handle)?;
    let observed_slowness = active_effects.iter().any(|effect| {
        effect.effect_id == "minecraft:slowness"
            && effect.amplifier == 0
            && effect.remaining_duration_ticks > 0
    });
    entity::remove_effect(pig_handle, "minecraft:slowness")?;
    let original_max_health = sjml::entity_attributes::read(
        pig_handle,
        sjml::entity_attributes::LivingAttribute::MaxHealth,
    )?;
    let temporary_max_health = if original_max_health.base_value < 1024.0 {
        original_max_health.base_value + 1.0
    } else {
        original_max_health.base_value - 1.0
    };
    let changed_max_health = (|| {
        sjml::entity_attributes::set_base_value(
            pig_handle,
            sjml::entity_attributes::LivingAttribute::MaxHealth,
            temporary_max_health,
        )?;
        sjml::entity_attributes::read(
            pig_handle,
            sjml::entity_attributes::LivingAttribute::MaxHealth,
        )
    })();
    let restore_max_health = sjml::entity_attributes::set_base_value(
        pig_handle,
        sjml::entity_attributes::LivingAttribute::MaxHealth,
        original_max_health.base_value,
    );
    let changed_max_health = changed_max_health?;
    restore_max_health?;
    let restored_max_health = sjml::entity_attributes::read(
        pig_handle,
        sjml::entity_attributes::LivingAttribute::MaxHealth,
    )?;
    let original_velocity = entity::velocity(pig_handle)?;
    let temporary_y = if original_velocity.y <= MAX_ENTITY_VELOCITY_COMPONENT - 0.125 {
        original_velocity.y + 0.125
    } else {
        original_velocity.y - 0.125
    };
    let temporary_velocity =
        EntityVelocity::new(original_velocity.x, temporary_y, original_velocity.z)?;
    let changed_velocity = (|| {
        entity::set_velocity(pig_handle, temporary_velocity)?;
        entity::velocity(pig_handle)
    })();
    let restore_velocity = entity::set_velocity(pig_handle, original_velocity);
    let changed_velocity = changed_velocity?;
    restore_velocity?;
    entity::damage(pig_handle, 1.0)?;
    if snapshots.is_empty()
        || !snapshots
            .iter()
            .map(|snapshot| snapshot.uuid.as_str())
            .collect::<Vec<_>>()
            .windows(2)
            .all(|pair| pair[0] < pair[1])
        || snapshots.iter().any(|snapshot| {
            snapshot.uuid.len() != 36
                || snapshot.entity_type_id.split_once(':').is_none()
                || snapshot.dimension_id.split_once(':').is_none()
                || snapshot.entity_type_id == "minecraft:player"
                || !snapshot.x.is_finite()
                || !snapshot.y.is_finite()
                || !snapshot.z.is_finite()
        })
        || !snapshots
            .iter()
            .any(|snapshot| snapshot.entity_type_id == "minecraft:pig")
        || (changed_max_health.base_value - temporary_max_health).abs() > 0.0001
        || (restored_max_health.base_value - original_max_health.base_value).abs() > 0.0001
        || changed_velocity != temporary_velocity
        || !observed_slowness
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity read contract failed",
        ));
    }
    Ok(format!(
        "read {} loaded non-player entity snapshot(s)",
        snapshots.len()
    ))
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct EntityDataArguments {
    entity_id: String,
}

#[command(name = "entity.data.roundtrip")]
fn roundtrip_entity_data(arguments: EntityDataArguments) -> SjmlResult<String> {
    let handle = entity::loaded()?
        .into_iter()
        .find(|handle| {
            entity::snapshot(*handle).is_ok_and(|snapshot| snapshot.uuid == arguments.entity_id)
        })
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "requested loaded non-player entity was not found",
            )
        })?;
    if entity_data::read(handle, "gametest/missing", &mut [])? != entity_data::ENTITY_DATA_MISSING {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "an unset entity attachment key must be absent",
        ));
    }
    entity_data::set(handle, "gametest/empty", &[])?;
    let mut empty_value_buffer = [0; 1];
    let empty_length = entity_data::read(handle, "gametest/empty", &mut empty_value_buffer)?;
    let removed_empty = entity_data::remove(handle, "gametest/empty")?;
    let removed_empty_again = entity_data::remove(handle, "gametest/empty")?;
    if empty_length != 0 || !removed_empty || removed_empty_again {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "empty entity attachment values or remove results are incorrect",
        ));
    }
    let payload = [0_u8, 42, 255];
    entity_data::set(handle, "gametest/probe", &payload)?;
    let mut read_payload = [0; 3];
    let read_length = entity_data::read(handle, "gametest/probe", &mut read_payload)?;
    if read_length != payload.len() || read_payload != payload {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity attachment read did not return the stored byte payload",
        ));
    }
    Ok("entity data attachment roundtrip succeeded".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct InventoryArguments {
    chest_x: i32,
    chest_y: i32,
    chest_z: i32,
    empty_x: i32,
    empty_y: i32,
    empty_z: i32,
}

#[command(name = "inventory.inspect")]
fn inspect_inventory(arguments: InventoryArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    let chest_position = BlockPos {
        x: arguments.chest_x,
        y: arguments.chest_y,
        z: arguments.chest_z,
    };
    let contains_expected_stack = |snapshot: &InventorySnapshot| {
        snapshot.slots.iter().flatten().any(|stack| {
            stack.item_id == "minecraft:diamond"
                && stack.count == 7
                && stack.max_stack_size == 64
                && stack.slot_limit == 99
        })
    };
    let chest = inventory::read(world, chest_position, None)?
        .ok_or_else(|| SjmlError::new(ErrorCode::InvalidArgument, "chest inventory is missing"))?;
    let sided_chest =
        inventory::read(world, chest_position, Some(Direction::Up))?.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "sided chest inventory is missing",
            )
        })?;
    let detailed_stack =
        inventory::read_stack(world, chest_position, None, 0)?.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::InvalidArgument,
                "detailed chest stack snapshot is missing",
            )
        })?;
    let empty_block = inventory::read(
        world,
        BlockPos {
            x: arguments.empty_x,
            y: arguments.empty_y,
            z: arguments.empty_z,
        },
        None,
    )?;
    if !contains_expected_stack(&chest) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "unsided inventory snapshot did not contain the expected stack: {:?}",
                chest.slots
            ),
        ));
    }
    if !contains_expected_stack(&sided_chest) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "sided inventory snapshot did not contain the expected stack: {:?}",
                sided_chest.slots
            ),
        ));
    }
    if detailed_stack.item_id != "minecraft:diamond"
        || detailed_stack.count != 7
        || detailed_stack.max_stack_size != 64
        || detailed_stack.damage.is_some()
        || detailed_stack.max_damage.is_some()
        || detailed_stack.custom_name.as_deref() != Some("SJML Test Diamond")
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!("detailed inventory stack snapshot was unexpected: {detailed_stack:?}"),
        ));
    }
    if let Some(snapshot) = empty_block {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "stone unexpectedly exposed an inventory: {:?}",
                snapshot.slots
            ),
        ));
    }
    Ok("loaded chest snapshot and absent inventory are correct".to_owned())
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct BlockEntityArguments {
    chest_x: i32,
    chest_y: i32,
    chest_z: i32,
    stone_x: i32,
    stone_y: i32,
    stone_z: i32,
}

#[command(name = "block_entity.inspect")]
fn inspect_block_entity(arguments: BlockEntityArguments) -> SjmlResult<String> {
    let world = world::get("minecraft:overworld")?.ok_or_else(|| {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "the real overworld must already be loaded",
        )
    })?;
    let chest_position = BlockPos {
        x: arguments.chest_x,
        y: arguments.chest_y,
        z: arguments.chest_z,
    };
    let chest = world.block_entity(chest_position)?.ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "chest block entity is missing")
    })?;
    let snapshot = chest.snapshot()?;
    let type_id = chest.type_id()?;
    let block_state = chest.block_state()?;
    let inventory = chest.inventory(None)?.ok_or_else(|| {
        SjmlError::new(ErrorCode::InvalidArgument, "chest item handler is missing")
    })?;
    let nbt = chest.nbt()?;
    let stone = world.block_entity(BlockPos {
        x: arguments.stone_x,
        y: arguments.stone_y,
        z: arguments.stone_z,
    })?;
    let has_diamond_nbt = nbt_contains_string(&nbt, "minecraft:diamond");
    let has_diamond_slot = inventory.slots.iter().flatten().any(|slot| {
        slot.item_id == "minecraft:diamond"
            && slot.count == 7
            && slot.max_stack_size == 64
            && slot.slot_limit == 99
    });
    if type_id != "minecraft:chest"
        || snapshot.type_id != type_id
        || snapshot.dimension_id != "minecraft:overworld"
        || snapshot.position != chest_position
        || !block_state.starts_with("minecraft:chest[")
        || snapshot.block_state != block_state
        || !has_diamond_slot
        || !has_diamond_nbt
        || stone.is_some()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "block entity snapshot, inventory, NBT or absent-block contract failed",
        ));
    }
    Ok("block entity snapshot, inventory and typed NBT are correct".to_owned())
}

fn nbt_contains_string(value: &NbtValue, expected: &str) -> bool {
    match value {
        NbtValue::String(value) => value == expected,
        NbtValue::List(values) => values
            .iter()
            .any(|value| nbt_contains_string(value, expected)),
        NbtValue::Compound(values) => values
            .iter()
            .any(|(_, value)| nbt_contains_string(value, expected)),
        _ => false,
    }
}
