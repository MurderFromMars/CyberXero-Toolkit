//! I/O utilities for protocol message serialization/deserialization.

use anyhow::{Context, Result};
use rkyv::api::high;
use rkyv::rancor::Error;
use rkyv::ser::allocator::ArenaHandle;
use rkyv::util::AlignedVec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Write a rkyv-serialized message to a writer.
///
/// The format is: [8-byte length (u64, little-endian)][message bytes]
pub async fn write_message<W, M>(writer: &mut W, message: &M) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
    for<'a> M: rkyv::Serialize<high::HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
{
    let bytes = high::to_bytes(message).context("Failed to serialize message")?;
    let len = bytes.len() as u64;
    let len_bytes = len.to_le_bytes();
    writer.write_all(&len_bytes).await?;
    writer.write_all(&bytes).await?;
    Ok(())
}

/// Read an rkyv-serialized message from a reader.
///
/// Returns `None` on EOF, `Some(message)` on success.
/// Uses unchecked deserialization since we control the data source.
pub async fn read_message<R, M>(reader: &mut R) -> Result<Option<M>>
where
    R: AsyncReadExt + Unpin,
    M: rkyv::Archive,
    M::Archived: rkyv::Deserialize<M, high::HighDeserializer<Error>>,
{
    // Read length prefix (u64, little-endian)
    let mut len_bytes = [0u8; 8];
    match reader.read_exact(&mut len_bytes).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Ok(None);
        }
        Err(e) => return Err(e.into()),
    }
    let len = u64::from_le_bytes(len_bytes) as usize;

    // Read the message bytes
    let mut buffer = vec![0u8; len];
    reader.read_exact(&mut buffer).await?;

    // Deserialize using unchecked high-level API (we control the data source)
    let message: M = unsafe { high::from_bytes_unchecked(&buffer[..]) }
        .context("Failed to deserialize message")?;
    Ok(Some(message))
}
