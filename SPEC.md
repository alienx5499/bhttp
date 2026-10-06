# Binary HTTP Protocol Specification (bHTTP/1.0)

## 1. Overview & Protocol Goals

bHTTP replaces HTTP/1.x delimiter scanning (`\r\n\r\n`) with deterministic binary framing inspired by HTTP/2 and HPACK. All communication runs over a single, long-lived TCP connection carrying multiplexed logical streams.

Core protocol guarantees:
- **Deterministic Framing:** Fixed 9-byte header tells the receiver exactly how many payload bytes follow.
- **Forward Compatibility:** Any endpoint encountering an unrecognized frame type skips the payload cleanly and continues processing without dropping the connection.
- **Zero Delimiter Ambiguity:** Length-prefixed payloads prevent request smuggling and parsing desyncs.

---

## 2. Wire Format & Header Layout

Every frame on the wire starts with a fixed 9-byte header:

<img width="5630" height="760" alt="Adaptive Navigation-2026-09-26-100252" src="https://github.com/user-attachments/assets/f0897805-181d-45e7-bf09-06e6f389fd49" />

### 2.1 Header Fields

| Field | Width | Purpose |
|---|:---:|---|
| **Length** | 24 bits (3B) | Unsigned big-endian integer indicating payload size (0 to 16,777,215 bytes). |
| **Type** | 8 bits (1B) | Identifies frame semantics (HEADERS, DATA, SETTINGS, etc.). |
| **Flags** | 8 bits (1B) | Bitfield modifying frame handling (`0x01` END_STREAM, `0x04` END_HEADERS). |
| **Stream ID** | 31 bits (4B) | Big-endian stream identifier. The top bit is reserved (`R = 0`). Stream 0 is connection-level. |

### 2.2 Design Rationale (Why 24 / 8 / 8 / 31?)
- **24-bit Length:** Supports single-frame payloads up to 16 MB. A 16-bit length (64 KB) causes heavy fragmentation for typical web assets, while a 32-bit length wastes a byte in every single frame header across millions of messages.
- **8-bit Type:** 256 frame types. 6 are defined in v1.0, leaving 250 slots open for future extensions.
- **8-bit Flags:** Dedicated bit positions for `END_STREAM` and `END_HEADERS`.
- **31-bit Stream ID:** Allows 2.14 billion concurrent streams before rollover. Odd IDs are client-initiated; even IDs are server-initiated.

---

## 3. Frame Types & State Transitions

### 3.1 Defined Frame Types

| Type Code | Name | Description |
|:---:|---|---|
| `0x01` | **`HEADERS`** | Carries HPACK-compressed header fields. |
| `0x02` | **`DATA`** | Carries raw stream body bytes. |
| `0x03` | **`SETTINGS`** | Connection configuration. |
| `0x04` | **`PING`** | Heartbeat; receiver echoes payload verbatim. |
| `0x05` | **`GOAWAY`** | Graceful shutdown; connection drain. |
| `0x07` | **`RST_STREAM`** | Aborts a specific stream. |
| `0x08..=0xFF` | **`UNKNOWN`** | Reserved for extensions. **Receiver MUST skip cleanly.** |

### 3.2 Unknown Frame Forward Compatibility

A receiver that encounters an unrecognized frame type **must not** close the connection. It reads the 9-byte header, consumes `Length` bytes from the socket into a discard sink, and resumes parsing the next frame:

<img width="4495" height="2800" src="https://github.com/user-attachments/assets/98fb7f49-217c-4ced-abeb-d25944eb6f48" />

---

## 4. Header Compression (HPACK-Lite)

bHTTP compresses common headers using a 10-entry static table combined with length-prefixed literals.

### 4.1 10-Entry Static Table (1-Based)

| Index | Name | Index | Name |
|:---:|---|:---:|---|
| **1** | `:method` | **6** | `content-length` |
| **2** | `:path` | **7** | `connection` |
| **3** | `:status` | **8** | `user-agent` |
| **4** | `host` | **9** | `accept` |
| **5** | `content-type` | **10** | `server` |

### 4.2 Encodings
1. **Indexed Name (`0x80 | index`):** Emits `[1B tag] [2B value_len] [value bytes]`.
   - Example: `:method: GET` $\rightarrow$ `81 00 03 47 45 54` (6 bytes total).
2. **Literal Name (`0x00`):** Emits `[0x00] [2B name_len] [name bytes] [2B value_len] [value bytes]`.

---

## 5. Client & Server Lifecycle

<img width="4715" height="3035" src="https://github.com/user-attachments/assets/13bb0b68-7fd4-4d30-a89c-5596f736c6a4" />

1. **Client (`bcurl`):** Connects to `host:port`, allocates odd stream IDs ($1, 3, 5\dots$), writes `HEADERS` frame, streams body to stdout, and exits non-zero on 4xx/5xx responses.
2. **Server (`bserve`):** Listens on specified port, parses incoming frames, sanitizes paths against directory traversal, replies with `HEADERS` + `DATA`, and keeps the connection open for subsequent requests.
