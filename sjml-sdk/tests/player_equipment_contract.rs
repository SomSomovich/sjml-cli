use sjml::{from_bytes, player, to_bytes, ErrorCode, PlayerHandle};

#[test]
fn equipment_schema_round_trips_all_six_named_slots() {
    let equipment = player::PlayerEquipment {
        main_hand: Some(
            player::PlayerEquipmentStack::new("minecraft:diamond_sword", 1, 1, 37, 1561).unwrap(),
        ),
        off_hand: None,
        feet: Some(
            player::PlayerEquipmentStack::new("minecraft:iron_boots", 1, 1, 0, 195).unwrap(),
        ),
        legs: None,
        chest: None,
        head: Some(
            player::PlayerEquipmentStack::new("minecraft:carved_pumpkin", 1, 64, 0, 0).unwrap(),
        ),
    };

    let encoded = to_bytes(&equipment).unwrap();
    assert_eq!(
        from_bytes::<player::PlayerEquipment>(&encoded).unwrap(),
        equipment
    );
}

#[test]
fn equipment_stack_constructor_rejects_invalid_ids_sizes_and_durability() {
    for (id, count, max_stack, damage, max_damage) in [
        ("Diamond:Sword", 1, 1, 0, 10),
        ("minecraft:", 1, 1, 0, 10),
        ("minecraft:diamond_sword", 0, 1, 0, 10),
        ("minecraft:diamond_sword", 2, 1, 0, 10),
        ("minecraft:diamond_sword", 1, 4097, 0, 10),
        ("minecraft:diamond_sword", 1, 1, 11, 10),
        ("minecraft:diamond_sword", 1, 1, 0, 1_000_001),
    ] {
        assert_eq!(
            player::PlayerEquipmentStack::new(id, count, max_stack, damage, max_damage)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument,
            "accepted invalid equipment tuple: {id}, {count}, {max_stack}, {damage}, {max_damage}"
        );
    }
}

#[test]
fn native_equipment_read_reports_unsupported_import() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::equipment(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}
