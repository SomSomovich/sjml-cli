//! Bounded reads and increments for vanilla registry-backed player statistic families.
//!
//! The host resolves every target ID against the registry selected by its family. A valid ID in
//! another registry does not make an invalid item/block/entity target valid.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::entity::status_result;
#[cfg(target_arch = "wasm32")]
use crate::{to_bytes, Decoder, StableDecode};
use crate::{ErrorCode, PlayerHandle, SjmlError, SjmlResult};

/// Maximum amount one call can add to a registry-backed vanilla stat.
pub const MAX_INCREMENT: u32 = 1_000_000;
/// Maximum UTF-8 byte length of a canonical registry target ID.
pub const MAX_STATISTIC_ID_BYTES: usize = 256;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_player_statistics_v1")]
extern "C" {
    #[link_name = "read_registry_stat"]
    fn imported_read_registry_stat(
        player: i32,
        player_length: i32,
        family: i32,
        statistic_id: i32,
        statistic_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "increment_registry_stat"]
    fn imported_increment_registry_stat(
        player: i32,
        player_length: i32,
        family: i32,
        statistic_id: i32,
        statistic_id_length: i32,
        amount: i32,
    ) -> i32;
}

/// Supported vanilla registry-backed player statistic families.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum StatisticFamily {
    /// Number of times a registered block type was mined.
    BlockMined = 0,
    /// Number of times a registered item type was used.
    ItemUsed = 1,
    /// Number of times a registered item type was broken.
    ItemBroken = 2,
    /// Number of times a registered item type was crafted.
    ItemCrafted = 3,
    /// Number of times a registered entity type was killed by this player.
    EntityKilled = 4,
    /// Number of times this player was killed by a registered entity type.
    EntityKilledBy = 5,
}

/// One registered target in a vanilla statistic family.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlayerStatistic {
    pub family: StatisticFamily,
    pub registry_id: String,
}

impl PlayerStatistic {
    /// Creates a statistic target after checking the canonical bounded `namespace:path` syntax.
    pub fn new(family: StatisticFamily, registry_id: impl Into<String>) -> SjmlResult<Self> {
        let registry_id = registry_id.into();
        validate_registry_id(&registry_id)?;
        Ok(Self {
            family,
            registry_id,
        })
    }
}

/// Reads one supported vanilla registry-backed statistic for a live online server player.
///
/// Requires `entity.read`. Registry IDs are checked against the registry chosen by `family`; for
/// example, `ITEM_USED` requires an item ID, while `BLOCK_MINED` requires a block ID.
pub fn read(player: PlayerHandle, statistic: &PlayerStatistic) -> SjmlResult<u32> {
    validate_registry_id(&statistic.registry_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_player = to_bytes(&player)?;
        let id = statistic.registry_id.as_bytes();
        let bytes = read_output(4, |output, capacity| {
            // SAFETY: the encoded handle, ID, and output span stay live for this synchronous server import.
            unsafe {
                imported_read_registry_stat(
                    encoded_player.as_ptr() as usize as u32 as i32,
                    encoded_player.len() as i32,
                    statistic.family as i32,
                    id.as_ptr() as usize as u32 as i32,
                    id.len() as i32,
                    output,
                    capacity,
                )
            }
        })?;
        if bytes > i32::MAX as u32 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned a statistic value outside Minecraft's signed integer range",
            ));
        }
        Ok(bytes)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = player;
        Err(unsupported_target())
    }
}

