# Rigging Design Document

## Overview

Rigging is a transport layer abstraction library that enables Servo-based applications to communicate over various transport mechanisms including TCP, Unix Domain Sockets, Named Pipes, and Tor.

## Goals

1. **Transport Abstraction**: Unified interface for different transport types
2. **URL-Based Configuration**: Encode transport in URL for easy configuration
3. **Composability**: Chain transports (e.g., TCP over Tor)
4. **Platform Support**: Linux, macOS, and Windows
5. **Async-First**: Built on Tokio for high-performance async I/O

## Non-Goals

1. Not a full HTTP client library (use with hyper)
2. Not implementing SOCKS protocol (Tor uses binary IPC)
3. Not handling TLS (separate concern)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│              (Compass, Harbor, Servo)                       │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    Rigging Library                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ TransportUrl│  │TransportChain│  │    Connectors      │  │
│  │   Parser    │  │  Composer   │  │                     │  │
│  └─────────────┘  └─────────────┘  │  ┌───┐ ┌───┐ ┌───┐ │  │
│                                     │  │TCP│ │UDS│ │Tor│ │  │
│                                     │  └───┘ └───┘ └───┘ │  │
│                                     └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    System Layer                              │
│         TCP Sockets | Unix Sockets | Named Pipes            │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Transport Enum

```rust
pub enum Transport {
    Tcp,        // Standard TCP/IP
    Unix,       // Unix Domain Socket
    NamedPipe,  // Windows Named Pipe
    Tor,        // Tor via Corsair daemon
    Ssh,        // SSH tunnel (future)
    Quic,       // QUIC protocol (future)
}
```

### 2. TransportUrl

Extended URL format that encodes transport information:

```
scheme::transport//authority/path?query#fragment

Components:
- scheme: http, https
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

### 3. Connector Trait

```rust
#[async_trait]
pub trait Connector: Send + Sync {
    /// Connect to the target specified by the URL
    async fn connect(&self, url: &TransportUrl) -> Result<Connection, TransportError>;

    /// Check if this connector supports the given transport
    fn supports(&self, transport: Transport) -> bool;
}
```

### 4. TransportChain

Composes multiple transports:

```rust
pub struct TransportChain {
    transports: Vec<Transport>,
}

impl TransportChain {
    pub fn new() -> Self;
    pub fn push(self, transport: Transport) -> Self;
    pub fn connect(&self, url: &TransportUrl) -> Result<Connection, TransportError>;
}
```

## Transport Implementations

### TCP Connector
- Standard TCP socket connection
- Uses `tokio::net::TcpStream`
- Supports IPv4 and IPv6

### Unix Connector
- Unix Domain Socket connection
- Uses `tokio::net::UnixStream`
- Supports both abstract and filesystem sockets
- Linux and macOS only

### Named Pipe Connector (Planned)
- Windows Named Pipe connection
- Uses `tokio::net::windows::named_pipe`
- Windows only

### Tor Connector
- Connects via Corsair daemon
- Binary IPC protocol over Unix socket
- **Not SOCKS5** - uses custom protocol for efficiency

## Binary IPC Protocol (Tor)

Communication with Corsair uses length-prefixed bincode:

```
┌──────────────┬─────────────────────────────┐
│ Length (u32) │ Bincode-encoded Message     │
└──────────────┴─────────────────────────────┘
```

**Request Message:**
```rust
struct ConnectRequest {
    host: String,
    port: u16,
}
```

**Response Message:**
```rust
struct ConnectResponse {
    success: bool,
    error: Option<String>,
}
```

After successful connection, the socket becomes a bidirectional relay.

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Unsupported transport: {0:?}")]
    UnsupportedTransport(Transport),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corsair daemon not available")]
    CorsairUnavailable,
}
```

## Feature Flags

```toml
[features]
default = ["tcp", "unix"]
tcp = []
unix = []
named-pipe = []
tor = ["bincode"]
ssh = []
quic = ["quinn"]
full = ["tcp", "unix", "named-pipe", "tor", "ssh", "quic"]
```

## Thread Safety

All connectors implement `Send + Sync`:
- Connectors can be shared across threads
- Connection handles are not shared (moved to handler task)
- Use `Arc<dyn Connector>` for shared ownership

## Performance Considerations

1. **Connection Pooling**: Not implemented in Rigging (application responsibility)
2. **Buffering**: Use `BufReader`/`BufWriter` at application level
3. **Zero-Copy**: Where possible, avoid copying data

## Security Considerations

1. **Unix Socket Permissions**: Set appropriate file permissions (0600)
2. **Tor Circuit Isolation**: Each connection may use same circuit
3. **No Plaintext Secrets**: Never log sensitive data

## Future Extensions

1. **QUIC Support**: Via `quinn` crate
2. **SSH Tunneling**: Via `russh` crate
3. **mTLS**: Client certificate authentication
4. **Connection Pooling**: Optional built-in pooling
5. **Metrics**: Connection statistics and tracing
