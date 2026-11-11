use sjml::entity_attributes::{self, LivingAttribute};
use sjml::{EntityHandle, ErrorCode};

#[test]
fn public_attribute_ids_and_bounded_base_values_are_stable() {
    assert_eq!(
        LivingAttribute::MaxHealth.id(),
        "minecraft:generic.max_health"
    );
    assert_eq!(
        LivingAttribute::MovementSpeed.id(),
        "minecraft:generic.movement_speed"
    );
    assert_eq!(
        LivingAttribute::AttackDamage.id(),
        "minecraft:generic.attack_damage"
    );
    assert_eq!(LivingAttribute::Armor.id(), "minecraft:generic.armor");
    assert_eq!(
        LivingAttribute::FollowRange.id(),
        "minecraft:generic.follow_range"
    );
    assert_eq!(
        LivingAttribute::KnockbackResistance.id(),
        "minecraft:generic.knockback_resistance"
    );

    let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        entity_attributes::set_base_value(handle, LivingAttribute::MaxHealth, 0.0)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity_attributes::set_base_value(handle, LivingAttribute::MovementSpeed, f64::INFINITY)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity_attributes::set_base_value(handle, LivingAttribute::Armor, 30.01)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_attribute_calls_fail_closed_after_local_validation() {
    let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        entity_attributes::read(handle, LivingAttribute::MaxHealth)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity_attributes::set_base_value(handle, LivingAttribute::MaxHealth, 40.0)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}
