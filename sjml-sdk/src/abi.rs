use core::fmt;
use core::marker::PhantomData;

use crate::serialization::{Decoder, Encoder, StableDecode, StableEncode};
use crate::{ErrorCode, SjmlError, SjmlResult};

/// Marker trait that gives a handle its compile-time and stable ABI kind.
pub trait HandleKind: 'static {
    const KIND_CODE: u32;
}

macro_rules! handle_kind {
    ($marker:ident, $code:literal) => {
        #[derive(Debug)]
        pub enum $marker {}

        impl HandleKind for $marker {
            const KIND_CODE: u32 = $code;
        }
    };
}

handle_kind!(EntityTag, 1);
handle_kind!(PlayerTag, 2);
handle_kind!(WorldTag, 3);
handle_kind!(BlockTag, 4);
handle_kind!(BlockEntityTag, 5);
handle_kind!(ItemStackTag, 6);
handle_kind!(EventSubscriptionTag, 7);
handle_kind!(TaskTag, 8);
handle_kind!(NetworkChannelTag, 9);
handle_kind!(StorageScopeTag, 10);
handle_kind!(RenderResourceTag, 11);
handle_kind!(GlResourceTag, 12);

/// Opaque guest-side token for one host-owned resource.
///
/// The value stores three ABI u64 words. Its Rust type parameter prevents
/// accidentally passing one resource kind where another is required.
pub struct Handle<K: HandleKind> {
    owner_token: u64,
    context_token: u64,
    slot_token: u64,
    marker: PhantomData<fn() -> K>,
}

impl<K: HandleKind> Handle<K> {
    pub const KIND_CODE: u32 = K::KIND_CODE;

    /// Creates a handle from its ABI owner/context and one-based slot fields.
    pub fn new(
        owner_token: u64,
        context_token: u64,
        index: u32,
        generation: u32,
    ) -> SjmlResult<Self> {
        if owner_token == 0 || context_token == 0 || index == 0 || generation == 0 {
            return Err(SjmlError::new(
                ErrorCode::InvalidHandle,
                "ABI handles require non-zero owner, context, index, and generation values",
            ));
        }
        Ok(Self {
            owner_token,
            context_token,
            slot_token: (u64::from(generation) << 32) | u64::from(index),
            marker: PhantomData,
        })
    }

    /// Decodes the exact three-word handle representation from the ABI.
    pub fn from_words(owner_token: u64, context_token: u64, slot_token: u64) -> SjmlResult<Self> {
        let index = slot_token as u32;
        let generation = (slot_token >> 32) as u32;
        Self::new(owner_token, context_token, index, generation)
    }

    pub const fn owner_token(&self) -> u64 {
        self.owner_token
    }

    pub const fn context_token(&self) -> u64 {
        self.context_token
    }

    pub const fn slot_token(&self) -> u64 {
        self.slot_token
    }

    pub const fn index(&self) -> u32 {
        self.slot_token as u32
    }

    pub const fn generation(&self) -> u32 {
        (self.slot_token >> 32) as u32
    }
}

impl<K: HandleKind> Copy for Handle<K> {}

impl<K: HandleKind> Clone for Handle<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: HandleKind> PartialEq for Handle<K> {
    fn eq(&self, other: &Self) -> bool {
        self.owner_token == other.owner_token
            && self.context_token == other.context_token
            && self.slot_token == other.slot_token
    }
}

impl<K: HandleKind> Eq for Handle<K> {}

impl<K: HandleKind> fmt::Debug for Handle<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Handle")
            .field("kind", &K::KIND_CODE)
            .field("owner_token", &self.owner_token)
            .field("context_token", &self.context_token)
            .field("index", &self.index())
            .field("generation", &self.generation())
            .finish()
    }
}

impl<K: HandleKind> StableEncode for Handle<K> {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u64(self.owner_token)?;
        encoder.write_u64(self.context_token)?;
        encoder.write_u64(self.slot_token)
    }
}

impl<K: HandleKind> StableDecode for Handle<K> {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let owner_token = decoder.read_u64()?;
        let context_token = decoder.read_u64()?;
        let slot_token = decoder.read_u64()?;
        Self::from_words(owner_token, context_token, slot_token)
    }
}

