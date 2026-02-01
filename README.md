# Rigging

Transport layer and stable embedding API for the Servo browser engine.

Rigging provides two main capabilities:

1. **Stable Embedding API** - A simple, stable interface for embedding Servo in applications
2. **Transport Layer** - Extended network transport support (Unix sockets, Tor, etc.)

## Stable Embedding API

Rigging provides a stable API layer that isolates applications from Servo's internal APIs. This makes it easier to upgrade Servo versions without rewriting your application code.

```rust
use rigging::embed::{BrowserBuilder, BrowserConfig};

// Simple usage
BrowserBuilder::new()
    .url("http::unix///tmp/app.sock/")
    .title("My App")
    .size(1200, 800)
    .run()?;

// With full configuration
let config = BrowserConfig::new("http://localhost/")
    .with_title("My App")
    .with_size(1200, 800)
    .with_devtools(true);

BrowserBuilder::new()
    .config(config)
    .on_event(|event| println!("Event: {:?}", event))
    .run()?;
```

### API Stability

The embedding API is designed for stability:
- `BrowserConfig` fields are stable; new fields have defaults
- `BrowserBuilder` methods are stable
- `BrowserEvent` variants are stable; new events may be added

When upgrading Servo, only the internal `backend.rs` implementation needs changes.

## Transport Layer

Rigging extends standard URLs with transport specifications.

### Supported Transports

| Transport | URL Syntax | Status | Platform | Description |
|-----------|------------|--------|----------|-------------|
| **TCP** | `http::tcp//host:port/` | ✅ Implemented | All | Standard TCP/IP (default) |
| **Unix** | `http::unix///path.sock/` | ✅ Implemented | Linux, macOS | Unix Domain Sockets |
| **Named Pipe** | `http::pipe//name/` | 🚧 Planned | Windows | Windows Named Pipes |
| **Tor** | `http::tor//host/` | 🚧 Planned | All | Tor network via Corsair daemon |
| **SSH** | `http::ssh//host/` | 📋 Future | All | SSH tunneling via `russh` |
| **QUIC** | `http::quic//host/` | 📋 Future | All | QUIC/HTTP3 via `quinn` |

### Transport URL Examples

```
http::unix///tmp/app.sock/api/data    # Unix socket (absolute path)
http::unix//var/run/app.sock          # Unix socket (relative path)
http::tcp//localhost:8080             # Explicit TCP
http::tor//example.onion              # Tor hidden service
http::pipe//myapp                     # Windows named pipe
http::ssh//user@host:22/              # SSH tunnel (future)
http::quic//host:443/                 # QUIC/HTTP3 (future)
```

### Transport Chaining (Future)

Rigging will support chaining transports for composed connections:

```
http::tor+unix///tmp/app.sock/        # Tor over Unix socket
http::ssh+tcp//host/                  # SSH tunnel over TCP
```

### Transport URL Parsing Example

```rust
use rigging::{TransportUrl, Transport};

let url = TransportUrl::parse("http::unix///tmp/app.sock/api")?;
assert_eq!(url.transport(), Transport::Unix);
assert_eq!(url.unix_socket_path(), Some("/tmp/app.sock"));
```

## Repository Relationships

```
┌─────────────────────────────────────────────────────────────┐
│                    servo/servo (upstream)                    │
│                    - Base browser engine                     │
│                    - TCP/HTTPS only                          │
└──────────────────────────┬──────────────────────────────────┘
                           │ patches from Rigging
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  marctjones/servo (fork)                     │
│                  - Upstream + transport patches              │
│                  - transport-layer branch                    │
└──────────────────────────┬──────────────────────────────────┘
                           │ embedded via
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                       Rigging                                │
│                  - Stable embedding API                      │
│                  - Transport URL parsing                     │
│                  - Servo backend integration                 │
└──────────────────────────┬──────────────────────────────────┘
                           │ used by
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Harbor / Compass (applications)                 │
│              - Single dependency on Rigging                  │
│              - Stable API contract                           │
└─────────────────────────────────────────────────────────────┘
```

