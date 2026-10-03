use crate::frame::{FrameFlags, FrameHeader, FrameType, RawFrame};
use crate::hpack::{HeaderDecoder, HeaderEncoder};
use std::io::{self, Error, ErrorKind};

/// High-level representation of an incoming or outgoing binary HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryRequest {
    pub stream_id: u32,
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl BinaryRequest {
    pub fn new(
        stream_id: u32,
        method: impl Into<String>,
        path: impl Into<String>,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            stream_id,
            method: method.into(),
            path: path.into(),
            headers,
            body,
        }
    }

    /// Build the binary frames representing this request.
    /// Request framing rules:
    ///   - Emits a HEADERS frame containing :method, :path, and all request headers.
    ///   - If the request has NO body, the HEADERS frame sets both END_HEADERS and END_STREAM.
    ///   - If the request HAS a body, the HEADERS frame sets END_HEADERS, and an additional
    ///     DATA frame follows with the payload bytes and flag END_STREAM.
    pub fn to_frames(&self) -> Vec<RawFrame> {
        // Pre-allocate header pairs vector with capacity for pseudo-headers + custom headers
        let mut header_pairs = Vec::with_capacity(self.headers.len() + 2);
        // Prepend required HTTP/2 pseudo-headers
        header_pairs.push((":method", self.method.as_str()));
        header_pairs.push((":path", self.path.as_str()));
        for (k, v) in &self.headers {
            header_pairs.push((k.as_str(), v.as_str()));
        }

        // Compress headers into binary block using HPACK static table + literals
        let encoded_headers = HeaderEncoder::encode(header_pairs);
        let has_body = !self.body.is_empty();

        // Flag END_HEADERS indicates header block is complete in this frame
        let mut flags = FrameFlags::end_headers();
        // If there is no subsequent DATA frame, close the client stream with END_STREAM
        if !has_body {
            flags = flags.with_end_stream();
        }

        let headers_header = FrameHeader::new(
            encoded_headers.len() as u32,
            FrameType::Headers,
            flags,
            self.stream_id,
        );
        let mut frames = vec![RawFrame::new(headers_header, encoded_headers)];

        // Append DATA frame containing body bytes if present
        if has_body {
            let data_header = FrameHeader::new(
                self.body.len() as u32,
                FrameType::Data,
                FrameFlags::end_stream(),
                self.stream_id,
            );
            frames.push(RawFrame::new(data_header, self.body.clone()));
        }

        frames
    }

    /// Parse a BinaryRequest from an incoming HEADERS frame payload and stream ID.
    pub fn from_headers_payload(stream_id: u32, payload: &[u8]) -> io::Result<Self> {
        let headers = HeaderDecoder::decode(payload)?;
        let mut method = None;
        let mut path = None;
        let mut other_headers = Vec::new();

        for (k, v) in headers {
            if k == ":method" {
                method = Some(v);
            } else if k == ":path" {
                path = Some(v);
            } else {
                other_headers.push((k, v));
            }
        }

        let method = method
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing pseudo-header :method"))?;
        let path =
            path.ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing pseudo-header :path"))?;

        Ok(Self {
            stream_id,
            method,
            path,
            headers: other_headers,
            body: Vec::new(),
        })
    }
}
