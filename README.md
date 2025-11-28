# Rigging 🪢

Transport layer library for Servo-based applications.

Rigging provides transport abstractions for connecting to web servers over various protocols:

- **TCP** - Standard TCP/IP connections
- **Unix Domain Sockets** - Local IPC on Linux/macOS
- **Named Pipes** - Local IPC on Windows (planned)
- **Tor** - Anonymous connections via SOCKS5 proxy

## Transport URL Syntax

Rigging extends standard URLs with explicit transport specifications:

```
http://example.com/                     # Standard TCP (implicit)
http::unix///tmp/app.sock/api/data      # Unix socket (absolute path)
http::unix//var/run/app.sock            # Unix socket (relative path)
http::tcp//localhost:8080               # Explicit TCP
http::tor//example.onion                # Tor network
```

## Usage

### Basic URL Parsing

```rust
use rigging::{TransportUrl, Transport};

// Parse a transport-aware URL
let url = TransportUrl::parse("http::unix///tmp/app.sock/api")?;

assert_eq!(url.transport(), Transport::Unix);
assert_eq!(url.unix_socket_path(), Some("/tmp/app.sock"));
assert_eq!(url.path(), "/api");
```

### Unix Socket Connection

```rust
use rigging::UnixConnector;

let connector = UnixConnector::new("/tmp/app.sock");
let connection = connector.connect().await?;
// Use with hyper client...
```

### Tor Connection (via Corsair daemon)

```rust
use rigging::tor_connector::TorConnector;

let connector = TorConnector::new(); // Uses default socket path
let connection = connector.connect("example.onion", 80).await?;
```

### Composed Connector

```rust
use rigging::composed::ComposedConnector;

let connector = ComposedConnector::new();

// Automatically routes based on URL transport
let conn = connector.connect("http::unix///tmp/app.sock/api").await?;
let conn = connector.connect("http://example.com/").await?;
let conn = connector.connect("http://example.onion/").await?;  // Auto-Tor
```

## Features

- `unix` (default) - Unix domain socket support
- `tcp` (default) - TCP/IP support
- `tor` - Tor network support (requires Corsair daemon)
- `named-pipe` - Windows named pipe support (planned)

## Integration with Servo Projects

Rigging is used by:

- **Compass** - Privacy-focused web browser
- **Harbor** - Local app framework for UDS/named pipe access

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Compass   │     │   Harbor    │     │   Other     │
│  (browser)  │     │ (app frame) │     │   Apps      │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │   Rigging   │
                    │ (transport) │
                    └──────┬──────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
┌──────▼──────┐     ┌──────▼──────┐     ┌──────▼──────┐
│    Unix     │     │    TCP      │     │    Tor      │
│   Sockets   │     │   (direct)  │     │  (Corsair)  │
└─────────────┘     └─────────────┘     └─────────────┘
```

## License

Mozilla Public License 2.0 (MPL-2.0)

## Related Projects

- [Compass](https://github.com/marctjones/compass) - Privacy-focused browser
- [Harbor](https://github.com/marctjones/harbor) - Local app framework
- [Corsair](https://github.com/marctjones/corsair) - Tor proxy daemon
