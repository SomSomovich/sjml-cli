//! Bounded lookup against the language currently active in the physical client.
//!
//! Package-owned `assets/<package-id>/lang/*.json` files participate through
//! Minecraft's ordinary resource reload and language selection. The returned
//! value is detached UTF-8 text; this API does not expose language or resource
//! manager objects.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Capability required to resolve translations from the local client resources.
pub const CLIENT_LOCALIZATION_READ_CAPABILITY: &str = "client.localization.read";
/// Maximum UTF-8 bytes in one translation key.
pub const MAX_TRANSLATION_KEY_BYTES: usize = 256;
/// Maximum UTF-8 bytes returned by one lookup.
pub const MAX_TRANSLATION_BYTES: usize = 4096;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_localization_v1")]
extern "C" {
    #[link_name = "translate"]
    fn imported_translate(
        key_pointer: i32,
        key_length: i32,
        output_pointer: i32,
        output_capacity: i32,
        output_length_pointer: i32,
    ) -> i32;
}

/// Resolves one Minecraft translation key using the language active on this client.
///
/// Missing keys follow Minecraft's normal fallback and return the key itself.
/// Package-owned language files must use the normal resource path, for example
/// `assets/example/lang/en_us.json`, and are visible after Minecraft reloads
/// resources. Translation values are bounded to 4,096 UTF-8 bytes.
pub fn translate(key: &str) -> SjmlResult<String> {
    validate_key(key)?;
    #[cfg(target_arch = "wasm32")]
    {
        let key_bytes = key.as_bytes();
        let key_pointer = allocate_host_input(key_bytes.len() as u32);
        let output_pointer = allocate_host_input(MAX_TRANSLATION_BYTES as u32);
        let length_pointer = allocate_host_input(core::mem::size_of::<u32>() as u32);
        if key_pointer == 0 || output_pointer == 0 || length_pointer == 0 {
            // SAFETY: all non-zero values came from the allocator with the exact sizes below.
            unsafe {
                deallocate_host_input(key_pointer, key_bytes.len() as u32);
                deallocate_host_input(output_pointer, MAX_TRANSLATION_BYTES as u32);
                deallocate_host_input(length_pointer, core::mem::size_of::<u32>() as u32);
            }
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "could not allocate client localization buffers",
            ));
        }
        // SAFETY: key_pointer is a live allocation with exactly key_bytes.len() writable bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(
                key_bytes.as_ptr(),
                key_pointer as *mut u8,
                key_bytes.len(),
            );
            core::ptr::write_bytes(output_pointer as *mut u8, 0, MAX_TRANSLATION_BYTES);
            core::ptr::write_bytes(length_pointer as *mut u8, 0, core::mem::size_of::<u32>());
        }
        // SAFETY: the host reads the initialized key, writes at most the documented output bound,
        // and initializes the four-byte length only during this synchronous call.
        let status = unsafe {
            imported_translate(
                key_pointer as i32,
                key_bytes.len() as i32,
                output_pointer as i32,
                MAX_TRANSLATION_BYTES as i32,
                length_pointer as i32,
            )
        };
        let result = if status != 0 {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client translation lookup",
            ))
        } else {
            // SAFETY: the host initialized the four-byte output length on success.
            let length =
                unsafe { core::ptr::read_unaligned(length_pointer as *const u32) } as usize;
            if length > MAX_TRANSLATION_BYTES {
                Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "host returned a client translation beyond the output bound",
                ))
            } else {
                // SAFETY: the host wrote exactly `length` bytes to the live bounded output span.
                let bytes =
                    unsafe { core::slice::from_raw_parts(output_pointer as *const u8, length) };
                String::from_utf8(bytes.to_vec()).map_err(|_| {
                    SjmlError::new(
                        ErrorCode::InvalidUtf8,
                        "host returned a client translation that is not valid UTF-8",
                    )
                })
            }
        };
        // SAFETY: release each exact live allocation after the host call has returned.
        unsafe {
            deallocate_host_input(key_pointer, key_bytes.len() as u32);
            deallocate_host_input(output_pointer, MAX_TRANSLATION_BYTES as u32);
            deallocate_host_input(length_pointer, core::mem::size_of::<u32>() as u32);
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
        Err(unsupported())
    }
}

fn validate_key(key: &str) -> SjmlResult<()> {
    if key.is_empty()
        || key.trim().is_empty()
        || key.len() > MAX_TRANSLATION_KEY_BYTES
        || key.chars().any(char::is_control)
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "translation key must contain 1..=256 UTF-8 bytes and no control characters",
        ));
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "client localization requires the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{translate, MAX_TRANSLATION_KEY_BYTES};
    use crate::ErrorCode;

    #[test]
    fn rejects_empty_control_and_oversized_keys_before_host_access() {
        for key in ["", " \t", "line\nbreak"] {
            assert_eq!(
                translate(key).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        let oversized = "x".repeat(MAX_TRANSLATION_KEY_BYTES + 1);
        assert_eq!(
            translate(&oversized).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn valid_key_fails_closed_without_a_wasm_host() {
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            translate("screen.example.title").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
