# Annotated Byte-by-Byte Frame Hexdump

This document provides a byte-level trace and field annotation for a complete bHTTP request and response cycle, captured using `bcurl -v localhost:9000/index.html`.

---

## 1. Outgoing Request (`bcurl` -> `bserve`)

### Frame: `HEADERS` (Stream ID: 1, Length: 37 Bytes)

```text
Wire Bytes:
00 00 25 01 05 00 00 00 01 81 00 03 47 45 54 82
00 0B 2F 69 6E 64 65 78 2E 68 74 6D 6C 84 00 0E
6C 6F 63 61 6C 68 6F 73 74 3A 39 30 30 30
```

### Detailed Byte Breakdown:

| Offset | Hex Bytes | Value / Field | Interpretation & Semantic Meaning |
|---|---|---|---|
| `0x0000` | `00 00 25` | Length = 37 | **24-bit Payload Length:** Exactly 37 payload bytes follow the header. |
| `0x0003` | `01` | Type = 0x01 | **Frame Type:** `HEADERS` frame. |
| `0x0004` | `05` | Flags = 0x05 | **Bitflags:** `END_STREAM (0x01) \| END_HEADERS (0x04)`. Header block is complete and no `DATA` frame follows. |
| `0x0005` | `00 00 00 01`| Stream ID = 1 | **31-bit Stream Identifier:** Client stream 1 (odd number). Reserved bit is 0. |
| `0x0009` | `81` | Static Index 1 | **HPACK Tag:** `0x80 \| 1` $\rightarrow$ Name is `:method`. |
| `0x000A` | `00 03` | Val Length = 3 | **16-bit Value Length:** Value string has length 3. |
| `0x000C` | `47 45 54` | "GET" | **Value Bytes:** ASCII string `"GET"`. |
| `0x000F` | `82` | Static Index 2 | **HPACK Tag:** `0x80 \| 2` $\rightarrow$ Name is `:path`. |
| `0x0010` | `00 0B` | Val Length = 11 | **16-bit Value Length:** Value string has length 11. |
| `0x0012` | `2F 69 ... 6C` | "/index.html" | **Value Bytes:** ASCII string `"/index.html"`. |
| `0x001D` | `84` | Static Index 4 | **HPACK Tag:** `0x80 \| 4` $\rightarrow$ Name is `host`. |
| `0x001E` | `00 0E` | Val Length = 14 | **16-bit Value Length:** Value string has length 14. |
| `0x0020` | `6C 6F ... 30` | "localhost:9000"| **Value Bytes:** ASCII string `"localhost:9000"`. |

---

## 2. Incoming Response (`bserve` -> `bcurl`)

### Frame 1: `HEADERS` (Stream ID: 1, Length: 52 Bytes)

```text
Wire Bytes:
00 00 34 01 04 00 00 00 01 83 00 03 32 30 30 85
00 18 74 65 78 74 2F 68 74 6D 6C 3B 20 63 68 61
72 73 65 74 3D 75 74 66 2D 38 86 00 03 33 30 36
8A 00 0A 62 73 65 72 76 65 2F 31 2E 30
```

### Detailed Byte Breakdown:

| Offset | Hex Bytes | Value / Field | Interpretation & Semantic Meaning |
|---|---|---|---|
| `0x0000` | `00 00 34` | Length = 52 | **24-bit Payload Length:** 52 bytes of header data. |
| `0x0003` | `01` | Type = 0x01 | **Frame Type:** `HEADERS` frame. |
| `0x0004` | `04` | Flags = 0x04 | **Bitflags:** `END_HEADERS (0x04)`. More frames follow on this stream (`DATA`). |
| `0x0005` | `00 00 00 01`| Stream ID = 1 | **Stream Identifier:** Matches client request stream 1. |
| `0x0009` | `83` | Static Index 3 | **HPACK Tag:** `0x80 \| 3` $\rightarrow$ Name is `:status`. |
| `0x000A` | `00 03` | Val Length = 3 | **16-bit Value Length:** 3 bytes. |
| `0x000C` | `32 30 30` | "200" | **Value Bytes:** Status code `"200"` (`200 OK`). |
| `0x000F` | `85` | Static Index 5 | **HPACK Tag:** `0x80 \| 5` $\rightarrow$ Name is `content-type`. |
| `0x0010` | `00 18` | Val Length = 24 | **16-bit Value Length:** 24 bytes. |
| `0x0012` | `74 65 ... 38` | "text/html; charset=utf-8" | **Value Bytes:** MIME type specification. |
| `0x002A` | `86` | Static Index 6 | **HPACK Tag:** `0x80 \| 6` $\rightarrow$ Name is `content-length`. |
| `0x002B` | `00 03` | Val Length = 3 | **16-bit Value Length:** 3 bytes. |
| `0x002D` | `33 30 36` | "306" | **Value Bytes:** Body size is 306 bytes. |
| `0x0030` | `8A` | Static Index 10| **HPACK Tag:** `0x80 \| 10` $\rightarrow$ Name is `server`. |
| `0x0031` | `00 0A` | Val Length = 10 | **16-bit Value Length:** 10 bytes. |
| `0x0033` | `62 73 ... 30` | "bserve/1.0" | **Value Bytes:** Server implementation identifier. |

---

### Frame 2: `DATA` (Stream ID: 1, Length: 306 Bytes)

```text
Header:
00 01 32 02 01 00 00 00 01
Payload (First 32 bytes):
3C 21 44 4F 43 54 59 50 45 20 68 74 6D 6C 3E 0A  |<!DOCTYPE html>.|
3C 68 74 6D 6C 20 6C 61 6E 67 3D 22 65 6E 22 3E  |<html lang="en">|
... (remaining HTML bytes up to offset 0x0132) ...
```

### Detailed Byte Breakdown:

| Offset | Hex Bytes | Value / Field | Interpretation & Semantic Meaning |
|---|---|---|---|
| `0x0000` | `00 01 32` | Length = 306 | **24-bit Payload Length:** `0x0132` = 306 bytes. |
| `0x0003` | `02` | Type = 0x02 | **Frame Type:** `DATA` frame. |
| `0x0004` | `01` | Flags = 0x01 | **Bitflags:** `END_STREAM (0x01)`. Concludes stream 1. |
| `0x0005` | `00 00 00 01`| Stream ID = 1 | **Stream Identifier:** Matches client stream 1. |
| `0x0009` | `3C 21 ...` | HTML Content | **Payload Bytes:** Exactly 306 bytes of raw HTML file data. |
