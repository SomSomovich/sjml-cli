//! Bounded reads and base-value updates for selected vanilla living-entity attributes.
//!
//! Dynamic attribute access is deliberately narrower than registry-time attribute declarations:
//! only six vanilla IDs are exposed, reads require `entity.read`, and writes also require
//! `entity.attributes.write`.

#[cfg(target_arch = "wasm32")]
use crate::to_bytes;
use crate::{EntityHandle, ErrorCode, SjmlError, SjmlResult};

#[cfg(any(target_arch = "wasm32", test))]
const SNAPSHOT_BYTES: usize = 20;
#[cfg(target_arch = "wasm32")]
const BASE_VALUE_BYTES: usize = 12;

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
        }
    }

    const fn range(self) -> (f64, f64) {
        match self {
            Self::MaxHealth => (1.0, 1024.0),
            Self::MovementSpeed => (0.0, 1.0),
            Self::AttackDamage | Self::FollowRange => (0.0, 1024.0),
            Self::Armor => (0.0, 30.0),
            Self::KnockbackResistance => (0.0, 1.0),
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
    use super::{decode_values, AttributeValues, LivingAttribute};
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
