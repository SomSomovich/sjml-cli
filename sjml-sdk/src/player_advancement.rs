//! Bounded, criterion-level access to vanilla advancement progress for online players.

use sjml::{ErrorCode, PlayerHandle, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Maximum number of criteria represented for one advancement.
pub const MAX_PLAYER_ADVANCEMENT_CRITERIA: usize = 256;
/// Maximum UTF-8 size accepted for an advancement or criterion ID.
pub const MAX_PLAYER_ADVANCEMENT_CRITERION_ID_BYTES: usize = 256;
/// Maximum stable schema-v1 response size for one advancement progress snapshot.
pub const MAX_PLAYER_ADVANCEMENT_PROGRESS_BYTES: u32 = 68_108;

/// One criterion and its current completion state in a vanilla advancement.
#[derive(Clone, Debug, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAdvancementCriterion {
    /// Exact criterion key defined by the active advancement.
    pub id: String,
    /// Whether vanilla has awarded this criterion to the player.
    pub achieved: bool,
}

impl PlayerAdvancementCriterion {
    /// Creates a bounded criterion snapshot.
    pub fn new(id: impl Into<String>, achieved: bool) -> SjmlResult<Self> {
        let criterion = Self {
            id: id.into(),
            achieved,
        };
        validate_criterion_id(&criterion.id)?;
        Ok(criterion)
    }
}

/// Detached criterion-level progress for one exact registered vanilla advancement.
#[derive(Clone, Debug, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct PlayerAdvancementProgress {
    /// Canonical advancement ID requested by the caller.
    pub advancement_id: String,
    /// All registered criteria for this advancement, sorted by criterion ID.
    pub criteria: Vec<PlayerAdvancementCriterion>,
}

impl PlayerAdvancementProgress {
    /// Creates a bounded progress snapshot and validates criterion identities and uniqueness.
    pub fn new(
        advancement_id: impl Into<String>,
        criteria: Vec<PlayerAdvancementCriterion>,
    ) -> SjmlResult<Self> {
        let progress = Self {
            advancement_id: advancement_id.into(),
            criteria,
        };
        progress.validate(None)?;
        Ok(progress)
    }

    /// Returns the criterion state if this advancement defines the given exact criterion ID.
    pub fn criterion(&self, criterion_id: &str) -> Option<&PlayerAdvancementCriterion> {
        self.criteria
            .iter()
            .find(|criterion| criterion.id == criterion_id)
    }

    /// Returns true when every registered criterion is complete.
    pub fn is_complete(&self) -> bool {
        self.criteria.iter().all(|criterion| criterion.achieved)
    }

    fn validate(&self, expected_advancement_id: Option<&str>) -> SjmlResult<()> {
        validate_advancement_id(&self.advancement_id)?;
        if expected_advancement_id.is_some_and(|expected| expected != self.advancement_id) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned progress for a different advancement ID",
            ));
        }
        if self.criteria.len() > MAX_PLAYER_ADVANCEMENT_CRITERIA {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "advancement progress exceeds the 256-criterion limit",
            ));
        }
        let mut previous: Option<&str> = None;
        for criterion in &self.criteria {
            validate_criterion_id(&criterion.id)?;
            if previous.is_some_and(|id| id >= criterion.id.as_str()) {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "advancement criterion IDs must be unique and sorted",
                ));
            }
            previous = Some(&criterion.id);
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_player_v1")]
extern "C" {
    #[link_name = "advancement_progress"]
    fn imported_advancement_progress(
        handle: i32,
        handle_length: i32,
        advancement_id: i32,
        advancement_id_length: i32,
        output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "grant_advancement_criterion"]
    fn imported_grant_advancement_criterion(
        handle: i32,
        handle_length: i32,
        advancement_id: i32,
        advancement_id_length: i32,
        criterion_id: i32,
        criterion_id_length: i32,
        changed_output: i32,
        capacity: i32,
    ) -> i32;
    #[link_name = "revoke_advancement_criterion"]
    fn imported_revoke_advancement_criterion(
        handle: i32,
        handle_length: i32,
        advancement_id: i32,
        advancement_id_length: i32,
        criterion_id: i32,
        criterion_id_length: i32,
        changed_output: i32,
        capacity: i32,
    ) -> i32;
}