/// Adds a bounded nonnegative amount to one supported vanilla registry-backed statistic.
///
/// Requires `entity.read` and `player.statistics.write`. Zero is a validated no-op; positive
/// amounts use Minecraft's `awardStat` path and cannot overflow its signed 32-bit counter.
pub fn increment(player: PlayerHandle, statistic: &PlayerStatistic, amount: u32) -> SjmlResult<()> {
    validate_registry_id(&statistic.registry_id)?;
    if amount > MAX_INCREMENT {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player statistic increment must be between 0 and 1000000",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_player = to_bytes(&player)?;
        let id = statistic.registry_id.as_bytes();
        // SAFETY: handle and target bytes are immutable throughout this synchronous server call.
        let status = unsafe {
            imported_increment_registry_stat(
                encoded_player.as_ptr() as usize as u32 as i32,
                encoded_player.len() as i32,
                statistic.family as i32,
                id.as_ptr() as usize as u32 as i32,
                id.len() as i32,
                amount as i32,
            )
        };
        status_result(status, "host rejected the player statistic increment")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = player;
        Err(unsupported_target())
    }
}

fn validate_registry_id(value: &str) -> SjmlResult<()> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(invalid_registry_id());
    };
    if value.len() < 3
        || value.len() > MAX_STATISTIC_ID_BYTES
        || namespace.is_empty()
        || path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains("//")
        || value.bytes().filter(|byte| *byte == b':').count() != 1
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        })
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-/".contains(&byte)
        })
        || path.split('/').any(|part| part == "." || part == "..")
    {
        return Err(invalid_registry_id());
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn read_output(capacity: u32, invoke: impl FnOnce(i32, i32) -> i32) -> SjmlResult<u32> {
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate player statistic output",
        ));
    }
    // SAFETY: initialize the full allocation before the host writes its four-byte value.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: output remains live and initialized until the four-byte stable value is decoded.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        Decoder::new(bytes).and_then(|mut decoder| u32::decode(&mut decoder))
    } else {
        Err(crate::SjmlError::from_abi_code(
            status as u32,
            "host rejected the player statistic read",
        ))
    };
    // SAFETY: release the exact allocation after host access and decoding are complete.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

fn invalid_registry_id() -> SjmlError {
    SjmlError::new(
        ErrorCode::InvalidArgument,
        "player statistic ID must be a canonical namespace:path value of 3–256 bytes",
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "registry-backed player statistics require the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{validate_registry_id, PlayerStatistic, StatisticFamily, MAX_INCREMENT};
    use crate::{ErrorCode, PlayerHandle};

    #[test]
    fn registry_id_validation_accepts_canonical_targets_and_rejects_malformed_values() {
        for id in ["minecraft:stone", "mod_name:deep/path/item-1.2"] {
            assert!(validate_registry_id(id).is_ok(), "{id}");
        }
        for id in [
            "minecraft",
            ":stone",
            "minecraft:",
            "Minecraft:stone",
            "minecraft:stone/",
            "minecraft:../stone",
            "minecraft:a//b",
            "minecraft:stone:extra",
            "minecraft:é",
        ] {
            assert_eq!(
                validate_registry_id(id).unwrap_err().code(),
                ErrorCode::InvalidArgument,
                "{id}"
            );
        }
        let long_id = format!("minecraft:{}", "a".repeat(247));
        assert_eq!(
            validate_registry_id(&long_id).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn families_keep_the_host_wire_codes_and_target_metadata() {
        let families = [
            StatisticFamily::BlockMined,
            StatisticFamily::ItemUsed,
            StatisticFamily::ItemBroken,
            StatisticFamily::ItemCrafted,
            StatisticFamily::EntityKilled,
            StatisticFamily::EntityKilledBy,
        ];
        for (code, family) in families.into_iter().enumerate() {
            assert_eq!(family as usize, code);
        }
        let statistic =
            PlayerStatistic::new(StatisticFamily::BlockMined, "minecraft:stone").unwrap();
        assert_eq!(statistic.family, StatisticFamily::BlockMined);
        assert_eq!(statistic.registry_id, "minecraft:stone");
        assert_eq!(MAX_INCREMENT, 1_000_000);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn valid_operations_fail_closed_on_native_targets() {
        let handle = PlayerHandle::new(1, 1, 1, 1).unwrap();
        let statistic = PlayerStatistic::new(StatisticFamily::ItemUsed, "minecraft:stick").unwrap();
        assert_eq!(
            super::read(handle, &statistic).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            super::increment(handle, &statistic, 1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
