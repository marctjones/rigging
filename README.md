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

Rigging extends standard URLs with transport specifications:

```
http::unix///tmp/app.sock/api/data    # Unix socket (absolute path)
http::unix//var/run/app.sock          # Unix socket (relative path)
http::tcp//localhost:8080             # Explicit TCP
http::tor//example.onion              # Tor network
http::pipe//myapp                     # Windows named pipe
```

### Transport URL Example

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

## Patch Contents

Rigging also provides patches for adding transport support to Servo:

| Patch | Description |
|-------|-------------|
| `0001-transport-url.patch` | TransportUrl parsing with scheme::transport syntax |
| `0002-unix-connector.patch` | Unix Domain Socket connector for hyper |
| `0003-transport-types.patch` | Transport enum and TransportChain types |
| `0004-http-loader.patch` | HTTP dispatch modifications for multi-transport |
| `0005-net-lib.patch` | Module exports for transport code |
| `0006-net-cargo.patch` | Dependencies (hyperlocal, etc.) |
| `0007-shared-net-lib.patch` | Shared transport type exports |
| `0008-tor-connector.patch` | Tor connector via Corsair IPC |

## Related Projects

- **[Servo](https://github.com/servo/servo)** - The upstream browser engine
- **[marctjones/servo](https://github.com/marctjones/servo)** - Fork with Rigging patches applied
- **[Harbor](https://github.com/marctjones/harbor)** - Local desktop app framework
- **[Compass](https://github.com/marctjones/compass)** - Privacy-focused browser
- **[Corsair](https://github.com/marctjones/corsair)** - Tor daemon for Compass

## License

MPL-2.0, same as Servo.