pub type EntityHandle = Handle<EntityTag>;
pub type PlayerHandle = Handle<PlayerTag>;
pub type WorldHandle = Handle<WorldTag>;
pub type BlockHandle = Handle<BlockTag>;
pub type BlockEntityHandle = Handle<BlockEntityTag>;
pub type ItemStackHandle = Handle<ItemStackTag>;
pub type EventSubscriptionHandle = Handle<EventSubscriptionTag>;
pub type TaskHandle = Handle<TaskTag>;
pub type NetworkChannelHandle = Handle<NetworkChannelTag>;
pub type StorageScopeHandle = Handle<StorageScopeTag>;
pub type RenderResourceHandle = Handle<RenderResourceTag>;
pub type GlResourceHandle = Handle<GlResourceTag>;

/// Packs an ABI v1 `ByteSpan` as a wasm32 i64 return: pointer in low bits,
/// length in high bits. The byte representation is pointer then length in LE.
pub const fn pack_byte_span(pointer: u32, length: u32) -> u64 {
    ((length as u64) << 32) | pointer as u64
}

/// Splits a packed ABI v1 `ByteSpan` returned by a guest callback.
pub const fn unpack_byte_span(span: u64) -> (u32, u32) {
    (span as u32, (span >> 32) as u32)
}

#[cfg(any(target_arch = "wasm32", test))]
fn guest_span_within_memory(pointer: u32, length: u32, memory_pages: u32) -> bool {
    let end = u64::from(pointer) + u64::from(length);
    let memory_bytes = u64::from(memory_pages) * 65_536;
    end <= memory_bytes
}

/// # Safety
///
/// For non-empty input, `pointer` and `length` must be the exact pair for a
/// live allocation from `allocate_host_input` in this guest instance. The
/// caller must initialize every byte and leave the allocation live and
/// unmodified until this call returns.
#[cfg(target_arch = "wasm32")]
pub(crate) unsafe fn dispatch_payload<I, O, F>(pointer: u32, length: u32, handler: F) -> u64
where
    I: StableDecode,
    O: StableEncode,
    F: FnOnce(I) -> SjmlResult<O>,
{
    // SAFETY: the caller provides a live, initialized callback input allocation.
    let payload = unsafe { read_guest_payload(pointer, length) };
    let result = match payload.and_then(|bytes| crate::from_bytes(&bytes)) {
        Ok(input) => handler(input),
        Err(error) => Err(error),
    };
    allocate_response(result)
}

/// Allocates guest-owned memory for a host-to-guest callback input span.
///
/// The host must pair every successful non-empty allocation with
/// [`deallocate_host_input`] using the exact pointer and byte length.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_alloc"]
pub extern "C" fn allocate_host_input(length: u32) -> u32 {
    use core::alloc::Layout;
    use std::alloc::alloc;

    if length == 0 {
        return 0;
    }
    let Ok(layout) = Layout::from_size_align(length as usize, 1) else {
        return 0;
    };
    // SAFETY: the returned allocation is owned by the guest and released by the
    // paired export with the same layout. The host validates and writes its size.
    let pointer = unsafe { alloc(layout) };
    pointer as usize as u32
}

/// Releases one input allocation previously returned by `sjml_v1_alloc`.
///
/// # Safety
///
/// A nonzero pair must match the pointer and length of one live allocation
/// returned by `allocate_host_input` in this guest instance. Release it only
/// once, after any callback using it has returned.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_dealloc_input"]
pub unsafe extern "C" fn deallocate_host_input(pointer: u32, length: u32) {
    use core::alloc::Layout;
    use std::alloc::dealloc;

    if pointer == 0 || length == 0 {
        return;
    }
    let Ok(layout) = Layout::from_size_align(length as usize, 1) else {
        return;
    };
    // SAFETY: the caller supplies a live allocation with this exact layout.
    unsafe { dealloc(pointer as usize as *mut u8, layout) };
}

/// # Safety
///
/// For non-empty input, the caller must provide a live, initialized callback
/// input allocation and must keep it unmodified while it is copied.
#[cfg(target_arch = "wasm32")]
unsafe fn read_guest_payload(pointer: u32, length: u32) -> SjmlResult<Vec<u8>> {
    use core::arch::wasm32;

    let length = length as usize;
    if length > crate::MAX_PAYLOAD_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "guest callback input exceeds the 1 MiB ABI payload limit",
        ));
    }
    let memory_pages = wasm32::memory_size(0) as u32;
    if !guest_span_within_memory(pointer, length as u32, memory_pages) {
        return Err(SjmlError::new(
            ErrorCode::MemoryOutOfBounds,
            "guest callback input is outside WebAssembly linear memory",
        ));
    }
    if length == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: the caller provides a live, initialized allocation, and the
    // span was checked against this module's current linear memory.
    Ok(unsafe { core::slice::from_raw_parts(pointer as *const u8, length) }.to_vec())
}

