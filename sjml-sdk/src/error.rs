use core::fmt;

/// Stable error values shared with SJML ABI v1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ErrorCode {
    IncompatibleAbiVersion = 1,
    MemoryOutOfBounds = 2,
    PayloadTooLarge = 3,
    InvalidUtf8 = 4,
    InvalidBoolean = 5,
    ArithmeticOverflow = 6,
    InvalidArgument = 7,
    InvalidHandle = 16,
    StaleHandle = 17,
    HandleOwnerMismatch = 18,
    HandleContextMismatch = 19,
    HandleKindMismatch = 20,
    HandleNotTransferable = 21,
    HandleTableClosed = 22,
    HandleTableFull = 23,
    PermissionDenied = 24,
    InvalidTransfer = 25,
    TaskLimitReached = 26,
    TaskRejected = 27,
    ServiceNotFound = 28,
    ServiceVersionIncompatible = 29,
    ServiceAccessDenied = 30,
    ServiceBusy = 31,
    ServiceCallFailed = 32,
    ServiceTimeout = 33,
    RegistrationLimitReached = 34,
    UnknownErrorCode = 255,
}

impl ErrorCode {
    /// Maps a host numeric status to the stable SDK error enumeration.
    pub const fn from_abi_code(code: u32) -> Self {
        match code {
            1 => Self::IncompatibleAbiVersion,
            2 => Self::MemoryOutOfBounds,
            3 => Self::PayloadTooLarge,
            4 => Self::InvalidUtf8,
            5 => Self::InvalidBoolean,
            6 => Self::ArithmeticOverflow,
            7 => Self::InvalidArgument,
            16 => Self::InvalidHandle,
            17 => Self::StaleHandle,
            18 => Self::HandleOwnerMismatch,
            19 => Self::HandleContextMismatch,
            20 => Self::HandleKindMismatch,
            21 => Self::HandleNotTransferable,
            22 => Self::HandleTableClosed,
            23 => Self::HandleTableFull,
            24 => Self::PermissionDenied,
            25 => Self::InvalidTransfer,
            26 => Self::TaskLimitReached,
            27 => Self::TaskRejected,
            28 => Self::ServiceNotFound,
            29 => Self::ServiceVersionIncompatible,
            30 => Self::ServiceAccessDenied,
            31 => Self::ServiceBusy,
            32 => Self::ServiceCallFailed,
            33 => Self::ServiceTimeout,
            34 => Self::RegistrationLimitReached,
            _ => Self::UnknownErrorCode,
        }
    }

    pub const fn abi_code(self) -> u32 {
        self as u32
    }
}

/// Typed SDK failure with the stable host-visible error code and a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SjmlError {
    code: ErrorCode,
    message: String,
}

impl SjmlError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn from_abi_code(code: u32, message: impl Into<String>) -> Self {
        Self::new(ErrorCode::from_abi_code(code), message)
    }

    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub const fn abi_code(&self) -> u32 {
        self.code.abi_code()
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SjmlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "SJML ABI error {}: {}",
            self.abi_code(),
            self.message
        )
    }
}

impl std::error::Error for SjmlError {}

/// Result type returned by guest callbacks and host operations.
pub type SjmlResult<T> = core::result::Result<T, SjmlError>;
