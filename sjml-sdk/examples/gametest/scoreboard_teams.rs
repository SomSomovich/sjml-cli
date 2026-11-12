use sjml::{command, ErrorCode, SjmlError, SjmlResult, StableDecode};

const TEAM_ID: &str = "gametest_players";

#[derive(StableDecode)]
#[sjml(version = 1)]
struct TeamArguments {
    action: String,
    player_uuid: String,
}

fn expected_options() -> sjml::scoreboard::TeamOptions {
    sjml::scoreboard::TeamOptions {
        display_name: "GameTest Players".to_owned(),
        prefix: "[GT] ".to_owned(),
        suffix: " ★".to_owned(),
        color: sjml::scoreboard::TeamColor::Aqua,
        name_tag_visibility: sjml::scoreboard::TeamVisibility::HideForOtherTeams,
        death_message_visibility: sjml::scoreboard::TeamVisibility::HideForOwnTeam,
        collision_rule: sjml::scoreboard::TeamCollisionRule::PushOtherTeams,
        friendly_fire: false,
        see_friendly_invisibles: true,
    }
}

#[command(name = "scoreboard.team.control")]
fn control_owned_team(arguments: TeamArguments) -> SjmlResult<String> {
    match arguments.action.as_str() {
        "check" => {
            let options = expected_options();
            sjml::scoreboard::create_team(TEAM_ID)?;
            sjml::scoreboard::set_team_options(TEAM_ID, &options)?;
            sjml::scoreboard::add_player_to_team(TEAM_ID, &arguments.player_uuid)?;
            if sjml::scoreboard::team(TEAM_ID)? != options {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "scoreboard team options did not round-trip through the host ABI",
                ));
            }
            Ok("owned team created, configured, joined, and read back".to_owned())
        }
        "cleanup" => {
            sjml::scoreboard::remove_player_from_team(TEAM_ID, &arguments.player_uuid)?;
            sjml::scoreboard::remove_team(TEAM_ID)?;
            Ok("owned team and membership removed".to_owned())
        }
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "scoreboard team action must be 'check' or 'cleanup'",
        )),
    }
}
