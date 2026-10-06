# bHTTP: Binary HTTP Protocol Engine

A pure Rust implementation of binary HTTP with multiplexed streaming, static header table compression, and persistent connections over raw TCP.

- **Track 1 (`bserve`):** Static file server over binary HTTP framing with path traversal protection and persistent sockets.
- **Track 2 (`bcurl`):** Command-line client with `-v` frame inspection, body streaming, and non-zero exit on error codes.
- **Specification:** [SPEC.md](SPEC.md) — Protocol wire format and state machine.
- **Hexdump:** [HEXDUMP.md](HEXDUMP.md) — Annotated byte-by-byte frame breakdown.

---

## Architecture

<img width="5511" height="3141" src="https://github.com/user-attachments/assets/f878f3a4-bbc1-4822-86fc-21dc6ec79e5b" />

### Protocol Highlights
1. **Deterministic 9-Byte Framing:** `[24b Length] [8b Type] [8b Flags] [31b Stream ID]`.
2. **Forward Compatibility:** Any endpoint encountering an unknown frame type skips exactly `Length` bytes cleanly through a streaming discard sink without terminating the connection.
3. **HPACK Static Table:** 10 pre-indexed header names (`:method`, `:path`, `:status`, `host`, `content-type`, etc.) combined with length-prefixed literals.
4. **Zero Crates:** Built exclusively on Rust's standard library (`std::net`, `std::io`, `std::fs`, `std::thread`).

---

## Build & Run

### 1. Build Binaries
```bash
cargo build --release
```
Release binaries are placed in `./target/release/bserve` and `./target/release/bcurl`.

### 2. Run Server (`bserve`)
```bash
./target/release/bserve ./www 9000
```

### 3. Run Client (`bcurl`)
In another terminal:
```bash
# Fetch index.html with verbose frame hexdumps
./target/release/bcurl -v localhost:9000/index.html

# Fetch plain text without verbose logs
./target/release/bcurl localhost:9000/test.txt

# Test 404 handling (exits with code 1)
./target/release/bcurl localhost:9000/missing.html
```

---

## Testing

```bash
cargo test
```

Verifies:
- 9-byte header serialization and big-endian bitpacking round-trips.
- Unknown frame type skip handling and streaming discard resilience.
- HPACK static table indexing and literal fallback.
- Directory traversal defenses (`..` escapes rejected).
- Full end-to-end client-server frame exchange over a single persistent TCP socket.
