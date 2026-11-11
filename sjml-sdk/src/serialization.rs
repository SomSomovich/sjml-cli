use crate::{ErrorCode, SjmlError, SjmlResult};

/// Maximum encoded value accepted by the current SJML ABI v1 payload contract.
pub const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;
/// Maximum UTF-8 string encoded as one stable value.
pub const MAX_STRING_BYTES: usize = 64 * 1024;
/// Maximum number of elements in a single encoded collection.
pub const MAX_COLLECTION_ITEMS: usize = 65_536;
// Limit aggregate Vec backing storage to bound expansion from small untrusted payloads.
const MAX_DECODED_COLLECTION_BYTES: usize = 64 * MAX_PAYLOAD_BYTES;

/// Encoder for the SDK's deterministic, little-endian value format.
#[derive(Debug, Default)]
pub struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn write_u8(&mut self, value: u8) -> SjmlResult<()> {
        self.write_raw(&[value])
    }

    pub fn write_u16(&mut self, value: u16) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_u32(&mut self, value: u32) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_u64(&mut self, value: u64) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_i8(&mut self, value: i8) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_i16(&mut self, value: i16) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_i32(&mut self, value: i32) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_i64(&mut self, value: i64) -> SjmlResult<()> {
        self.write_raw(&value.to_le_bytes())
    }

    pub fn write_f32(&mut self, value: f32) -> SjmlResult<()> {
        self.write_u32(value.to_bits())
    }

    pub fn write_f64(&mut self, value: f64) -> SjmlResult<()> {
        self.write_u64(value.to_bits())
    }

    pub fn write_length(&mut self, length: usize) -> SjmlResult<()> {
        let length = u32::try_from(length).map_err(|_| {
            SjmlError::new(
                ErrorCode::ArithmeticOverflow,
                "stable value length does not fit the ABI u32 field",
            )
        })?;
        self.write_u32(length)
    }

    pub fn write_raw(&mut self, bytes: &[u8]) -> SjmlResult<()> {
        let new_length = self.bytes.len().checked_add(bytes.len()).ok_or_else(|| {
            SjmlError::new(
                ErrorCode::ArithmeticOverflow,
                "encoded payload length overflowed",
            )
        })?;
        if new_length > MAX_PAYLOAD_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                format!("encoded payload exceeds the {MAX_PAYLOAD_BYTES}-byte ABI limit"),
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

/// Bounds-checked decoder for deterministic SJML wire values.
pub struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
    decoded_collection_bytes: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> SjmlResult<Self> {
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                format!("input payload exceeds the {MAX_PAYLOAD_BYTES}-byte ABI limit"),
            ));
        }
        Ok(Self {
            bytes,
            offset: 0,
            decoded_collection_bytes: 0,
        })
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    pub fn read_u8(&mut self) -> SjmlResult<u8> {
        Ok(self.read_raw(1)?[0])
    }

    pub fn read_u16(&mut self) -> SjmlResult<u16> {
        Ok(u16::from_le_bytes(
            self.read_raw(2)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_u32(&mut self) -> SjmlResult<u32> {
        Ok(u32::from_le_bytes(
            self.read_raw(4)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_u64(&mut self) -> SjmlResult<u64> {
        Ok(u64::from_le_bytes(
            self.read_raw(8)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_i8(&mut self) -> SjmlResult<i8> {
        Ok(i8::from_le_bytes(
            self.read_raw(1)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_i16(&mut self) -> SjmlResult<i16> {
        Ok(i16::from_le_bytes(
            self.read_raw(2)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_i32(&mut self) -> SjmlResult<i32> {
        Ok(i32::from_le_bytes(
            self.read_raw(4)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_i64(&mut self) -> SjmlResult<i64> {
        Ok(i64::from_le_bytes(
            self.read_raw(8)?.try_into().expect("fixed-width read"),
        ))
    }

    pub fn read_f32(&mut self) -> SjmlResult<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f64(&mut self) -> SjmlResult<f64> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub fn read_length(&mut self, max: usize) -> SjmlResult<usize> {
        let length = self.read_u32()? as usize;
        if length > max {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                format!("stable collection length {length} exceeds its {max}-item limit"),
            ));
        }
        Ok(length)
    }

    pub fn read_raw(&mut self, length: usize) -> SjmlResult<&'a [u8]> {
        let end = self.offset.checked_add(length).ok_or_else(|| {
            SjmlError::new(ErrorCode::ArithmeticOverflow, "decoder range overflowed")
        })?;
        if end > self.bytes.len() {
            return Err(SjmlError::new(
                ErrorCode::MemoryOutOfBounds,
                "stable value ends outside the provided byte span",
            ));
        }
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn reserve_collection<T>(&mut self, length: usize) -> SjmlResult<Vec<T>> {
        let requested_bytes = length
            .checked_mul(core::mem::size_of::<T>())
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "decoded collection storage exceeds the supported memory budget",
                )
            })?;
        let total_bytes = self
            .decoded_collection_bytes
            .checked_add(requested_bytes)
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "decoded collection storage exceeds the supported memory budget",
                )
            })?;
        if total_bytes > MAX_DECODED_COLLECTION_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                format!(
                    "decoded collections exceed the {MAX_DECODED_COLLECTION_BYTES}-byte memory budget"
                ),
            ));
        }

        let mut values = Vec::new();
        values.try_reserve_exact(length).map_err(|_| {
            SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not reserve memory for the decoded collection",
            )
        })?;
        self.decoded_collection_bytes = total_bytes;
        Ok(values)
    }
}

/// Encodes a value in the versioned SJML little-endian format.
pub trait StableEncode {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()>;
}

/// Decodes a value from the versioned SJML little-endian format.
pub trait StableDecode: Sized {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self>;
}

/// Encodes one value and enforces the ABI v1 payload bound.
pub fn to_bytes<T: StableEncode + ?Sized>(value: &T) -> SjmlResult<Vec<u8>> {
    let mut encoder = Encoder::new();
    value.encode(&mut encoder)?;
    Ok(encoder.into_bytes())
}

/// Decodes exactly one value; trailing data is rejected.
pub fn from_bytes<T: StableDecode>(bytes: &[u8]) -> SjmlResult<T> {
    let mut decoder = Decoder::new(bytes)?;
    let value = T::decode(&mut decoder)?;
    if decoder.remaining() != 0 {
        return Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "stable value has trailing bytes after its complete encoding",
        ));
    }
    Ok(value)
}

macro_rules! impl_integer {
    ($type:ty, $write:ident, $read:ident) => {
        impl StableEncode for $type {
            fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
                encoder.$write(*self)
            }
        }

        impl StableDecode for $type {
            fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
                decoder.$read()
            }
        }
    };
}

impl_integer!(u8, write_u8, read_u8);
impl_integer!(u16, write_u16, read_u16);
impl_integer!(u32, write_u32, read_u32);
impl_integer!(u64, write_u64, read_u64);
impl_integer!(i8, write_i8, read_i8);
impl_integer!(i16, write_i16, read_i16);
impl_integer!(i32, write_i32, read_i32);
impl_integer!(i64, write_i64, read_i64);
impl_integer!(f32, write_f32, read_f32);
impl_integer!(f64, write_f64, read_f64);

impl StableEncode for bool {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u8(u8::from(*self))
    }
}