/// Reads every criterion and its completion state for one registered advancement.
///
/// Requires `entity.read` and a live player handle in a logical-server module. The requested
/// advancement must exist in the active vanilla advancement registry and define no more than 256
/// criteria. Results contain only bounded IDs and booleans, not advancement objects or rewards.
pub fn progress(
    handle: PlayerHandle,
    advancement_id: &str,
) -> SjmlResult<PlayerAdvancementProgress> {
    validate_advancement_id(advancement_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = sjml::to_bytes(&handle)?;
        let snapshot = read_output::<PlayerAdvancementProgress>(
            MAX_PLAYER_ADVANCEMENT_PROGRESS_BYTES,
            |output, capacity| {
                // SAFETY: both strings and output allocation stay live through the synchronous import.
                unsafe {
                    imported_advancement_progress(
                        encoded_handle.as_ptr() as usize as u32 as i32,
                        encoded_handle.len() as u32 as i32,
                        advancement_id.as_ptr() as usize as u32 as i32,
                        advancement_id.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
        )?;
        snapshot.validate(Some(advancement_id))?;
        Ok(snapshot)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

/// Grants one exact, registered criterion on an online player.
///
/// Requires `entity.read` and `player.advancement.write` in a logical-server module. The
/// advancement and criterion must both exist in the active registry; only the selected criterion
/// is awarded. Completing the final required criterion can trigger vanilla rewards and dependent
/// advancements. Returns true only if vanilla progress changed.
pub fn grant_criterion(
    handle: PlayerHandle,
    advancement_id: &str,
    criterion_id: &str,
) -> SjmlResult<bool> {
    change_criterion(handle, advancement_id, criterion_id, true)
}

/// Revokes one exact, registered criterion from an online player.
///
/// Requires `entity.read` and `player.advancement.write` in a logical-server module. Only the
/// selected criterion is revoked. Vanilla rewards or already-completed dependent advancements are
/// not undone. Returns true only if vanilla progress changed.
pub fn revoke_criterion(
    handle: PlayerHandle,
    advancement_id: &str,
    criterion_id: &str,
) -> SjmlResult<bool> {
    change_criterion(handle, advancement_id, criterion_id, false)
}

fn change_criterion(
    handle: PlayerHandle,
    advancement_id: &str,
    criterion_id: &str,
    grant: bool,
) -> SjmlResult<bool> {
    validate_advancement_id(advancement_id)?;
    validate_criterion_id(criterion_id)?;
    #[cfg(target_arch = "wasm32")]
    {
        let encoded_handle = sjml::to_bytes(&handle)?;
        let mut changed = 0_u8;
        // SAFETY: borrowed strings and the byte result remain live during the synchronous host call.
        let status = unsafe {
            let operation = if grant {
                imported_grant_advancement_criterion
            } else {
                imported_revoke_advancement_criterion
            };
            operation(
                encoded_handle.as_ptr() as usize as u32 as i32,
                encoded_handle.len() as u32 as i32,
                advancement_id.as_ptr() as usize as u32 as i32,
                advancement_id.len() as u32 as i32,
                criterion_id.as_ptr() as usize as u32 as i32,
                criterion_id.len() as u32 as i32,
                &mut changed as *mut u8 as usize as u32 as i32,
                1,
            )
        };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the player advancement criterion change",
            ));
        }
        match changed {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "host returned an invalid player advancement change result",
            )),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, grant);
        Err(unsupported_target())
    }
}

fn validate_advancement_id(advancement_id: &str) -> SjmlResult<()> {
    let valid = advancement_id
        .split_once(':')
        .is_some_and(|(namespace, path)| {
            !namespace.is_empty()
                && !path.is_empty()
                && namespace.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
                })
                && path.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
                })
        });
    if advancement_id.len() > MAX_PLAYER_ADVANCEMENT_CRITERION_ID_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "player advancement ID exceeds 256 UTF-8 bytes",
        ));
    }
    if !valid {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "player advancement ID must be a canonical lowercase namespace:path value",
        ));
    }
    Ok(())
}

fn validate_criterion_id(criterion_id: &str) -> SjmlResult<()> {
    if criterion_id.is_empty()
        || criterion_id.len() > MAX_PLAYER_ADVANCEMENT_CRITERION_ID_BYTES
        || criterion_id.chars().any(char::is_control)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "criterion ID must contain 1–256 UTF-8 bytes without control characters",
        ));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn read_output<T: StableDecode>(
    capacity: u32,
    invoke: impl FnOnce(i32, i32) -> i32,
) -> SjmlResult<T> {
    let pointer = sjml::abi::allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate the advancement progress output buffer",
        ));
    }
    // SAFETY: initialize output padding so the stable decoder can safely consume the fixed buffer.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        // SAFETY: allocator-owned output remains live through the bounded stable decode.
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        sjml::Decoder::new(bytes).and_then(|mut decoder| T::decode(&mut decoder))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the player advancement progress read",
        ))
    };
    // SAFETY: release the same owned output allocation exactly once.
    unsafe { sjml::abi::deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "player advancement progress requires the wasm32-unknown-unknown target",
    )
}
