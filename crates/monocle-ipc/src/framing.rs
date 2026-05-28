//! 4-byte little-endian length-prefix framing for IPC messages.
//!
//! # Wire Format (BC-2.05.002, SS-ipc.md v1.4.0 §Framing Protocol)
//!
//! Each framed message is:
//! ```text
//! [ 4 bytes: u32 LE payload length ] [ N bytes: UTF-8 JSON payload ]
//! ```
//!
//! No trailing newline or null terminator.
//!
//! # Size Limit
//!
//! `MAX_MESSAGE_BYTES = 262,144` (256 KiB). Messages larger than this limit are rejected
//! with [`crate::error::IpcError::MessageTooLarge`].

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::IpcError;

/// Maximum IPC message payload size: 256 KiB (262,144 bytes).
///
/// Both `write_framed` and `read_framed` enforce this limit.
pub const MAX_MESSAGE_BYTES: usize = 262_144;

/// Serialize `value` to JSON and write it to `writer` using 4-byte LE length-prefix framing.
///
/// # Wire Format
///
/// Writes `[u32 LE length][JSON bytes]`.  No trailing delimiter.
///
/// # Errors
///
/// - [`IpcError::SerializeError`] — if `serde_json::to_vec(value)` fails.
/// - [`IpcError::MessageTooLarge`] — if the serialized JSON exceeds `MAX_MESSAGE_BYTES`.
/// - [`IpcError::IoError`] — if writing to `writer` fails.
pub async fn write_framed<W, T>(writer: &mut W, value: &T) -> Result<(), IpcError>
where
    W: AsyncWriteExt + Unpin,
    T: serde::Serialize,
{
    let payload = serde_json::to_vec(value)?;
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(IpcError::MessageTooLarge);
    }
    // Safety: we just checked payload.len() <= MAX_MESSAGE_BYTES <= u32::MAX.
    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(&payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Write a pre-serialized payload to `writer` using 4-byte LE length-prefix framing.
///
/// Unlike [`write_framed`], this function does NOT serialize `value` — the caller
/// already has the serialized bytes. This avoids a double-serialize pattern when the
/// caller needs both the byte count (for a size check) and the bytes themselves.
///
/// # Wire Format
///
/// Writes `[u32 LE length][payload bytes]`.  No trailing delimiter.
///
/// # Errors
///
/// - [`IpcError::MessageTooLarge`] — if `payload.len()` exceeds `MAX_MESSAGE_BYTES`.
/// - [`IpcError::IoError`] — if writing to `writer` fails.
pub async fn write_framed_bytes<W>(writer: &mut W, payload: &[u8]) -> Result<(), IpcError>
where
    W: AsyncWriteExt + Unpin,
{
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(IpcError::MessageTooLarge);
    }
    // Safety: we just checked payload.len() <= MAX_MESSAGE_BYTES <= u32::MAX.
    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Read a 4-byte LE length-prefix framed message from `reader` and deserialize it.
///
/// # Wire Format
///
/// Reads `[u32 LE length][JSON bytes]`.
///
/// # Errors
///
/// - [`IpcError::Disconnected`] — if the reader returns EOF (0 bytes on the first read).
/// - [`IpcError::MessageTooLarge`] — if the declared length exceeds `MAX_MESSAGE_BYTES`.
/// - [`IpcError::IoError`] — if reading from `reader` fails mid-stream.
/// - [`IpcError::SerializeError`] — if the payload is not valid JSON for `T`.
pub async fn read_framed<R, T>(reader: &mut R) -> Result<T, IpcError>
where
    R: AsyncReadExt + Unpin,
    T: serde::de::DeserializeOwned,
{
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(IpcError::Disconnected);
        }
        Err(e) => return Err(IpcError::IoError(e)),
    }
    let payload_len = u32::from_le_bytes(len_buf) as usize;
    if payload_len > MAX_MESSAGE_BYTES {
        return Err(IpcError::MessageTooLarge);
    }
    let mut payload = vec![0u8; payload_len];
    match reader.read_exact(&mut payload).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(IpcError::Disconnected);
        }
        Err(e) => return Err(IpcError::IoError(e)),
    }
    let value = serde_json::from_slice(&payload)?;
    Ok(value)
}
