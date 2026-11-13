//! Bounded reads and base-value updates for vanilla living-entity attributes on loaded
//! non-player entities and live server players.
//!
//! Dynamic attribute access is deliberately narrower than arbitrary registry access:
//! vanilla IDs exposed by [`LivingAttribute`] require `entity.read`, and writes also require
//! `entity.attributes.write`.

use crate::registry::AttributeModifierOperation;
#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
use crate::{EntityHandle, ErrorCode, PlayerHandle, SjmlError, SjmlResult};

#[cfg(any(target_arch = "wasm32", test))]
const SNAPSHOT_BYTES: usize = 20;
#[cfg(target_arch = "wasm32")]
const BASE_VALUE_BYTES: usize = 12;
#[cfg(any(target_arch = "wasm32", test))]
const MODIFIER_REQUEST_BYTES: usize = 16;
#[cfg(any(target_arch = "wasm32", test))]
const MODIFIER_REMOVE_RESULT_BYTES: usize = 5;

/// Maximum absolute amount accepted by [`add_transient_modifier`].
pub const MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT: f64 = 1_024.0;
/// Maximum length of one module-owned modifier key.
pub const MAX_ATTRIBUTE_MODIFIER_KEY_BYTES: usize = 64;

/// A vanilla living-entity attribute with a stable registry ID and bounded writable base value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivingAttribute {
    /// `minecraft:generic.max_health`, in 1–1024 health points.
    MaxHealth,
    /// `minecraft:generic.movement_speed`, in 0–1 blocks per tick.
    MovementSpeed,
    /// `minecraft:generic.attack_damage`, in 0–1024 damage points.
    AttackDamage,
    /// `minecraft:generic.armor`, in 0–30 armor points.
    Armor,
    /// `minecraft:generic.follow_range`, in 0–1024 blocks.
    FollowRange,
    /// `minecraft:generic.knockback_resistance`, in 0–1.
    KnockbackResistance,
    /// `minecraft:generic.armor_toughness`, in 0–20.
    ArmorToughness,
    /// `minecraft:generic.attack_knockback`, in 0–5.
    AttackKnockback,
    /// `minecraft:generic.attack_speed`, in 0–1024.
    AttackSpeed,
    /// `minecraft:player.block_break_speed`, in 0–1024.
    BlockBreakSpeed,
    /// `minecraft:player.block_interaction_range`, in 0–64 blocks.
    BlockInteractionRange,
    /// `minecraft:generic.burning_time`, in 0–1024.
    BurningTime,
    /// `minecraft:generic.explosion_knockback_resistance`, in 0–1.
    ExplosionKnockbackResistance,
    /// `minecraft:player.entity_interaction_range`, in 0–64 blocks.
    EntityInteractionRange,
    /// `minecraft:generic.fall_damage_multiplier`, in 0–100.
    FallDamageMultiplier,
    /// `minecraft:generic.flying_speed`, in 0–1024.
    FlyingSpeed,
    /// `minecraft:generic.gravity`, in -1–1.
    Gravity,
    /// `minecraft:generic.jump_strength`, in 0–32.
    JumpStrength,
    /// `minecraft:generic.luck`, in -1024–1024.
    Luck,
    /// `minecraft:generic.max_absorption`, in 0–2048.
    MaxAbsorption,
    /// `minecraft:player.mining_efficiency`, in 0–1024.
    MiningEfficiency,
    /// `minecraft:generic.movement_efficiency`, in 0–1.
    MovementEfficiency,
    /// `minecraft:generic.oxygen_bonus`, in 0–1024.
    OxygenBonus,
    /// `minecraft:generic.safe_fall_distance`, in -1024–1024 blocks.
    SafeFallDistance,
    /// `minecraft:generic.scale`, in 0.0625–16.
    Scale,
    /// `minecraft:player.sneaking_speed`, in 0–1.
    SneakingSpeed,
    /// `minecraft:zombie.spawn_reinforcements`, in 0–1.
    SpawnReinforcementsChance,
    /// `minecraft:generic.step_height`, in 0–10 blocks.
    StepHeight,
    /// `minecraft:player.submerged_mining_speed`, in 0–20.
    SubmergedMiningSpeed,
    /// `minecraft:player.sweeping_damage_ratio`, in 0–1.
    SweepingDamageRatio,
    /// `minecraft:generic.water_movement_efficiency`, in 0–1.
    WaterMovementEfficiency,
}

