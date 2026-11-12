use crate::{ErrorCode, SjmlError, SjmlResult, MAX_PAYLOAD_BYTES};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

const REQUEST_VERSION: u32 = 1;

/// Creates a future for a bounded, versioned cross-module service call.
///
/// `maximum_response_bytes` controls the guest output buffer allocated while the
/// request is pending. Keep it close to the service's expected response size.
pub fn service_call(
    provider_package_id: &str,
    service_id: &str,
    version_constraint: &str,
    request: &[u8],
    maximum_response_bytes: usize,
) -> SjmlResult<ServiceCallFuture> {
    let request = encode_request(provider_package_id, service_id, version_constraint, request)?;
    if maximum_response_bytes > MAX_PAYLOAD_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "service response buffer exceeds the 1 MiB ABI payload limit",
        ));
    }
    Ok(ServiceCallFuture {
        request,
        response: vec![0; maximum_response_bytes],
        operation_id: None,
        completed: false,
    })
}

/// Future returned by [`service_call`]. Dropping it cancels its host operation handle.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub struct ServiceCallFuture {
    request: Vec<u8>,
    response: Vec<u8>,
    operation_id: Option<i32>,
    completed: bool,
}

impl Future for ServiceCallFuture {
    type Output = SjmlResult<Vec<u8>>;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            return Poll::Ready(Err(SjmlError::new(
                ErrorCode::InvalidHandle,
                "service-call future was polled after completion",
            )));
        }
        #[cfg(target_arch = "wasm32")]
        {
            if self.operation_id.is_none() {
                let operation_id = unsafe {
                    imported_service_start(
                        self.request.as_ptr() as usize as u32 as i32,
                        self.request.len() as u32 as i32,
                        self.response.as_mut_ptr() as usize as u32 as i32,
                        self.response.len() as u32 as i32,
                    )
                };
                if operation_id <= 0 {
                    self.completed = true;
                    let code = operation_id.unsigned_abs();
                    return Poll::Ready(Err(SjmlError::from_abi_code(
                        code,
                        "host rejected the service operation",
                    )));
                }
                self.operation_id = Some(operation_id);
            }

            let operation_id = self
                .operation_id
                .expect("operation id was just initialized");
            let packed = unsafe { imported_service_poll(operation_id) } as u64;
            let status = packed as u32 as i32;
            let response_length = (packed >> 32) as u32 as usize;
            if status == 0 {
                return Poll::Pending;
            }
            if status == 1 {
                if response_length > self.response.len() {
                    self.finish_operation();
                    return Poll::Ready(Err(SjmlError::new(
                        ErrorCode::PayloadTooLarge,
                        "service response exceeds the guest-provided output buffer",
                    )));
                }
                self.response.truncate(response_length);
                self.finish_operation();
                return Poll::Ready(Ok(core::mem::take(&mut self.response)));
            }
            self.finish_operation();
            let code = status.unsigned_abs();
            Poll::Ready(Err(SjmlError::from_abi_code(
                code,
                "host service operation failed",
            )))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.completed = true;
            Poll::Ready(Err(SjmlError::new(
                ErrorCode::TaskRejected,
                "host service operations require the wasm32-unknown-unknown target",
            )))
        }
    }
}

impl ServiceCallFuture {
    #[cfg(target_arch = "wasm32")]
    fn finish_operation(&mut self) {
        if let Some(operation_id) = self.operation_id.take() {
            let _ = unsafe { imported_service_cancel(operation_id) };
        }
        self.completed = true;
    }
}

impl Drop for ServiceCallFuture {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        if !self.completed {
            if let Some(operation_id) = self.operation_id.take() {
                let _ = unsafe { imported_service_cancel(operation_id) };
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_async_v1")]
extern "C" {
    #[link_name = "service_start"]
    fn imported_service_start(
        request_pointer: i32,
        request_length: i32,
        response_pointer: i32,
        response_capacity: i32,
    ) -> i32;
    #[link_name = "service_poll"]
    fn imported_service_poll(operation_id: i32) -> i64;
    #[link_name = "service_cancel"]
    fn imported_service_cancel(operation_id: i32) -> i32;
}

fn encode_request(
    provider_package_id: &str,
    service_id: &str,
    version_constraint: &str,
    request: &[u8],
) -> SjmlResult<Vec<u8>> {
    if !valid_package_id(provider_package_id)
        || !valid_service_id(service_id)
        || version_constraint.is_empty()
    {
        return Err(SjmlError::new(
            ErrorCode::InvalidTransfer,
            "service request contains an invalid provider, service, or version constraint",
        ));
    }
    if service_id.len() > 512 || version_constraint.len() > 512 {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "service id or version constraint exceeds the 512-byte ABI field limit",
        ));
    }
    let total = 4usize
        .checked_add(4 + provider_package_id.len())
        .and_then(|total| total.checked_add(4 + service_id.len()))
        .and_then(|total| total.checked_add(4 + version_constraint.len()))
        .and_then(|total| total.checked_add(4 + request.len()))
        .ok_or_else(|| {
            SjmlError::new(
                ErrorCode::ArithmeticOverflow,
                "service request size calculation overflowed",
            )
        })?;
    if total > MAX_PAYLOAD_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "encoded service request exceeds the 1 MiB ABI payload limit",
        ));
    }
    let mut output = Vec::with_capacity(total);
    output.extend_from_slice(&REQUEST_VERSION.to_le_bytes());
    write_field(&mut output, provider_package_id.as_bytes());
    write_field(&mut output, service_id.as_bytes());
    write_field(&mut output, version_constraint.as_bytes());
    write_field(&mut output, request);
    Ok(output)
}

fn write_field(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(bytes);
}

fn valid_package_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

fn valid_service_id(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|part| {
            !part.is_empty()
                && part.as_bytes()[0].is_ascii_lowercase()
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || byte == b'_'
                        || byte == b'-'
                })
        })
}

#[cfg(test)]
mod tests {
    use super::encode_request;
    use crate::{ErrorCode, MAX_PAYLOAD_BYTES};

    #[test]
    fn service_request_uses_the_core_little_endian_versioned_wire_format() {
        let encoded = encode_request("provider", "inventory.query", "^1.2", &[0, 1, 255]).unwrap();
        assert_eq!(&encoded[0..4], &1u32.to_le_bytes());
        assert_eq!(&encoded[4..8], &8u32.to_le_bytes());
        assert_eq!(&encoded[8..16], b"provider");
        assert_eq!(ErrorCode::from_abi_code(26), ErrorCode::TaskLimitReached);
    }

    #[test]
    fn service_request_rejects_invalid_names_and_oversized_payloads() {
        assert_eq!(
            encode_request("Provider", "inventory.query", "*", &[])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidTransfer
        );
        assert_eq!(
            encode_request(
                "provider",
                "inventory.query",
                "*",
                &vec![0; MAX_PAYLOAD_BYTES]
            )
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
        assert_eq!(
            encode_request("provider", &format!("a{}", "b".repeat(512)), "*", &[])
                .unwrap_err()
                .code(),
            ErrorCode::PayloadTooLarge
        );
    }
}
