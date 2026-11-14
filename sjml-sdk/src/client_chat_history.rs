//! Bounded read-only access to the physical client's retained vanilla chat display backlog.
//!
//! The snapshot is copied from `ChatComponent` on the client main thread. It contains plain text,
//! the message's GUI tick counter, and newest-first order only; it does not expose Minecraft
//! components, styles, click or hover actions, signatures, or the separate sent-command history.

#[cfg(target_arch = "wasm32")]
use sjml::abi::{allocate_host_input, deallocate_host_input};
#[cfg(target_arch = "wasm32")]
use sjml::from_bytes;
use sjml::{Decoder, Encoder, ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Capability required to read the local rendered chat backlog.
pub const CLIENT_CHAT_HISTORY_READ_CAPABILITY: &str = "client.chat.history.read";
/// Maximum number of newest-first messages returned by one request.
pub const MAX_CLIENT_CHAT_HISTORY_MESSAGES: usize = 64;
/// Maximum UTF-8 text bytes returned for one message.
pub const MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES: usize = 384;
/// Maximum size of one encoded chat-history result.
pub const MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES: usize = 32 * 1024;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "sjml_client_chat_history_v1")]
extern "C" {
    #[link_name = "read_recent"]
    fn imported_read_recent(
        max_messages: i32,
        output_pointer: i32,
        output_capacity: i32,
        output_length_pointer: i32,
    ) -> i32;
}

/// One detached plain-text message retained by the client's vanilla chat display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatHistoryEntry {
    /// Zero-based index in the snapshot, with zero being the newest retained message.
    pub order: u32,
    /// Raw unsigned GUI tick value when vanilla added this message; it is not wall-clock time.
    pub added_tick: u32,
    /// Plain, unstyled text. It may end at a UTF-8 boundary if [`Self::truncated`] is true.
    pub text: String,
    /// Whether the source text was clipped or normalized to satisfy the bounded text contract.
    pub truncated: bool,
}

/// Immutable newest-first snapshot of retained vanilla chat display messages.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChatHistorySnapshot {
    /// The requested number of newest messages, or fewer if the GUI has not retained that many.
    pub messages: Vec<ChatHistoryEntry>,
}

impl StableEncode for ChatHistorySnapshot {
    fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
        validate_snapshot(self)?;
        encoder.write_u32(1)?;
        encoder.write_length(self.messages.len())?;
        for message in &self.messages {
            encoder.write_u32(message.order)?;
            encoder.write_u32(message.added_tick)?;
            encoder.write_u8(u8::from(message.truncated))?;
            encoder.write_length(message.text.len())?;
            encoder.write_raw(message.text.as_bytes())?;
        }
        Ok(())
    }
}

impl StableDecode for ChatHistorySnapshot {
    fn decode(decoder: &mut Decoder<'_>) -> SjmlResult<Self> {
        let version = decoder.read_u32()?;
        if version != 1 {
            return Err(SjmlError::new(
                ErrorCode::IncompatibleAbiVersion,
                format!("ChatHistorySnapshot schema version {version} is unsupported"),
            ));
        }
        let count = decoder.read_length(MAX_CLIENT_CHAT_HISTORY_MESSAGES)?;
        let mut messages = Vec::with_capacity(count);
        for index in 0..count {
            let order = decoder.read_u32()?;
            let added_tick = decoder.read_u32()?;
            let truncated = match decoder.read_u8()? {
                0 => false,
                1 => true,
                value => {
                    return Err(SjmlError::new(
                        ErrorCode::InvalidBoolean,
                        format!("client chat history truncated flag must be 0 or 1, found {value}"),
                    ));
                }
            };
            let text_length = decoder.read_length(MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES)?;
            let text =
                String::from_utf8(decoder.read_raw(text_length)?.to_vec()).map_err(|_| {
                    SjmlError::new(
                        ErrorCode::InvalidUtf8,
                        "client chat history text contains malformed UTF-8",
                    )
                })?;
            if order != index as u32 {
                return Err(SjmlError::new(
                    ErrorCode::InvalidArgument,
                    "client chat history order must be contiguous and newest first",
                ));
            }
            messages.push(ChatHistoryEntry {
                order,
                added_tick,
                text,
                truncated,
            });
        }
        let snapshot = Self { messages };
        validate_snapshot(&snapshot)?;
        Ok(snapshot)
    }
}

