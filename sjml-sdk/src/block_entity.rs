//! Block-entity snapshots and bounded persistent typed data.
//! Raw NBT snapshots require `block_entity.nbt.read`; persistent data reads require
//! `block_entity.data.read`. Persistent writes require `block_entity.data.write`
//! and `world.write` and are available only for the calling module's own SJML blocks.
#[cfg(any(target_arch = "wasm32", test))]
use crate::to_bytes;
#[cfg(target_arch = "wasm32")]
use crate::{
    abi::{allocate_host_input, deallocate_host_input},
    Decoder,
};
use crate::{
    BlockEntityHandle, BlockPos, Direction, ErrorCode, InventorySnapshot, SjmlError, SjmlResult,
    StableDecode, StableEncode, WorldHandle,
};

/// Detached values describing a loaded block entity and its current block state.
#[derive(Debug, Clone, Eq, PartialEq, StableEncode, StableDecode)]
#[sjml(version = 1)]
pub struct BlockEntitySnapshot {
    pub type_id: String,
    pub dimension_id: String,
    pub position: BlockPos,
    pub block_state: String,
}

/// A bounded persistent value stored on a block entity registered by this module.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PersistentDataValue {
    Boolean(bool),
    Integer(i32),
    String(String),
}

/// Maximum UTF-8 bytes in one persistent-data key.
pub const MAX_PERSISTENT_DATA_KEY_BYTES: usize = 64;
/// Maximum UTF-8 bytes in one persistent-data string value.
pub const MAX_PERSISTENT_DATA_STRING_BYTES: usize = 2_048;

/// Maximum UTF-8 bytes in one literal line exposed by the sign-text API.
pub const MAX_SIGN_TEXT_LINE_BYTES: usize = 2_048;
/// Maximum bytes returned for four literal lines plus their bounded wire framing.
pub const MAX_SIGN_TEXT_RESPONSE_BYTES: usize = 2 * 4 + 4 * (4 + MAX_SIGN_TEXT_LINE_BYTES);

/// Which side of a double-sided vanilla sign a text operation addresses.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SignSide {
    /// The front face of the sign.
    Front,
    /// The back face of the sign.
    Back,
}

impl SignSide {
    #[cfg(any(target_arch = "wasm32", test))]
    const fn wire_value(self) -> i32 {
        match self {
            Self::Front => 0,
            Self::Back => 1,
        }
    }
}

/// Four detached plain-text lines from one side of a vanilla sign.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SignTextLines {
    lines: [String; 4],
}

impl SignTextLines {
    /// Creates four bounded printable lines. Formatting and click actions are not accepted.
    pub fn new(lines: [String; 4]) -> SjmlResult<Self> {
        for line in &lines {
            validate_sign_line(line)?;
        }
        Ok(Self { lines })
    }

    /// Returns the four lines in sign order.
    pub fn lines(&self) -> &[String; 4] {
        &self.lines
    }
}

