//! Bounded module-owned opaque byte attachments saved with loaded non-player entities.
//!
//! Unlike [`crate::storage`]'s entity scope, attachment bytes live inside the entity's own
//! persistent NBT and follow that entity as it is saved, moved, or unloaded. Each key belongs to
//! the current package/module pair and is isolated from other modules.

#[cfg(any(target_arch = "wasm32", test))]
use crate::Encoder;
use crate::{EntityHandle, ErrorCode, SjmlError, SjmlResult};

/// Maximum UTF-8 size of one module-local attachment key.
pub const MAX_ENTITY_DATA_KEY_BYTES: usize = 128;
/// Maximum payload size for one attachment value.
pub const MAX_ENTITY_DATA_VALUE_BYTES: usize = 16 * 1024;
/// Sentinel returned by [`read`] when the key is missing; a present value returns its byte length.
pub const ENTITY_DATA_MISSING: usize = usize::MAX;
#[cfg(any(target_arch = "wasm32", test))]
const MAX_READ_RESPONSE_BYTES: usize = 8 + MAX_ENTITY_DATA_VALUE_BYTES;

#[cfg(any(target_arch = "wasm32", test))]
const SCHEMA_VERSION: u32 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const OP_GET: u8 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const OP_SET: u8 = 2;
#[cfg(any(target_arch = "wasm32", test))]
const OP_REMOVE: u8 = 3;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_entity_v1")]
extern "C" {
    #[link_name = "data_read"]
    fn imported_data_read(request: i32, request_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "data_write"]
    fn imported_data_write(request: i32, request_length: i32) -> i32;
    #[link_name = "data_remove"]
    fn imported_data_remove(request: i32, request_length: i32, output: i32, capacity: i32) -> i32;
}

/// Copies one module-owned attachment into `output` and returns its length.
///
/// The module must request and receive `entity.data.read`. The supplied `EntityHandle` must be a
/// current server-side non-player handle from [`crate::entity::loaded`]. Keys are local to the
/// module and use bounded lowercase path syntax. A missing key returns [`ENTITY_DATA_MISSING`]; a
/// present empty value returns `0`. If the value does not fit in `output`, this returns
/// [`ErrorCode::PayloadTooLarge`] without copying a partial value.
pub fn read(handle: EntityHandle, key: &str, output: &mut [u8]) -> SjmlResult<usize> {
    validate_key(key)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = encode_request(OP_GET, handle, key, None)?;
        read_value_output(
            MAX_READ_RESPONSE_BYTES,
            |output, capacity| {
                // SAFETY: request and output allocations stay live for the synchronous host call.
                unsafe {
                    imported_data_read(
                        request.as_ptr() as usize as u32 as i32,
                        request.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
            output,
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, output);
        Err(unsupported_target())
    }
}

/// Replaces this module's value for `key` on one current entity.
///
/// The module must request and receive `entity.data.write`. One value is limited to 16 KiB; the
/// host additionally enforces per-module entry count and aggregate per-entity byte quotas.
pub fn set(handle: EntityHandle, key: &str, value: &[u8]) -> SjmlResult<()> {
    validate_key(key)?;
    if value.len() > MAX_ENTITY_DATA_VALUE_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            format!("entity data value exceeds the {MAX_ENTITY_DATA_VALUE_BYTES}-byte limit"),
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let request = encode_request(OP_SET, handle, key, Some(value))?;
        // SAFETY: the host copies this bounded frame and validates the live entity handle.
        let status = unsafe {
            imported_data_write(
                request.as_ptr() as usize as u32 as i32,
                request.len() as u32 as i32,
            )
        };
        status_result(status, "host rejected the entity data write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, value);
        Err(unsupported_target())
    }
}

/// Removes this module's value for `key`; returns whether a value was present.
///
/// This requires `entity.data.write`, just like [`set`]. Removing the last value also removes the
/// module and SJML attachment compounds from the entity NBT.
pub fn remove(handle: EntityHandle, key: &str) -> SjmlResult<bool> {
    validate_key(key)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = encode_request(OP_REMOVE, handle, key, None)?;
        read_output(
            5,
            |output, capacity| {
                // SAFETY: the request and response allocations stay live for the synchronous call.
                unsafe {
                    imported_data_remove(
                        request.as_ptr() as usize as u32 as i32,
                        request.len() as u32 as i32,
                        output,
                        capacity,
                    )
                }
            },
            decode_removed_response,
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported_target())
    }
}

fn validate_key(key: &str) -> SjmlResult<()> {
    let invalid = || {
        SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity data key must be a bounded lowercase path using letters, digits, '.', '_', '-' or '/'",
        )
    };
    if key.is_empty()
        || key.len() > MAX_ENTITY_DATA_KEY_BYTES
        || !key.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b'-' | b'/')
        })
    {
        return Err(invalid());
    }
    for segment in key.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(invalid());
        }
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn encode_request(
    operation: u8,
    handle: EntityHandle,
    key: &str,
    value: Option<&[u8]>,
) -> SjmlResult<Vec<u8>> {
    let handle_bytes = crate::to_bytes(&handle)?;
    let mut encoder = Encoder::new();
    encoder.write_u32(SCHEMA_VERSION)?;
    encoder.write_u8(operation)?;
    encoder.write_raw(&handle_bytes)?;
    encoder.write_length(key.len())?;
    encoder.write_raw(key.as_bytes())?;
    if let Some(value) = value {
        encoder.write_length(value.len())?;
        encoder.write_raw(value)?;
    }
    Ok(encoder.into_bytes())
}

