//! Bounded, path-only discovery of the calling package's archived client assets.
//!
//! Results list relative names under `assets/<package-id>/`. The host indexes only the current
//! package archive; it never enumerates Minecraft's global resource manager and does not return
//! file contents.

#[cfg(target_arch = "wasm32")]
use crate::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use crate::serialization::Decoder;
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Capability required to list path names from the calling package's own client assets.
pub const CLIENT_ASSETS_READ_CAPABILITY: &str = "client.assets.read";
/// Maximum UTF-8 bytes in one relative path prefix.
pub const MAX_CLIENT_ASSET_PREFIX_BYTES: usize = 256;
/// Maximum number of paths returned by one query.
pub const MAX_CLIENT_ASSET_RESULTS: usize = 60;
/// Maximum UTF-8 bytes in one returned relative asset path.
pub const MAX_CLIENT_ASSET_PATH_BYTES: usize = 1_024;
/// Maximum versioned response bytes accepted from the host.
pub const MAX_CLIENT_ASSET_RESPONSE_BYTES: usize = 61_689;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_assets_v1")]
extern "C" {
    #[link_name = "list"]
    fn imported_list(
        prefix_pointer: i32,
        prefix_length: i32,
        result_limit: i32,
        output_pointer: i32,
        output_capacity: i32,
        output_length_pointer: i32,
    ) -> i32;
}

/// A sorted page of package-owned paths. `truncated` indicates that more prefix matches exist.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClientAssetList {
    /// Paths relative to `assets/<package-id>/`, such as `textures/gui/status.png`.
    pub paths: Vec<String>,
    /// Whether the result limit omitted additional matching paths.
    pub truncated: bool,
}

/// Lists path names under a relative asset prefix in the calling package.
///
/// An empty prefix lists from the package asset root. Results are sorted, capped at
/// [`MAX_CLIENT_ASSET_RESULTS`], and contain paths only; this call never reads or returns file bytes.
/// Prefixes must use canonical lowercase relative paths, for example `textures/gui/`.
pub fn list(prefix: &str, limit: usize) -> SjmlResult<ClientAssetList> {
    validate_prefix(prefix)?;
    if !(1..=MAX_CLIENT_ASSET_RESULTS).contains(&limit) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            format!("client asset result limit must be between 1 and {MAX_CLIENT_ASSET_RESULTS}"),
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let prefix_bytes = prefix.as_bytes();
        // The host accepts a zero-length prefix, but a one-byte allocation keeps the pointer valid
        // on allocators that return zero for a zero-sized request.
        let prefix_allocation = prefix_bytes.len().max(1);
        let prefix_pointer = allocate_host_input(prefix_allocation as u32);
        let output_pointer = allocate_host_input(MAX_CLIENT_ASSET_RESPONSE_BYTES as u32);
        let length_pointer = allocate_host_input(core::mem::size_of::<u32>() as u32);
        if prefix_pointer == 0 || output_pointer == 0 || length_pointer == 0 {
            // SAFETY: nonzero pointers are live allocations with the exact sizes below.
            unsafe {
                deallocate_host_input(prefix_pointer, prefix_allocation as u32);
                deallocate_host_input(output_pointer, MAX_CLIENT_ASSET_RESPONSE_BYTES as u32);
                deallocate_host_input(length_pointer, core::mem::size_of::<u32>() as u32);
            }
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "could not allocate bounded client asset buffers",
            ));
        }
        // SAFETY: `prefix_pointer` owns at least one writable byte, even for an empty prefix.
        unsafe {
            if !prefix_bytes.is_empty() {
                core::ptr::copy_nonoverlapping(
                    prefix_bytes.as_ptr(),
                    prefix_pointer as *mut u8,
                    prefix_bytes.len(),
                );
            }
            core::ptr::write_bytes(
                output_pointer as *mut u8,
                0,
                MAX_CLIENT_ASSET_RESPONSE_BYTES,
            );
            core::ptr::write_bytes(length_pointer as *mut u8, 0, core::mem::size_of::<u32>());
        }
        // SAFETY: the host reads the initialized prefix and may write only to the bounded output
        // and four-byte length allocations supplied to this synchronous call.
        let status = unsafe {
            imported_list(
                prefix_pointer as i32,
                prefix_bytes.len() as i32,
                limit as i32,
                output_pointer as i32,
                MAX_CLIENT_ASSET_RESPONSE_BYTES as i32,
                length_pointer as i32,
            )
        };
        let result = if status != 0 {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client asset list request",
            ))
        } else {
            // SAFETY: successful host calls initialize this four-byte little-endian length.
            let length =
                unsafe { core::ptr::read_unaligned(length_pointer as *const u32) } as usize;
            if length > MAX_CLIENT_ASSET_RESPONSE_BYTES {
                Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "host returned client asset paths beyond the response bound",
                ))
            } else {
                // SAFETY: the host wrote no more than `length` bytes into the bounded output span.
                let bytes =
                    unsafe { core::slice::from_raw_parts(output_pointer as *const u8, length) };
                decode_response(bytes)
            }
        };
        // SAFETY: release each exact live allocation after the synchronous host call.
        unsafe {
            deallocate_host_input(prefix_pointer, prefix_allocation as u32);
            deallocate_host_input(output_pointer, MAX_CLIENT_ASSET_RESPONSE_BYTES as u32);
            deallocate_host_input(length_pointer, core::mem::size_of::<u32>() as u32);
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (prefix, limit);
        Err(unsupported())
    }
}

