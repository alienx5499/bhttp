use crate::frame::RawFrame;

/// Formats raw binary frames into standard annotated hex and ASCII dumps.
pub struct FrameDumper;

impl FrameDumper {
    /// Print an annotated hexdump of an outgoing or incoming binary frame.
    pub fn dump_frame(direction: &str, frame: &RawFrame) {
        let encoded_header = frame.header.encode();
        eprintln!(
            "[{}] Frame Type: {}, Flags: {}, Stream ID: {}, Payload Len: {} bytes",
            direction,
            frame.header.frame_type,
            frame.header.flags,
            frame.header.stream_id,
            frame.header.length
        );

        eprintln!("  Header Bytes (9B):");
        Self::print_bytes(&encoded_header, 4);

        if !frame.payload.is_empty() {
            eprintln!("  Payload Bytes ({}B):", frame.payload.len());
            Self::print_bytes(&frame.payload, 4);
        }
        eprintln!();
    }

    fn print_bytes(bytes: &[u8], indent: usize) {
        let pad = " ".repeat(indent);
        for (row_idx, chunk) in bytes.chunks(16).enumerate() {
            let offset = row_idx * 16;
            let mut hex_part = String::with_capacity(48);
            let mut ascii_part = String::with_capacity(16);

            for (i, &b) in chunk.iter().enumerate() {
                hex_part.push_str(&format!("{:02X} ", b));
                if i == 7 {
                    hex_part.push(' ');
                }
                if b.is_ascii_graphic() || b == b' ' {
                    ascii_part.push(b as char);
                } else {
                    ascii_part.push('.');
                }
            }

            eprintln!("{}{:04X}:  {:<49} |{}|", pad, offset, hex_part, ascii_part);
        }
    }
}