impl LivingAttribute {
    /// Returns the stable vanilla registry ID used by the host ABI.
    pub const fn id(self) -> &'static str {
        match self {
            Self::MaxHealth => "minecraft:generic.max_health",
            Self::MovementSpeed => "minecraft:generic.movement_speed",
            Self::AttackDamage => "minecraft:generic.attack_damage",
            Self::Armor => "minecraft:generic.armor",
            Self::FollowRange => "minecraft:generic.follow_range",
            Self::KnockbackResistance => "minecraft:generic.knockback_resistance",
            Self::ArmorToughness => "minecraft:generic.armor_toughness",
            Self::AttackKnockback => "minecraft:generic.attack_knockback",
            Self::AttackSpeed => "minecraft:generic.attack_speed",
            Self::BlockBreakSpeed => "minecraft:player.block_break_speed",
            Self::BlockInteractionRange => "minecraft:player.block_interaction_range",
            Self::BurningTime => "minecraft:generic.burning_time",
            Self::ExplosionKnockbackResistance => {
                "minecraft:generic.explosion_knockback_resistance"
            }
            Self::EntityInteractionRange => "minecraft:player.entity_interaction_range",
            Self::FallDamageMultiplier => "minecraft:generic.fall_damage_multiplier",
            Self::FlyingSpeed => "minecraft:generic.flying_speed",
            Self::Gravity => "minecraft:generic.gravity",
            Self::JumpStrength => "minecraft:generic.jump_strength",
            Self::Luck => "minecraft:generic.luck",
            Self::MaxAbsorption => "minecraft:generic.max_absorption",
            Self::MiningEfficiency => "minecraft:player.mining_efficiency",
            Self::MovementEfficiency => "minecraft:generic.movement_efficiency",
            Self::OxygenBonus => "minecraft:generic.oxygen_bonus",
            Self::SafeFallDistance => "minecraft:generic.safe_fall_distance",
            Self::Scale => "minecraft:generic.scale",
            Self::SneakingSpeed => "minecraft:player.sneaking_speed",
            Self::SpawnReinforcementsChance => "minecraft:zombie.spawn_reinforcements",
            Self::StepHeight => "minecraft:generic.step_height",
            Self::SubmergedMiningSpeed => "minecraft:player.submerged_mining_speed",
            Self::SweepingDamageRatio => "minecraft:player.sweeping_damage_ratio",
            Self::WaterMovementEfficiency => "minecraft:generic.water_movement_efficiency",
        }
    }

    const fn range(self) -> (f64, f64) {
        match self {
            Self::MaxHealth => (1.0, 1024.0),
            Self::MovementSpeed => (0.0, 1.0),
            Self::AttackDamage | Self::FollowRange => (0.0, 1024.0),
            Self::Armor => (0.0, 30.0),
            Self::KnockbackResistance => (0.0, 1.0),
            Self::ArmorToughness => (0.0, 20.0),
            Self::AttackKnockback => (0.0, 5.0),
            Self::AttackSpeed | Self::BlockBreakSpeed | Self::MiningEfficiency => (0.0, 1024.0),
            Self::BlockInteractionRange | Self::EntityInteractionRange => (0.0, 64.0),
            Self::BurningTime => (0.0, 1024.0),
            Self::ExplosionKnockbackResistance
            | Self::MovementEfficiency
            | Self::SneakingSpeed
            | Self::SpawnReinforcementsChance
            | Self::SweepingDamageRatio
            | Self::WaterMovementEfficiency => (0.0, 1.0),
            Self::FallDamageMultiplier => (0.0, 100.0),
            Self::FlyingSpeed | Self::OxygenBonus => (0.0, 1024.0),
            Self::Gravity => (-1.0, 1.0),
            Self::JumpStrength => (0.0, 32.0),
            Self::Luck => (-1024.0, 1024.0),
            Self::MaxAbsorption => (0.0, 2048.0),
            Self::SafeFallDistance => (-1024.0, 1024.0),
            Self::Scale => (0.0625, 16.0),
            Self::StepHeight => (0.0, 10.0),
            Self::SubmergedMiningSpeed => (0.0, 20.0),
        }
    }

    fn validate_base(self, value: f64) -> SjmlResult<()> {
        let (minimum, maximum) = self.range();
        if !value.is_finite() || value < minimum || value > maximum {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!(
                    "base value is outside the supported range for {}",
                    self.id()
                ),
            ));
        }
        Ok(())
    }
}

/// A detached attribute snapshot. `value` includes active vanilla and modded modifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeValues {
    /// The directly stored base value.
    pub base_value: f64,
    /// Current effective value after all active modifiers.
    pub value: f64,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_entity_v1")]
