use crate::{ErrorCode, SjmlError, SjmlResult, MAX_STRING_BYTES};

/// Stable log level values passed to the `sjml.log` host import.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml")]
extern "C" {
    #[link_name = "abi_version"]
    fn imported_abi_version() -> i32;
    #[link_name = "log"]
    fn imported_log(level: i32, pointer: i32, length: i32) -> i32;
}

/// Returns the host-supported ABI major version.
///
/// This call requires the host to allowlist the `sjml.abi_version` import.
pub fn abi_major_version() -> SjmlResult<u32> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the function is a fixed numeric SJML ABI import.
        let version = unsafe { imported_abi_version() };
        if version <= 0 {
            return Err(SjmlError::from_abi_code(
                version.unsigned_abs(),
                "host rejected the ABI version request",
            ));
        }
        Ok(version as u32)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(unsupported_target())
    }
}

/// Writes a UTF-8 message through the capability-checked `sjml.log` import.
pub fn log(level: LogLevel, message: &str) -> SjmlResult<()> {
    if message.len() > MAX_STRING_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            format!("log message exceeds the {MAX_STRING_BYTES}-byte ABI string limit"),
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: the host call is synchronous and the message slice stays live for its duration.
        let status = unsafe {
            imported_log(
                level as i32,
                message.as_ptr() as usize as u32 as i32,
                message.len() as u32 as i32,
            )
        };
        if status == 0 {
            Ok(())
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the log request",
            ))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = level;
        let _ = message;
        Err(unsupported_target())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported_target() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "SJML host imports require the wasm32-unknown-unknown target",
    )
}
