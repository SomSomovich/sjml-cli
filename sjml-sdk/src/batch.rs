//! One-transition batching for explicitly batchable, capability-gated read imports.
//!
//! Operations run in request order against the same module instance and guest memory. The host
//! only includes imports whose adapter opted into batching; every imported operation still runs
//! its ordinary capability, owner, context, and thread checks.

use crate::{ErrorCode, SjmlError, SjmlResult, MAX_PAYLOAD_BYTES};

const REQUEST_SCHEMA: u32 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const RESPONSE_SCHEMA: u32 = 1;
const MAX_CALLS: usize = 1024;
const MAX_ARGUMENTS: usize = 16;
const MAX_NAME_BYTES: usize = 128;
const VALUE_BYTES: usize = 1 + core::mem::size_of::<u64>();

/// A raw numeric argument/result in a batch import call.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BatchValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl BatchValue {
    fn encode(self, output: &mut Vec<u8>) {
        let (tag, bits) = match self {
            Self::I32(value) => (0, value as u32 as u64),
            Self::I64(value) => (1, value as u64),
            Self::F32(value) => (2, value.to_bits() as u64),
            Self::F64(value) => (3, value.to_bits()),
        };
        output.push(tag);
        output.extend_from_slice(&bits.to_le_bytes());
    }

    #[cfg(any(target_arch = "wasm32", test))]
    fn decode(tag: u8, bits: u64) -> SjmlResult<Self> {
        match tag {
            0 if bits >> 32 == 0 => Ok(Self::I32(bits as u32 as i32)),
            1 => Ok(Self::I64(bits as i64)),
            2 if bits >> 32 == 0 => Ok(Self::F32(f32::from_bits(bits as u32))),
            3 => Ok(Self::F64(f64::from_bits(bits))),
            0 | 2 => Err(invalid("32-bit batch value has nonzero upper bits")),
            _ => Err(invalid("batch value has an unknown numeric type tag")),
        }
    }
}

/// Results from one operation. Values are returned exactly as the host import declares them.
#[derive(Clone, Debug, PartialEq)]
pub struct CallResult {
    pub values: Vec<BatchValue>,
}

struct Call {
    module: String,
    name: String,
    arguments: Vec<BatchValue>,
}

/// A bounded sequence of approved host reads submitted through one ABI import.
pub struct Batch {
    calls: Vec<Call>,
    encoded_bytes: usize,
}

impl Default for Batch {
    fn default() -> Self {
        Self::new()
    }
}

