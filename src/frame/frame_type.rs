use std::fmt;

/// Binary protocol frame types matching HTTP/2 framing semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// Carries HPACK-compressed header blocks (e.g. :method, :path, :status).
    Headers,
    /// Carries raw stream body payload bytes (e.g. file content or request body).
    Data,
    /// Advertises connection configuration parameters (e.g. max frame size).
    Settings,
    /// Round-trip heartbeat and liveness ping; recipient must echo payload.
    Ping,
    /// Initiates graceful connection shutdown without abruptly dropping in-flight streams.
    GoAway,
    /// Aborts an active stream without terminating the underlying TCP connection.
    RstStream,
    /// Forward-compatibility fallback: any unknown frame byte (e.g. from v2) is preserved
    /// so the receiver can cleanly skip its payload without failing.
    Unknown(u8),
}

impl FrameType {
    pub const HEADERS_BYTE: u8 = 0x01;
    pub const DATA_BYTE: u8 = 0x02;
    pub const SETTINGS_BYTE: u8 = 0x03;
    pub const PING_BYTE: u8 = 0x04;
    pub const GOAWAY_BYTE: u8 = 0x05;
    pub const RST_STREAM_BYTE: u8 = 0x07;

    /// Parse a 1-byte frame type identifier with fallback to Unknown for version 2 extension.
    pub fn from_byte(b: u8) -> Self {
        match b {
            Self::HEADERS_BYTE => FrameType::Headers,
            Self::DATA_BYTE => FrameType::Data,
            Self::SETTINGS_BYTE => FrameType::Settings,
            Self::PING_BYTE => FrameType::Ping,
            Self::GOAWAY_BYTE => FrameType::GoAway,
            Self::RST_STREAM_BYTE => FrameType::RstStream,
            other => FrameType::Unknown(other),
        }
    }

    /// Return the raw wire byte representation.
    pub fn to_byte(self) -> u8 {
        match self {
            FrameType::Headers => Self::HEADERS_BYTE,
            FrameType::Data => Self::DATA_BYTE,
            FrameType::Settings => Self::SETTINGS_BYTE,
            FrameType::Ping => Self::PING_BYTE,
            FrameType::GoAway => Self::GOAWAY_BYTE,
            FrameType::RstStream => Self::RST_STREAM_BYTE,
            FrameType::Unknown(b) => b,
        }
    }

    /// Returns true if this frame type is unknown to this version of the protocol.
    pub fn is_unknown(self) -> bool {
        matches!(self, FrameType::Unknown(_))
    }
}

impl fmt::Display for FrameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameType::Headers => write!(f, "HEADERS(0x01)"),
            FrameType::Data => write!(f, "DATA(0x02)"),
            FrameType::Settings => write!(f, "SETTINGS(0x03)"),
            FrameType::Ping => write!(f, "PING(0x04)"),
            FrameType::GoAway => write!(f, "GOAWAY(0x05)"),
            FrameType::RstStream => write!(f, "RST_STREAM(0x07)"),
            FrameType::Unknown(b) => write!(f, "UNKNOWN(0x{:02X})", b),
        }
    }
}
