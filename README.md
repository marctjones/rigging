# Rigging

Transport layer patches for the Servo browser engine.

Rigging is a **patch set** that adds extended network transport capabilities to Servo, enabling:

- **Unix Domain Sockets** - Connect to local services over UDS (Linux/macOS)
- **Named Pipes** - Connect to local services on Windows
- **Tor Support** - Anonymous connections via the Corsair daemon
- **Transport-aware URLs** - Encode transport type directly in URLs

## Overview

Rigging is NOT a standalone library. It is a collection of patches that modify Servo's network stack to support multiple transport mechanisms beyond standard TCP/HTTPS.

### Repository Relationships

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
│                  - Upstream + Rigging patches                │
│                  - transport-layer branch                    │
└──────────────────────────┬──────────────────────────────────┘
                           │ depends on
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Harbor / Compass (applications)                 │
│              - Use patched Servo for rendering               │
│              - Extended transport capabilities               │
└─────────────────────────────────────────────────────────────┘
```

## Transport URL Syntax

Rigging extends standard URLs with transport specifications:

```
http::unix///tmp/app.sock/api/data    # Unix socket (absolute path)
http::unix//var/run/app.sock          # Unix socket (relative path)
http::tcp//localhost:8080             # Explicit TCP
http::tor//example.onion              # Tor network
http::pipe//myapp                     # Windows named pipe
```

## Using Rigging

### Option 1: Use the Pre-patched Fork (Recommended)

The easiest approach is to depend on the already-patched Servo fork:

```toml
# Cargo.toml
[dependencies]
servo = { git = "https://github.com/marctjones/servo", branch = "transport-layer" }
```

### Option 2: Apply Patches to Fresh Servo

If you need to apply patches to a specific Servo version:

```bash
# Clone upstream Servo
git clone https://github.com/servo/servo.git
cd servo

# Clone Rigging and apply patches
git clone https://github.com/marctjones/rigging.git ../rigging
../rigging/apply-patches.sh .

# Build
cargo build --package servoshell
```

## Patch Contents

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

## Maintaining Patches

When the Servo fork is updated, regenerate patches:

```bash
./regenerate-patches.sh /path/to/servo-fork
git add patches/
git commit -m "Update patches from servo fork"
git push
```

## Related Projects

- **[Servo](https://github.com/servo/servo)** - The upstream browser engine
- **[marctjones/servo](https://github.com/marctjones/servo)** - Fork with Rigging patches applied
- **[Harbor](https://github.com/marctjones/harbor)** - Local desktop app framework
- **[Compass](https://github.com/marctjones/compass)** - Privacy-focused browser
- **[Corsair](https://github.com/marctjones/corsair)** - Tor daemon for Compass

## License

MPL-2.0, same as Servo.
