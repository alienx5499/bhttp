pub mod connection;
pub mod file_server;

pub use connection::ServerConnection;
pub use file_server::FileServer;

use std::io;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

/// Launch the multithreaded binary HTTP server on the specified port.
pub fn start_server(root_dir: &str, port: u16) -> io::Result<()> {
    let file_server = Arc::new(FileServer::new(root_dir));
    let addr = format!("0.0.0.0:{}", port);
    // Bind TCP socket to the specified port
    let listener = TcpListener::bind(&addr)?;
    println!("bserve: listening on {} (root: {})", addr, root_dir);

    // Accept incoming TCP connections and spawn a thread per client connection.
    // Each thread maintains a persistent connection loop to handle multiple streams.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let fs = Arc::clone(&file_server);
                thread::spawn(move || {
                    let mut conn = ServerConnection::new(stream, fs);
                    if let Err(e) = conn.run() {
                        let _ = e;
                    }
                });
            }
            Err(e) => {
                eprintln!("[bserve] Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}
