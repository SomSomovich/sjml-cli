use sjml::events::{
    BlockBreakAttempt, EffectTickEvent, EntityDeath, EntityDeathAttempt, PlayerJoined, PlayerLeft,
    PlayerRespawned, ResourcesReloaded,
};
use sjml::player::PlayerTeleport;
use sjml::prelude::PlayerMoved;
use sjml::registry::{register_villager_trade, VillagerTradeItem, VillagerTradeOffer};
use sjml::{
    client_command, command, event, from_bytes, mod_init, mod_register, mod_shutdown, service,
    to_bytes, EntityHandle, ErrorCode, PlayerHandle, PlayerSnapshot, SjmlError, SjmlResult,
    StableDecode, StableEncode, WorldHandle,
};

#[derive(Debug, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 7)]
struct PlayerMessage {
    player_id: u32,
    accepted: bool,
    text: String,
}

#[mod_init]
fn initialize() -> SjmlResult<()> {
    Ok(())
}

#[mod_shutdown]
fn shutdown() -> SjmlResult<()> {
    Ok(())
}

#[mod_register]
fn register_demo_trade() -> SjmlResult<()> {
    let offer = VillagerTradeOffer::new(
        "minecraft:farmer",
        1,
        VillagerTradeItem::new("minecraft:emerald", 18)?,
        VillagerTradeItem::new("minecraft:bread", 4)?,
    )?;
    register_villager_trade("emerald_for_bread", offer)
}

#[event(name = "player.joined")]
fn on_player_joined(message: PlayerMessage) -> SjmlResult<()> {
    assert_eq!(message.player_id, 0x0102_0304);
    Ok(())
}

#[event(name = "resources.reloaded")]
fn on_resources_reloaded(event: ResourcesReloaded) -> SjmlResult<()> {
    assert_eq!(event.tick_count, 1234);
    Ok(())
}

#[event(name = "player.moved")]
fn on_player_moved(_: PlayerMoved) -> SjmlResult<()> {
    Ok(())
}

#[event(name = "player.respawned")]
fn on_player_respawned(_: PlayerRespawned) -> SjmlResult<()> {
    Ok(())
}

#[event(name = "entity.death_attempt")]
fn on_entity_death_attempt(_: EntityDeathAttempt) -> SjmlResult<bool> {
    Ok(false)
}

#[event(name = "block.break_attempt")]
fn on_block_break_attempt(_: BlockBreakAttempt) -> SjmlResult<bool> {
    Ok(false)
}

#[command(name = "demo.echo")]
fn echo(message: String) -> SjmlResult<String> {
    Ok(message)
}

#[client_command(name = "demo.local_echo")]
fn local_echo(message: String) -> SjmlResult<String> {
    Ok(message)
}

#[service(name = "demo.player_count")]
fn player_count(_: ()) -> SjmlResult<u32> {
    Ok(12)
}

#[test]
fn derived_records_use_explicit_version_and_little_endian_fields() {
    let value = PlayerMessage {
        player_id: 0x0102_0304,
        accepted: true,
        text: "hi".to_owned(),
    };

    let bytes = to_bytes(&value).expect("encode record");

    assert_eq!(
        bytes,
        [
            7, 0, 0, 0, // schema version
            4, 3, 2, 1, // u32
            1, // bool
            2, 0, 0, 0, b'h', b'i', // length-prefixed UTF-8
        ]
    );
    assert_eq!(from_bytes::<PlayerMessage>(&bytes).unwrap(), value);
}

#[test]
fn player_joined_event_record_has_the_documented_v1_field_order() {
    let value = PlayerJoined {
        player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        display_name: "Alice".to_owned(),
    };

    let bytes = to_bytes(&value).expect("encode player.joined record");

    let mut expected = vec![1, 0, 0, 0, 36, 0, 0, 0];
    expected.extend_from_slice(b"00000000-0000-0000-0000-000000000001");
    expected.extend_from_slice(&[5, 0, 0, 0]);
    expected.extend_from_slice(b"Alice");
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<PlayerJoined>(&bytes).unwrap(), value);
}

#[test]
fn resources_reloaded_record_has_the_documented_v1_field_order() {
    let value = ResourcesReloaded { tick_count: 1234 };
    let bytes = to_bytes(&value).expect("encode resources.reloaded record");
    let mut expected = vec![1, 0, 0, 0];
    expected.extend_from_slice(&1234_i64.to_le_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<ResourcesReloaded>(&bytes).unwrap(), value);
    assert_eq!(on_resources_reloaded(value), Ok(()));
}