impl Batch {
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            encoded_bytes: 2 * core::mem::size_of::<u32>(),
        }
    }

    /// Adds one import call and returns its position in the eventual response.
    pub fn push(
        &mut self,
        module: &str,
        name: &str,
        arguments: &[BatchValue],
    ) -> SjmlResult<usize> {
        if self.calls.len() >= MAX_CALLS {
            return Err(payload_too_large("batch exceeds its 1,024-call limit"));
        }
        validate_name(module, "import module")?;
        validate_name(name, "import name")?;
        if module == "sjml_core_v1" && name == "batch" {
            return Err(invalid("the batch dispatcher cannot call itself"));
        }
        if arguments.len() > MAX_ARGUMENTS {
            return Err(payload_too_large(
                "batch import exceeds its 16-argument limit",
            ));
        }
        let encoded_call_bytes = 2 * core::mem::size_of::<u16>()
            + module.len()
            + name.len()
            + core::mem::size_of::<u16>()
            + arguments.len() * VALUE_BYTES;
        let new_encoded_bytes = self
            .encoded_bytes
            .checked_add(encoded_call_bytes)
            .ok_or_else(|| payload_too_large("batch request length overflowed"))?;
        if new_encoded_bytes > MAX_PAYLOAD_BYTES {
            return Err(payload_too_large(
                "encoded batch exceeds the 1 MiB ABI limit",
            ));
        }
        let call = Call {
            module: module.to_owned(),
            name: name.to_owned(),
            arguments: arguments.to_vec(),
        };
        let index = self.calls.len();
        self.calls.push(call);
        self.encoded_bytes = new_encoded_bytes;
        Ok(index)
    }

    /// Submits all operations as one bounded host transition.
    pub fn execute(&self) -> SjmlResult<Vec<CallResult>> {
        if self.calls.is_empty() {
            return Err(invalid("cannot execute an empty batch"));
        }
        let request = encode_request(&self.calls)?;
        #[cfg(target_arch = "wasm32")]
        {
            execute_host(&request, self.calls.len())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = request;
            Err(SjmlError::new(
                ErrorCode::TaskRejected,
                "batched host imports require the wasm32-unknown-unknown target",
            ))
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn execute_host(request: &[u8], expected_calls: usize) -> SjmlResult<Vec<CallResult>> {
    use crate::abi::{allocate_host_input, deallocate_host_input};

    let response_capacity = 2 * core::mem::size_of::<u32>()
        + expected_calls * (core::mem::size_of::<u16>() + MAX_ARGUMENTS * VALUE_BYTES);
    let request_pointer = allocate_host_input(request.len() as u32);
    if request_pointer == 0 {
        return Err(payload_too_large("could not allocate the batch request"));
    }
    // SAFETY: the host input allocation is live, large enough, and uniquely owned here.
    unsafe {
        core::ptr::copy_nonoverlapping(request.as_ptr(), request_pointer as *mut u8, request.len());
    }

    let response_pointer = allocate_host_input(response_capacity as u32);
    if response_pointer == 0 {
        // SAFETY: release the exact request allocation after the host rejected the second allocation.
        unsafe { deallocate_host_input(request_pointer, request.len() as u32) };
        return Err(payload_too_large("could not allocate the batch response"));
    }

    // SAFETY: both exact live allocations remain owned until the synchronous host call and copy finish.
    let packed = unsafe {
        imported_batch(
            request_pointer as i32,
            request.len() as i32,
            response_pointer as i32,
            response_capacity as i32,
        ) as u64
    };
    // SAFETY: release the exact input allocation once after the host call has returned.
    unsafe { deallocate_host_input(request_pointer, request.len() as u32) };

    let status = packed as u32 as i32;
    let response_length = (packed >> 32) as u32 as usize;
    let result = if status != 0 {
        if status < 0 {
            Err(SjmlError::from_abi_code(
                status.unsigned_abs(),
                "host rejected the batch request",
            ))
        } else {
            Err(invalid("host returned an invalid batch status"))
        }
    } else if response_length > response_capacity {
        Err(payload_too_large(
            "host batch response exceeds its output allocation",
        ))
    } else {
        // SAFETY: successful host status reports an initialized response span within the live allocation.
        let bytes =
            unsafe { core::slice::from_raw_parts(response_pointer as *const u8, response_length) };
        decode_response(bytes, expected_calls)
    };
    // SAFETY: release the exact response allocation once after copying and decoding.
    unsafe { deallocate_host_input(response_pointer, response_capacity as u32) };
    result
}

fn encode_request(calls: &[Call]) -> SjmlResult<Vec<u8>> {
    if calls.is_empty() || calls.len() > MAX_CALLS {
        return Err(payload_too_large(
            "batch call count is outside its supported range",
        ));
    }
    let mut output = Vec::new();
    output.extend_from_slice(&REQUEST_SCHEMA.to_le_bytes());
    output.extend_from_slice(&(calls.len() as u32).to_le_bytes());
    for call in calls {
        put_name(&mut output, &call.module, "import module")?;
        put_name(&mut output, &call.name, "import name")?;
        if call.arguments.len() > MAX_ARGUMENTS {
            return Err(payload_too_large(
                "batch import exceeds its 16-argument limit",
            ));
        }
        output.extend_from_slice(&(call.arguments.len() as u16).to_le_bytes());
        for argument in &call.arguments {
            argument.encode(&mut output);
        }
        if output.len() > MAX_PAYLOAD_BYTES {
            return Err(payload_too_large(
                "encoded batch exceeds the 1 MiB ABI limit",
            ));
        }
    }
    Ok(output)
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_response(bytes: &[u8], expected_calls: usize) -> SjmlResult<Vec<CallResult>> {
    if bytes.len() < 8 || bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(payload_too_large(
            "batch response is empty or exceeds its ABI limit",
        ));
    }
    let mut decoder = SliceDecoder::new(bytes);
    if decoder.read_u32()? != RESPONSE_SCHEMA {
        return Err(SjmlError::new(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported batch response schema",
        ));
    }
    let count = decoder.read_u32()? as usize;
    if count != expected_calls || count > MAX_CALLS {
        return Err(invalid(
            "batch response call count does not match the request",
        ));
    }
    let mut results = Vec::with_capacity(count);
    for _ in 0..count {
        let value_count = decoder.read_u16()? as usize;
        if value_count > MAX_ARGUMENTS {
            return Err(payload_too_large("batch response has too many values"));
        }
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            let tag = decoder.read_u8()?;
            let bits = decoder.read_u64()?;
            values.push(BatchValue::decode(tag, bits)?);
        }
        results.push(CallResult { values });
    }
    decoder.finish()?;
    Ok(results)
}

#[cfg(any(target_arch = "wasm32", test))]
struct SliceDecoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

#[cfg(any(target_arch = "wasm32", test))]
impl<'a> SliceDecoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self) -> SjmlResult<u8> {
        let value = *self.bytes.get(self.offset).ok_or_else(truncated)?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> SjmlResult<u16> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> SjmlResult<u32> {
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> SjmlResult<u64> {
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_exact(&mut self, count: usize) -> SjmlResult<&'a [u8]> {
        let end = self.offset.checked_add(count).ok_or_else(|| {
            SjmlError::new(
                ErrorCode::ArithmeticOverflow,
                "batch response offset overflowed",
            )
        })?;
        let bytes = self.bytes.get(self.offset..end).ok_or_else(truncated)?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(self) -> SjmlResult<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid("batch response contains trailing bytes"))
        }
    }
}