/// Reads up to `max_messages` entries from the local vanilla chat display backlog.
///
/// `max_messages` must be in `1..=64`. Entries are returned newest first. The GUI tick value is
/// bounded metadata, not a server timestamp; no sent-message history or message actions are read.
pub fn read_recent(max_messages: usize) -> SjmlResult<ChatHistorySnapshot> {
    if !(1..=MAX_CLIENT_CHAT_HISTORY_MESSAGES).contains(&max_messages) {
        return Err(SjmlError::new(
            ErrorCode::InvalidArgument,
            "client chat history request count must be within 1..=64",
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let output_pointer = allocate_host_input(MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES as u32);
        let length_pointer = allocate_host_input(core::mem::size_of::<u32>() as u32);
        if output_pointer == 0 || length_pointer == 0 {
            // SAFETY: each nonzero pointer came from the allocator with the exact sizes below.
            unsafe {
                deallocate_host_input(output_pointer, MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES as u32);
                deallocate_host_input(length_pointer, core::mem::size_of::<u32>() as u32);
            }
            return Err(SjmlError::new(
                ErrorCode::UnknownErrorCode,
                "could not allocate client chat history buffers",
            ));
        }
        // SAFETY: both pointers are live allocations with the exact sizes passed to the allocator.
        unsafe {
            core::ptr::write_bytes(
                output_pointer as *mut u8,
                0,
                MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES,
            );
            core::ptr::write_bytes(length_pointer as *mut u8, 0, core::mem::size_of::<u32>());
        }
        // SAFETY: the host reads the requested limit and writes at most the fixed payload bound
        // plus a four-byte output length during this synchronous client-main-thread call.
        let status = unsafe {
            imported_read_recent(
                max_messages as i32,
                output_pointer as i32,
                MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES as i32,
                length_pointer as i32,
            )
        };
        let result = if status != 0 {
            Err(SjmlError::from_abi_code(
                status as u32,
                "host rejected the client chat history request",
            ))
        } else {
            // SAFETY: the host initializes this four-byte length on success.
            let length =
                unsafe { core::ptr::read_unaligned(length_pointer as *const u32) } as usize;
            if length > MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES {
                Err(SjmlError::new(
                    ErrorCode::PayloadTooLarge,
                    "host returned client chat history beyond the payload limit",
                ))
            } else {
                // SAFETY: the host writes exactly `length` bytes to the live output allocation.
                let bytes =
                    unsafe { core::slice::from_raw_parts(output_pointer as *const u8, length) };
                match from_bytes::<ChatHistorySnapshot>(bytes) {
                    Ok(snapshot) if snapshot.messages.len() <= max_messages => Ok(snapshot),
                    Ok(_) => Err(SjmlError::new(
                        ErrorCode::PayloadTooLarge,
                        "host returned more chat history entries than requested",
                    )),
                    Err(error) => Err(error),
                }
            }
        };
        // SAFETY: release each exact live allocation after the host call and decoding have completed.
        unsafe {
            deallocate_host_input(output_pointer, MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES as u32);
            deallocate_host_input(length_pointer, core::mem::size_of::<u32>() as u32);
        }
        result
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = max_messages;
        Err(SjmlError::new(
            ErrorCode::UnknownErrorCode,
            "client chat history requires the wasm32-unknown-unknown target",
        ))
    }
}

fn validate_snapshot(snapshot: &ChatHistorySnapshot) -> SjmlResult<()> {
    if snapshot.messages.len() > MAX_CLIENT_CHAT_HISTORY_MESSAGES {
        return Err(SjmlError::new(
            ErrorCode::PayloadTooLarge,
            "client chat history exceeds the 64-message limit",
        ));
    }
    let mut total_bytes = 8_usize;
    for (index, message) in snapshot.messages.iter().enumerate() {
        if message.order != index as u32 {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "client chat history order must be contiguous and newest first",
            ));
        }
        if message.text.len() > MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "client chat history message exceeds 384 UTF-8 bytes",
            ));
        }
        total_bytes = total_bytes
            .checked_add(13 + message.text.len())
            .ok_or_else(|| {
                SjmlError::new(ErrorCode::PayloadTooLarge, "chat history size overflowed")
            })?;
        if total_bytes > MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES {
            return Err(SjmlError::new(
                ErrorCode::PayloadTooLarge,
                "client chat history exceeds the 32 KiB payload limit",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        read_recent, ChatHistoryEntry, ChatHistorySnapshot, MAX_CLIENT_CHAT_HISTORY_MESSAGES,
        MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES, MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES,
    };
    use sjml::{from_bytes, to_bytes, ErrorCode};

    fn entry(order: u32, text: &str) -> ChatHistoryEntry {
        ChatHistoryEntry {
            order,
            added_tick: 123,
            text: text.to_owned(),
            truncated: false,
        }
    }

    #[test]
    fn snapshot_codec_round_trips_order_tick_plain_text_and_truncation() {
        assert_eq!(
            super::CLIENT_CHAT_HISTORY_READ_CAPABILITY,
            "client.chat.history.read"
        );
        let snapshot = ChatHistorySnapshot {
            messages: vec![
                ChatHistoryEntry {
                    truncated: true,
                    ..entry(0, "latest clipped")
                },
                entry(1, "older"),
            ],
        };
        let encoded = to_bytes(&snapshot).unwrap();
        assert!(encoded.len() <= MAX_CLIENT_CHAT_HISTORY_PAYLOAD_BYTES);
        assert_eq!(
            from_bytes::<ChatHistorySnapshot>(&encoded).unwrap(),
            snapshot
        );
    }

    #[test]
    fn snapshot_codec_rejects_wrong_order_text_bound_and_invalid_boolean() {
        assert_eq!(
            to_bytes(&ChatHistorySnapshot {
                messages: vec![entry(1, "not newest index")],
            })
            .unwrap_err()
            .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            to_bytes(&ChatHistorySnapshot {
                messages: vec![entry(
                    0,
                    &"x".repeat(MAX_CLIENT_CHAT_HISTORY_TEXT_BYTES + 1)
                )],
            })
            .unwrap_err()
            .code(),
            ErrorCode::PayloadTooLarge
        );
        let mut invalid_boolean = vec![1, 0, 0, 0, 1, 0, 0, 0];
        invalid_boolean.extend_from_slice(&0_u32.to_le_bytes());
        invalid_boolean.extend_from_slice(&123_u32.to_le_bytes());
        invalid_boolean.push(2);
        assert_eq!(
            from_bytes::<ChatHistorySnapshot>(&invalid_boolean)
                .unwrap_err()
                .code(),
            ErrorCode::InvalidBoolean
        );
    }

    #[test]
    fn read_api_validates_limits_then_fails_closed_outside_wasm() {
        for invalid in [0, MAX_CLIENT_CHAT_HISTORY_MESSAGES + 1] {
            assert_eq!(
                read_recent(invalid).unwrap_err().code(),
                ErrorCode::InvalidArgument
            );
        }
        assert_eq!(
            read_recent(1).unwrap_err().code(),
            ErrorCode::UnknownErrorCode
        );
    }
}
