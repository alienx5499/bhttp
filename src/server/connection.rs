use super::file_server::FileServer;
use crate::frame::{FrameHeader, FrameType, RawFrame};
use crate::proto::{BinaryRequest, BinaryResponse};
use std::io::{self, ErrorKind};
use std::net::TcpStream;
use std::sync::Arc;

/// Handles a persistent binary HTTP connection on the server.
pub struct ServerConnection {
    stream: TcpStream,
    file_server: Arc<FileServer>,
}

impl ServerConnection {
    pub fn new(stream: TcpStream, file_server: Arc<FileServer>) -> Self {
        Self {
            stream,
            file_server,
        }
    }

    /// Process incoming binary frames indefinitely until client disconnects or sends GOAWAY.
    /// Implements persistent connection semantics: multiple logical streams are served sequentially
    /// over this single TCP stream without terminating or re-dialing.
    pub fn run(&mut self) -> io::Result<()> {
        loop {
            // Step 1: Read the fixed 9-byte header to determine frame boundaries
            let header = match FrameHeader::read_from(&mut self.stream) {
                Ok(h) => h,
                // Clean TCP FIN from client indicating connection shutdown
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    break;
                }
                // Client forcibly reset connection (RST packet)
                Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                    break;
                }
                Err(e) => return Err(e),
            };

            // Step 2: Non-negotiable forward-compatibility rule
            // A receiver meeting a frame type it does not know MUST skip it cleanly based on
            // the 24-bit length without aborting the TCP session.
            if header.frame_type.is_unknown() {
                eprintln!(
                    "[bserve] Notice: skipping unknown frame type {} (length: {} bytes)",
                    header.frame_type, header.length
                );
                // Stream-discard exactly header.length bytes in O(1) memory
                RawFrame::skip_payload(&mut self.stream, header.length)?;
                continue;
            }

            // Step 3: For known frame types, read the exact payload bytes
            let frame = RawFrame::read_payload(&mut self.stream, header)?;

            // Step 4: Dispatch frame based on its type code
            match frame.header.frame_type {
                FrameType::Headers => {
                    self.handle_headers_frame(frame)?;
                }
                FrameType::Ping => {
                    // Echo PING back with identical payload for round-trip latency checks
                    let pong = RawFrame::new(frame.header, frame.payload);
                    pong.write_to(&mut self.stream)?;
                }
                FrameType::GoAway => {
                    // Client requested graceful connection closure
                    eprintln!("[bserve] Client requested GOAWAY, closing connection");
                    break;
                }
                FrameType::Data | FrameType::Settings | FrameType::RstStream => {
                    // Standard frames accepted without error in v1.0
                    continue;
                }
                FrameType::Unknown(_) => unreachable!(),
            }
        }

        Ok(())
    }

    /// Decode HEADERS payload, validate pseudo-headers, resolve static file, and send response frames.
    fn handle_headers_frame(&mut self, frame: RawFrame) -> io::Result<()> {
        let stream_id = frame.header.stream_id;

        // Step 1: Deserialize binary HPACK header block into request model
        let req = match BinaryRequest::from_headers_payload(stream_id, &frame.payload) {
            Ok(r) => r,
            Err(e) => {
                // Return 400 Bad Request if headers are malformed or missing pseudo-headers
                let resp =
                    BinaryResponse::bad_request(stream_id, &format!("Malformed frame: {}", e));
                for f in resp.to_frames() {
                    f.write_to(&mut self.stream)?;
                }
                return Ok(());
            }
        };

        // Step 2: Validate method (bserve static file server only supports GET and HEAD)
        let resp = if req.method != "GET" && req.method != "HEAD" {
            BinaryResponse::method_not_allowed(stream_id)
        } else {
            // Step 3: Resolve requested path safely inside the web root
            match self.file_server.resolve_path(&req.path) {
                Some(file_path) => match self.file_server.read_file(&file_path) {
                    Some((bytes, mime)) => BinaryResponse::ok(stream_id, bytes, mime),
                    None => BinaryResponse::not_found(
                        stream_id,
                        &format!("Unable to read file: {}", req.path),
                    ),
                },
                None => {
                    BinaryResponse::not_found(stream_id, &format!("File not found: {}", req.path))
                }
            }
        };

        // Step 4: Serialize response into HEADERS + DATA frames and send over socket
        for f in resp.to_frames() {
            f.write_to(&mut self.stream)?;
        }

        Ok(())
    }
}
