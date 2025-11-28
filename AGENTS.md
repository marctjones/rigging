# AI Agent Development Guide for Rigging

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on the Rigging transport library.

## Project Overview

**Rigging** is a transport layer abstraction library for Servo-based applications. It provides:
- Transport-aware URL parsing (`http::unix:///tmp/app.sock/`)
- Multiple transport backends (TCP, Unix sockets, Named pipes, Tor)
- Composable transport chains
- Async I/O via Tokio

## Repository Structure

```
rigging/
├── Cargo.toml           # Package manifest with features
├── src/
│   ├── lib.rs           # Main library exports
│   ├── types.rs         # Core types (Transport enum, TransportChain)
│   ├── transport_url.rs # URL parsing for transport-aware URLs
│   ├── unix_connector.rs    # Unix Domain Socket connector
│   ├── tcp_connector.rs     # TCP connector
│   ├── tor_connector.rs     # Tor connector (via Corsair)
│   └── composed.rs      # Transport composition
├── README.md
├── DESIGN.md
└── IMPLEMENTATION_PLAN.md
```

## Coding Standards

### Rust Guidelines
- **Edition**: Rust 2021
- **Async Runtime**: Tokio
- **Error Handling**: Use `thiserror` for library errors, `anyhow` for binaries
- **Serialization**: `serde` with `bincode` for binary protocols
- **Documentation**: All public items must have doc comments

### Code Style
```rust
// Good: Descriptive error types
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

// Good: Feature-gated implementations
#[cfg(feature = "unix")]
pub mod unix_connector;

// Good: Async trait pattern
#[async_trait]
pub trait Connector: Send + Sync {
    async fn connect(&self, url: &TransportUrl) -> Result<Box<dyn AsyncReadWrite>, TransportError>;
}
```

### Testing
- Unit tests in same file as implementation
- Integration tests in `tests/` directory
- Use `#[tokio::test]` for async tests
- Test each transport independently

## Key Concepts

### Transport-Aware URLs
```
scheme::transport//path/url_path

Examples:
http::unix///tmp/app.sock/api/v1     # Unix socket with absolute path
http::unix//var/run/app.sock/        # Unix socket with relative path
http::pipe//myapp/api                # Windows named pipe
http::tor//example.onion/            # Tor hidden service
https::tcp//example.com/             # Standard TCP (explicit)
```

### Transport Chain
Transports can be composed:
```rust
// TCP through Tor
let chain = TransportChain::new()
    .push(Transport::Tor)
    .push(Transport::Tcp);
```

## Development Tasks

### Adding a New Transport

1. Create connector file: `src/{transport}_connector.rs`
2. Implement the `Connector` trait
3. Add feature flag to `Cargo.toml`
4. Export in `lib.rs` with feature gate
5. Add URL parsing support in `transport_url.rs`
6. Write unit tests
7. Update documentation

### Example Connector Implementation
```rust
// src/quic_connector.rs
use crate::{TransportError, TransportUrl};
use async_trait::async_trait;

pub struct QuicConnector {
    // connector state
}

impl QuicConnector {
    pub fn new() -> Self {
        Self { }
    }
}

#[async_trait]
impl Connector for QuicConnector {
    async fn connect(&self, url: &TransportUrl) -> Result<Connection, TransportError> {
        // Implementation
    }
}
```

## Common Commands

```bash
# Build with all features
cargo build --all-features

# Build with specific feature
cargo build --features unix

# Run tests
cargo test

# Run specific test
cargo test test_unix_connector

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Integration Points

### With Corsair (Tor)
- Rigging's `TorConnector` communicates with Corsair daemon
- Uses binary IPC protocol over Unix socket
- See `tor_connector.rs` for protocol details

### With Harbor (Local Apps)
- Harbor uses Rigging for Unix socket connections
- URL format: `http::unix///tmp/app.sock/`

### With Compass (Browser)
- Compass uses Rigging for all network connections
- Supports runtime transport switching

## Important Notes

1. **No SOCKS5**: Tor connections use binary IPC, not SOCKS5
2. **Platform Support**: Unix sockets on Linux/macOS, Named pipes on Windows
3. **Async Only**: All I/O is async via Tokio
4. **Feature Flags**: Each transport is behind a feature flag for minimal builds

## Related Projects

- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
- [Harbor](https://github.com/marctjones/harbor) - Local app framework
- [Compass](https://github.com/marctjones/compass) - Privacy browser
- [Servo](https://github.com/servo/servo) - Browser engine
