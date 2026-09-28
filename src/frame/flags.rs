use std::fmt;

/// Bitflags representing frame control options in the 9-byte header (Byte 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameFlags(pub u8);

impl FrameFlags {
    /// No flags set (0x00).
    pub const NONE: u8 = 0x00;
    /// END_STREAM (0x01): Signals that this is the final frame sent on this stream.
    /// Closes the sender's half of the stream.
    pub const END_STREAM: u8 = 0x01;
    /// END_HEADERS (0x04): Signals that the entire header block is contained in this frame.
    pub const END_HEADERS: u8 = 0x04;
    /// PADDED (0x08): Reserved for security padding.
    pub const PADDED: u8 = 0x08;

    pub fn new(bits: u8) -> Self {
        Self(bits)
    }

    pub fn end_stream() -> Self {
        Self(Self::END_STREAM)
    }

    pub fn end_headers() -> Self {
        Self(Self::END_HEADERS)
    }

    pub fn end_all() -> Self {
        Self(Self::END_STREAM | Self::END_HEADERS)
    }

    pub fn has_end_stream(self) -> bool {
        (self.0 & Self::END_STREAM) != 0
    }

    pub fn has_end_headers(self) -> bool {
        (self.0 & Self::END_HEADERS) != 0
    }

    pub fn with_end_stream(mut self) -> Self {
        self.0 |= Self::END_STREAM;
        self
    }

    pub fn with_end_headers(mut self) -> Self {
        self.0 |= Self::END_HEADERS;
        self
    }

    pub fn bits(self) -> u8 {
        self.0
    }
}

impl fmt::Display for FrameFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.has_end_stream() {
            flags.push("END_STREAM");
        }
        if self.has_end_headers() {
            flags.push("END_HEADERS");
        }
        if flags.is_empty() {
            write!(f, "0x00")
        } else {
            write!(f, "0x{:02X} ({})", self.0, flags.join("|"))
        }
    }
}
