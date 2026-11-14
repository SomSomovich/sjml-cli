#[allow(dead_code)]
#[path = "../examples/gametest/entity_left_level.rs"]
mod gametest_fixture;

use sjml::entity_left_level::{
    EntityLeftLevel, ENTITY_LEFT_LEVEL_EVENT_NAME, MAX_ENTITY_LEFT_LEVEL_CALLBACKS,
    MAX_ENTITY_LEFT_LEVEL_PAYLOAD_BYTES,
};
use sjml::{from_bytes, to_bytes, ErrorCode, StableDecode, StableEncode};

const ENTITY_ID: &str = "123e4567-e89b-12d3-a456-426614174000";

#[test]
fn wire_shape_matches_host_schema_and_round_trips() {
    let event = EntityLeftLevel {
        entity_id: ENTITY_ID.to_owned(),
        entity_type_id: "minecraft:pig".to_owned(),
        dimension_id: "minecraft:the_nether".to_owned(),
        x: 12.5,
        y: -64.0,
        z: 0.25,
    };
    let mut expected = Vec::new();
    expected.extend_from_slice(&1i32.to_le_bytes());
    for value in [ENTITY_ID, "minecraft:pig", "minecraft:the_nether"] {
        expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
        expected.extend_from_slice(value.as_bytes());
    }
    expected.extend_from_slice(&12.5f64.to_bits().to_le_bytes());
    expected.extend_from_slice(&(-64.0f64).to_bits().to_le_bytes());
    expected.extend_from_slice(&0.25f64.to_bits().to_le_bytes());

    let encoded = to_bytes(&event).expect("event should encode");
    assert_eq!(encoded, expected);
    assert_eq!(from_bytes::<EntityLeftLevel>(&encoded).unwrap(), event);
    assert_eq!(ENTITY_LEFT_LEVEL_EVENT_NAME, "entity.left_level");
    assert_eq!(MAX_ENTITY_LEFT_LEVEL_CALLBACKS, 64);
    assert_eq!(MAX_ENTITY_LEFT_LEVEL_PAYLOAD_BYTES, 588);
}

#[test]
fn validates_identity_resource_locations_and_finite_position() {
    let valid = EntityLeftLevel {
        entity_id: ENTITY_ID.to_owned(),
        entity_type_id: "minecraft:pig".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: 0.0,
        y: 70.0,
        z: -12.0,
    };
    assert_eq!(
        from_bytes::<EntityLeftLevel>(&to_bytes(&valid).unwrap()).unwrap(),
        valid
    );

    for invalid in [
        EntityLeftLevel {
            entity_id: "not-a-uuid".to_owned(),
            ..valid.clone()
        },
        EntityLeftLevel {
            entity_type_id: "Minecraft:pig".to_owned(),
            ..valid.clone()
        },
        EntityLeftLevel {
            dimension_id: "minecraft:".to_owned(),
            ..valid.clone()
        },
        EntityLeftLevel {
            x: f64::NAN,
            ..valid.clone()
        },
        EntityLeftLevel {
            y: f64::INFINITY,
            ..valid.clone()
        },
        EntityLeftLevel {
            z: f64::NEG_INFINITY,
            ..valid.clone()
        },
    ] {
        assert_eq!(
            to_bytes(&invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    let oversized = EntityLeftLevel {
        entity_type_id: format!("x:{}", "a".repeat(255)),
        ..valid
    };
    assert_eq!(
        to_bytes(&oversized).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );
}

#[test]
fn rejects_incompatible_malformed_and_trailing_payloads() {
    let event = EntityLeftLevel {
        entity_id: ENTITY_ID.to_owned(),
        entity_type_id: "minecraft:pig".to_owned(),
        dimension_id: "minecraft:overworld".to_owned(),
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let encoded = to_bytes(&event).unwrap();

    let mut wrong_version = encoded.clone();
    wrong_version[..4].copy_from_slice(&2i32.to_le_bytes());
    assert_eq!(
        from_bytes::<EntityLeftLevel>(&wrong_version)
            .unwrap_err()
            .code(),
        ErrorCode::IncompatibleAbiVersion
    );
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(from_bytes::<EntityLeftLevel>(&trailing).is_err());

    let mut non_finite = encoded.clone();
    let x_offset =
        4 + 3 * 4 + ENTITY_ID.len() + "minecraft:pig".len() + "minecraft:overworld".len();
    non_finite[x_offset..x_offset + 8].copy_from_slice(&f64::NAN.to_bits().to_le_bytes());
    assert_eq!(
        from_bytes::<EntityLeftLevel>(&non_finite)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );

    let mut malformed_utf8 = Vec::new();
    malformed_utf8.extend_from_slice(&1i32.to_le_bytes());
    for value in [ENTITY_ID.as_bytes(), b"minecraft:pig" as &[u8]] {
        malformed_utf8.extend_from_slice(&(value.len() as u32).to_le_bytes());
        malformed_utf8.extend_from_slice(value);
    }
    malformed_utf8.extend_from_slice(&3u32.to_le_bytes());
    malformed_utf8.extend_from_slice(&[0xff, b'a', b'b']);
    for position in [1.0f64, 2.0, 3.0] {
        malformed_utf8.extend_from_slice(&position.to_bits().to_le_bytes());
    }
    assert_eq!(
        from_bytes::<EntityLeftLevel>(&malformed_utf8)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidUtf8
    );
}

#[test]
fn type_implements_stable_codec_traits() {
    fn assert_stable<T: StableEncode + StableDecode>() {}
    assert_stable::<EntityLeftLevel>();
}

#[test]
fn dedicated_server_gametest_guest_fixture_compiles() {
    let _initialize = gametest_fixture::initialize;
}
