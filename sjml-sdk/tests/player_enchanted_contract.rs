use sjml::{
    events::{
        PlayerEnchanted, PlayerEnchantedEnchantment, MAX_PLAYER_ENCHANTED_ENCHANTMENTS,
        MAX_PLAYER_ENCHANTED_ID_BYTES, MAX_PLAYER_ENCHANTED_PAYLOAD_BYTES,
        PLAYER_ENCHANTED_EVENT_NAME, PLAYER_ENCHANTMENTS_OBSERVE_CAPABILITY,
    },
    from_bytes, to_bytes, ErrorCode,
};

const PLAYER_ID: &str = "00000000-0000-0000-0000-000000000001";

fn event() -> PlayerEnchanted {
    PlayerEnchanted {
        player_id: PLAYER_ID.into(),
        item_id: "minecraft:diamond_sword".into(),
        count: 1,
        damage: 29,
        max_damage: 1_561,
        enchantments: vec![
            PlayerEnchantedEnchantment {
                id: "minecraft:sharpness".into(),
                level: 3,
            },
            PlayerEnchantedEnchantment {
                id: "minecraft:unbreaking".into(),
                level: 2,
            },
        ],
    }
}

#[test]
fn schema_matches_the_host_wire_format_and_round_trips() {
    let event = event();
    let mut expected = 1_u32.to_le_bytes().to_vec();
    for value in [PLAYER_ID, "minecraft:diamond_sword"] {
        expected.extend_from_slice(&(value.len() as u32).to_le_bytes());
        expected.extend_from_slice(value.as_bytes());
    }
    expected.extend_from_slice(&1_u32.to_le_bytes());
    expected.extend_from_slice(&29_u32.to_le_bytes());
    expected.extend_from_slice(&1_561_u32.to_le_bytes());
    expected.extend_from_slice(&2_u32.to_le_bytes());
    for (id, level) in [("minecraft:sharpness", 3_u32), ("minecraft:unbreaking", 2)] {
        expected.extend_from_slice(&(id.len() as u32).to_le_bytes());
        expected.extend_from_slice(id.as_bytes());
        expected.extend_from_slice(&level.to_le_bytes());
    }

    assert_eq!(to_bytes(&event).unwrap(), expected);
    assert_eq!(from_bytes::<PlayerEnchanted>(&expected).unwrap(), event);
    assert_eq!(MAX_PLAYER_ENCHANTED_ENCHANTMENTS, 32);
    assert_eq!(MAX_PLAYER_ENCHANTED_ID_BYTES, 256);
    assert_eq!(MAX_PLAYER_ENCHANTED_PAYLOAD_BYTES, 16 * 1024);
    assert_eq!(PLAYER_ENCHANTED_EVENT_NAME, "player.enchanted");
    assert_eq!(
        PLAYER_ENCHANTMENTS_OBSERVE_CAPABILITY,
        "player.enchantments.observe"
    );
}

#[test]
fn schema_rejects_invalid_values_duplicates_wrong_version_and_trailing_bytes() {
    let base = event();
    for invalid in [
        PlayerEnchanted {
            player_id: "not-a-uuid".into(),
            ..base.clone()
        },
        PlayerEnchanted {
            item_id: "Minecraft:diamond_sword".into(),
            ..base.clone()
        },
        PlayerEnchanted {
            count: 0,
            ..base.clone()
        },
        PlayerEnchanted {
            damage: 1_562,
            ..base.clone()
        },
        PlayerEnchanted {
            enchantments: vec![PlayerEnchantedEnchantment {
                id: "minecraft:sharpness".into(),
                level: 0,
            }],
            ..base.clone()
        },
        PlayerEnchanted {
            enchantments: vec![base.enchantments[0].clone(), base.enchantments[0].clone()],
            ..base.clone()
        },
        PlayerEnchanted {
            enchantments: Vec::new(),
            ..base.clone()
        },
    ] {
        assert_eq!(
            to_bytes(&invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    let oversized = PlayerEnchanted {
        item_id: format!("minecraft:{}", "x".repeat(MAX_PLAYER_ENCHANTED_ID_BYTES)),
        ..base
    };
    assert_eq!(
        to_bytes(&oversized).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );

    let mut wrong_version = to_bytes(&event()).unwrap();
    wrong_version[..4].copy_from_slice(&2_u32.to_le_bytes());
    assert_eq!(
        from_bytes::<PlayerEnchanted>(&wrong_version)
            .unwrap_err()
            .code(),
        ErrorCode::IncompatibleAbiVersion
    );
    let mut trailing = to_bytes(&event()).unwrap();
    trailing.push(0);
    assert_eq!(
        from_bytes::<PlayerEnchanted>(&trailing).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
}
