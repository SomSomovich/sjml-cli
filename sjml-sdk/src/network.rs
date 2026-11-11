//! Manifest-declared, typed network channels and module-local callback subscriptions.

use crate::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// A validated channel identity. Local references resolve to the calling package and module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelRef {
    owner: Option<(String, String)>,
    name: String,
}

impl ChannelRef {
    pub fn local(name: &str) -> SjmlResult<Self> {
        validate_channel(name)?;
        Ok(Self {
            owner: None,
            name: name.to_owned(),
        })
    }

    /// References an explicitly named owner; foreign subscriptions require `network.external`.
    pub fn new(package_id: &str, module_id: &str, name: &str) -> SjmlResult<Self> {
        if !valid_owner(package_id) || !valid_owner(module_id) {
            return Err(invalid("network owner must contain 1..=64 lowercase ASCII letters, digits, '_' or '-', starting with a letter"));
        }
        validate_channel(name)?;
        Ok(Self {
            owner: Some((package_id.to_owned(), module_id.to_owned())),
            name: name.to_owned(),
        })
    }

    pub fn package_id(&self) -> Option<&str> {
        self.owner.as_ref().map(|owner| owner.0.as_str())
    }
    pub fn module_id(&self) -> Option<&str> {
        self.owner.as_ref().map(|owner| owner.1.as_str())
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl StableEncode for ChannelRef {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(1)?;
        match &self.owner {
            None => encoder.write_u8(0)?,
            Some((package, module)) => {
                encoder.write_u8(1)?;
                package.encode(encoder)?;
                module.encode(encoder)?;
            }
        }
        self.name.encode(encoder)
    }
}

impl StableDecode for ChannelRef {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported network channel reference schema",
            ));
        }
        match decoder.read_u8()? {
            0 => Self::local(&String::decode(decoder)?),
            1 => Self::new(
                &String::decode(decoder)?,
                &String::decode(decoder)?,
                &String::decode(decoder)?,
            ),
            _ => Err(invalid("unknown network channel reference scope")),
        }
    }
}

fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= 64
        && owner.as_bytes()[0].is_ascii_lowercase()
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte))
}

fn validate_channel(name: &str) -> SjmlResult<()> {
    if name.is_empty()
        || name.len() > 128
        || !name.split('.').all(|part| {
            !part.is_empty()
                && part.as_bytes()[0].is_ascii_lowercase()
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_-".contains(&byte)
                })
        })
    {
        return Err(invalid("network channel must contain 1..=128 lowercase ASCII letters, digits, '_', '-' or '.'; each dot-separated segment must start with a letter"));
    }
    Ok(())
}

fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn test_context() -> NetworkContext {
        NetworkContext {
            snapshot: NetworkContextSnapshot {
                channel: ChannelRef::local("chat").unwrap(),
                direction: NetworkDirection::ClientToServer,
                side: LogicalSide::Server,
                player_uuid: None,
            },
            reply_scope: None,
        }
    }

    #[test]
    fn callback_removal_prevents_further_delivery_without_reusing_ids() {
        let delivered = Rc::new(Cell::new(0));
        let capture = delivered.clone();
        let mut registry = Callbacks::default();
        let id = registry
            .insert::<u32, _>(move |_, value| {
                capture.set(value);
                Ok(())
            })
            .unwrap();
        let callback = registry.get(id).unwrap();
        callback.borrow_mut()(test_context(), &crate::to_bytes(&17_u32).unwrap()).unwrap();
        assert_eq!(delivered.get(), 17);
        registry.remove(id);
        assert_eq!(
            registry.get(id).err().unwrap().code(),
            ErrorCode::StaleHandle
        );
        assert!(registry.insert::<u32, _>(|_, _| Ok(())).unwrap() > id);
    }

    #[test]
    fn malformed_payload_does_not_invoke_callback() {
        let delivered = Rc::new(Cell::new(false));
        let capture = delivered.clone();
        let mut registry = Callbacks::default();
        let id = registry
            .insert::<bool, _>(move |_, _| {
                capture.set(true);
                Ok(())
            })
            .unwrap();
        let callback = registry.get(id).unwrap();
        assert_eq!(
            callback.borrow_mut()(test_context(), &[2])
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
        assert!(!delivered.get());
        assert!(callback.borrow_mut()(test_context(), &[0, 0]).is_err());
        assert!(!delivered.get());
    }

    #[test]
    fn callback_failures_do_not_remove_other_callbacks() {
        let mut registry = Callbacks::default();
        let failed = registry
            .insert::<(), _>(|_, _| Err(invalid("handler failed")))
            .unwrap();
        let healthy = registry.insert::<(), _>(|_, _| Ok(())).unwrap();
        assert!(registry.get(failed).unwrap().borrow_mut()(test_context(), &[]).is_err());
        assert!(registry.get(healthy).unwrap().borrow_mut()(test_context(), &[]).is_ok());
    }

    #[test]
    fn callback_registry_is_bounded() {
        let mut registry = Callbacks::default();
        for _ in 0..MAX_SUBSCRIPTIONS {
            registry.insert::<(), _>(|_, _| Ok(())).unwrap();
        }
        assert_eq!(
            registry.insert::<(), _>(|_, _| Ok(())).unwrap_err().code(),
            ErrorCode::RegistrationLimitReached
        );
        registry.remove(1);
        assert!(registry.insert::<(), _>(|_, _| Ok(())).is_ok());
    }

    #[test]
    fn reply_authority_expires_even_when_handler_returns_an_error() {
        let retained = Rc::new(RefCell::new(None));
        let capture = retained.clone();
        let mut registry = Callbacks::default();
        let id = registry
            .insert::<(), _>(move |context, _| {
                assert!(context.reply_scope.as_ref().unwrap().active.get());
                *capture.borrow_mut() = Some(context.clone());
                Err(invalid("handler failure"))
            })
            .unwrap();
        let result = invoke_callback(registry.get(id).unwrap(), test_context(), &[]);
        assert_eq!(result.unwrap_err().code(), ErrorCode::InvalidArgument);
        let context = retained.borrow();
        assert_eq!(
            context.as_ref().unwrap().reply(&()).unwrap_err().code(),
            ErrorCode::StaleHandle
        );
    }

    #[test]
    fn callback_can_remove_itself_without_borrowing_the_registry_during_execution() {
        let registry = Rc::new(RefCell::new(Callbacks::default()));
        let capture = registry.clone();
        let id = registry
            .borrow_mut()
            .insert::<(), _>(move |_, _| {
                capture.borrow_mut().remove(1);
                Ok(())
            })
            .unwrap();
        let callback = registry.borrow().get(id).unwrap();
        invoke_callback(callback, test_context(), &[]).unwrap();
        assert_eq!(
            registry.borrow().get(id).err().unwrap().code(),
            ErrorCode::StaleHandle
        );
    }

    #[test]
    fn delivery_payload_uses_remaining_bytes_without_a_collection_item_limit() {
        let mut bytes = 1_u32.to_le_bytes().to_vec();
        bytes.extend(7_u64.to_le_bytes());
        bytes.extend(crate::to_bytes(&test_context()).unwrap());
        bytes.extend(vec![0; 70_000]);
        let decoded: Delivery = crate::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.callback_id, 7);
        assert!(decoded.context.is_server());
        assert_eq!(decoded.payload.len(), 70_000);
        bytes.resize(bytes.len() + MAX_MESSAGE_BYTES, 0);
        assert_eq!(
            crate::from_bytes::<Delivery>(&bytes).err().unwrap().code(),
            ErrorCode::PayloadTooLarge
        );
    }

    #[test]
    fn exhausted_callback_ids_never_wrap_and_steal_an_old_delivery() {
        let mut registry = Callbacks {
            next: Some(u64::MAX),
            ..Callbacks::default()
        };
        let last = registry.insert::<(), _>(|_, _| Ok(())).unwrap();
        assert_eq!(last, u64::MAX);
        registry.remove(last);
        assert_eq!(
            registry.insert::<(), _>(|_, _| Ok(())).unwrap_err().code(),
            ErrorCode::RegistrationLimitReached
        );
    }
}

