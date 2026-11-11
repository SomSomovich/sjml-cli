use sjml::player::{
    set_experience, set_food, set_health, MAX_PLAYER_EXPERIENCE_LEVEL, MAX_PLAYER_FOOD_LEVEL,
    MAX_PLAYER_HEALTH,
};
use sjml::{ErrorCode, PlayerHandle};

#[test]
fn health_is_bounded_locally_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    for health in [f32::NAN, f32::INFINITY, -0.01, MAX_PLAYER_HEALTH + 1.0] {
        assert_eq!(
            set_health(handle, health).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        set_health(handle, 20.0).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn food_level_and_saturation_are_validated_as_a_pair() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    for (food, saturation) in [
        (MAX_PLAYER_FOOD_LEVEL + 1, 0.0),
        (10, f32::NAN),
        (10, f32::INFINITY),
        (10, 10.1),
        (0, 0.1),
    ] {
        assert_eq!(
            set_food(handle, food, saturation).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        set_food(handle, 20, 5.0).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn experience_level_and_progress_are_bounded_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    for (level, progress) in [
        (MAX_PLAYER_EXPERIENCE_LEVEL + 1, 0.0),
        (10, f32::NAN),
        (10, f32::INFINITY),
        (10, -0.01),
        (10, 1.0),
    ] {
        assert_eq!(
            set_experience(handle, level, progress).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        set_experience(handle, 10, 0.5).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}
