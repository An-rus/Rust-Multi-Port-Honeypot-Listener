# Rust Multi-Port Honeypot Listener

A lightweight, async TCP honeypot built with **Tokio** that listens on common service ports, captures incoming traffic, and logs forensic data for every connection attempt. Designed as the core capture layer for a larger threat-detection pipeline.

## Overview

The honeypot binds to a set of well-known ports and impersonates the services typically found there. Any inbound connection is intercepted, its raw payload is captured, and a structured **forensic event** is written to disk in JSON Lines format for downstream analysis (e.g. by a threat detector or SIEM).

## Features

- **Async, non-blocking I/O** via Tokio — each port and each connection runs on its own spawned task.
- **Protocol-aware handling** — dedicated handlers for SSH and HTTP traffic (see `services::ssh` / `services::http`), with a generic fallback for other ports.
- **Forensic event logging** — every captured connection is serialized as JSON with:
  - Timestamp (UTC)
  - Source and destination IP/port
  - Base64-encoded raw payload
  - Detected protocol
  - Unique session ID (UUID v4)
- **Append-only JSON log** (`events.json`) for easy ingestion by other tools.
- **Persistent listeners** — each port binds once and then loops on `accept()`, so multiple connections (sequential or concurrent) are all captured, not just the first one.
- **Graceful error handling** — bind, accept, read, and save failures are logged to stderr instead of panicking the process.
- **Graceful shutdown** on `Ctrl+C`.

## Monitored Ports

| Port | Protocol | Handling             |
|------|----------|-----------------------|
| 22   | SSH      | `services::ssh` handler |
| 80   | HTTP     | `services::http` handler |
| 21   | FTP      | Generic capture        |
| 3306 | MySQL    | Generic capture        |

Unrecognized ports fall back to generic capture, reading up to 4096 bytes from the socket and logging the payload with protocol `"Unknown"`.

## Project Structure

```
.
├── src/
│   ├── main.rs           # Listener setup, event model, port dispatch
│   └── services/
│       ├── ssh.rs        # SSH-specific connection handler
│       └── http.rs       # HTTP-specific connection handler
└── events.json           # Generated at runtime — forensic event log
```

## Dependencies

- [`tokio`](https://crates.io/crates/tokio) — async runtime and networking
- [`serde`](https://crates.io/crates/serde) / `serde_json` — serialization
- [`chrono`](https://crates.io/crates/chrono) — UTC timestamps
- [`base64`](https://crates.io/crates/base64) — payload encoding
- [`uuid`](https://crates.io/crates/uuid) — session ID generation

## Building & Running

```bash
cargo build --release
sudo ./target/release/<binary_name>
```

> **Note:** Ports below 1024 (22, 80, 21) require elevated privileges on most systems (`sudo` on Linux/macOS). On macOS specifically, binding to privileged ports may require additional configuration.

The honeypot will run until interrupted with `Ctrl+C`, at which point it shuts down gracefully and prints `[-] Honeypot stopped`.

## Sample Forensic Event

```json
{
  "timestamp": "2026-07-13T14:32:10.512Z",
  "source_ip": "192.168.1.42",
  "dest_ip": "0.0.0.0:3306",
  "source_port": 51422,
  "dest_port": 3306,
  "payload_b64": "SGVsbG8gd29ybGQ=",
  "detected_protocol": "MySQL",
  "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

## Known Limitations

- No rate-limiting or connection timeout is currently implemented, leaving the generic handler's `read()` call to wait indefinitely if a client connects but sends no data.
- Bind/accept/read/save errors are logged via `eprintln!` rather than a structured logger, so long-term monitoring would benefit from proper log tooling.

## Roadmap Ideas

- Add configurable port list via CLI args or config file.
- Feed `events.json` into the companion real-time threat detector / alerting system.
- Add per-connection read timeouts to avoid indefinitely blocked tasks.
- Add structured logging (e.g. `tracing`) instead of `println!`/`eprintln!`.
