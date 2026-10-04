use std::env;
use std::process;

fn main() {
    // Read command line arguments: expects <www-directory> <port>
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <www-directory> <port>", args[0]);
        eprintln!("Example: {} ./www 9000", args[0]);
        process::exit(1);
    }

    let root_dir = &args[1];
    // Validate port is a valid u16 (1..=65535)
    let port: u16 = match args[2].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Error: Invalid port number '{}'", args[2]);
            process::exit(1);
        }
    };

    // Track 1 requirement: start binary HTTP server accepting TCP connections
    if let Err(e) = bhttp::start_server(root_dir, port) {
        eprintln!("[bserve] Fatal server error: {}", e);
        process::exit(1);
    }
}
