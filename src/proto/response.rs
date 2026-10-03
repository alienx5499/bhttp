use super::status::BinaryStatus;
use crate::frame::{FrameFlags, FrameHeader, FrameType, RawFrame};
use crate::hpack::{HeaderDecoder, HeaderEncoder};
use std::io::{self, Error, ErrorKind};

/// High-level representation of an incoming or outgoing binary HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryResponse {
    pub stream_id: u32,
    pub status: BinaryStatus,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl BinaryResponse {
    pub fn new(
        stream_id: u32,
        status: BinaryStatus,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            stream_id,
            status,
            headers,
            body,
        }
    }

    pub fn ok(stream_id: u32, body: Vec<u8>, content_type: &str) -> Self {
        let headers = vec![
            ("content-type".to_string(), content_type.to_string()),
            ("content-length".to_string(), body.len().to_string()),
            ("server".to_string(), "bserve/1.0".to_string()),
        ];
        Self::new(stream_id, BinaryStatus::Ok, headers, body)
    }

    pub fn not_found(stream_id: u32, msg: &str) -> Self {
        let body = format!("404 Not Found: {}\n", msg).into_bytes();
        let headers = vec![
            ("content-type".to_string(), "text/plain".to_string()),
            ("content-length".to_string(), body.len().to_string()),
        ];
        Self::new(stream_id, BinaryStatus::NotFound, headers, body)
    }

    pub fn bad_request(stream_id: u32, msg: &str) -> Self {
        let body = format!("400 Bad Request: {}\n", msg).into_bytes();
        let headers = vec![
            ("content-type".to_string(), "text/plain".to_string()),
            ("content-length".to_string(), body.len().to_string()),
        ];
        Self::new(stream_id, BinaryStatus::BadRequest, headers, body)
    }

    pub fn method_not_allowed(stream_id: u32) -> Self {
        let body = b"405 Method Not Allowed\n".to_vec();
        let headers = vec![
            ("content-type".to_string(), "text/plain".to_string()),
            ("content-length".to_string(), body.len().to_string()),
        ];
        Self::new(stream_id, BinaryStatus::MethodNotAllowed, headers, body)
    }

    /// Serialize into binary HEADERS frame, plus DATA frame if body is present.
    pub fn to_frames(&self) -> Vec<RawFrame> {
        let status_str = self.status.code().to_string();
        let mut header_pairs = Vec::with_capacity(self.headers.len() + 1);
        header_pairs.push((":status", status_str.as_str()));
        for (k, v) in &self.headers {
            header_pairs.push((k.as_str(), v.as_str()));
        }

        let encoded_headers = HeaderEncoder::encode(header_pairs);
        let has_body = !self.body.is_empty();

        let mut flags = FrameFlags::end_headers();
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

    /// Parse response headers from a HEADERS frame payload.
    pub fn from_headers_payload(stream_id: u32, payload: &[u8]) -> io::Result<Self> {
        let headers = HeaderDecoder::decode(payload)?;
        let mut status = None;
        let mut other_headers = Vec::new();

        for (k, v) in headers {
            if k == ":status" {
                let code = v
                    .parse::<u16>()
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                status = Some(BinaryStatus::from_code(code));
            } else {
                other_headers.push((k, v));
            }
        }

        let status = status
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Missing pseudo-header :status"))?;

        Ok(Self {
            stream_id,
            status,
            headers: other_headers,
            body: Vec::new(),
        })
    }
}