/// One immutable NBT value. The encoding uses standard NBT tag numbers and a versioned root.
#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<NbtValue>),
    Compound(Vec<(String, NbtValue)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 65_536;
fn invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
fn limit() -> SjmlError {
    SjmlError::new(
        ErrorCode::PayloadTooLarge,
        "NBT snapshot exceeds its limits",
    )
}
impl NbtValue {
    fn tag(&self) -> u8 {
        match self {
            Self::Byte(_) => 1,
            Self::Short(_) => 2,
            Self::Int(_) => 3,
            Self::Long(_) => 4,
            Self::Float(_) => 5,
            Self::Double(_) => 6,
            Self::ByteArray(_) => 7,
            Self::String(_) => 8,
            Self::List(_) => 9,
            Self::Compound(_) => 10,
            Self::IntArray(_) => 11,
            Self::LongArray(_) => 12,
        }
    }
    fn write_node(
        &self,
        e: &mut crate::Encoder,
        depth: usize,
        nodes: &mut usize,
    ) -> SjmlResult<()> {
        *nodes += 1;
        if depth > MAX_DEPTH || *nodes > MAX_NODES {
            return Err(limit());
        }
        match self {
            Self::Byte(v) => {
                e.write_u8(1)?;
                v.encode(e)
            }
            Self::Short(v) => {
                e.write_u8(2)?;
                v.encode(e)
            }
            Self::Int(v) => {
                e.write_u8(3)?;
                v.encode(e)
            }
            Self::Long(v) => {
                e.write_u8(4)?;
                v.encode(e)
            }
            Self::Float(v) => {
                e.write_u8(5)?;
                v.encode(e)
            }
            Self::Double(v) => {
                e.write_u8(6)?;
                v.encode(e)
            }
            Self::ByteArray(v) => {
                e.write_u8(7)?;
                v.encode(e)
            }
            Self::String(v) => {
                e.write_u8(8)?;
                v.encode(e)
            }
            Self::List(v) => {
                e.write_u8(9)?;
                if v.len() > MAX_NODES {
                    return Err(limit());
                }
                e.write_length(v.len())?;
                let first = v.first().map(Self::tag);
                for item in v {
                    if Some(item.tag()) != first {
                        return Err(invalid("heterogeneous NBT list"));
                    }
                    item.write_node(e, depth + 1, nodes)?;
                }
                Ok(())
            }
            Self::Compound(v) => {
                e.write_u8(10)?;
                if v.len() > MAX_NODES {
                    return Err(limit());
                }
                e.write_length(v.len())?;
                let mut keys = std::collections::HashSet::new();
                for (key, item) in v {
                    if !keys.insert(key) {
                        return Err(invalid("duplicate NBT compound key"));
                    }
                    key.encode(e)?;
                    item.write_node(e, depth + 1, nodes)?;
                }
                Ok(())
            }
            Self::IntArray(v) => {
                e.write_u8(11)?;
                v.encode(e)
            }
            Self::LongArray(v) => {
                e.write_u8(12)?;
                v.encode(e)
            }
        }
    }
    fn read_node(d: &mut crate::Decoder<'_>, depth: usize, nodes: &mut usize) -> SjmlResult<Self> {
        *nodes += 1;
        if depth > MAX_DEPTH || *nodes > MAX_NODES {
            return Err(limit());
        }
        Ok(match d.read_u8()? {
            1 => Self::Byte(i8::decode(d)?),
            2 => Self::Short(i16::decode(d)?),
            3 => Self::Int(i32::decode(d)?),
            4 => Self::Long(i64::decode(d)?),
            5 => Self::Float(f32::decode(d)?),
            6 => Self::Double(f64::decode(d)?),
            7 => Self::ByteArray(Vec::<i8>::decode(d)?),
            8 => Self::String(String::decode(d)?),
            9 => {
                let n = d.read_length(MAX_NODES)?;
                let mut v = Vec::new();
                for _ in 0..n {
                    let item = Self::read_node(d, depth + 1, nodes)?;
                    if v.first()
                        .is_some_and(|first: &Self| first.tag() != item.tag())
                    {
                        return Err(invalid("heterogeneous NBT list"));
                    }
                    v.push(item);
                }
                Self::List(v)
            }
            10 => {
                let n = d.read_length(MAX_NODES)?;
                let mut v = Vec::new();
                let mut keys = std::collections::HashSet::new();
                for _ in 0..n {
                    let key = String::decode(d)?;
                    if !keys.insert(key.clone()) {
                        return Err(invalid("duplicate NBT compound key"));
                    }
                    v.push((key, Self::read_node(d, depth + 1, nodes)?));
                }
                Self::Compound(v)
            }
            11 => Self::IntArray(Vec::<i32>::decode(d)?),
            12 => Self::LongArray(Vec::<i64>::decode(d)?),
            _ => return Err(invalid("unknown NBT tag")),
        })
    }
}
impl StableEncode for NbtValue {
    fn encode(&self, e: &mut crate::Encoder) -> SjmlResult<()> {
        e.write_u32(1)?;
        self.write_node(e, 1, &mut 0)
    }
}
impl StableDecode for NbtValue {
    fn decode(d: &mut crate::Decoder<'_>) -> SjmlResult<Self> {
        if d.read_u32()? != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                "unsupported NBT schema",
            ));
        }
        Self::read_node(d, 1, &mut 0)
    }
}

