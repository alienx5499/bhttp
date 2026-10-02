use super::static_table::get_static_name;
use std::io::{self, Error, ErrorKind};
use std::str;

/// Deserializes binary HPACK-style representation into key-value header pairs.
pub struct HeaderDecoder;

impl HeaderDecoder {
    /// Decode wire bytes into a list of (name, value) strings.
    /// Iterates sequentially through the payload:
    ///   - Checks tag byte high bit (0x80) to route between indexed vs literal fields
    ///   - Validates that byte slices fit within the payload bounds before slicing
    ///   - Validates UTF-8 encoding of all strings
    pub fn decode(payload: &[u8]) -> io::Result<Vec<(String, String)>> {
        let len = payload.len();
        // Heuristic: average header entry is ~16 bytes, pre-allocate to minimize reallocations
        let mut headers = Vec::with_capacity(len / 16 + 1);
        let mut offset = 0;

        while offset < len {
            // Read 1-byte mechanism tag
            let tag = payload[offset];
            offset += 1;

            // Route to indexed or literal sub-parser
            let entry = if (tag & 0x80) != 0 {
                Self::decode_indexed_entry(tag, payload, &mut offset, len)?
            } else {
                Self::decode_literal_entry(payload, &mut offset, len)?
            };

            headers.push(entry);
        }

        Ok(headers)
    }

    /// Decode an indexed entry (Mechanism 1: 0x80 | index followed by 2-byte value length).
    #[inline]
    fn decode_indexed_entry(
        tag: u8,
        payload: &[u8],
        offset: &mut usize,
        len: usize,
    ) -> io::Result<(String, String)> {
        let index = tag & 0x7F;
        let static_name = get_static_name(index).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Invalid static table index: {}", index),
            )
        })?;

        let val_str = Self::read_length_prefixed_str(payload, offset, len, "value")?;
        Ok((static_name.to_string(), val_str.to_string()))
    }

    /// Decode a literal entry (Mechanism 2: 0x00 followed by 2-byte name len + name + 2-byte val len + val).
    #[inline]
    fn decode_literal_entry(
        payload: &[u8],
        offset: &mut usize,
        len: usize,
    ) -> io::Result<(String, String)> {
        let name_str = Self::read_length_prefixed_str(payload, offset, len, "literal name")?;
        let val_str = Self::read_length_prefixed_str(payload, offset, len, "literal value")?;
        Ok((name_str.to_string(), val_str.to_string()))
    }

    /// Read a 2-byte big-endian length prefix followed by UTF-8 string bytes in O(L) time without heap copies.
    #[inline]
    fn read_length_prefixed_str<'a>(
        payload: &'a [u8],
        offset: &mut usize,
        len: usize,
        field_name: &str,
    ) -> io::Result<&'a str> {
        if *offset + 2 > len {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!("Truncated {} length", field_name),
            ));
        }
        let str_len = u16::from_be_bytes([payload[*offset], payload[*offset + 1]]) as usize;
        *offset += 2;

        if *offset + str_len > len {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!("Truncated {} bytes", field_name),
            ));
        }

        let slice = &payload[*offset..*offset + str_len];
        *offset += str_len;

        str::from_utf8(slice).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::super::encoder::HeaderEncoder;
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let input = [
            (":method", "GET"),
            (":path", "/index.html"),
            ("host", "localhost:9000"),
            ("x-custom-key", "custom-value-123"),
        ];
        let bytes = HeaderEncoder::encode(input);
        let decoded = HeaderDecoder::decode(&bytes).expect("Failed to decode headers");

        assert_eq!(decoded.len(), 4);
        assert_eq!(decoded[0], (":method".into(), "GET".into()));
        assert_eq!(decoded[1], (":path".into(), "/index.html".into()));
        assert_eq!(decoded[2], ("host".into(), "localhost:9000".into()));
        assert_eq!(
            decoded[3],
            ("x-custom-key".into(), "custom-value-123".into())
        );
    }
}
