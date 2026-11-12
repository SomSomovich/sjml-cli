//! Read-only block-entity snapshots. NBT requires the separate `block_entity.nbt.read` grant.
#[cfg(target_arch = "wasm32")]
use crate::{
    abi::{allocate_host_input, deallocate_host_input},
    to_bytes, Decoder,
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
#[cfg(not(target_arch = "wasm32"))]
fn unsupported() -> SjmlError {
    SjmlError::new(
        ErrorCode::UnknownErrorCode,
        "block entity reads require wasm32-unknown-unknown",
    )
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
