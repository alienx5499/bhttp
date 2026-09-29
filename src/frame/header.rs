use super::flags::FrameFlags;
use super::frame_type::FrameType;
use std::io::{self, Read, Write};

pub const FRAME_HEADER_LEN: usize = 9;

/// Fixed 9-byte frame header matching RFC 7540 §4.1 layout:
/// - 24 bits: Payload Length
/// - 8 bits: Frame Type
/// - 8 bits: Flags
/// - 31 bits: Stream ID (1 reserved bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub length: u32,
    pub frame_type: FrameType,
    pub flags: FrameFlags,
    pub stream_id: u32,
}

impl FrameHeader {
    pub const MAX_PAYLOAD_LEN: u32 = 0x00FF_FFFF; // 16,777,215 bytes (24 bits)

    pub fn new(length: u32, frame_type: FrameType, flags: FrameFlags, stream_id: u32) -> Self {
        assert!(
            length <= Self::MAX_PAYLOAD_LEN,
            "Frame payload length exceeds 24-bit maximum"
        );
        Self {
            length,
            frame_type,
            flags,
            stream_id: stream_id & 0x7FFF_FFFF,
        }
    }

    /// Serialize the 9-byte frame header into fixed wire bytes without heap allocation.
    /// Wire layout:
    ///   Bytes 0..3: 24-bit big-endian payload length (supports frames up to 16,777,215 bytes)
    ///   Byte 3:     8-bit frame type identifier (e.g. 0x01 = HEADERS, 0x02 = DATA)
    ///   Byte 4:     8-bit frame flags (bit 0 = END_STREAM, bit 2 = END_HEADERS)
    ///   Bytes 5..9: 31-bit stream ID (big-endian, MSB reserved as 0)
    pub fn encode(&self) -> [u8; FRAME_HEADER_LEN] {
        let mut buf = [0u8; FRAME_HEADER_LEN];
        // High 8 bits of 24-bit length
        buf[0] = ((self.length >> 16) & 0xFF) as u8;
        // Middle 8 bits of 24-bit length
        buf[1] = ((self.length >> 8) & 0xFF) as u8;
        // Low 8 bits of 24-bit length
        buf[2] = (self.length & 0xFF) as u8;
        // Frame type discriminator byte
        buf[3] = self.frame_type.to_byte();
        // Active bitflags byte
        buf[4] = self.flags.bits();
        // 31-bit stream identifier (clear highest bit to reserve it for future use)
        let sid_bytes = (self.stream_id & 0x7FFF_FFFF).to_be_bytes();
        buf[5..9].copy_from_slice(&sid_bytes);
        buf
    }

    /// Parse a 9-byte frame header from wire bytes.
    /// Reconstitutes the 24-bit length, type, flags, and 31-bit stream ID.
    pub fn decode(bytes: &[u8; FRAME_HEADER_LEN]) -> Self {
        // Reassemble the 3-byte big-endian integer into a u32
        let length = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        // Map byte to known frame enum, falling back to Unknown(u8) for forward compatibility
        let frame_type = FrameType::from_byte(bytes[3]);
        // Extract bitflags (END_STREAM = 0x01, END_HEADERS = 0x04)
        let flags = FrameFlags::new(bytes[4]);
        // Read 4-byte big-endian stream ID and mask out the 1 reserved bit
        let stream_id = u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) & 0x7FFF_FFFF;

        Self {
            length,
            frame_type,
            flags,
            stream_id,
        }
    }

    /// Write header directly to an I/O stream.
    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.encode())
    }

    /// Read a fixed 9-byte header from an I/O stream.
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut buf = [0u8; FRAME_HEADER_LEN];
        reader.read_exact(&mut buf)?;
        Ok(Self::decode(&buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let header = FrameHeader::new(1024, FrameType::Headers, FrameFlags::end_all(), 1);
        let encoded = header.encode();
        assert_eq!(encoded.len(), 9);
        let decoded = FrameHeader::decode(&encoded);
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_unknown_frame_type_preservation() {
        let header = FrameHeader::new(42, FrameType::Unknown(0xFE), FrameFlags::new(0), 3);
        let encoded = header.encode();
        assert_eq!(encoded[3], 0xFE);
        let decoded = FrameHeader::decode(&encoded);
        assert_eq!(decoded.frame_type, FrameType::Unknown(0xFE));
        assert_eq!(decoded.length, 42);
    }
}