#[test]
fn client_command_callback_uses_the_typed_sdk_contract() {
    assert_eq!(
        local_echo("hello locally".to_owned()),
        Ok("hello locally".to_owned())
    );
}

#[test]
fn mod_register_can_declare_a_bounded_villager_trade() {
    let error = register_demo_trade().unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnknownErrorCode);
}

#[test]
fn player_left_event_record_uses_the_same_stable_player_identity_layout() {
    let value = PlayerLeft {
        player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        display_name: "Alice".to_owned(),
    };

    let bytes = to_bytes(&value).expect("encode player.left record");
    let mut expected = vec![1, 0, 0, 0, 36, 0, 0, 0];
    expected.extend_from_slice(b"00000000-0000-0000-0000-000000000001");
    expected.extend_from_slice(&[5, 0, 0, 0]);
    expected.extend_from_slice(b"Alice");
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<PlayerLeft>(&bytes).unwrap(), value);
}

#[test]
fn player_respawned_event_record_has_the_documented_v1_field_order() {
    let event = PlayerRespawned {
        player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        display_name: "Alice".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: -1.5,
        y: 64.0,
        z: 2.25,
        end_conquered: true,
    };

    let mut expected = 1_u32.to_le_bytes().to_vec();
    for field in [
        event.player_id.as_str(),
        event.display_name.as_str(),
        event.dimension_id.as_str(),
    ] {
        expected.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected.extend_from_slice(field.as_bytes());
    }
    expected.extend_from_slice(&event.x.to_le_bytes());
    expected.extend_from_slice(&event.y.to_le_bytes());
    expected.extend_from_slice(&event.z.to_le_bytes());
    expected.push(1);

    assert_eq!(to_bytes(&event).unwrap(), expected);
    let decoded = from_bytes::<PlayerRespawned>(&expected).unwrap();
    assert_eq!(decoded, event);
    on_player_respawned(decoded).unwrap();
}

#[test]
fn player_moved_event_record_has_the_documented_v1_field_order() {
    let event = PlayerMoved {
        player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        display_name: "Alice".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: -1.5,
        y: 64.0,
        z: 2.25,
        yaw: 90.0,
        pitch: -15.5,
    };

    let mut expected = 1_u32.to_le_bytes().to_vec();
    for field in [
        event.player_id.as_str(),
        event.display_name.as_str(),
        event.dimension_id.as_str(),
    ] {
        expected.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected.extend_from_slice(field.as_bytes());
    }
    expected.extend_from_slice(&event.x.to_le_bytes());
    expected.extend_from_slice(&event.y.to_le_bytes());
    expected.extend_from_slice(&event.z.to_le_bytes());
    expected.extend_from_slice(&event.yaw.to_le_bytes());
    expected.extend_from_slice(&event.pitch.to_le_bytes());
    assert_eq!(to_bytes(&event).unwrap(), expected);
    let decoded = from_bytes::<PlayerMoved>(&expected).unwrap();
    assert_eq!(decoded, event);
    on_player_moved(decoded).unwrap();
}

#[test]
fn player_teleport_destination_has_the_documented_v1_field_order() {
    let destination = PlayerTeleport {
        x: -12.5,
        y: 70.0,
        z: 24.25,
        yaw: 190.0,
        pitch: -25.0,
    };

    let mut expected = 1_u32.to_le_bytes().to_vec();
    expected.extend_from_slice(&destination.x.to_le_bytes());
    expected.extend_from_slice(&destination.y.to_le_bytes());
    expected.extend_from_slice(&destination.z.to_le_bytes());
    expected.extend_from_slice(&destination.yaw.to_le_bytes());
    expected.extend_from_slice(&destination.pitch.to_le_bytes());

    assert_eq!(expected.len(), 36);
    assert_eq!(to_bytes(&destination).unwrap(), expected);
    assert_eq!(
        from_bytes::<PlayerTeleport>(&expected).unwrap(),
        destination
    );
}

#[test]
fn entity_death_event_record_has_the_documented_v1_field_order() {
    let value = EntityDeath {
        entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        entity_type_id: "minecraft:pig".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: -1.5,
        y: 64.0,
        z: 2.25,
    };
    let bytes = to_bytes(&value).expect("encode entity.death record");
    let mut expected = vec![1, 0, 0, 0];
    for field in [
        "00000000-0000-0000-0000-000000000001",
        "minecraft:pig",
        "minecraft:overworld",
    ] {
        expected.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected.extend_from_slice(field.as_bytes());
    }
    expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
    expected.extend_from_slice(&(64.0_f64).to_le_bytes());
    expected.extend_from_slice(&(2.25_f64).to_le_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<EntityDeath>(&bytes).unwrap(), value);
}