extern "C" {
    #[link_name = "attribute"]
    fn imported_attribute(
        handle: i32,
        handle_length: i32,
        attribute_id: i32,
        attribute_id_length: i32,
        output: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "set_attribute_base"]
    fn imported_set_attribute_base(
        handle: i32,
        handle_length: i32,
        attribute_id: i32,
        attribute_id_length: i32,
        value: i32,
        value_length: i32,
    ) -> i32;
    #[link_name = "add_attribute_modifier"]
    fn imported_add_attribute_modifier(
        handle: i32,
        handle_length: i32,
        attribute_id: i32,
        attribute_id_length: i32,
        key: i32,
        key_length: i32,
        request: i32,
        request_length: i32,
    ) -> i32;
    #[link_name = "remove_attribute_modifier"]
    fn imported_remove_attribute_modifier(
        handle: i32,
        handle_length: i32,
        key: i32,
        key_length: i32,
        output: i32,
        output_capacity: i32,
    ) -> i32;
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_player_v1")]
extern "C" {
    #[link_name = "attribute"]
    fn imported_player_attribute(
        handle: i32,
        handle_length: i32,
        attribute_id: i32,
        attribute_id_length: i32,
        output: i32,
        output_capacity: i32,
    ) -> i32;
    #[link_name = "set_attribute_base"]
    fn imported_set_player_attribute_base(
        handle: i32,
        handle_length: i32,
        attribute_id: i32,
        attribute_id_length: i32,
        value: i32,
        value_length: i32,
    ) -> i32;
}

/// Reads base and effective values for one loaded living non-player entity.
///
/// The host validates the entity session, owning server thread, and `entity.read`. The attribute
/// must exist on the selected living entity and belong to [`LivingAttribute`]'s fixed allowlist.
pub fn read(handle: EntityHandle, attribute: LivingAttribute) -> SjmlResult<AttributeValues> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let mut output = [0_u8; SNAPSHOT_BYTES];
        // SAFETY: pointers reference live fixed-size local buffers for this synchronous host call.
        let status = unsafe {
            imported_attribute(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                attribute.id().as_ptr() as usize as u32 as i32,
                attribute.id().len() as u32 as i32,
                output.as_mut_ptr() as usize as u32 as i32,
                output.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity attribute read")?;
        decode_values(&output)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, attribute);
        Err(unsupported_target())
    }
}

/// Replaces an allowlisted attribute's base value on a loaded living non-player entity.
///
/// The module must request and receive both `entity.read` and `entity.attributes.write`. The host
/// rechecks the live handle and server thread for every call. This changes only the base value;
/// temporary effects and other attribute modifiers remain in effect.
pub fn set_base_value(
    handle: EntityHandle,
    attribute: LivingAttribute,
    base_value: f64,
) -> SjmlResult<()> {
    attribute.validate_base(base_value)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_value = encode_base_value(base_value);
        // SAFETY: both buffers stay live during the synchronous call; the host copies them before returning.
        let status = unsafe {
            imported_set_attribute_base(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                attribute.id().as_ptr() as usize as u32 as i32,
                attribute.id().len() as u32 as i32,
                encoded_value.as_ptr() as usize as u32 as i32,
                encoded_value.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity attribute mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Adds an owner-scoped transient vanilla modifier to one loaded living non-player entity.
///
/// The attribute must be in [`LivingAttribute`]'s allowlist. `key` identifies the modifier only
/// inside this module and entity, and must match `[a-z0-9_.-]{1,64}`. A duplicate key is rejected.
/// The modifier is not saved to entity NBT and is removed when the module shuts down, the entity
/// leaves its level, or [`remove_transient_modifier`] is called. The module must request and
/// receive both `entity.read` and `entity.attributes.write`.
pub fn add_transient_modifier(
    handle: EntityHandle,
    attribute: LivingAttribute,
    key: &str,
    amount: f64,
    operation: AttributeModifierOperation,
) -> SjmlResult<()> {
    validate_modifier_key(key)?;
    validate_modifier_amount(amount)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let request = encode_modifier_request(amount, operation);
        // SAFETY: all borrowed strings and fixed-size buffers stay live during the synchronous call.
        let status = unsafe {
            imported_add_attribute_modifier(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                attribute.id().as_ptr() as usize as u32 as i32,
                attribute.id().len() as u32 as i32,
                key.as_ptr() as usize as u32 as i32,
                key.len() as u32 as i32,
                request.as_ptr() as usize as u32 as i32,
                request.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity attribute modifier")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, attribute, operation);
        Err(unsupported_target())
    }
}

/// Removes a module-owned transient modifier by key from one loaded living non-player entity.
///
/// Returns `true` if the module-owned vanilla modifier was present and removed. Other modules'
/// modifiers cannot be removed. Calls require both `entity.read` and `entity.attributes.write`.
pub fn remove_transient_modifier(handle: EntityHandle, key: &str) -> SjmlResult<bool> {
    validate_modifier_key(key)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let mut output = [0_u8; MODIFIER_REMOVE_RESULT_BYTES];
        // SAFETY: pointers reference live fixed-size buffers for this synchronous host call.
        let status = unsafe {
            imported_remove_attribute_modifier(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                key.as_ptr() as usize as u32 as i32,
                key.len() as u32 as i32,
                output.as_mut_ptr() as usize as u32 as i32,
                output.len() as u32 as i32,
            )
        };
        status_result(
            status,
            "host rejected the entity attribute modifier removal",
        )?;
        decode_modifier_remove_result(&output)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, key);
        Err(unsupported_target())
    }
}

fn validate_modifier_key(key: &str) -> SjmlResult<()> {
    if key.is_empty()
        || key.len() > MAX_ATTRIBUTE_MODIFIER_KEY_BYTES
        || !key.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity attribute modifier keys must match [a-z0-9_.-]{1,64}",
        ));
    }
    Ok(())
}

