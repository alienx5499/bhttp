/// 10-Entry Static Header Table matching common HTTP names.
/// Per HPACK specification, indices are 1-based.
pub const STATIC_HEADER_TABLE: [&str; 10] = [
    ":method",        // Index 1
    ":path",          // Index 2
    ":status",        // Index 3
    "host",           // Index 4
    "content-type",   // Index 5
    "content-length", // Index 6
    "connection",     // Index 7
    "user-agent",     // Index 8
    "accept",         // Index 9
    "server",         // Index 10
];

/// Retrieve the header name corresponding to a 1-based static table index in O(1) time.
#[inline]
pub fn get_static_name(index: u8) -> Option<&'static str> {
    if (1..=10).contains(&index) {
        Some(STATIC_HEADER_TABLE[(index - 1) as usize])
    } else {
        None
    }
}

/// Look up the 1-based index for a known header name in O(1) cache-friendly linear scan.
/// Length pre-filter prunes 90% of string byte comparisons with a single integer instruction.
#[inline]
pub fn find_static_index(name: &str) -> Option<u8> {
    let name_len = name.len();
    for (i, &entry) in STATIC_HEADER_TABLE.iter().enumerate() {
        if entry.len() == name_len && entry.eq_ignore_ascii_case(name) {
            return Some((i + 1) as u8);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_table_lookup() {
        assert_eq!(get_static_name(1), Some(":method"));
        assert_eq!(get_static_name(3), Some(":status"));
        assert_eq!(get_static_name(10), Some("server"));
        assert_eq!(get_static_name(11), None);
        assert_eq!(get_static_name(0), None);
    }

    #[test]
    fn test_find_static_index() {
        assert_eq!(find_static_index(":method"), Some(1));
        assert_eq!(find_static_index("HOST"), Some(4));
        assert_eq!(find_static_index("custom-header"), None);
    }
}