#[cfg(any(target_arch = "wasm32", test))]
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// Maximum live guest callbacks per module instance, independent of host quotas.
pub const MAX_SUBSCRIPTIONS: usize = 1024;

#[cfg(any(target_arch = "wasm32", test))]
type Callback = Rc<RefCell<Box<dyn FnMut(NetworkContext, &[u8]) -> SjmlResult<()>>>>;

#[cfg(any(target_arch = "wasm32", test))]
struct CallbackEntry {
    callback: Callback,
    #[cfg(target_arch = "wasm32")]
    handle: Option<NetworkSubscriptionHandle>,
}

#[cfg(any(target_arch = "wasm32", test))]
struct Callbacks {
    next: Option<u64>,
    entries: BTreeMap<u64, CallbackEntry>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl Default for Callbacks {
    fn default() -> Self {
        Self {
            next: Some(1),
            entries: BTreeMap::new(),
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl Callbacks {
    fn insert<T, F>(&mut self, mut callback: F) -> SjmlResult<u64>
    where
        T: StableDecode + 'static,
        F: FnMut(NetworkContext, T) -> SjmlResult<()> + 'static,
    {
        if self.entries.len() >= MAX_SUBSCRIPTIONS {
            return Err(SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "network callback limit reached",
            ));
        }
        let id = self.next.ok_or_else(|| {
            SjmlError::new(
                ErrorCode::RegistrationLimitReached,
                "network callback identity space exhausted",
            )
        })?;
        self.next = id.checked_add(1);
        self.entries.insert(
            id,
            CallbackEntry {
                callback: Rc::new(RefCell::new(Box::new(move |context, payload| {
                    callback(context, crate::from_bytes::<T>(payload)?)
                }))),
                #[cfg(target_arch = "wasm32")]
                handle: None,
            },
        );
        Ok(id)
    }

    fn get(&self, id: u64) -> SjmlResult<Callback> {
        self.entries
            .get(&id)
            .map(|entry| entry.callback.clone())
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "network callback is no longer subscribed",
                )
            })
    }

    fn remove(&mut self, id: u64) {
        self.entries.remove(&id);
    }
}

/// Host-scoped network subscription token (reserved ABI handle kind 9).
pub type NetworkSubscriptionHandle = crate::NetworkChannelHandle;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static CALLBACKS: RefCell<Callbacks> = RefCell::new(Callbacks::default());
    static DISPATCHING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_network_v1")]
extern "C" {
    #[link_name = "subscribe"]
    fn imported_subscribe(
        channel_pointer: i32,
        channel_length: i32,
        callback_id: i64,
        handle_output_pointer: i32,
    ) -> i32;
    #[link_name = "unsubscribe"]
    fn imported_unsubscribe(handle_pointer: i32, handle_length: i32) -> i32;
}

/// Subscribes one typed closure to a manifest-declared channel.
///
/// Host imports must not deliver callbacks inline. After this function returns, the host may
/// queue callback dispatch on the module's owning lane. The handle remains subscribed until
/// explicit unsubscription or module unload; dropping a copied handle does not unsubscribe it.
pub fn subscribe<T, F>(channel: ChannelRef, callback: F) -> SjmlResult<NetworkSubscriptionHandle>
where
    T: StableDecode + 'static,
    F: FnMut(NetworkContext, T) -> SjmlResult<()> + 'static,
{
    #[cfg(target_arch = "wasm32")]
    {
        use crate::abi::{allocate_host_input, deallocate_host_input};
        let channel_bytes = crate::to_bytes(&channel)?;
        let id = CALLBACKS.with(|registry| registry.borrow_mut().insert::<T, F>(callback))?;
        const HANDLE_BYTES: u32 = 24;
        let output = allocate_host_input(HANDLE_BYTES);
        if output == 0 {
            CALLBACKS.with(|registry| registry.borrow_mut().remove(id));
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "could not allocate network handle output",
            ));
        }
        // SAFETY: initialize the full live output allocation before exposing it to the host.
        unsafe { core::ptr::write_bytes(output as *mut u8, 0, HANDLE_BYTES as usize) };
        // SAFETY: both borrowed channel bytes and the exact handle output allocation are live.
        let status = unsafe {
            imported_subscribe(
                channel_bytes.as_ptr() as i32,
                channel_bytes.len() as i32,
                id as i64,
                output as i32,
            )
        };
        let result = if status == 0 {
            // SAFETY: the complete output allocation was initialized and remains live.
            crate::from_bytes::<NetworkSubscriptionHandle>(unsafe {
                core::slice::from_raw_parts(output as *const u8, HANDLE_BYTES as usize)
            })
        } else {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected network subscription",
            ))
        };
        // SAFETY: release exactly the allocation returned above, once the host call has ended.
        unsafe { deallocate_host_input(output, HANDLE_BYTES) };
        match result {
            Ok(handle) => {
                CALLBACKS.with(|registry| {
                    registry
                        .borrow_mut()
                        .entries
                        .get_mut(&id)
                        .expect("host cannot dispatch inline")
                        .handle = Some(handle);
                });
                Ok(handle)
            }
            Err(error) => {
                CALLBACKS.with(|registry| registry.borrow_mut().remove(id));
                Err(error)
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (channel, callback);
        Err(unsupported())
    }
}

