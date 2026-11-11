use sjml::storage::{
    StableRecord, StorageScope, MAX_SCHEMA_ID_BYTES, MAX_STORAGE_BATCH_BYTES,
    MAX_STORAGE_KEY_BYTES, MAX_STORAGE_RECORD_BYTES,
};
use sjml::{to_bytes, BlockPos, StableDecode, StableEncode, Uuid};

#[derive(Debug, StableEncode, StableDecode)]
#[sjml(version = 4, id = "contract.record")]
struct ContractRecord {
    value: u32,
}

#[derive(Debug, StableEncode, StableDecode)]
#[sjml(version = 2)]
struct FallbackIdentityRecord {
    value: u8,
}

#[test]
fn stable_decode_derive_exposes_record_identity_without_changing_payload_prefix() {
    assert_eq!(ContractRecord::SCHEMA_VERSION, 4);
    assert_eq!(ContractRecord::schema_id(), "contract.record");
    let bytes = to_bytes(&ContractRecord { value: 11 }).unwrap();
    assert_eq!(u32::from_le_bytes(bytes[..4].try_into().unwrap()), 4);

    assert_eq!(FallbackIdentityRecord::SCHEMA_VERSION, 2);
    assert!(FallbackIdentityRecord::schema_id().contains("FallbackIdentityRecord"));
}

#[test]
fn save_and_dimension_scope_wire_bytes_are_versioned_and_canonical() {
    assert_eq!(to_bytes(&StorageScope::Save).unwrap(), [1, 0, 0, 0, 0]);

    let client_profile = to_bytes(&StorageScope::ClientProfile).unwrap();
    assert_eq!(client_profile, [1, 0, 0, 0, 5]);
    assert_eq!(
        sjml::from_bytes::<StorageScope>(&client_profile).unwrap(),
        StorageScope::ClientProfile
    );

    let dimension = StorageScope::dimension("minecraft:overworld").unwrap();
    assert_eq!(
        to_bytes(&dimension).unwrap(),
        [
            1, 0, 0, 0, 1, // scope schema and dimension kind
            19, 0, 0, 0, // UTF-8 byte length
            b'm', b'i', b'n', b'e', b'c', b'r', b'a', b'f', b't', b':', b'o', b'v', b'e', b'r',
            b'w', b'o', b'r', b'l', b'd',
        ]
    );
}

#[test]
fn player_entity_and_block_entity_scopes_encode_detached_stable_identities() {
    let uuid = Uuid::from_bytes([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
    let player = to_bytes(&StorageScope::Player(uuid)).unwrap();
    let entity = to_bytes(&StorageScope::Entity(uuid)).unwrap();
    assert_eq!(&player[..5], &[1, 0, 0, 0, 2]);
    assert_eq!(&player[5..], uuid.as_bytes());
    assert_eq!(&entity[..5], &[1, 0, 0, 0, 3]);
    assert_eq!(&entity[5..], uuid.as_bytes());

    let block =
        StorageScope::block_entity("example:moon", BlockPos { x: -2, y: 64, z: 9 }).unwrap();
    let bytes = to_bytes(&block).unwrap();
    assert_eq!(&bytes[..5], &[1, 0, 0, 0, 4]);
    let dimension_length_offset = 5;
    assert_eq!(
        u32::from_le_bytes(
            bytes[dimension_length_offset..dimension_length_offset + 4]
                .try_into()
                .unwrap()
        ),
        12
    );
    assert_eq!(bytes.len(), 5 + 4 + 12 + 4 + 12);
    assert_eq!(
        u32::from_le_bytes(bytes[21..25].try_into().unwrap()),
        1,
        "BlockPos keeps its existing stable version prefix"
    );
}

#[test]
fn scopes_reject_noncanonical_dimension_ids_and_record_limits_are_explicit() {
    assert!(StorageScope::dimension("Minecraft:Overworld").is_err());
    assert!(
        StorageScope::block_entity("missing_namespace", BlockPos { x: 0, y: 0, z: 0 }).is_err()
    );
    assert_eq!(MAX_STORAGE_RECORD_BYTES, 1024 * 1024);
    assert_eq!(MAX_STORAGE_BATCH_BYTES, 1024 * 1024);
    assert_eq!(MAX_STORAGE_KEY_BYTES, 256);
    assert_eq!(MAX_SCHEMA_ID_BYTES, 256);
}
