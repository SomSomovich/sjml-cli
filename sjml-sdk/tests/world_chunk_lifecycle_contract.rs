#[path = "../examples/gametest/world_chunk_lifecycle.rs"]
#[allow(dead_code)]
mod gametest_fixture;

use sjml::world_chunk_lifecycle::{
    WorldChunkLifecycle, MAX_WORLD_CHUNK_COORDINATE, MAX_WORLD_CHUNK_DIMENSION_ID_BYTES,
    MAX_WORLD_CHUNK_LIFECYCLE_CALLBACKS, MAX_WORLD_CHUNK_LIFECYCLE_PAYLOAD_BYTES,
    WORLD_CHUNK_LOADED_EVENT_NAME, WORLD_CHUNK_UNLOADED_EVENT_NAME,
};
use sjml::{from_bytes, to_bytes, ErrorCode, StableDecode, StableEncode};

#[test]
fn wire_shape_matches_host_schema_and_round_trips() {
    let event = WorldChunkLifecycle {
        dimension_id: "minecraft:the_nether".to_owned(),
        chunk_x: -12,
        chunk_z: 345,
    };
    let encoded = to_bytes(&event).expect("event should encode");
    let dimension = b"minecraft:the_nether";
    let mut expected = Vec::new();
    expected.extend_from_slice(&1i32.to_le_bytes());
    expected.extend_from_slice(&(dimension.len() as u32).to_le_bytes());
    expected.extend_from_slice(dimension);
    expected.extend_from_slice(&(-12i32).to_le_bytes());
    expected.extend_from_slice(&345i32.to_le_bytes());

    assert_eq!(encoded, expected);
    assert_eq!(from_bytes::<WorldChunkLifecycle>(&encoded).unwrap(), event);
    assert_eq!(WORLD_CHUNK_LOADED_EVENT_NAME, "world.chunk_loaded");
    assert_eq!(WORLD_CHUNK_UNLOADED_EVENT_NAME, "world.chunk_unloaded");
    assert_eq!(MAX_WORLD_CHUNK_LIFECYCLE_CALLBACKS, 64);
    assert_eq!(MAX_WORLD_CHUNK_LIFECYCLE_PAYLOAD_BYTES, 272);
}

#[test]
fn accepts_dimension_and_coordinate_bounds() {
    let event = WorldChunkLifecycle {
        dimension_id: format!("a:{}", "b".repeat(MAX_WORLD_CHUNK_DIMENSION_ID_BYTES - 2)),
        chunk_x: -MAX_WORLD_CHUNK_COORDINATE,
        chunk_z: MAX_WORLD_CHUNK_COORDINATE,
    };
    assert_eq!(
        from_bytes::<WorldChunkLifecycle>(&to_bytes(&event).unwrap()).unwrap(),
        event
    );
}

#[test]
fn rejects_invalid_wire_schemas_strings_and_coordinates() {
    let event = WorldChunkLifecycle {
        dimension_id: "minecraft:overworld".to_owned(),
        chunk_x: 0,
        chunk_z: 0,
    };
    let encoded = to_bytes(&event).unwrap();

    let mut wrong_schema = encoded.clone();
    wrong_schema[..4].copy_from_slice(&2i32.to_le_bytes());
    assert_eq!(
        from_bytes::<WorldChunkLifecycle>(&wrong_schema)
            .unwrap_err()
            .code(),
        ErrorCode::IncompatibleAbiVersion
    );
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(from_bytes::<WorldChunkLifecycle>(&trailing).is_err());

    for invalid in [
        WorldChunkLifecycle {
            dimension_id: "Minecraft:overworld".to_owned(),
            ..event.clone()
        },
        WorldChunkLifecycle {
            dimension_id: "minecraft:".to_owned(),
            ..event.clone()
        },
        WorldChunkLifecycle {
            dimension_id: format!("a:{}", "b".repeat(255)),
            ..event.clone()
        },
        WorldChunkLifecycle {
            chunk_x: MAX_WORLD_CHUNK_COORDINATE + 1,
            ..event.clone()
        },
        WorldChunkLifecycle {
            chunk_z: -MAX_WORLD_CHUNK_COORDINATE - 1,
            ..event.clone()
        },
    ] {
        assert_eq!(
            to_bytes(&invalid).unwrap_err().code(),
            if invalid.dimension_id.len() > MAX_WORLD_CHUNK_DIMENSION_ID_BYTES {
                ErrorCode::PayloadTooLarge
            } else {
                ErrorCode::InvalidArgument
            }
        );
    }

    let mut malformed_utf8 = Vec::from(1i32.to_le_bytes());
    malformed_utf8.extend_from_slice(&1u32.to_le_bytes());
    malformed_utf8.push(0xff);
    malformed_utf8.extend_from_slice(&0i32.to_le_bytes());
    malformed_utf8.extend_from_slice(&0i32.to_le_bytes());
    assert_eq!(
        from_bytes::<WorldChunkLifecycle>(&malformed_utf8)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidUtf8
    );
}

#[test]
fn type_exposes_stable_codec_traits() {
    fn assert_stable<T: StableEncode + StableDecode>() {}
    assert_stable::<WorldChunkLifecycle>();
}

#[test]
fn dedicated_server_gametest_guest_fixture_compiles() {
    let _initialize = gametest_fixture::initialize;
}
