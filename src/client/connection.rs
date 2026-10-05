use super::hexdump::FrameDumper;
use crate::frame::{FrameHeader, FrameType, RawFrame};
use crate::proto::{BinaryRequest, BinaryResponse};
use std::io::{self, Error, ErrorKind};
use std::net::{TcpStream, ToSocketAddrs};

/// Client connection maintaining a persistent socket and stream counter.
pub struct ClientConnection {
    stream: TcpStream,
    verbose: bool,
    next_stream_id: u32,
}

impl ClientConnection {
    pub fn connect(addr: impl ToSocketAddrs, verbose: bool) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self {
            stream,
            verbose,
            next_stream_id: 1,
        })
    }

    /// Execute a request over the existing TCP stream without opening a new connection.
    /// Follows HTTP/2 client stream rules:
    ///   - Increments stream ID by 2 each request (client-initiated streams are odd numbers: 1, 3, 5...)
    ///   - Reuses the existing self.stream socket (Track 2: "never opens a second connection")
    ///   - Dumps outgoing frames to stderr if verbose mode (-v) is active
    pub fn send_request(
        &mut self,
        method: &str,
        path: &str,
        host: &str,
        extra_headers: Vec<(String, String)>,
    ) -> io::Result<BinaryResponse> {
        // Allocate next odd stream ID
        let stream_id = self.next_stream_id;
        self.next_stream_id += 2;

        // Populate host header + caller-provided headers
        let mut headers = vec![("host".to_string(), host.to_string())];
        headers.extend(extra_headers);

        // Build domain request and serialize to HEADERS (+ optional DATA) frames
        let req = BinaryRequest::new(stream_id, method, path, headers, Vec::new());
        for frame in req.to_frames() {
            if self.verbose {
                FrameDumper::dump_frame("SEND", &frame);
            }
            frame.write_to(&mut self.stream)?;
        }

        // Wait on the same socket for incoming response frames
        self.receive_response(stream_id)
    }

    /// Reads frames for the response until END_STREAM is encountered.
    fn receive_response(&mut self, stream_id: u32) -> io::Result<BinaryResponse> {
        let mut response: Option<BinaryResponse> = None;
        let mut body = Vec::new();

        loop {
            let header = FrameHeader::read_from(&mut self.stream)?;
            if header.frame_type.is_unknown() {
                if self.verbose {
                    eprintln!(
                        "[bcurl] Notice: skipping unknown frame type {}",
                        header.frame_type
                    );
                }
                RawFrame::skip_payload(&mut self.stream, header.length)?;
                continue;
            }

            let frame = RawFrame::read_payload(&mut self.stream, header)?;
            if self.verbose {
                FrameDumper::dump_frame("RECV", &frame);
            }

            match frame.header.frame_type {
                FrameType::Headers => {
                    let mut resp = BinaryResponse::from_headers_payload(stream_id, &frame.payload)?;
                    resp.stream_id = frame.header.stream_id;
                    response = Some(resp);
                }
                FrameType::Data => {
                    body.extend_from_slice(&frame.payload);
                }
                FrameType::GoAway => {
                    return Err(Error::new(
                        ErrorKind::ConnectionAborted,
                        "Server sent GOAWAY",
                    ));
                }
                _ => {}
            }

            if frame.header.flags.has_end_stream() {
                break;
            }
        }

        let mut final_resp = response.ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "Connection closed before headers were received",
            )
        })?;
        final_resp.body = body;
        Ok(final_resp)
    }
}