/// Removes the host subscription and its guest closure after successful host acknowledgement.
pub fn unsubscribe(subscription: NetworkSubscriptionHandle) -> SjmlResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let id = CALLBACKS
            .with(|registry| {
                registry
                    .borrow()
                    .entries
                    .iter()
                    .find_map(|(&id, entry)| (entry.handle == Some(subscription)).then_some(id))
            })
            .ok_or_else(|| {
                SjmlError::new(
                    ErrorCode::StaleHandle,
                    "network subscription is not active in this guest instance",
                )
            })?;
        let bytes = crate::to_bytes(&subscription)?;
        // SAFETY: the handle's borrowed stable encoding remains live for the host call.
        let status = unsafe { imported_unsubscribe(bytes.as_ptr() as i32, bytes.len() as i32) };
        if status != 0 {
            return Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected network unsubscription",
            ));
        }
        CALLBACKS.with(|registry| registry.borrow_mut().remove(id));
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = subscription;
        Err(unsupported())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "network operations require wasm32-unknown-unknown and host networking imports",
    )
}

#[cfg(any(target_arch = "wasm32", test))]
struct Delivery {
    callback_id: u64,
    context: NetworkContext,
    payload: Vec<u8>,
}

#[cfg(any(target_arch = "wasm32", test))]
impl StableDecode for Delivery {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported network delivery schema",
            ));
        }
        let callback_id = decoder.read_u64()?;
        let context = NetworkContext::decode(decoder)?;
        if decoder.remaining() > MAX_MESSAGE_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "network delivery message exceeds its payload bound",
            ));
        }
        let payload = decoder.read_raw(decoder.remaining())?.to_vec();
        Ok(Self {
            callback_id,
            context,
            payload,
        })
    }
}

/// Dispatches one queued host delivery to its stored typed guest closure.
///
/// # Safety
/// The host must supply one initialized live allocation returned by `sjml_v1_alloc`, retain
/// it until return, and invoke this export only after the originating guest invocation ends.
/// Both the delivery envelope and its message share the ABI's 1 MiB input bound. The host
/// must release the input via `sjml_v1_dealloc_input` and the packed response via `sjml_v1_dealloc`.
#[cfg(target_arch = "wasm32")]
#[export_name = "sjml_v1_network_dispatch"]
pub unsafe extern "C" fn dispatch(pointer: u32, length: u32) -> u64 {
    // SAFETY: the caller guarantees the exact live initialized callback allocation.
    unsafe {
        crate::abi::dispatch_payload(pointer, length, |delivery: Delivery| {
            if DISPATCHING.with(|active| active.replace(true)) {
                return Err(SjmlError::new(
                    ErrorCode::ServiceBusy,
                    "reentrant guest network dispatch is forbidden",
                ));
            }
            struct DispatchGuard;
            impl Drop for DispatchGuard {
                fn drop(&mut self) {
                    DISPATCHING.with(|active| active.set(false));
                }
            }
            let _guard = DispatchGuard;
            let callback =
                CALLBACKS.with(|registry| registry.borrow().get(delivery.callback_id))?;
            invoke_callback(callback, delivery.context, &delivery.payload)
        })
    }
}