## Feature Flags

- `unix` - Unix Domain Socket support (default)
- `tcp` - TCP transport support (default)
- `tor` - Tor transport via Corsair daemon
- `named-pipe` - Windows Named Pipe support
- `servo` - Enable embedded Servo browser engine

## Usage

### As Embedding API (Recommended)

Add Rigging as a dependency:

```toml
[dependencies]
rigging = { git = "https://github.com/marctjones/rigging", features = ["unix"] }
```

Use the embedding API in your application:

```rust
use rigging::embed::{BrowserBuilder, BrowserConfig};

fn main() -> Result<(), rigging::EmbedError> {
    BrowserBuilder::new()
        .url("http://localhost:8080/")
        .title("My Application")
        .run()
}
```

### For Transport URL Parsing Only

If you only need transport URL parsing without browser embedding:

```rust
use rigging::{TransportUrl, Transport};

let url = TransportUrl::parse("http::unix///tmp/app.sock/api")?;
println!("Transport: {:?}", url.transport());
println!("Socket: {:?}", url.unix_socket_path());
```

## Current Status

**Phase 1 (Issue #48): ✅ COMPLETE**
- All 9 Servo patches apply cleanly to current Servo main (commit c0583492d60, Jan 27 2026)
- ServoBuilder::with_connector() API implemented
- Patched Servo compiles successfully
- See [PATCHES_STATUS.md](./PATCHES_STATUS.md) for details

**Phase 2 (Issues #45, #49-51): ✅ COMPLETE**
- ✅ Servoshell embedding code imported (src/servoshell/)
- ✅ Import visibility issue solved (servo:: not libservo::)
- ✅ All dependencies loading correctly
- ✅ Servoshell updated to current Servo API
- ✅ **Rigging compiles successfully with servo feature!**
- Note: Headless mode working, headed mode stubbed (will extract from servoshell later)
- WebDriver support temporarily disabled (will port later if needed)

**Phase 3: IN PROGRESS**
- Next: Test window creation, integrate ComposedConnector

## Servo Patches

Rigging provides patches for adding transport support to Servo:

| Patch | Description | Status |
|-------|-------------|--------|
| `0001-transport-url.patch` | TransportUrl parsing with scheme::transport syntax | ✅ Applies |
| `0002-unix-connector.patch` | Unix Domain Socket connector for hyper | ✅ Applies |
| `0003-transport-types.patch` | Transport enum and TransportChain types | ✅ Applies |
| `0005-net-lib.patch` | Module exports for transport code | ✅ Applies |
| `0006-net-cargo.patch` | Dependencies (tokio-socks, hyperlocal, aws-lc-rs) | ✅ Applies |
| `0007-shared-net-lib.patch` | Shared transport type exports | ✅ Applies |
| `0008-tor-connector.patch` | Tor connector via Corsair IPC | ✅ Applies |
| `0009-connector-injection.patch` | ServoBuilder::with_connector() API + Unpin fix | ✅ Applies |
| `0010-connector-aws-lc-rs-fix.patch` | Fix aws-lc-rs import structure | ✅ Applies |
| `0004-http-loader.patch` | HTTP dispatch modifications | ⏸️ Optional (deferred) |

**All patches target Servo main branch** (commit c0583492d60, Jan 27 2026)

## Related Projects

- **[Servo](https://github.com/servo/servo)** - The upstream browser engine
- **[marctjones/servo](https://github.com/marctjones/servo)** - Fork with Rigging patches applied
- **[Harbor](https://github.com/marctjones/harbor)** - Local desktop app framework
- **[Compass](https://github.com/marctjones/compass)** - Privacy-focused browser
- **[Corsair](https://github.com/marctjones/corsair)** - Tor daemon for Compass

## License

MPL-2.0, same as Servo.