#[cfg(target_arch = "wasm32")]
fn decode_response(bytes: &[u8]) -> SjmlResult<ClientAssetList> {
    if bytes.len() > MAX_CLIENT_ASSET_RESPONSE_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "client asset response exceeds its byte bound",
        ));
    }
    let mut decoder = Decoder::new(bytes)?;
    let schema = decoder.read_u32()?;
    if schema != 1 {
        return Err(SjmlError::new(
            ErrorCode::IncompatibleAbiVersion,
            format!("client asset response schema {schema} is unsupported"),
        ));
    }
    let truncated = match decoder.read_u8()? {
        0 => false,
        1 => true,
        value => {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                format!("client asset truncated flag must be 0 or 1, found {value}"),
            ));
        }
    };
    let count = decoder.read_length(MAX_CLIENT_ASSET_RESULTS)?;
    let mut paths = Vec::with_capacity(count);
    for _ in 0..count {
        let length = decoder.read_length(MAX_CLIENT_ASSET_PATH_BYTES)?;
        if length == 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "client asset path must not be empty",
            ));
        }
        let path = String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
            SjmlError::new(
                ErrorCode::InvalidUtf8,
                "client asset path is not valid UTF-8",
            )
        })?;
        validate_asset_path(&path)?;
        if paths
            .last()
            .is_some_and(|previous: &String| previous >= &path)
        {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "client asset paths must be unique and sorted",
            ));
        }
        paths.push(path);
    }
    if decoder.remaining() != 0 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "client asset response has trailing bytes",
        ));
    }
    Ok(ClientAssetList { paths, truncated })
}

fn validate_prefix(prefix: &str) -> SjmlResult<()> {
    if prefix.is_empty() {
        return Ok(());
    }
    if prefix.len() > MAX_CLIENT_ASSET_PREFIX_BYTES
        || !prefix.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
        })
        || prefix.starts_with('/')
    {
        return Err(invalid_prefix());
    }
    validate_segments(prefix, true)
}

#[cfg(target_arch = "wasm32")]
fn validate_asset_path(path: &str) -> SjmlResult<()> {
    if path.is_empty()
        || path.len() > MAX_CLIENT_ASSET_PATH_BYTES
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"/._-".contains(&byte)
        })
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "client asset path is not canonical or exceeds 1,024 bytes",
        ));
    }
    validate_segments(path, false)
}

fn validate_segments(path: &str, allow_trailing_slash: bool) -> SjmlResult<()> {
    let segments = path.split('/').collect::<Vec<_>>();
    for (index, segment) in segments.iter().enumerate() {
        if *segment == "."
            || *segment == ".."
            || segment.is_empty()
                && !(allow_trailing_slash && path.ends_with('/') && index == segments.len() - 1)
        {
            return Err(invalid_prefix());
        }
    }
    Ok(())
}

fn invalid_prefix() -> SjmlError {
    SjmlError::new(
        ErrorCode::InvalidArgument,
        "client asset prefix must be a canonical relative path of at most 256 bytes",
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "client asset discovery requires the wasm32-unknown-unknown target",
    )
}

#[cfg(test)]
mod tests {
    use super::{list, validate_prefix, MAX_CLIENT_ASSET_PREFIX_BYTES, MAX_CLIENT_ASSET_RESULTS};
    use crate::ErrorCode;

    #[test]
    fn validates_relative_asset_prefixes_and_result_bounds() {
        assert!(validate_prefix("").is_ok());
        assert!(validate_prefix("textures/gui/").is_ok());
        assert!(validate_prefix("textures/gui/i").is_ok());
        for prefix in [
            "../",
            "/textures",
            "textures//gui/",
            "textures/../",
            "Textures/",
        ] {
            assert_eq!(
                validate_prefix(prefix).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            validate_prefix(&"a".repeat(MAX_CLIENT_ASSET_PREFIX_BYTES + 1))
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(list("", 0).unwrap_err().code(), ErrorCode::InvalidArgument);
        assert_eq!(
            list("", MAX_CLIENT_ASSET_RESULTS + 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn valid_native_lookup_fails_closed_without_a_wasm_host() {
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(
            list("textures/gui/", 12).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
