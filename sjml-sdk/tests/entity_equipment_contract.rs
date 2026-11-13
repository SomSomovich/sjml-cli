use sjml::{entity, from_bytes, to_bytes, EntityHandle, ErrorCode};

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_equipment_read_reports_unsupported_import() {
    let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        entity::equipment(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}

#[test]
fn equipment_schema_round_trips_the_six_slots_in_fixed_order() {
    let equipment = entity::EntityEquipment {
        main_hand: Some(
            entity::EntityEquipmentStack::new("minecraft:iron_sword", 1, 12, 250).unwrap(),
        ),
        off_hand: None,
        head: Some(entity::EntityEquipmentStack::new("minecraft:carved_pumpkin", 1, 0, 0).unwrap()),
        chest: None,
        legs: None,
        feet: Some(entity::EntityEquipmentStack::new("minecraft:iron_boots", 1, 4, 195).unwrap()),
    };

    let encoded = to_bytes(&equipment).expect("encode equipment schema");
    assert_eq!(
        from_bytes::<entity::EntityEquipment>(&encoded).expect("decode equipment schema"),
        equipment
    );
    assert_eq!(&encoded[..4], &1_u32.to_le_bytes());
    assert_eq!(entity::MAX_ENTITY_EQUIPMENT_RESPONSE_BYTES, 1666);
}

#[test]
fn equipment_stack_constructor_rejects_bad_ids_counts_and_durability() {
    for (id, count, damage, max_damage) in [
        ("Minecraft:iron_sword", 1, 0, 250),
        ("minecraft:", 1, 0, 250),
        ("minecraft:iron_sword", 0, 0, 250),
        ("minecraft:iron_sword", 4097, 0, 250),
        ("minecraft:iron_sword", 1, 251, 250),
        ("minecraft:iron_sword", 1, 0, 1_000_001),
    ] {
        assert_eq!(
            entity::EntityEquipmentStack::new(id, count, damage, max_damage)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "accepted invalid equipment tuple: {id}, {count}, {damage}, {max_damage}"
        );
    }
}

#[test]
fn equipment_wire_bytes_match_host_schema_v1() {
    let equipment = entity::EntityEquipment {
        main_hand: None,
        off_hand: Some(entity::EntityEquipmentStack::new("minecraft:stone", 2, 0, 0).unwrap()),
        head: None,
        chest: None,
        legs: None,
        feet: None,
    };
    let mut expected = 1_u32.to_le_bytes().to_vec();
    expected.push(0);
    expected.push(1);
    expected.extend_from_slice(&1_u32.to_le_bytes());
    expected.extend_from_slice(&15_u32.to_le_bytes());
    expected.extend_from_slice(b"minecraft:stone");
    expected.extend_from_slice(&2_u32.to_le_bytes());
    expected.extend_from_slice(&0_u32.to_le_bytes());
    expected.extend_from_slice(&0_u32.to_le_bytes());
    expected.extend_from_slice(&[0, 0, 0, 0]);

    assert_eq!(to_bytes(&equipment).unwrap(), expected);
}