/// An RFC UUID represented as sixteen bytes in canonical textual (network) order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Uuid([u8; 16]);

impl Uuid {
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl core::str::FromStr for Uuid {
    type Err = SjmlError;
    fn from_str(value: &str) -> SjmlResult<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 36 || [8, 13, 18, 23].iter().any(|&i| bytes[i] != b'-') {
            return Err(invalid(
                "UUID must use the canonical 8-4-4-4-12 hexadecimal form",
            ));
        }
        let mut output = [0; 16];
        let mut offset = 0;
        for (index, byte) in bytes.iter().copied().enumerate() {
            if [8, 13, 18, 23].contains(&index) {
                continue;
            }
            let nibble = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(invalid("UUID contains a non-hexadecimal character")),
            };
            output[offset / 2] |= nibble << (if offset % 2 == 0 { 4 } else { 0 });
            offset += 1;
        }
        Ok(Self(output))
    }
}
impl core::fmt::Display for Uuid {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (index, byte) in self.0.iter().enumerate() {
            if [4, 6, 8, 10].contains(&index) {
                formatter.write_str("-")?;
            }
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
impl StableEncode for Uuid {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_raw(&self.0)
    }
}
impl StableDecode for Uuid {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        Ok(Self(
            decoder
                .read_raw(16)?
                .try_into()
                .expect("fixed-width UUID read"),
        ))
    }
}

/// The actual authenticated transport direction, independent of local receiving side.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum NetworkDirection {
    ClientToServer = 1,
    ServerToClient = 2,
}

/// The logical receiving side; integrated-server callbacks still have side `Server`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LogicalSide {
    Client = 1,
    Server = 2,
}

impl StableEncode for NetworkDirection {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(*self as u32)
    }
}
impl StableDecode for NetworkDirection {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u32()? {
            1 => Ok(Self::ClientToServer),
            2 => Ok(Self::ServerToClient),
            _ => Err(invalid("unknown network direction")),
        }
    }
}
impl StableEncode for LogicalSide {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(*self as u32)
    }
}
impl StableDecode for LogicalSide {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        match decoder.read_u32()? {
            1 => Ok(Self::Client),
            2 => Ok(Self::Server),
            _ => Err(invalid("unknown logical network side")),
        }
    }
}

/// Read-only fields of a received context. `NetworkContext` dereferences to this snapshot
/// without providing mutable access, so `context.channel` remains convenient and immutable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkContextSnapshot {
    pub channel: ChannelRef,
    pub direction: NetworkDirection,
    pub side: LogicalSide,
    /// Authenticated connection identity supplied by the host, never decoded from the peer payload.
    pub player_uuid: Option<Uuid>,
}

#[derive(Debug)]
struct ReplyScope {
    active: std::cell::Cell<bool>,
}

/// Detached authenticated delivery metadata with an opaque, callback-scoped reply capability.
///
/// Cloning or serializing metadata does not extend reply lifetime. Deserialization grants no
/// reply capability. Field access is read-only; there is deliberately no `DerefMut`.
///
/// ```compile_fail
/// use sjml::network::{LogicalSide, NetworkContext};
/// fn cannot_rewrite_authenticated_side(mut context: NetworkContext) {
///     context.side = LogicalSide::Client;
/// }
/// ```
#[derive(Clone, Debug)]
pub struct NetworkContext {
    snapshot: NetworkContextSnapshot,
    reply_scope: Option<std::rc::Rc<ReplyScope>>,
}
impl core::ops::Deref for NetworkContext {
    type Target = NetworkContextSnapshot;
    fn deref(&self) -> &Self::Target {
        &self.snapshot
    }
}
impl NetworkContext {
    pub fn is_server(&self) -> bool {
        self.side == LogicalSide::Server
    }
    pub fn is_client(&self) -> bool {
        self.side == LogicalSide::Client
    }

