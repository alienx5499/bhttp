use bhttp::frame::{FrameFlags, FrameHeader, FrameType, RawFrame};
use bhttp::proto::BinaryStatus;
use bhttp::{ClientConnection, FileServer, ServerConnection};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

#[test]
fn test_end_to_end_server_client() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let file_server = Arc::new(FileServer::new("./www"));

    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let mut conn = ServerConnection::new(stream, file_server);
            let _ = conn.run();
        }
    });

    let mut client = ClientConnection::connect(format!("127.0.0.1:{}", port), false).unwrap();

    // 1. Valid request for index.html
    let resp = client
        .send_request("GET", "/index.html", "localhost", Vec::new())
        .unwrap();
    assert_eq!(resp.status, BinaryStatus::Ok);
    let body_str = String::from_utf8(resp.body).unwrap();
    assert!(body_str.contains("Binary HTTP Protocol Works!"));

    // 2. Request for non-existent file on the same connection
    let resp_404 = client
        .send_request("GET", "/not_found.html", "localhost", Vec::new())
        .unwrap();
    assert_eq!(resp_404.status, BinaryStatus::NotFound);
}

#[test]
fn test_unknown_frame_type_skipped_cleanly() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let file_server = Arc::new(FileServer::new("./www"));

    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let mut conn = ServerConnection::new(stream, file_server);
            let _ = conn.run();
        }
    });

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();

    // Send an unknown frame type (0xFE) with a 12-byte payload
    let unknown_payload = b"future_v2_data";
    let unknown_hdr = FrameHeader::new(
        unknown_payload.len() as u32,
        FrameType::Unknown(0xFE),
        FrameFlags::new(0),
        0,
    );
    let unknown_frame = RawFrame::new(unknown_hdr, unknown_payload.to_vec());
    unknown_frame.write_to(&mut stream).unwrap();

    // Immediately follow with a valid GET /test.txt request on the SAME stream
    let req = bhttp::BinaryRequest::new(1, "GET", "/test.txt", Vec::new(), Vec::new());
    for f in req.to_frames() {
        f.write_to(&mut stream).unwrap();
    }

    // Read response frames: the server must have skipped the 0xFE frame and answered the GET request
    let headers_frame = RawFrame::read_from(&mut stream).unwrap();
    assert_eq!(headers_frame.header.frame_type, FrameType::Headers);
    let resp = bhttp::BinaryResponse::from_headers_payload(1, &headers_frame.payload).unwrap();
    assert_eq!(resp.status, BinaryStatus::Ok);

    let data_frame = RawFrame::read_from(&mut stream).unwrap();
    assert_eq!(data_frame.header.frame_type, FrameType::Data);
    let text = String::from_utf8(data_frame.payload).unwrap();
    assert!(text.contains("Hello from Binary HTTP"));
}

#[test]
fn test_large_unknown_frame_skipped_cleanly() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let file_server = Arc::new(FileServer::new("./www"));

    thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let mut conn = ServerConnection::new(stream, file_server);
            let _ = conn.run();
        }
    });

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();

    // 64 KB unknown frame payload
    let large_payload = vec![0xAA; 65536];
    let unknown_hdr = FrameHeader::new(
        large_payload.len() as u32,
        FrameType::Unknown(0xAA),
        FrameFlags::new(0),
        0,
    );
    let unknown_frame = RawFrame::new(unknown_hdr, large_payload);
    unknown_frame.write_to(&mut stream).unwrap();

    // Send valid request immediately after
    let req = bhttp::BinaryRequest::new(3, "GET", "/test.txt", Vec::new(), Vec::new());
    for f in req.to_frames() {
        f.write_to(&mut stream).unwrap();
    }

    let headers_frame = RawFrame::read_from(&mut stream).unwrap();
    let resp = bhttp::BinaryResponse::from_headers_payload(3, &headers_frame.payload).unwrap();
    assert_eq!(resp.status, BinaryStatus::Ok);
}