#[cfg(target_arch = "wasm32")]
fn allocate_response<O: StableEncode>(result: SjmlResult<O>) -> u64 {
    let bytes = match result {
        Ok(value) => {
            let mut payload = Encoder::new();
            match value.encode(&mut payload) {
                Ok(()) => bounded_success_frame(payload.into_bytes()),
                Err(error) => error_frame(error),
            }
        }
        Err(error) => error_frame(error),
    };
    let boxed = bytes.into_boxed_slice();
    let length = boxed.len() as u32;
    let pointer = Box::into_raw(boxed) as *mut u8 as u32;
    pack_byte_span(pointer, length)
}

#[cfg(any(target_arch = "wasm32", test))]
fn bounded_success_frame(payload: Vec<u8>) -> Vec<u8> {
    let total_length = payload.len().checked_add(4);
    if total_length.is_none_or(|length| length > crate::MAX_PAYLOAD_BYTES) {
        return error_frame(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "callback result and status exceed the 1 MiB ABI payload limit",
        ));
    }
    response_frame(0, payload)
}

#[cfg(any(target_arch = "wasm32", test))]
fn response_frame(status: u32, payload: Vec<u8>) -> Vec<u8> {
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&status.to_le_bytes());
    frame.extend_from_slice(&payload);
    frame
}

#[cfg(any(target_arch = "wasm32", test))]
fn error_frame(error: SjmlError) -> Vec<u8> {
    let mut message = error.message().as_bytes();
    if message.len() > crate::MAX_STRING_BYTES {
        let mut boundary = crate::MAX_STRING_BYTES;
        while !error.message().is_char_boundary(boundary) {
            boundary -= 1;
        }
        message = &error.message().as_bytes()[..boundary];
    }
    let mut frame = Vec::with_capacity(8 + message.len());
    frame.extend_from_slice(&error.abi_code().to_le_bytes());
    frame.extend_from_slice(&(message.len() as u32).to_le_bytes());
    frame.extend_from_slice(message);
    frame
}

/// Releases the exact output span returned by an SJML callback export.
///
/// # Safety
///
/// A nonzero pair must match the pointer and length of one live output span
/// returned by a callback in this guest instance. Release it only once,
/// after any reads of its contents have finished.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_dealloc"]
pub unsafe extern "C" fn dealloc_response(pointer: u32, length: u32) {
    if pointer == 0 || length == 0 {
        return;
    }
    let slice = core::ptr::slice_from_raw_parts_mut(pointer as *mut u8, length as usize);
    // SAFETY: the caller supplies the live allocation originally returned by the callback.
    unsafe { drop(Box::from_raw(slice)) };
}

#[cfg(test)]
mod tests {
    use super::{bounded_success_frame, guest_span_within_memory};
    use crate::{ErrorCode, MAX_PAYLOAD_BYTES};

    #[test]
    fn wasm32_span_bounds_cover_the_full_four_gib_address_space_without_overflow() {
        assert!(guest_span_within_memory(u32::MAX - 3, 4, 65_536));
        assert!(!guest_span_within_memory(u32::MAX, 2, 65_536));
        assert!(!guest_span_within_memory(1, 1, 0));
        assert!(guest_span_within_memory(0, 0, 0));
    }

    #[test]
    fn callback_success_frame_includes_its_status_word_in_the_payload_limit() {
        let fits = bounded_success_frame(vec![0; MAX_PAYLOAD_BYTES - 4]);
        assert_eq!(fits.len(), MAX_PAYLOAD_BYTES);
        assert_eq!(u32::from_le_bytes(fits[0..4].try_into().unwrap()), 0);

        let exceeds = bounded_success_frame(vec![0; MAX_PAYLOAD_BYTES - 3]);
        assert_eq!(
            u32::from_le_bytes(exceeds[0..4].try_into().unwrap()),
            ErrorCode::PayloadTooLarge.abi_code()
        );
        assert!(exceeds.len() < MAX_PAYLOAD_BYTES);
    }
}
