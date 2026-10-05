use bhttp::ClientConnection;
use std::env;
use std::io::{self, Write};
use std::process;

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.is_empty() {
        eprintln!("Usage: bcurl [-v] <host:port/path>");
        eprintln!("Example: bcurl -v localhost:9000/index.html");
        process::exit(1);
    }

    // Parse flags: look for -v or --verbose
    let mut verbose = false;
    let mut target_arg = None;

    for arg in raw_args {
        if arg == "-v" || arg == "--verbose" {
            verbose = true;
        } else if target_arg.is_none() {
            target_arg = Some(arg);
        }
    }

    let target = match target_arg {
        Some(t) => t,
        None => {
            eprintln!("Error: Missing target URL");
            process::exit(1);
        }
    };

    // Strip http:// prefix if user provided it
    let stripped = target.trim_start_matches("http://");
    let (host_port, path) = match stripped.find('/') {
        Some(slash_idx) => (&stripped[..slash_idx], &stripped[slash_idx..]),
        None => (stripped, "/"),
    };

    // Track 2 requirement: connect once to host:port
    let mut client = match ClientConnection::connect(host_port, verbose) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[bcurl] Connection error to {}: {}", host_port, e);
            process::exit(1);
        }
    };

    // Send binary GET request over the persistent socket
    match client.send_request("GET", path, host_port, Vec::new()) {
        Ok(resp) => {
            // Track 2 requirement: body bytes stream directly to stdout
            let _ = io::stdout().write_all(&resp.body);
            let _ = io::stdout().flush();

            // Track 2 requirement: exit non-zero if server returned a 4xx or 5xx status code
            let status_code = resp.status.code();
            if status_code >= 400 {
                eprintln!("\n[bcurl] HTTP error status: {}", resp.status);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("[bcurl] Request failed: {}", e);
            process::exit(1);
        }
    }
}