    /// Resolves the authenticated UUID through `entity.read`-gated player APIs.
    /// Returns `None` when no player identity exists or its online session has ended.
    pub fn player(&self) -> SjmlResult<Option<Player>> {
        let handles = crate::player::online()?;
        let Some(uuid) = self.player_uuid else {
            return Ok(None);
        };
        for handle in handles {
            match crate::player::snapshot(handle) {
                Ok(snapshot) if snapshot.uuid.parse::<Uuid>()? == uuid => {
                    return Ok(Some(Player { handle }))
                }
                Ok(_) => {}
                Err(error) if error.code() == ErrorCode::StaleHandle => {}
                Err(error) => return Err(error),
            }
        }
        Ok(None)
    }

    /// Replies only while this exact callback is executing and its connection remains open.
    /// The host determines the authenticated peer; no guest-supplied connection ID exists.
    pub fn reply<T: StableEncode + ?Sized>(&self, message: &T) -> SjmlResult<()> {
        if !self
            .reply_scope
            .as_ref()
            .is_some_and(|scope| scope.active.get())
        {
            return Err(SjmlError::new(
                ErrorCode::StaleHandle,
                "network reply scope expired or was not issued by a live delivery",
            ));
        }
        let bytes = encode_message(message)?;
        #[cfg(target_arch = "wasm32")]
        {
            let channel = crate::to_bytes(&self.channel)?;
            // SAFETY: channel and bounded message encodings remain live throughout the host call.
            let status = unsafe {
                imported_reply(
                    channel.as_ptr() as i32,
                    channel.len() as i32,
                    bytes.as_ptr() as i32,
                    bytes.len() as i32,
                )
            };
            status_result(status, "host rejected network reply")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = bytes;
            Err(unsupported())
        }
    }
}
impl StableEncode for NetworkContext {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        encoder.write_u32(1)?;
        self.channel.encode(encoder)?;
        self.direction.encode(encoder)?;
        self.side.encode(encoder)?;
        self.player_uuid.encode(encoder)
    }
}
impl StableDecode for NetworkContext {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        if decoder.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported network context schema",
            ));
        }
        let channel = ChannelRef::decode(decoder)?;
        let direction = NetworkDirection::decode(decoder)?;
        let side = LogicalSide::decode(decoder)?;
        let player_uuid = Option::<Uuid>::decode(decoder)?;
        if !matches!(
            (direction, side),
            (NetworkDirection::ClientToServer, LogicalSide::Server)
                | (NetworkDirection::ServerToClient, LogicalSide::Client)
        ) {
            return Err(invalid(
                "network context direction does not match its receiving side",
            ));
        }
        Ok(Self {
            snapshot: NetworkContextSnapshot {
                channel,
                direction,
                side,
                player_uuid,
            },
            reply_scope: None,
        })
    }
}

/// A module-scoped online player reference. Host handles are kept private and checked on use.
#[derive(Clone, Copy, Debug)]
pub struct Player {
    handle: crate::PlayerHandle,
}
impl Player {
    /// Wraps a typed handle obtained from the existing permitted player API.
    /// Wrapping performs no authorization; the host validates the handle on every operation.
    pub fn from_handle(handle: crate::PlayerHandle) -> Self {
        Self { handle }
    }
    pub fn snapshot(self) -> SjmlResult<crate::PlayerSnapshot> {
        crate::player::snapshot(self.handle)
    }
}

impl From<crate::PlayerHandle> for Player {
    fn from(handle: crate::PlayerHandle) -> Self {
        Self::from_handle(handle)
    }
}

// Maximum delivery envelope: schema + callback ID + context schema + fully qualified channel
// (schema, scope, two 64-byte owners, one 128-byte name) + direction + side + optional UUID.
const MAX_DELIVERY_OVERHEAD: usize = 4 + 8 + 4 + (4 + 1 + (4 + 64) * 2 + 4 + 128) + 4 + 4 + 1 + 16;
/// Maximum message encoding that leaves room for the full authenticated delivery envelope.
pub const MAX_MESSAGE_BYTES: usize = crate::MAX_PAYLOAD_BYTES - MAX_DELIVERY_OVERHEAD;