/// A module-scoped block-entity reference; its numeric handle remains private.
#[derive(Debug, Clone, Copy)]
pub struct BlockEntity {
    handle: BlockEntityHandle,
}
impl WorldHandle {
    /// Finds an entity only in an already loaded chunk.
    pub fn block_entity(self, position: BlockPos) -> SjmlResult<Option<BlockEntity>> {
        get(self, position)
    }
}
impl BlockEntity {
    pub fn snapshot(self) -> SjmlResult<BlockEntitySnapshot> {
        snapshot(self.handle)
    }
    pub fn type_id(self) -> SjmlResult<String> {
        Ok(self.snapshot()?.type_id)
    }
    pub fn block_state(self) -> SjmlResult<String> {
        Ok(self.snapshot()?.block_state)
    }
    pub fn inventory(self, side: Option<Direction>) -> SjmlResult<Option<InventorySnapshot>> {
        inventory(self.handle, side)
    }
    pub fn nbt(self) -> SjmlResult<NbtValue> {
        nbt(self.handle)
    }
    /// Reads this module's typed value from a live block entity that this module registered.
    pub fn get_data(self, key: &str) -> SjmlResult<Option<PersistentDataValue>> {
        get_data(self.handle, key)
    }
    /// Sets a persistent string, signed integer, or Boolean on this module's registered block entity.
    pub fn set_data(self, key: &str, value: PersistentDataValue) -> SjmlResult<()> {
        set_data(self.handle, key, value)
    }
    /// Removes one persistent value from this module's registered block entity.
    pub fn remove_data(self, key: &str) -> SjmlResult<()> {
        remove_data(self.handle, key)
    }

    /// Reads the selected side of a loaded vanilla sign as four plain-text lines.
    pub fn sign_text(self, side: SignSide) -> SjmlResult<SignTextLines> {
        read_sign_text(self.handle, side)
    }