impl StableDecode for bool {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(SjmlError::new(
                ErrorCode::InvalidBoolean,
                format!("boolean wire value must be 0 or 1, found {value}"),
            )),
        }
    }
}

impl StableEncode for str {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        if self.len() > MAX_STRING_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                format!("UTF-8 string exceeds the {MAX_STRING_BYTES}-byte ABI limit"),
            ));
        }
        encoder.write_length(self.len())?;
        encoder.write_raw(self.as_bytes())
    }
}

impl StableEncode for String {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        self.as_str().encode(encoder)
    }
}

impl StableDecode for String {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let length = decoder.read_length(MAX_STRING_BYTES)?;
        String::from_utf8(decoder.read_raw(length)?.to_vec()).map_err(|_| {
            SjmlError::new(
                ErrorCode::InvalidUtf8,
                "stable string contains malformed UTF-8",
            )
        })
    }
}

impl StableEncode for () {
    fn encode(&self, _: &mut Encoder) -> SjmlResult<()> {
        Ok(())
    }
}

impl StableDecode for () {
    fn decode(_: &mut Decoder<'_>) -> SjmlResult<Self> {
        Ok(())
    }
}

impl<T: StableEncode> StableEncode for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        if self.len() > MAX_COLLECTION_ITEMS {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                format!("stable collection exceeds the {MAX_COLLECTION_ITEMS}-item limit"),
            ));
        }
        encoder.write_length(self.len())?;
        for value in self {
            value.encode(encoder)?;
        }
        Ok(())
    }
}