#[test]
fn entity_death_attempt_record_has_detached_player_data_and_cancellable_bool_contract() {
    let event = EntityDeathAttempt {
        entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        entity_type_id: "minecraft:player".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        player_display_name: Some("Alice".to_owned()),
        x: -1.5,
        y: 64.0,
        z: 2.25,
    };
    let bytes = to_bytes(&event).expect("encode entity.death_attempt record");
    let mut expected = vec![1, 0, 0, 0];
    for field in [
        "00000000-0000-0000-0000-000000000001",
        "minecraft:player",
        "minecraft:overworld",
    ] {
        expected.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected.extend_from_slice(field.as_bytes());
    }
    expected.push(1);
    expected.extend_from_slice(&[5, 0, 0, 0]);
    expected.extend_from_slice(b"Alice");
    expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
    expected.extend_from_slice(&(64.0_f64).to_le_bytes());
    expected.extend_from_slice(&(2.25_f64).to_le_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<EntityDeathAttempt>(&bytes).unwrap(), event);
    assert_eq!(on_entity_death_attempt(event), Ok(false));

    let non_player = EntityDeathAttempt {
        entity_type_id: "minecraft:pig".to_owned(),
        player_display_name: None,
        x: 1.0,
        y: 2.0,
        z: 3.0,
        ..from_bytes::<EntityDeathAttempt>(&bytes).unwrap()
    };
    let mut expected_none = vec![1, 0, 0, 0];
    for field in [
        "00000000-0000-0000-0000-000000000001",
        "minecraft:pig",
        "minecraft:overworld",
    ] {
        expected_none.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected_none.extend_from_slice(field.as_bytes());
    }
    expected_none.push(0);
    expected_none.extend_from_slice(&(1.0_f64).to_le_bytes());
    expected_none.extend_from_slice(&(2.0_f64).to_le_bytes());
    expected_none.extend_from_slice(&(3.0_f64).to_le_bytes());
    assert_eq!(to_bytes(&non_player).unwrap(), expected_none);
    assert_eq!(
        from_bytes::<EntityDeathAttempt>(&expected_none).unwrap(),
        non_player
    );
}

#[test]
fn block_break_attempt_record_has_the_documented_v1_field_order_and_bool_callback() {
    let event = BlockBreakAttempt {
        player_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        block_id: "minecraft:stone".to_owned(),
        x: -3,
        y: 320,
        z: i32::MIN,
    };
    let bytes = to_bytes(&event).expect("encode block.break_attempt record");
    let mut expected = vec![1, 0, 0, 0];
    for field in [
        "00000000-0000-0000-0000-000000000001",
        "minecraft:overworld",
        "minecraft:stone",
    ] {
        expected.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected.extend_from_slice(field.as_bytes());
    }
    expected.extend_from_slice(&(-3_i32).to_le_bytes());
    expected.extend_from_slice(&(320_i32).to_le_bytes());
    expected.extend_from_slice(&i32::MIN.to_le_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<BlockBreakAttempt>(&bytes).unwrap(), event);
    assert_eq!(on_block_break_attempt(event), Ok(false));
}

#[test]
fn effect_tick_event_record_has_the_documented_v1_field_order() {
    let value = EffectTickEvent {
        effect_id: "sjml_game_test:content/gametest_tick".to_owned(),
        entity_id: "00000000-0000-0000-0000-000000000001".to_owned(),
        entity_type_id: "minecraft:pig".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: -1.5,
        y: 64.0,
        z: 2.25,
        amplifier: 2,
        duration_ticks: 100,
        tick_interval: 20,
    };
    let bytes = to_bytes(&value).expect("encode entity.effect.tick record");
    let mut expected = vec![1, 0, 0, 0];
    for field in [
        "sjml_game_test:content/gametest_tick",
        "00000000-0000-0000-0000-000000000001",
        "minecraft:pig",
        "minecraft:overworld",
    ] {
        expected.extend_from_slice(&(field.len() as u32).to_le_bytes());
        expected.extend_from_slice(field.as_bytes());
    }
    expected.extend_from_slice(&(-1.5_f64).to_le_bytes());
    expected.extend_from_slice(&(64.0_f64).to_le_bytes());
    expected.extend_from_slice(&(2.25_f64).to_le_bytes());
    expected.extend_from_slice(&2_i32.to_le_bytes());
    expected.extend_from_slice(&100_i32.to_le_bytes());
    expected.extend_from_slice(&20_u32.to_le_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(from_bytes::<EffectTickEvent>(&bytes).unwrap(), value);
}

#[test]
fn malformed_boolean_uses_the_stable_abi_error_code() {
    let error = from_bytes::<bool>(&[2]).unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidBoolean);
    assert_eq!(error.abi_code(), 5);
}

