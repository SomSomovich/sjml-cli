use sjml::{from_bytes, player, to_bytes, ErrorCode, PlayerHandle};

#[test]
fn player_ability_snapshot_uses_schema_one_and_ordered_boolean_fields() {
    let abilities = player::PlayerAbilities {
        may_fly: true,
        flying: false,
        invulnerable: true,
        may_build: false,
    };
    let encoded = to_bytes(&abilities).unwrap();
    assert_eq!(encoded, [1, 0, 0, 0, 1, 0, 1, 0]);
    assert_eq!(
        from_bytes::<player::PlayerAbilities>(&encoded).unwrap(),
        abilities
    );
}

#[test]
fn native_ability_reads_and_flight_changes_report_unsupported_imports() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::abilities(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_flying(handle, true).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}
