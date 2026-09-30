pub mod flags;
pub mod frame_type;
pub mod header;

pub use flags::FrameFlags;
pub use frame_type::FrameType;
pub use header::{FRAME_HEADER_LEN, FrameHeader};

use std::io::{self, Read, Write};

/// An in-memory binary frame containing a 9-byte header and payload bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl RawFrame {
    pub fn new(header: FrameHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// Write both the 9-byte header and payload bytes to the stream.
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        self.header.write_to(writer)?;
        if !self.payload.is_empty() {
            writer.write_all(&self.payload)?;
        }
        writer.flush()
    }

    /// Read the next frame from a stream (header followed by full payload).
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let header = FrameHeader::read_from(reader)?;
        Self::read_payload(reader, header)
    }

    /// Read only the payload corresponding to an already-decoded header.
    /// This enables two-phase reads: check header first, then decide whether to read or skip.
    pub fn read_payload(reader: &mut impl Read, header: FrameHeader) -> io::Result<Self> {
        let mut payload = vec![0u8; header.length as usize];
        if header.length > 0 {
            reader.read_exact(&mut payload)?;
        }
        Ok(Self { header, payload })
    }

    /// Cleanly discard `length` bytes from reader in O(1) memory using io::copy into io::sink.
    /// Defends against heap exhaustion attacks from arbitrary unknown frame sizes by streaming
    /// through an internal 8KB stack buffer instead of allocating a dynamic vector.
    pub fn skip_payload(reader: &mut impl Read, length: u32) -> io::Result<()> {
        if length > 0 {
            let mut take = reader.take(length as u64);
            io::copy(&mut take, &mut io::sink())?;
        }
        Ok(())
    }
}