fn validate_modifier_amount(amount: f64) -> SjmlResult<()> {
    if !amount.is_finite() || amount.abs() > MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity attribute modifier amount must be finite and within -1,024..=1,024",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn encode_modifier_request(
    amount: f64,
    operation: AttributeModifierOperation,
) -> [u8; MODIFIER_REQUEST_BYTES] {
    let mut encoded = [0_u8; MODIFIER_REQUEST_BYTES];
    encoded[..4].copy_from_slice(&1_u32.to_le_bytes());
    encoded[4..12].copy_from_slice(&amount.to_le_bytes());
    encoded[12..].copy_from_slice(&(operation as i32).to_le_bytes());
    encoded
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_modifier_remove_result(bytes: &[u8; MODIFIER_REMOVE_RESULT_BYTES]) -> SjmlResult<bool> {
    if u32::from_le_bytes(bytes[..4].try_into().expect("fixed header")) != 1 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "unsupported entity attribute modifier removal schema",
        ));
    }
    match bytes[4] {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned an invalid entity attribute modifier removal result",
        )),
    }
}

/// Reads base and effective values for one live server player.
///
/// Requires `entity.read` and a logical-server module context. The host validates the active
/// player session and server thread for every call; client and static-registration contexts are
/// denied. The supported attribute IDs and value bounds are the same as [`LivingAttribute`].
pub fn read_player(
    handle: PlayerHandle,
    attribute: LivingAttribute,
) -> SjmlResult<AttributeValues> {
    #[cfg(target_arch = "wasm32")]
    {
        let encoded = to_bytes(&handle)?;
        let mut output = [0_u8; SNAPSHOT_BYTES];
        // SAFETY: pointers reference live fixed-size local buffers for this synchronous host call.
        let status = unsafe {
            imported_player_attribute(
                encoded.as_ptr() as usize as u32 as i32,
                encoded.len() as u32 as i32,
                attribute.id().as_ptr() as usize as u32 as i32,
                attribute.id().len() as u32 as i32,
                output.as_mut_ptr() as usize as u32 as i32,
                output.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the player attribute read")?;
        decode_values(&output)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, attribute);
        Err(unsupported_target())
    }
}

/// Replaces an allowlisted attribute's base value on one live server player.
///
/// The module must request and receive both `entity.read` and `entity.attributes.write`. The host
/// rechecks the live login session and server thread for every call. This updates only the base
/// value; active effects and other attribute modifiers remain in effect.
pub fn set_player_base_value(
    handle: PlayerHandle,
    attribute: LivingAttribute,
    base_value: f64,
) -> SjmlResult<()> {
    attribute.validate_base(base_value)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = to_bytes(&handle)?;
        let encoded_value = encode_base_value(base_value);
        // SAFETY: both buffers stay live during the synchronous call; the host copies them before returning.
        let status = unsafe {
            imported_set_player_attribute_base(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                attribute.id().as_ptr() as usize as u32 as i32,
                attribute.id().len() as u32 as i32,
                encoded_value.as_ptr() as usize as u32 as i32,
                encoded_value.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the player attribute mutation")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

#[cfg(target_arch = "wasm32")]
fn encode_base_value(value: f64) -> [u8; BASE_VALUE_BYTES] {
    let mut encoded = [0_u8; BASE_VALUE_BYTES];
    encoded[..4].copy_from_slice(&1_u32.to_le_bytes());
    encoded[4..].copy_from_slice(&value.to_le_bytes());
    encoded
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_values(bytes: &[u8; SNAPSHOT_BYTES]) -> SjmlResult<AttributeValues> {
    if u32::from_le_bytes(bytes[..4].try_into().expect("fixed header")) != 1 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "unsupported entity attribute response schema",
        ));
    }
    let values = AttributeValues {
        base_value: f64::from_le_bytes(bytes[4..12].try_into().expect("fixed base value")),
        value: f64::from_le_bytes(bytes[12..20].try_into().expect("fixed effective value")),
    };
    if !values.base_value.is_finite() || !values.value.is_finite() {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "host returned non-finite entity attribute values",
        ));
    }
    Ok(values)
}