impl<T: StableDecode> StableDecode for Vec<T> {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let length = decoder.read_length(MAX_COLLECTION_ITEMS)?;
        let mut values = decoder.reserve_collection::<T>(length)?;
        for _ in 0..length {
            values.push(T::decode(decoder)?);
        }
        Ok(values)
    }
}

impl<T: StableEncode> StableEncode for Option<T> {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        match self {
            None => encoder.write_u8(0),
            Some(value) => {
                encoder.write_u8(1)?;
                value.encode(encoder)
            }
        }
    }
}

impl<T: StableDecode> StableDecode for Option<T> {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(T::decode(decoder)?)),
            value => Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                format!("option discriminant must be 0 or 1, found {value}"),
            )),
        }
    }
}

impl<T: StableEncode, E: StableEncode> StableEncode for core::result::Result<T, E> {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        match self {
            Ok(value) => {
                encoder.write_u8(0)?;
                value.encode(encoder)
            }
            Err(error) => {
                encoder.write_u8(1)?;
                error.encode(encoder)
            }
        }
    }
}

impl<T: StableDecode, E: StableDecode> StableDecode for core::result::Result<T, E> {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u8()? {
            0 => Ok(Ok(T::decode(decoder)?)),
            1 => Ok(Err(E::decode(decoder)?)),
            value => Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                format!("result discriminant must be 0 or 1, found {value}"),
            )),
        }
    }
}

impl StableEncode for ErrorCode {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(self.abi_code())
    }
}

impl StableDecode for ErrorCode {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        Ok(Self::from_abi_code(decoder.read_u32()?))
    }
}

impl StableEncode for SjmlError {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        self.code().encode(encoder)?;
        self.message().encode(encoder)
    }
}

impl StableDecode for SjmlError {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let code = ErrorCode::decode(decoder)?;
        let message = String::decode(decoder)?;
        Ok(Self::new(code, message))
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoder, StableDecode, MAX_COLLECTION_ITEMS};
    use crate::{ErrorCode, SjmlError, SjmlResult};
    use std::sync::atomic::{AtomicBool, Ordering};

    static LARGE_VALUE_DECODE_CALLED: AtomicBool = AtomicBool::new(false);

    #[allow(dead_code)]
    struct LargeDecodedValue([u8; 1 << 16]);

    impl StableDecode for LargeDecodedValue {
        fn decode(_: &mut Decoder<'_>) -> SjmlResult<Self> {
            LARGE_VALUE_DECODE_CALLED.store(true, Ordering::Relaxed);
            Err(SjmlError::new(
                ErrorCode::MemoryOutOfBounds,
                "the input has no encoded element",
            ))
        }
    }

    #[test]
    fn truncated_collection_does_not_reserve_capacity_from_its_untrusted_count() {
        let encoded_count = (MAX_COLLECTION_ITEMS as u32).to_le_bytes();
        let mut decoder = Decoder::new(&encoded_count).unwrap();
        LARGE_VALUE_DECODE_CALLED.store(false, Ordering::Relaxed);

        let result = Vec::<LargeDecodedValue>::decode(&mut decoder);

        assert!(matches!(result, Err(error) if error.code() == ErrorCode::PayloadTooLarge));
        assert!(!LARGE_VALUE_DECODE_CALLED.load(Ordering::Relaxed));
    }

    #[test]
    fn maximum_count_of_empty_strings_remains_within_the_decoded_memory_budget() {
        let mut encoded = Vec::with_capacity(4 + MAX_COLLECTION_ITEMS * 4);
        encoded.extend_from_slice(&(MAX_COLLECTION_ITEMS as u32).to_le_bytes());
        for _ in 0..MAX_COLLECTION_ITEMS {
            encoded.extend_from_slice(&0_u32.to_le_bytes());
        }

        let values = crate::from_bytes::<Vec<String>>(&encoded).unwrap();

        assert_eq!(values.len(), MAX_COLLECTION_ITEMS);
        assert!(values.iter().all(String::is_empty));
    }
}
