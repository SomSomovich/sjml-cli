use sjml::{
    from_bytes, to_bytes, EntityTeleportAttempt, EntityTeleportAttemptAction, EntityTeleportCause,
    MAX_ENTITY_TELEPORT_ATTEMPT_COORDINATE,
};

fn event() -> EntityTeleportAttempt {
    EntityTeleportAttempt {
        entity_id: "68ccfd1a-588d-4eb4-a0c9-31046942d74f".into(),
        entity_type_id: "minecraft:pig".into(),
        dimension_id: "minecraft:overworld".into(),
        cause: EntityTeleportCause::EnderPearl,
        from_x: 1.25,
        from_y: 64.0,
        from_z: -2.5,
        target_x: 120.0,
        target_y: 70.0,
        target_z: 121.0,
    }
}

#[test]
fn event_schema_and_veto_response_round_trip() {
    let original = event();
    let encoded = to_bytes(&original).unwrap();
    assert_eq!(&encoded[..4], &1_i32.to_le_bytes());
    assert_eq!(
        from_bytes::<EntityTeleportAttempt>(&encoded).unwrap(),
        original
    );
    assert_eq!(
        from_bytes::<EntityTeleportAttemptAction>(
            &to_bytes(&EntityTeleportAttemptAction::Cancel).unwrap()
        )
        .unwrap(),
        EntityTeleportAttemptAction::Cancel
    );
}

#[test]
fn event_rejects_untrusted_ids_and_coordinates() {
    let mut invalid_id = event();
    invalid_id.entity_type_id = "MINECRAFT:pig".into();
    assert!(to_bytes(&invalid_id).is_err());

    let mut invalid_position = event();
    invalid_position.from_y = f64::INFINITY;
    assert!(to_bytes(&invalid_position).is_err());

    let mut out_of_bounds = event();
    out_of_bounds.target_x = MAX_ENTITY_TELEPORT_ATTEMPT_COORDINATE + 1.0;
    assert!(to_bytes(&out_of_bounds).is_err());
}

#[test]
fn response_uses_only_pass_or_cancel() {
    assert!(from_bytes::<EntityTeleportAttemptAction>(&[0]).is_ok());
    assert!(from_bytes::<EntityTeleportAttemptAction>(&[1]).is_ok());
    assert!(from_bytes::<EntityTeleportAttemptAction>(&[2]).is_err());
}