#[cfg(target_arch = "wasm32")]
fn status_result(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "entity attribute imports require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        decode_modifier_remove_result, decode_values, encode_modifier_request,
        validate_modifier_amount, validate_modifier_key, AttributeValues, LivingAttribute,
        MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT,
    };
    use crate::registry::AttributeModifierOperation;
    use crate::{EntityHandle, ErrorCode};

    #[test]
    fn attribute_ids_are_stable_and_base_values_are_bounded_before_host_access() {
        assert_eq!(
            LivingAttribute::MaxHealth.id(),
            "minecraft:generic.max_health"
        );
        assert_eq!(
            LivingAttribute::MovementSpeed.id(),
            "minecraft:generic.movement_speed"
        );
        assert!(LivingAttribute::MaxHealth.validate_base(20.0).is_ok());
        assert!(LivingAttribute::MaxHealth.validate_base(0.99).is_err());
        assert!(LivingAttribute::MovementSpeed
            .validate_base(f64::NAN)
            .is_err());
        assert!(LivingAttribute::Armor.validate_base(30.01).is_err());
        assert!(LivingAttribute::KnockbackResistance
            .validate_base(0.75)
            .is_ok());

        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            super::set_base_value(handle, LivingAttribute::MaxHealth, 20.0)
                .unwrap_err()
                .code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn transient_modifier_keys_and_amounts_are_bounded() {
        assert!(validate_modifier_key("speed_boost.v1").is_ok());
        assert!(validate_modifier_key(&"a".repeat(64)).is_ok());
        assert!(validate_modifier_key("").is_err());
        assert!(validate_modifier_key("UpperCase").is_err());
        assert!(validate_modifier_key("path/name").is_err());
        assert!(validate_modifier_key(&"a".repeat(65)).is_err());

        assert!(validate_modifier_amount(MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT).is_ok());
        assert!(validate_modifier_amount(-MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT).is_ok());
        assert!(validate_modifier_amount(f64::NAN).is_err());
        assert!(validate_modifier_amount(f64::INFINITY).is_err());
        assert!(validate_modifier_amount(MAX_TRANSIENT_ATTRIBUTE_MODIFIER_AMOUNT + 0.01).is_err());
    }

    #[test]
    fn transient_modifier_remove_result_requires_schema_and_a_stable_boolean() {
        assert!(decode_modifier_remove_result(&[1, 0, 0, 0, 1]).unwrap());
        assert!(!decode_modifier_remove_result(&[1, 0, 0, 0, 0]).unwrap());
        assert!(decode_modifier_remove_result(&[2, 0, 0, 0, 1]).is_err());
        assert!(decode_modifier_remove_result(&[1, 0, 0, 0, 2]).is_err());
    }

    #[test]
    fn transient_modifier_request_matches_schema_v1_little_endian_bytes() {
        let encoded = encode_modifier_request(0.5, AttributeModifierOperation::AddMultipliedBase);
        let mut expected = 1_u32.to_le_bytes().to_vec();
        expected.extend_from_slice(&0.5_f64.to_le_bytes());
        expected.extend_from_slice(&1_i32.to_le_bytes());
        assert_eq!(encoded.as_slice(), expected);
    }

    #[test]
    fn snapshot_decoder_requires_schema_one_and_finite_values() {
        let mut bytes = [0_u8; 20];
        bytes[..4].copy_from_slice(&1_u32.to_le_bytes());
        bytes[4..12].copy_from_slice(&32.0_f64.to_le_bytes());
        bytes[12..20].copy_from_slice(&40.0_f64.to_le_bytes());
        assert_eq!(
            decode_values(&bytes).unwrap(),
            AttributeValues {
                base_value: 32.0,
                value: 40.0
            }
        );

        bytes[..4].copy_from_slice(&2_u32.to_le_bytes());
        assert_eq!(
            decode_values(&bytes).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        bytes[..4].copy_from_slice(&1_u32.to_le_bytes());
        bytes[4..12].copy_from_slice(&f64::INFINITY.to_le_bytes());
        assert_eq!(
            decode_values(&bytes).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}