fn encode_message<T: StableEncode + ?Sized>(message: &T) -> SjmlResult<Vec<u8>> {
    let bytes = crate::to_bytes(message)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "network message and delivery envelope exceed the ABI payload bound",
        ));
    }
    Ok(bytes)
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_network_v1")]
extern "C" {
    #[link_name = "reply"]
    fn imported_reply(cp: i32, cl: i32, pp: i32, pl: i32) -> i32;
    #[link_name = "send_to_server"]
    fn imported_send_to_server(cp: i32, cl: i32, pp: i32, pl: i32) -> i32;
    #[link_name = "send_to"]
    fn imported_send_to(cp: i32, cl: i32, hp: i32, hl: i32, pp: i32, pl: i32) -> i32;
    #[link_name = "broadcast"]
    fn imported_broadcast(cp: i32, cl: i32, pp: i32, pl: i32) -> i32;
}

/// Queues a client-to-server message on an allowed manifest channel.
pub fn send_to_server<T: StableEncode + ?Sized>(
    channel: ChannelRef,
    message: &T,
) -> SjmlResult<()> {
    let bytes = encode_message(message)?;
    #[cfg(target_arch = "wasm32")]
    {
        let channel = crate::to_bytes(&channel)?;
        // SAFETY: borrowed channel/message encodings remain live throughout the synchronous import.
        status_result(
            unsafe {
                imported_send_to_server(
                    channel.as_ptr() as i32,
                    channel.len() as i32,
                    bytes.as_ptr() as i32,
                    bytes.len() as i32,
                )
            },
            "host rejected client-to-server send",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (channel, bytes);
        Err(unsupported())
    }
}

/// Queues a server-to-client message for one authenticated online player session.
pub fn send_to<T: StableEncode + ?Sized>(
    channel: ChannelRef,
    player: impl Into<Player>,
    message: &T,
) -> SjmlResult<()> {
    let player = player.into();
    let bytes = encode_message(message)?;
    #[cfg(target_arch = "wasm32")]
    {
        let channel = crate::to_bytes(&channel)?;
        let player = crate::to_bytes(&player.handle)?;
        // SAFETY: all borrowed stable encodings remain live for the complete synchronous import.
        status_result(
            unsafe {
                imported_send_to(
                    channel.as_ptr() as i32,
                    channel.len() as i32,
                    player.as_ptr() as i32,
                    player.len() as i32,
                    bytes.as_ptr() as i32,
                    bytes.len() as i32,
                )
            },
            "host rejected targeted server-to-client send",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (channel, player, bytes);
        Err(unsupported())
    }
}

/// Queues a server-to-client message for connected clients with the negotiated channel.
pub fn broadcast<T: StableEncode + ?Sized>(channel: ChannelRef, message: &T) -> SjmlResult<()> {
    let bytes = encode_message(message)?;
    #[cfg(target_arch = "wasm32")]
    {
        let channel = crate::to_bytes(&channel)?;
        // SAFETY: borrowed channel/message encodings remain live throughout the synchronous import.
        status_result(
            unsafe {
                imported_broadcast(
                    channel.as_ptr() as i32,
                    channel.len() as i32,
                    bytes.as_ptr() as i32,
                    bytes.len() as i32,
                )
            },
            "host rejected broadcast",
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (channel, bytes);
        Err(unsupported())
    }
}

#[cfg(target_arch = "wasm32")]
fn status_result(status: i32, message: &'static str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn invoke_callback(
    callback: Callback,
    mut context: NetworkContext,
    payload: &[u8],
) -> SjmlResult<()> {
    let scope = std::rc::Rc::new(ReplyScope {
        active: std::cell::Cell::new(true),
    });
    struct ReplyGuard(std::rc::Rc<ReplyScope>);
    impl Drop for ReplyGuard {
        fn drop(&mut self) {
            self.0.active.set(false);
        }
    }
    let _guard = ReplyGuard(scope.clone());
    context.reply_scope = Some(scope);
    let mut callback = callback.try_borrow_mut().map_err(|_| {
        SjmlError::new(
            ErrorCode::ServiceBusy,
            "network callback is already running",
        )
    })?;
    callback(context, payload)
}