fn put_name(output: &mut Vec<u8>, value: &str, label: &str) -> SjmlResult<()> {
    validate_name(value, label)?;
    let bytes = value.as_bytes();
    output.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    output.extend_from_slice(bytes);
    Ok(())
}

fn validate_name(value: &str, label: &str) -> SjmlResult<()> {
    let valid = !value.is_empty()
        && value.len() <= MAX_NAME_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(invalid(format!(
            "{label} must be a short ASCII import identifier"
        )))
    }
}

fn invalid(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

fn payload_too_large(message: impl Into<String>) -> SjmlError {
    SjmlError::new(ErrorCode::PayloadTooLarge, message)
}

#[cfg(any(target_arch = "wasm32", test))]
fn truncated() -> SjmlError {
    SjmlError::new(ErrorCode::MemoryOutOfBounds, "batch response is truncated")
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_core_v1")]
extern "C" {
    #[link_name = "batch"]
    fn imported_batch(
        request_pointer: i32,
        request_length: i32,
        response_pointer: i32,
        response_capacity: i32,
    ) -> i64;
}

#[cfg(test)]
mod tests {
    use super::{decode_response, encode_request, Batch, BatchValue, Call, CallResult};
    use crate::ErrorCode;

    #[test]
    fn encodes_a_versioned_typed_request_and_decodes_ordered_results() {
        let call = Call {
            module: "sjml_world_v1".to_owned(),
            name: "game_time".to_owned(),
            arguments: vec![BatchValue::I32(-1), BatchValue::F64(1.5)],
        };
        let bytes = encode_request(&[call]).unwrap();
        assert_eq!(&bytes[0..8], &[1, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(&bytes[8..10], &("sjml_world_v1".len() as u16).to_le_bytes());

        let response = [
            1, 0, 0, 0, 1, 0, 0, 0, // schema and call count
            2, 0, // result count
            0, 42, 0, 0, 0, 0, 0, 0, 0, // i32 result
            3, 0, 0, 0, 0, 0, 0, 248, 63, // f64 1.5
        ];
        assert_eq!(
            decode_response(&response, 1).unwrap(),
            vec![CallResult {
                values: vec![BatchValue::I32(42), BatchValue::F64(1.5)]
            }]
        );
    }

    #[test]
    fn validates_batch_names_counts_size_and_recursive_dispatch() {
        let mut batch = Batch::new();
        assert_eq!(
            batch.push("sjml_core_v1", "batch", &[]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            batch.push("bad module", "get", &[]).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            batch
                .push("sjml_world_v1", "get", &[BatchValue::I32(1); 17])
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
        for _ in 0..1024 {
            batch.push("sjml_world_v1", "loaded", &[]).unwrap();
        }
        assert_eq!(
            batch
                .push("sjml_world_v1", "loaded", &[])
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn rejects_truncated_trailing_and_wrong_count_responses() {
        assert_eq!(
            decode_response(&[1, 0, 0, 0], 1).unwrap_err().code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            decode_response(&[1, 0, 0, 0, 0, 0, 0, 0], 1)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidArgument
        );
        let mut trailing = vec![1, 0, 0, 0, 1, 0, 0, 0, 0, 0];
        trailing.push(0);
        assert_eq!(
            decode_response(&trailing, 1).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
}