    /// Replaces the selected side of a loaded, unwaxed vanilla sign with literal text.
    pub fn set_sign_text(self, side: SignSide, lines: &SignTextLines) -> SjmlResult<()> {
        set_sign_text(self.handle, side, lines)
    }
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_block_entity_v1")]
extern "C" {
    #[link_name = "get"]
    fn imported_get(w: i32, wl: i32, p: i32, pl: i32, o: i32, c: i32) -> i32;
    #[link_name = "snapshot"]
    fn imported_snapshot(h: i32, hl: i32, o: i32, c: i32) -> i32;
    #[link_name = "inventory"]
    fn imported_inventory(h: i32, hl: i32, s: i32, sl: i32, o: i32, c: i32) -> i32;
    #[link_name = "nbt"]
    fn imported_nbt(h: i32, hl: i32, o: i32, c: i32) -> i32;
    #[link_name = "get_data"]
    fn imported_get_data(request: i32, request_length: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_data"]
    fn imported_set_data(request: i32, request_length: i32) -> i32;
    #[link_name = "remove_data"]
    fn imported_remove_data(request: i32, request_length: i32) -> i32;
    #[link_name = "sign_text"]
    fn imported_sign_text(h: i32, hl: i32, side: i32, output: i32, capacity: i32) -> i32;
    #[link_name = "set_sign_text"]
    fn imported_set_sign_text(h: i32, hl: i32, side: i32, payload: i32, payload_length: i32)
        -> i32;
}
pub fn get(world: WorldHandle, pos: BlockPos) -> SjmlResult<Option<BlockEntity>> {
    #[cfg(target_arch = "wasm32")]
    {
        let w = to_bytes(&world)?;
        let p = to_bytes(&pos)?;
        let h: Option<BlockEntityHandle> = read_output(25, |o, c| unsafe {
            imported_get(
                w.as_ptr() as i32,
                w.len() as i32,
                p.as_ptr() as i32,
                p.len() as i32,
                o,
                c,
            )
        })?;
        Ok(h.map(|handle| BlockEntity { handle }))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (world, pos);
        Err(unsupported())
    }
}
pub fn snapshot(handle: BlockEntityHandle) -> SjmlResult<BlockEntitySnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let h = to_bytes(&handle)?;
        read_output(crate::MAX_PAYLOAD_BYTES as u32, |o, c| unsafe {
            imported_snapshot(h.as_ptr() as i32, h.len() as i32, o, c)
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}
fn inventory(
    handle: BlockEntityHandle,
    side: Option<Direction>,
) -> SjmlResult<Option<InventorySnapshot>> {
    #[cfg(target_arch = "wasm32")]
    {
        let h = to_bytes(&handle)?;
        let s = to_bytes(&side)?;
        let value: Option<InventorySnapshot> =
            read_output(crate::MAX_PAYLOAD_BYTES as u32, |o, c| unsafe {
                imported_inventory(
                    h.as_ptr() as i32,
                    h.len() as i32,
                    s.as_ptr() as i32,
                    s.len() as i32,
                    o,
                    c,
                )
            })?;
        if value
            .as_ref()
            .is_some_and(|v| v.slots.len() > crate::inventory::MAX_INVENTORY_SLOTS)
        {
            return Err(limit());
        }
        Ok(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, side);
        Err(unsupported())
    }
}
fn nbt(handle: BlockEntityHandle) -> SjmlResult<NbtValue> {
    #[cfg(target_arch = "wasm32")]
    {
        let h = to_bytes(&handle)?;
        read_output(crate::MAX_PAYLOAD_BYTES as u32, |o, c| unsafe {
            imported_nbt(h.as_ptr() as i32, h.len() as i32, o, c)
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const DATA_SCHEMA: u32 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const DATA_GET: u8 = 0;
#[cfg(any(target_arch = "wasm32", test))]
const DATA_SET: u8 = 1;
#[cfg(any(target_arch = "wasm32", test))]
const DATA_REMOVE: u8 = 2;
#[cfg(any(target_arch = "wasm32", test))]
const DATA_RESPONSE_CAPACITY: u32 = 4 + 1 + 1 + 4 + MAX_PERSISTENT_DATA_STRING_BYTES as u32;

fn get_data(handle: BlockEntityHandle, key: &str) -> SjmlResult<Option<PersistentDataValue>> {
    validate_data_key(key)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = encode_data_request(handle, DATA_GET, key, None)?;
        let bytes = read_raw_data_output(DATA_RESPONSE_CAPACITY, |output, capacity| unsafe {
            imported_get_data(
                request.as_ptr() as i32,
                request.len() as i32,
                output,
                capacity,
            )
        })?;
        decode_data_response(&bytes)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

fn set_data(handle: BlockEntityHandle, key: &str, value: PersistentDataValue) -> SjmlResult<()> {
    validate_data_key(key)?;
    validate_data_value(&value)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = encode_data_request(handle, DATA_SET, key, Some(value))?;
        let status = unsafe { imported_set_data(request.as_ptr() as i32, request.len() as i32) };
        status_data(status, "host rejected block-entity data write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, value);
        Err(unsupported())
    }
}

fn remove_data(handle: BlockEntityHandle, key: &str) -> SjmlResult<()> {
    validate_data_key(key)?;
    #[cfg(target_arch = "wasm32")]
    {
        let request = encode_data_request(handle, DATA_REMOVE, key, None)?;
        let status = unsafe { imported_remove_data(request.as_ptr() as i32, request.len() as i32) };
        status_data(status, "host rejected block-entity data removal")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handle;
        Err(unsupported())
    }
}

/// Reads four literal lines from one side of a loaded vanilla sign.
///
/// The handle must come from [`get`], the call must run on the logical-server thread, and the
/// module needs `block_entity.sign.read`. The host never loads a chunk.
pub fn read_sign_text(handle: BlockEntityHandle, side: SignSide) -> SjmlResult<SignTextLines> {
    #[cfg(target_arch = "wasm32")]
    {
        let handle_bytes = to_bytes(&handle)?;
        let response = read_raw_data_output(
            MAX_SIGN_TEXT_RESPONSE_BYTES as u32,
            |output, capacity| unsafe {
                imported_sign_text(
                    handle_bytes.as_ptr() as i32,
                    handle_bytes.len() as i32,
                    side.wire_value(),
                    output,
                    capacity,
                )
            },
        )?;
        decode_sign_text_response(&response)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, side);
        Err(unsupported())
    }
}

/// Replaces one side of a loaded unwaxed vanilla sign with four plain-text lines.
///
/// The write requires `block_entity.sign.write` and `world.write`, and must run on the logical-
/// server thread. The operation preserves the sign's text color and glowing-text setting while
/// replacing each line with a literal component that has no click or hover actions.
pub fn set_sign_text(
    handle: BlockEntityHandle,
    side: SignSide,
    lines: &SignTextLines,
) -> SjmlResult<()> {
    for line in &lines.lines {
        validate_sign_line(line)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let handle_bytes = to_bytes(&handle)?;
        let payload = encode_sign_text_request(lines)?;
        let capacity = u32::try_from(payload.len()).map_err(|_| data_limit())?;
        let pointer = allocate_host_input(capacity);
        if pointer == 0 {
            return Err(data_limit());
        }
        unsafe {
            core::ptr::copy_nonoverlapping(payload.as_ptr(), pointer as *mut u8, payload.len())
        };
        let status = unsafe {
            imported_set_sign_text(
                handle_bytes.as_ptr() as i32,
                handle_bytes.len() as i32,
                side.wire_value(),
                pointer as i32,
                capacity as i32,
            )
        };
        unsafe { deallocate_host_input(pointer, capacity) };
        status_data(status, "host rejected sign text write")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (handle, side, lines);
        Err(unsupported())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
const SIGN_TEXT_SCHEMA: u32 = 1;

fn validate_sign_line(line: &str) -> SjmlResult<()> {
    if line.len() > MAX_SIGN_TEXT_LINE_BYTES || line.chars().any(char::is_control) {
        return Err(data_invalid(
            "sign text lines must be printable and at most 2048 UTF-8 bytes",
        ));
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn encode_sign_text_request(lines: &SignTextLines) -> SjmlResult<Vec<u8>> {
    let mut encoder = crate::Encoder::new();
    encoder.write_u32(SIGN_TEXT_SCHEMA)?;
    encoder.write_u32(4)?;
    for line in &lines.lines {
        validate_sign_line(line)?;
        line.encode(&mut encoder)?;
    }
    Ok(encoder.into_bytes())
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_sign_text_response(bytes: &[u8]) -> SjmlResult<SignTextLines> {
    let mut input = bytes;
    let schema = take_data_u32(&mut input)?;
    if schema != SIGN_TEXT_SCHEMA {
        return Err(SjmlError::new(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported sign text response schema",
        ));
    }
    if take_data_u32(&mut input)? != 4 {
        return Err(data_invalid(
            "sign text response must contain exactly four lines",
        ));
    }
    let mut lines = Vec::with_capacity(4);
    for _ in 0..4 {
        let line = take_data_string(&mut input, MAX_SIGN_TEXT_LINE_BYTES)?;
        validate_sign_line(&line)?;
        lines.push(line);
    }
    ensure_data_padding(input)?;
    let lines: [String; 4] = lines
        .try_into()
        .map_err(|_| data_invalid("sign text response must contain four lines"))?;
    Ok(SignTextLines { lines })
}

fn validate_data_key(key: &str) -> SjmlResult<()> {
    if key.is_empty()
        || key.len() > MAX_PERSISTENT_DATA_KEY_BYTES
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(data_invalid("persistent data keys must contain 1..=64 lowercase ASCII letters, digits or underscores"));
    }
    Ok(())
}

fn validate_data_value(value: &PersistentDataValue) -> SjmlResult<()> {
    if let PersistentDataValue::String(text) = value {
        if text.len() > MAX_PERSISTENT_DATA_STRING_BYTES || text.chars().any(char::is_control) {
            return Err(data_invalid(
                "persistent data strings must be printable and at most 2048 UTF-8 bytes",
            ));
        }
    }
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn encode_data_request(
    handle: BlockEntityHandle,
    operation: u8,
    key: &str,
    value: Option<PersistentDataValue>,
) -> SjmlResult<Vec<u8>> {
    let mut output = Vec::new();
    output.extend_from_slice(&DATA_SCHEMA.to_le_bytes());
    output.push(operation);
    output.extend_from_slice(&to_bytes(&handle)?);
    put_data_string(&mut output, key.as_bytes())?;
    match value {
        None => output.push(0),
        Some(PersistentDataValue::Boolean(value)) => {
            output.push(1);
            output.push(u8::from(value));
        }
        Some(PersistentDataValue::Integer(value)) => {
            output.push(2);
            output.extend_from_slice(&value.to_le_bytes());
        }
        Some(PersistentDataValue::String(value)) => {
            validate_data_value(&PersistentDataValue::String(value.clone()))?;
            output.push(3);
            put_data_string(&mut output, value.as_bytes())?;
        }
    }
    if output.len() > crate::MAX_PAYLOAD_BYTES {
        return Err(data_limit());
    }
    Ok(output)
}

#[cfg(any(target_arch = "wasm32", test))]
fn put_data_string(output: &mut Vec<u8>, value: &[u8]) -> SjmlResult<()> {
    let length = u32::try_from(value.len()).map_err(|_| data_limit())?;
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(value);
    Ok(())
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_data_response(bytes: &[u8]) -> SjmlResult<Option<PersistentDataValue>> {
    let mut input = bytes;
    let schema = take_data_u32(&mut input)?;
    if schema != DATA_SCHEMA {
        return Err(SjmlError::new(
            ErrorCode::IncompatibleAbiVersion,
            "unsupported block-entity data response",
        ));
    }
    let present = take_data_byte(&mut input)?;
    if present == 0 {
        ensure_data_padding(input)?;
        return Ok(None);
    }
    if present != 1 {
        return Err(data_invalid("invalid block-entity data presence marker"));
    }
    let value = match take_data_byte(&mut input)? {
        1 => {
            let value = take_data_byte(&mut input)?;
            if value > 1 {
                return Err(data_invalid(
                    "block-entity Boolean data must be zero or one",
                ));
            }
            PersistentDataValue::Boolean(value == 1)
        }
        2 => PersistentDataValue::Integer(take_data_i32(&mut input)?),
        3 => {
            let value = take_data_string(&mut input, MAX_PERSISTENT_DATA_STRING_BYTES)?;
            PersistentDataValue::String(value)
        }
        _ => return Err(data_invalid("unknown block-entity data value kind")),
    };
    validate_data_value(&value)?;
    ensure_data_padding(input)?;
    Ok(Some(value))
}

#[cfg(any(target_arch = "wasm32", test))]
fn take_data_u32(input: &mut &[u8]) -> SjmlResult<u32> {
    let bytes = take_data(input, 4)?;
    Ok(u32::from_le_bytes(
        bytes.try_into().expect("four-byte integer"),
    ))
}
#[cfg(any(target_arch = "wasm32", test))]
fn take_data_i32(input: &mut &[u8]) -> SjmlResult<i32> {
    let bytes = take_data(input, 4)?;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("four-byte integer"),
    ))
}
#[cfg(any(target_arch = "wasm32", test))]
fn take_data_byte(input: &mut &[u8]) -> SjmlResult<u8> {
    Ok(take_data(input, 1)?[0])
}
#[cfg(any(target_arch = "wasm32", test))]
fn take_data_string(input: &mut &[u8], maximum: usize) -> SjmlResult<String> {
    let length = usize::try_from(take_data_u32(input)?).map_err(|_| data_limit())?;
    if length > maximum {
        return Err(data_limit());
    }
    String::from_utf8(take_data(input, length)?.to_vec())
        .map_err(|_| data_invalid("block-entity data is not valid UTF-8"))
}
#[cfg(any(target_arch = "wasm32", test))]
fn take_data<'a>(input: &mut &'a [u8], length: usize) -> SjmlResult<&'a [u8]> {
    if input.len() < length {
        return Err(data_invalid("block-entity data response is truncated"));
    }
    let (value, remaining) = input.split_at(length);
    *input = remaining;
    Ok(value)
}
#[cfg(any(target_arch = "wasm32", test))]
fn ensure_data_padding(input: &[u8]) -> SjmlResult<()> {
    if input.iter().any(|byte| *byte != 0) {
        return Err(data_invalid(
            "block-entity data response contains trailing bytes",
        ));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn read_raw_data_output(
    capacity: u32,
    invoke: impl FnOnce(i32, i32) -> i32,
) -> SjmlResult<Vec<u8>> {
    let pointer = allocate_host_input(capacity);
    if pointer == 0 {
        return Err(data_limit());
    }
    unsafe { core::ptr::write_bytes(pointer as *mut u8, 0, capacity as usize) };
    let status = invoke(pointer as i32, capacity as i32);
    let result = if status == 0 {
        Ok(
            unsafe { core::slice::from_raw_parts(pointer as *const u8, capacity as usize) }
                .to_vec(),
        )
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected block-entity data read",
        ))
    };
    unsafe { deallocate_host_input(pointer, capacity) };
    result
}

#[cfg(target_arch = "wasm32")]
fn status_data(status: i32, message: &str) -> SjmlResult<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(SjmlError::from_abi_code(status as u32, message))
    }
}

fn data_invalid(message: &'static str) -> SjmlError {
    SjmlError::new(ErrorCode::InvalidArgument, message)
}
#[cfg(any(target_arch = "wasm32", test))]
fn data_limit() -> SjmlError {
    SjmlError::new(
        ErrorCode::PayloadTooLarge,
        "block-entity data exceeds its bounded ABI limits",
    )
}
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "block entity access requires wasm32-unknown-unknown",
    )
}

#[cfg(test)]
mod persistent_data_tests {
    use super::*;

    #[test]
    fn request_encoder_uses_bounded_versioned_little_endian_values() {
        let handle = BlockEntityHandle::new(1, 2, 3, 4).unwrap();
        let get = encode_data_request(handle, DATA_GET, "active", None).unwrap();
        let remove = encode_data_request(handle, DATA_REMOVE, "active", None).unwrap();
        assert_eq!(get[4], DATA_GET);
        assert_eq!(remove[4], DATA_REMOVE);
        let bytes = encode_data_request(
            handle,
            DATA_SET,
            "active",
            Some(PersistentDataValue::Integer(-17)),
        )
        .unwrap();
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            DATA_SCHEMA
        );
        assert_eq!(bytes[4], DATA_SET);
        assert_eq!(&bytes[5..29], &to_bytes(&handle).unwrap());
        assert_eq!(u32::from_le_bytes(bytes[29..33].try_into().unwrap()), 6);
        assert_eq!(&bytes[33..39], b"active");
        assert_eq!(bytes[39], 2);
        assert_eq!(i32::from_le_bytes(bytes[40..44].try_into().unwrap()), -17);
    }

    #[test]
    fn response_decoder_accepts_optional_boolean_integer_and_string() {
        assert_eq!(
            decode_data_response(&padded(&[1, 0, 0, 0, 0])).unwrap(),
            None
        );
        assert_eq!(
            decode_data_response(&padded(&[1, 0, 0, 0, 1, 1, 1])).unwrap(),
            Some(PersistentDataValue::Boolean(true))
        );
        let mut integer = vec![1, 0, 0, 0, 1, 2];
        integer.extend_from_slice(&i32::MIN.to_le_bytes());
        assert_eq!(
            decode_data_response(&padded(&integer)).unwrap(),
            Some(PersistentDataValue::Integer(i32::MIN))
        );
        let mut string = vec![1, 0, 0, 0, 1, 3, 5, 0, 0, 0];
        string.extend_from_slice(b"hello");
        assert_eq!(
            decode_data_response(&padded(&string)).unwrap(),
            Some(PersistentDataValue::String("hello".into()))
        );
    }

    #[test]
    fn data_validation_rejects_noncanonical_keys_and_nonprintable_or_oversized_values() {
        assert!(validate_data_key("valid_2").is_ok());
        assert!(validate_data_key("Upper").is_err());
        assert!(validate_data_key("a/b").is_err());
        assert!(validate_data_value(&PersistentDataValue::String("line\nfeed".into())).is_err());
        assert!(validate_data_value(&PersistentDataValue::String(
            "x".repeat(MAX_PERSISTENT_DATA_STRING_BYTES + 1)
        ))
        .is_err());
    }

    fn padded(bytes: &[u8]) -> Vec<u8> {
        let mut result = vec![0; DATA_RESPONSE_CAPACITY as usize];
        result[..bytes.len()].copy_from_slice(bytes);
        result
    }
}

#[cfg(test)]
mod sign_text_tests {
    use super::*;

    #[test]
    fn four_line_wire_round_trip_is_bounded_and_little_endian() {
        let lines = SignTextLines::new([
            "front door".to_owned(),
            "雪の家".to_owned(),
            String::new(),
            "welcome".to_owned(),
        ])
        .unwrap();
        let request = encode_sign_text_request(&lines).unwrap();
        assert_eq!(
            u32::from_le_bytes(request[0..4].try_into().unwrap()),
            SIGN_TEXT_SCHEMA
        );
        assert_eq!(u32::from_le_bytes(request[4..8].try_into().unwrap()), 4);
        assert_eq!(
            decode_sign_text_response(&padded_sign(&request)).unwrap(),
            lines
        );
    }

    #[test]
    fn sign_lines_reject_control_characters_and_oversized_utf8() {
        assert!(SignTextLines::new([
            "line\nfeed".into(),
            String::new(),
            String::new(),
            String::new()
        ])
        .is_err());
        assert!(SignTextLines::new([
            "é".repeat(MAX_SIGN_TEXT_LINE_BYTES / 2 + 1),
            String::new(),
            String::new(),
            String::new(),
        ])
        .is_err());
        assert_eq!(SignSide::Front.wire_value(), 0);
        assert_eq!(SignSide::Back.wire_value(), 1);
    }

    #[test]
    fn sign_response_rejects_wrong_schema_count_invalid_utf8_and_nonzero_padding() {
        let lines =
            SignTextLines::new([String::new(), String::new(), String::new(), String::new()])
                .unwrap();
        let valid = encode_sign_text_request(&lines).unwrap();

        let mut wrong_schema = valid.clone();
        wrong_schema[0] = 2;
        assert_eq!(
            decode_sign_text_response(&padded_sign(&wrong_schema))
                .unwrap_err()
                .code(),
            ErrorCode::IncompatibleAbiVersion
        );

        let mut wrong_count = valid.clone();
        wrong_count[4] = 3;
        assert!(decode_sign_text_response(&padded_sign(&wrong_count)).is_err());

        let mut bad_padding = padded_sign(&valid);
        let last = bad_padding.len() - 1;
        bad_padding[last] = 1;
        assert!(decode_sign_text_response(&bad_padding).is_err());
    }

    fn padded_sign(payload: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0; MAX_SIGN_TEXT_RESPONSE_BYTES];
        bytes[..payload.len()].copy_from_slice(payload);
        bytes
    }
}
#[cfg(target_arch = "wasm32")]
fn read_output<T: StableDecode>(
    capacity: u32,
    invoke: impl FnOnce(i32, i32) -> i32,
) -> SjmlResult<T> {
    let p = allocate_host_input(capacity);
    if p == 0 {
        return Err(limit());
    }
    unsafe { core::ptr::write_bytes(p as *mut u8, 0, capacity as usize) };
    let status = invoke(p as i32, capacity as i32);
    let result = if status == 0 {
        let bytes = unsafe { core::slice::from_raw_parts(p as *const u8, capacity as usize) };
        Decoder::new(bytes).and_then(|mut d| T::decode(&mut d))
    } else {
        Err(SjmlError::from_abi_code(
            status as u32,
            "host rejected block entity read",
        ))
    };
    unsafe { deallocate_host_input(p, capacity) };
    result
}
