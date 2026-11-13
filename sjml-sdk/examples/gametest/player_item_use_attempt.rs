use sjml::{
    command, event,
    events::{
        self, EventPriority, ItemUseAction, ModuleItemUse, ModuleItemUseOnBlock,
        ModuleItemUseOnEntity, PlayerItemUseAttempt,
    },
    ErrorCode, SjmlError, SjmlResult, StableDecode,
};
use std::sync::Mutex;

struct ItemUseAttemptSnapshot {
    player_id: String,
    dimension_id: String,
    hand: String,
    item_id: String,
    count: u32,
}

static LAST_ITEM_USE_ATTEMPT: Mutex<Option<ItemUseAttemptSnapshot>> = Mutex::new(None);

pub(super) fn initialize() -> SjmlResult<()> {
    *LAST_ITEM_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.item_use_attempt state lock is poisoned",
        )
    })? = None;
    events::subscribe("player.item_use_attempt", EventPriority::Normal)?;
    events::subscribe("item.use", EventPriority::Normal)?;
    events::subscribe("item.use_on_block", EventPriority::Normal)?;
    events::subscribe("item.use_on_entity", EventPriority::Normal)?;
    Ok(())
}

#[event(name = "player.item_use_attempt")]
fn cancel_ender_pearl_use(attempt: PlayerItemUseAttempt) -> SjmlResult<bool> {
    let should_cancel = attempt.hand == "main_hand"
        && attempt.item_id == "minecraft:ender_pearl"
        && attempt.count == 3;
    *LAST_ITEM_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.item_use_attempt state lock is poisoned",
        )
    })? = Some(ItemUseAttemptSnapshot {
        player_id: attempt.player_id,
        dimension_id: attempt.dimension_id,
        hand: attempt.hand,
        item_id: attempt.item_id,
        count: attempt.count,
    });
    Ok(should_cancel)
}

#[event(name = "item.use")]
fn handle_module_item_use(attempt: ModuleItemUse) -> SjmlResult<ItemUseAction> {
    if attempt.hand == "main_hand"
        && attempt.item_id == "sjml_game_test:content/gametest_ruby"
        && attempt.count == 3
    {
        Ok(ItemUseAction::ConsumeOne)
    } else {
        Ok(ItemUseAction::Pass)
    }
}

#[event(name = "item.use_on_block")]
fn handle_module_item_use_on_block(attempt: ModuleItemUseOnBlock) -> SjmlResult<ItemUseAction> {
    if attempt.hand == "main_hand"
        && attempt.item_id == "sjml_game_test:content/gametest_ruby"
        && attempt.item_count == 2
        && attempt.block_id == "minecraft:stone"
    {
        Ok(ItemUseAction::ConsumeOne)
    } else {
        Ok(ItemUseAction::Pass)
    }
}

#[event(name = "item.use_on_entity")]
fn handle_module_item_use_on_entity(attempt: ModuleItemUseOnEntity) -> SjmlResult<ItemUseAction> {
    if attempt.hand == "main_hand"
        && attempt.item_id == "sjml_game_test:content/gametest_ruby"
        && attempt.item_count == 2
        && attempt.target_type_id == "minecraft:cow"
    {
        Ok(ItemUseAction::ConsumeOne)
    } else {
        Ok(ItemUseAction::Pass)
    }
}

#[derive(StableDecode)]
#[sjml(version = 1)]
struct ItemUseAttemptCheckArguments {
    player_id: String,
    dimension_id: String,
    hand: String,
    item_id: String,
    count: i32,
}

#[command(name = "player.item_use_attempt.check")]
fn check_item_use_attempt(arguments: ItemUseAttemptCheckArguments) -> SjmlResult<String> {
    if arguments.item_id == "minecraft:potion" {
        if !super::player_potion_use_attempt::matches_existing_item_use_check(
            &arguments.player_id,
            &arguments.dimension_id,
            &arguments.hand,
            &arguments.item_id,
            arguments.count,
        )? {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player.potion_use_attempt did not match the requested GameTest snapshot",
            ));
        }
        return Ok(
            "player.potion_use_attempt delivered and cancelled the expected potion use".to_owned(),
        );
    }
    if arguments.item_id == "minecraft:totem_of_undying" {
        if !super::player_totem_use_attempt::matches_existing_item_use_check(
            &arguments.player_id,
            &arguments.dimension_id,
            &arguments.hand,
            &arguments.item_id,
            arguments.count,
        )? {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "player.totem_use_attempt did not match the requested GameTest snapshot",
            ));
        }
        return Ok(
            "player.totem_use_attempt delivered and cancelled the expected totem use".to_owned(),
        );
    }

    let observed = LAST_ITEM_USE_ATTEMPT.lock().map_err(|_| {
        SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "player.item_use_attempt state lock is poisoned",
        )
    })?;
    let Some(observed) = observed.as_ref() else {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "the guest has not observed a player.item_use_attempt callback",
        ));
    };
    if observed.player_id != arguments.player_id
        || observed.dimension_id != arguments.dimension_id
        || observed.hand != arguments.hand
        || observed.item_id != arguments.item_id
        || observed.count != u32::try_from(arguments.count).unwrap_or(u32::MAX)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!(
                "player.item_use_attempt expected {}/{}/{}/{}/{}, received {}/{}/{}/{}/{}",
                arguments.player_id,
                arguments.dimension_id,
                arguments.hand,
                arguments.item_id,
                arguments.count,
                observed.player_id,
                observed.dimension_id,
                observed.hand,
                observed.item_id,
                observed.count
            ),
        ));
    }
    Ok("player.item_use_attempt delivered the expected detached item snapshot".to_owned())
}