#[cfg(target_arch = "wasm32")]
fn read_output<T>(
    capacity: usize,
    invoke: impl FnOnce(i32, i32) -> i32,
    decode: impl FnOnce(&[u8]) -> SjmlResult<T>,
) -> SjmlResult<T> {
    use crate::abi::{allocate_host_input, deallocate_host_input};
    let capacity = u32::try_from(capacity).map_err(|_| {
        SjmlError::new(
            ErrorCode::ArithmeticOverflow,
            "entity data output capacity overflowed",
        )
    })?;
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate entity data response buffer",
        ));
    }
    // SAFETY: initialize the full response buffer before host writes to it.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        decode(bytes)
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the entity data read",
        ))
    };
    // SAFETY: release exactly the allocation passed to the host, after decoding finishes.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(target_arch = "wasm32")]
fn read_value_output(
    capacity: usize,
    invoke: impl FnOnce(i32, i32) -> i32,
    output: &mut [u8],
) -> SjmlResult<usize> {
    use crate::abi::{allocate_host_input, deallocate_host_input};
    let capacity = u32::try_from(capacity).map_err(|_| {
        SjmlError::new(
            ErrorCode::ArithmeticOverflow,
            "entity data output capacity overflowed",
        )
    })?;
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "could not allocate entity data response buffer",
        ));
    }
    // SAFETY: initialize the full response buffer before host writes to it.
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        let bytes = unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) };
        decode_value_response(bytes, output)
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected the entity data read",
        ))
    };
    // SAFETY: release exactly the allocation passed to the host, after decoding finishes.
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(any(target_arch = "wasm32", test))]
fn read_le_u32(bytes: &[u8], offset: usize, description: &str) -> SjmlResult<u32> {
    let end = offset.checked_add(4).ok_or_else(|| {
        SjmlError::new(
            ErrorCode::ArithmeticOverflow,
            "entity data response offset overflowed",
        )
    })?;
    let Some(word) = bytes.get(offset..end) else {
        return Err(SjmlError::new(
            ErrorCode::MemoryOutOfBounds,
            format!("entity data {description} is truncated"),
        ));
    };
    Ok(u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_value_response(bytes: &[u8], output: &mut [u8]) -> SjmlResult<usize> {
    if bytes.len() < 4 {
        return Err(SjmlError::new(
            ErrorCode::MemoryOutOfBounds,
            "entity data response is truncated",
        ));
    }
    let schema = read_le_u32(bytes, 0, "response schema")?;
    if schema != SCHEMA_VERSION {
        return Err(SjmlError::new(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported entity data response schema",
        ));
    }
    if bytes.len() < 8 {
        return Err(SjmlError::new(
            ErrorCode::MemoryOutOfBounds,
            "entity data response length is truncated",
        ));
    }
    let length = read_le_u32(bytes, 4, "response length")? as i32;
    let payload = bytes.get(8..).ok_or_else(|| {
        SjmlError::new(
            ErrorCode::MemoryOutOfBounds,
            "entity data response payload is truncated",
        )
    })?;
    if length == -1 {
        return Ok(ENTITY_DATA_MISSING);
    }
    if length < 0
        || length as usize > MAX_ENTITY_DATA_VALUE_BYTES
        || length as usize > payload.len()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity data response length is invalid",
        ));
    }
    let value_length = length as usize;
    if value_length > output.len() {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "entity data value does not fit in the caller's output buffer",
        ));
    }
    output[..value_length].copy_from_slice(&payload[..value_length]);
    Ok(value_length)
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_removed_response(bytes: &[u8]) -> SjmlResult<bool> {
    if bytes.len() != 5 {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity data remove response has an invalid length",
        ));
    }
    if read_le_u32(bytes, 0, "response schema")? != SCHEMA_VERSION {
        return Err(SjmlError::new(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported entity data response schema",
        ));
    }
    match bytes[4] {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "entity data remove response is invalid",
        )),
    }
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
        "entity attachments require the wasm32-unknown-unknown target and host imports",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        decode_removed_response, decode_value_response, encode_request, read, remove, set,
        MAX_ENTITY_DATA_VALUE_BYTES, MAX_READ_RESPONSE_BYTES, OP_GET, OP_REMOVE, OP_SET,
    };
    use crate::{EntityHandle, ErrorCode};

    #[test]
    fn validates_local_keys_and_value_bounds_before_host_access() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        for key in ["", "../progress", "progress//old", "Bad", "x:foreign"] {
            let mut output = [];
            assert_eq!(
                read(handle, key, &mut output).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
            assert_eq!(
                remove(handle, key).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            set(
                handle,
                "progress",
                &vec![0; MAX_ENTITY_DATA_VALUE_BYTES + 1]
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn native_target_reports_missing_host_imports_for_valid_requests() {
        let handle = EntityHandle::new(1, 2, 3, 4).unwrap();
        let mut output = [0; 8];
        assert_eq!(
            read(handle, "progress", &mut output).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            set(handle, "progress", b"done").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
        assert_eq!(
            remove(handle, "progress").unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }

    #[test]
    fn request_frames_use_the_shared_little_endian_core_contract() {
        let handle = EntityHandle::new(4, 8, 12, 1).unwrap();
        let expected_handle = crate::to_bytes(&handle).unwrap();
        let key = b"quest/progress";

        let get = encode_request(OP_GET, handle, "quest/progress", None).unwrap();
        let mut expected = Vec::new();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.push(OP_GET);
        expected.extend_from_slice(&expected_handle);
        expected.extend_from_slice(&(key.len() as u32).to_le_bytes());
        expected.extend_from_slice(key);
        assert_eq!(get, expected);

        let payload = [0, 1, 255];
        let set = encode_request(OP_SET, handle, "quest/progress", Some(&payload)).unwrap();
        expected.clear();
        expected.extend_from_slice(&1_u32.to_le_bytes());
        expected.push(OP_SET);
        expected.extend_from_slice(&expected_handle);
        expected.extend_from_slice(&(key.len() as u32).to_le_bytes());
        expected.extend_from_slice(key);
        expected.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        expected.extend_from_slice(&payload);
        assert_eq!(set, expected);

        let remove = encode_request(OP_REMOVE, handle, "quest/progress", None).unwrap();
        assert_eq!(remove[4], OP_REMOVE);
        assert_eq!(&remove[5..29], expected_handle.as_slice());
    }

    #[test]
    fn response_decoder_distinguishes_missing_and_empty_values() {
        let mut output = [0; MAX_ENTITY_DATA_VALUE_BYTES];
        let mut missing = vec![0; MAX_READ_RESPONSE_BYTES];
        missing[..4].copy_from_slice(&1_u32.to_le_bytes());
        missing[4..8].copy_from_slice(&(-1_i32).to_le_bytes());
        assert_eq!(
            decode_value_response(&missing, &mut output).unwrap(),
            super::ENTITY_DATA_MISSING
        );

        let mut empty = vec![0; MAX_READ_RESPONSE_BYTES];
        empty[..4].copy_from_slice(&1_u32.to_le_bytes());
        empty[4..8].copy_from_slice(&0_i32.to_le_bytes());
        assert_eq!(decode_value_response(&empty, &mut []).unwrap(), 0);
        empty[12] = 1;
        assert_eq!(decode_value_response(&empty, &mut []).unwrap(), 0);

        missing[12] = 1;
        assert_eq!(
            decode_value_response(&missing, &mut output).unwrap(),
            super::ENTITY_DATA_MISSING
        );
        assert_eq!(
            decode_value_response(&[1, 0, 0], &mut output)
                .unwrap_err()
                .code(),
            ErrorCode::MemoryOutOfBounds
        );

        let mut wrong_schema = vec![0; 8];
        wrong_schema[4..8].copy_from_slice(&(-1_i32).to_le_bytes());
        assert_eq!(
            decode_value_response(&wrong_schema, &mut output)
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );
        let mut invalid_length = vec![0; 8];
        invalid_length[..4].copy_from_slice(&1_u32.to_le_bytes());
        invalid_length[4..8].copy_from_slice(&(-2_i32).to_le_bytes());
        assert_eq!(
            decode_value_response(&invalid_length, &mut output)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn response_decoder_copies_only_a_value_that_fits_the_caller_buffer() {
        let mut response = vec![0; MAX_READ_RESPONSE_BYTES];
        response[..4].copy_from_slice(&1_u32.to_le_bytes());
        response[4..8].copy_from_slice(&3_u32.to_le_bytes());
        response[8..11].copy_from_slice(&[7, 8, 9]);

        let mut too_small = [4, 5];
        assert_eq!(
            decode_value_response(&response, &mut too_small)
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(too_small, [4, 5]);

        let mut output = [0; 3];
        assert_eq!(decode_value_response(&response, &mut output).unwrap(), 3);
        assert_eq!(output, [7, 8, 9]);
    }

    #[test]
    fn remove_response_decoder_checks_fixed_width_fields() {
        assert!(decode_removed_response(&[1, 0, 0, 0, 1]).unwrap());
        assert_eq!(
            decode_removed_response(&[1, 0, 0, 0]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}
