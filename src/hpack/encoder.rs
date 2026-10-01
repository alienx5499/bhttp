use super::static_table::find_static_index;

/// Serializes key-value header pairs into binary HPACK-style representation.
pub struct HeaderEncoder;

impl HeaderEncoder {
    /// Encode a list of (name, value) headers into wire bytes.
    /// Wire encoding rules:
    ///   - If header name is in static table: emit [0x80 | index] [u16 value_len] [value bytes]
    ///   - Otherwise (custom header): emit [0x00] [u16 name_len] [name bytes] [u16 value_len] [value bytes]
    pub fn encode<'a, I>(headers: I) -> Vec<u8>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        // Pre-allocate buffer with typical header block capacity to avoid reallocations
        let mut buf = Vec::with_capacity(128);
        for (name, val) in headers {
            if let Some(index) = find_static_index(name) {
                // Mechanism 1: Indexed Name
                // High bit set (0x80) indicates the name is an index into the 10-entry static table.
                buf.push(0x80 | (index & 0x7F));

                // 2-byte big-endian value length prefix
                let val_bytes = val.as_bytes();
                buf.extend_from_slice(&(val_bytes.len() as u16).to_be_bytes());
                // Raw UTF-8 value payload bytes
                buf.extend_from_slice(val_bytes);
            } else {
                // Mechanism 2: Literal Name
                // Leading 0x00 tag indicates that the header name is supplied as a literal string.
                buf.push(0x00);

                // 2-byte name length + UTF-8 name bytes
                let name_bytes = name.as_bytes();
                buf.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
                buf.extend_from_slice(name_bytes);

                // 2-byte value length + UTF-8 value bytes
                let val_bytes = val.as_bytes();
                buf.extend_from_slice(&(val_bytes.len() as u16).to_be_bytes());
                buf.extend_from_slice(val_bytes);
            }
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_indexed_and_literal() {
        let headers = [(":method", "GET"), ("x-custom", "value")];
        let bytes = HeaderEncoder::encode(headers);

        // First header is indexed: 0x81, 0x00, 0x03, 'G', 'E', 'T'
        assert_eq!(bytes[0], 0x81);
        assert_eq!(&bytes[1..3], &[0x00, 0x03]);
        assert_eq!(&bytes[3..6], b"GET");

        // Second header is literal: 0x00, 0x00, 0x08, 'x-custom', 0x00, 0x05, 'value'
        assert_eq!(bytes[6], 0x00);
        assert_eq!(&bytes[7..9], &[0x00, 0x08]);
        assert_eq!(&bytes[9..17], b"x-custom");
        assert_eq!(&bytes[17..19], &[0x00, 0x05]);
        assert_eq!(&bytes[19..24], b"value");
    }
}