#[test]
fn typed_handles_keep_kind_and_the_24_byte_abi_layout() {
    let entity = EntityHandle::new(0x0102_0304_0506_0708, 9, 3, 4).unwrap();
    let encoded = to_bytes(&entity).unwrap();

    assert_eq!(encoded.len(), 24);
    assert_eq!(&encoded[0..8], &0x0102_0304_0506_0708u64.to_le_bytes());
    assert_eq!(&encoded[8..16], &9u64.to_le_bytes());
    assert_eq!(&encoded[16..24], &0x0000_0004_0000_0003u64.to_le_bytes());
    assert_eq!(from_bytes::<EntityHandle>(&encoded).unwrap(), entity);
    assert_eq!(EntityHandle::KIND_CODE, 1);
    assert_eq!(WorldHandle::KIND_CODE, 3);
    assert_eq!(PlayerHandle::KIND_CODE, 2);
    assert_eq!(initialize(), Ok(()));
    assert_eq!(shutdown(), Ok(()));
    assert_eq!(echo("test".to_owned()).unwrap(), "test");
    assert_eq!(player_count(()).unwrap(), 12);
    assert_eq!(
        on_player_joined(PlayerMessage {
            player_id: 0x0102_0304,
            accepted: true,
            text: "joined".to_owned(),
        }),
        Ok(())
    );
}

#[test]
fn player_snapshot_uses_versioned_stable_encoding() {
    let snapshot = PlayerSnapshot {
        uuid: "00000000-0000-0000-0000-000000000001".to_owned(),
        profile_name: "Alice".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: 1.25,
        y: -2.5,
        z: 3.75,
    };
    let encoded = to_bytes(&snapshot).expect("encode player snapshot");
    assert_eq!(u32::from_le_bytes(encoded[0..4].try_into().unwrap()), 1);
    assert_eq!(from_bytes::<PlayerSnapshot>(&encoded).unwrap(), snapshot);
}

#[test]
fn abi_error_values_round_trip_without_reassignment() {
    for (code, expected) in [
        (1, ErrorCode::IncompatibleAbiVersion),
        (16, ErrorCode::InvalidHandle),
        (26, ErrorCode::TaskLimitReached),
        (27, ErrorCode::TaskRejected),
        (28, ErrorCode::ServiceNotFound),
        (33, ErrorCode::ServiceTimeout),
        (7, ErrorCode::InvalidArgument),
        (34, ErrorCode::RegistrationLimitReached),
        (255, ErrorCode::UnknownErrorCode),
    ] {
        assert_eq!(ErrorCode::from_abi_code(code), expected);
        let error = SjmlError::from_abi_code(code, "host error");
        assert_eq!(error.code(), expected);
        assert_eq!(error.abi_code(), code);
    }
}

#[test]
fn schema_versions_and_trailing_bytes_are_rejected() {
    let wrong_version = [8, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, b'x'];
    assert_eq!(
        from_bytes::<PlayerMessage>(&wrong_version)
            .unwrap_err()
            .code(),
        ErrorCode::IncompatibleAbiVersion
    );

    let mut valid = to_bytes(&PlayerMessage {
        player_id: 1,
        accepted: true,
        text: "x".to_owned(),
    })
    .unwrap();
    valid.push(0);
    assert!(from_bytes::<PlayerMessage>(&valid).is_err());
}

#[test]
fn handle_construction_rejects_zero_scope_and_slot_components() {
    for result in [
        EntityHandle::new(0, 1, 1, 1),
        EntityHandle::new(1, 0, 1, 1),
        EntityHandle::new(1, 1, 0, 1),
        EntityHandle::new(1, 1, 1, 0),
    ] {
        assert_eq!(result.unwrap_err().code(), ErrorCode::InvalidHandle);
    }
}

#[test]
fn serialization_refuses_payloads_beyond_the_abi_limit() {
    let too_large = vec![b'x'; sjml::MAX_PAYLOAD_BYTES + 1];
    assert_eq!(
        to_bytes(&too_large).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );
    assert_eq!(
        from_bytes::<bool>(&vec![0; sjml::MAX_PAYLOAD_BYTES + 1])
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );
}

#[test]
fn host_imports_fail_with_a_typed_error_outside_wasm() {
    let error = sjml::host::abi_major_version().unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnknownErrorCode);
    assert!(error.message().contains("wasm32-unknown-unknown"));
}
