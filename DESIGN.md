# Rigging Design Document

## Overview

Rigging is a **patch set** for the Servo browser engine that adds extended network transport capabilities. It modifies Servo's network stack to support Unix Domain Sockets, Named Pipes, Tor, and other transport mechanisms beyond standard TCP/HTTPS.

## Goals

1. **Transport Abstraction**: Enable Servo to connect over multiple transport types
2. **URL-Based Configuration**: Encode transport in URL for easy configuration
3. **Composability**: Support chaining transports (e.g., TCP over Tor)
4. **Platform Support**: Linux, macOS, and Windows
5. **Async-First**: Maintain Servo's async I/O patterns via Tokio

## Non-Goals

1. Not a standalone library (patches applied to Servo)
2. Not implementing SOCKS5 protocol (Tor uses binary IPC via Corsair)
3. Not a general-purpose HTTP client

## Architecture

Rigging patches modify these Servo components:

```
┌─────────────────────────────────────────────────────────────┐
│                    Servo Browser Engine                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  components/net/                                             │
│  ├── transport_url.rs      [NEW] Transport-aware URL parser │
│  ├── unix_connector.rs     [NEW] Unix socket connector      │
│  ├── tor_connector.rs      [NEW] Tor via Corsair IPC        │
│  ├── http_loader.rs        [MOD] Multi-transport dispatch   │
│  ├── lib.rs                [MOD] Export new modules         │
│  └── Cargo.toml            [MOD] hyperlocal dependency      │
│                                                              │
│  components/shared/net/                                      │
│  ├── transport.rs          [NEW] Transport enum & chain     │
│  └── lib.rs                [MOD] Export transport types     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Patch Set Contents

| Patch | Files Modified | Purpose |
|-------|----------------|---------|
| 0001-transport-url | transport_url.rs | URL parsing with `scheme::transport//` syntax |
| 0002-unix-connector | unix_connector.rs | hyper connector for Unix sockets |
| 0003-transport-types | transport.rs | `Transport` enum, `TransportChain` |
| 0004-http-loader | http_loader.rs | Dispatch requests by transport type |
| 0005-net-lib | lib.rs | Export transport modules |
| 0006-net-cargo | Cargo.toml | Add hyperlocal, bincode deps |
| 0007-shared-net-lib | shared/net/lib.rs | Export shared types |
| 0008-tor-connector | tor_connector.rs | Corsair IPC protocol |

## Transport URL Syntax

Extended URL format encoding transport information:

```
scheme::transport//authority/path?query#fragment

Components:
- scheme: http, https, ws, wss
- transport: tcp, unix, pipe, tor
- authority: host:port or socket path
- path: URL path
- query: URL query parameters
- fragment: URL fragment
```

**Parsing Rules:**
- `http://example.com/` → TCP (implicit)
- `http::tcp//example.com/` → TCP (explicit)
- `http::unix///tmp/app.sock/` → Unix socket (absolute path, 3 slashes)
- `http::unix//var/run/app.sock/` → Unix socket (relative path, 2 slashes)
- `http::pipe//myapp/` → Windows named pipe
- `http::tor//example.onion/` → Tor hidden service
- `http://example.onion/` → Auto-detect Tor for .onion

## Transport Implementations

### TCP Connector (Existing Servo)
- Standard TCP socket connection via hyper
- Unmodified from upstream Servo

### Unix Connector (Rigging Addition)
- Unix Domain Socket connection
- Uses `hyperlocal` crate for hyper integration
- Linux and macOS only
- Socket path extracted from URL

### Named Pipe Connector (Planned)
- Windows Named Pipe connection
- Similar pattern to Unix connector
- Windows only

### Tor Connector (Rigging Addition)
- Connects via Corsair daemon
- Binary IPC protocol over Unix socket
- **Not SOCKS5** - uses custom protocol for efficiency

## Binary IPC Protocol (Tor/Corsair)

Communication with Corsair uses length-prefixed bincode:

```
┌──────────────┬─────────────────────────────┐
│ Length (u32) │ Bincode-encoded Message     │
└──────────────┴─────────────────────────────┘
```

**Request Types:**
```rust
enum CorsairRequest {
    Connect { host: String, port: u16 },
    NewIdentity,
    GetStatus,
}
```

**Response Types:**
```rust
enum CorsairResponse {
    Connected,
    Error { message: String },
    Status { bootstrap: u8, circuits: u32 },
}
```

After successful connection, the socket becomes a bidirectional relay.

## HTTP Dispatch Flow

Modified `http_loader.rs` dispatch logic:

```rust
fn dispatch_request(url: &TransportUrl) -> Result<Response> {
    match url.transport() {
        Transport::Tcp => standard_tcp_fetch(url),
        Transport::Unix => unix_socket_fetch(url),
        Transport::Tor => tor_fetch_via_corsair(url),
        Transport::NamedPipe => named_pipe_fetch(url),
    }
}
```

## Development Workflow

1. **Work in Servo fork** (marctjones/servo transport-layer branch)
2. **Make changes** to components/net/
3. **Commit** with descriptive message
4. **Regenerate patches** using `regenerate-patches.sh`
5. **Commit patches** to Rigging repository
6. **Push both** repos

## Upstream Tracking

Rigging patches must be rebased when upstream Servo changes:

1. Fetch upstream Servo changes
2. Rebase transport-layer branch
3. Resolve conflicts (usually in http_loader.rs)
4. Regenerate patches
5. Update Rigging repository

## Security Considerations

1. **Unix Socket Permissions**: Applications should set appropriate file permissions (0600)
2. **Tor Circuit Isolation**: Corsair manages circuit isolation
3. **No Plaintext Secrets**: Never log sensitive data in transport code
4. **Path Validation**: Unix socket paths are validated before connection

## Future Extensions

1. **QUIC Support**: Via `quinn` crate
2. **SSH Tunneling**: Via `russh` crate
3. **Named Pipe Completion**: Full Windows support
4. **Connection Pooling**: Per-transport connection pools
