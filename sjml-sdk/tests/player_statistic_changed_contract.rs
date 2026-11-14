use sjml::{
    events::{
        PlayerStatisticChanged, MAX_PLAYER_STATISTIC_CHANGED_PAYLOAD_BYTES,
        MAX_PLAYER_STATISTIC_ID_BYTES, PLAYER_STATISTIC_CHANGED_EVENT_NAME,
    },
    from_bytes, to_bytes, ErrorCode,
};

const PLAYER_ID: &str = "00000000-0000-0000-0000-000000000001";

#[test]
fn statistic_change_matches_the_host_schema_and_round_trips_signed_values() {
    let event = PlayerStatisticChanged {
        player_id: PLAYER_ID.into(),
        statistic_type_id: "minecraft:custom".into(),
        statistic_id: "minecraft:time_since_death".into(),
        previous_value: 73,
        current_value: 0,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    for value in [PLAYER_ID, "minecraft:custom", "minecraft:time_since_death"] {
        expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
        expected.extend_from_slice(value.as_bytes());
    }
    expected.extend_from_slice(&73_i32.to_le_bytes());
    expected.extend_from_slice(&0_i32.to_le_bytes());

    assert_eq!(to_bytes(&event).unwrap(), expected);
    assert_eq!(
        from_bytes::<PlayerStatisticChanged>(&expected).unwrap(),
        event
    );
    assert_eq!(MAX_PLAYER_STATISTIC_CHANGED_PAYLOAD_BYTES, 572);
    assert_eq!(
        PLAYER_STATISTIC_CHANGED_EVENT_NAME,
        "player.statistic_changed"
    );

    let reset = PlayerStatisticChanged {
        previous_value: 0,
        current_value: -1,
        ..event
    };
    assert_eq!(
        from_bytes::<PlayerStatisticChanged>(&to_bytes(&reset).unwrap()).unwrap(),
        reset
    );
}

#[test]
fn statistic_change_rejects_bad_identity_unchanged_values_and_oversized_ids() {
    let base = PlayerStatisticChanged {
        player_id: PLAYER_ID.into(),
        statistic_type_id: "minecraft:custom".into(),
        statistic_id: "minecraft:deaths".into(),
        previous_value: 0,
        current_value: 1,
    };
    for invalid in [
        PlayerStatisticChanged {
            player_id: "bad-player-id".into(),
            ..base.clone()
        },
        PlayerStatisticChanged {
            statistic_type_id: "Minecraft:custom".into(),
            ..base.clone()
        },
        PlayerStatisticChanged {
            statistic_id: "minecraft:".into(),
            ..base.clone()
        },
        PlayerStatisticChanged {
            current_value: 0,
            ..base.clone()
        },
    ] {
        assert_eq!(
            to_bytes(&invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    let oversized = PlayerStatisticChanged {
        statistic_id: format!("minecraft:{}", "x".repeat(MAX_PLAYER_STATISTIC_ID_BYTES)),
        ..base.clone()
    };
    assert_eq!(
        to_bytes(&oversized).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );

    let mut wrong_schema = to_bytes(&base).unwrap();
    wrong_schema[..4].copy_from_slice(&2_u32.to_le_bytes());
    assert_eq!(
        from_bytes::<PlayerStatisticChanged>(&wrong_schema)
            .unwrap_err()
            .code(),
        ErrorCode::IncompatibleAbiVersion
    );

    let mut trailing = to_bytes(&base).unwrap();
    trailing.push(0);
    assert_eq!(
        from_bytes::<PlayerStatisticChanged>(&trailing)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
