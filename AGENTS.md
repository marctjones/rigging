# AI Agent Development Guide for Rigging

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on the Rigging transport layer patches.

## Project Overview

**Rigging** is a patch set that adds extended network transport capabilities to Servo's network stack. It enables:
- Transport-aware URL parsing (`http::unix:///tmp/app.sock/`)
- Multiple transport backends (TCP, Unix sockets, Named pipes, Tor)
- Composable transport chains
- Async I/O via Tokio

**Important**: Rigging is NOT a standalone Rust library. It is a collection of patches applied to Servo.

## Repository Structure

```
rigging/
├── patches/                 # Git patches for Servo
│   ├── 0001-transport-url.patch      # TransportUrl parsing
│   ├── 0002-unix-connector.patch     # Unix socket connector
│   ├── 0003-transport-types.patch    # Transport enum and types
│   ├── 0004-http-loader.patch        # HTTP dispatch changes
│   ├── 0005-net-lib.patch            # Module exports
│   ├── 0006-net-cargo.patch          # Dependencies
│   ├── 0007-shared-net-lib.patch     # Shared type exports
│   └── 0008-tor-connector.patch      # Tor connector
├── apply-patches.sh         # Apply patches to Servo
├── regenerate-patches.sh    # Regenerate patches from fork
├── src/                     # Reference implementation (legacy)
├── README.md
├── DESIGN.md
└── IMPLEMENTATION_PLAN.md
```

## Architecture

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

## Development Workflow

### Making Changes to Transport Layer

1. **Work in the Servo fork** (marctjones/servo):
   ```bash
   cd /path/to/servo-fork
   git checkout transport-layer
   # Make changes to components/net/
   git commit -m "Description of change"
   ```

2. **Regenerate patches**:
   ```bash
   cd /path/to/rigging
   ./regenerate-patches.sh /path/to/servo-fork
   git add patches/
   git commit -m "Update patches"
   git push
   ```

3. **Push servo fork changes**:
   ```bash
   cd /path/to/servo-fork
   git push origin transport-layer
   ```

### Applying Patches to Fresh Servo

```bash
git clone https://github.com/servo/servo.git
./apply-patches.sh ./servo
cd servo && cargo build --package servoshell
```

## Key Files in Servo (After Patching)

| File | Purpose |
|------|---------|
| `components/net/transport_url.rs` | Transport-aware URL parsing |
| `components/net/unix_connector.rs` | Unix Domain Socket connector |
| `components/net/tor_connector.rs` | Tor connector via Corsair IPC |
| `components/net/http_loader.rs` | Modified for multi-transport dispatch |
| `components/shared/net/transport.rs` | Transport enum and TransportChain |

## Transport-Aware URLs

```
scheme::transport//path/url_path

Examples:
http::unix///tmp/app.sock/api/v1     # Unix socket with absolute path
http::unix//var/run/app.sock/        # Unix socket with relative path
http::pipe//myapp/api                # Windows named pipe
http::tor//example.onion/            # Tor hidden service
https::tcp//example.com/             # Standard TCP (explicit)
```

## Adding a New Transport

1. **Create connector in Servo fork**:
   - Add `components/net/{transport}_connector.rs`
   - Implement connector following `unix_connector.rs` pattern

2. **Modify http_loader.rs**:
   - Add dispatch logic for new transport type

3. **Update shared types**:
   - Add variant to `Transport` enum in `components/shared/net/transport.rs`

4. **Regenerate patches**:
   ```bash
   ./regenerate-patches.sh /path/to/servo-fork
   ```

## Common Commands

```bash
# Apply patches to Servo
./apply-patches.sh /path/to/servo

# Regenerate patches from fork
./regenerate-patches.sh /path/to/servo-fork

# Build patched Servo
cd /path/to/servo && cargo build --package servoshell

# Run tests for net component
cd /path/to/servo && cargo test --package net
```

## Integration Points

### With Corsair (Tor)
- Rigging's `TorConnector` communicates with Corsair daemon
- Uses binary IPC protocol over Unix socket
- Protocol: length-prefixed bincode messages

### With Harbor (Local Apps)
- Applications use patched Servo's Unix socket support
- URL format: `http::unix///tmp/app.sock/`

### With Compass (Browser)
- Uses patched Servo for all network connections
- Supports runtime transport switching via Corsair

## Important Notes

1. **Patch-based Development**: Always work in the Servo fork, then regenerate patches
2. **No SOCKS5**: Tor connections use binary IPC, not SOCKS5
3. **Platform Support**: Unix sockets on Linux/macOS, Named pipes on Windows
4. **Upstream Tracking**: Periodically rebase fork on upstream Servo

## Related Projects

- [Servo](https://github.com/servo/servo) - Upstream browser engine
- [marctjones/servo](https://github.com/marctjones/servo) - Fork with Rigging patches
- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
- [Harbor](https://github.com/marctjones/harbor) - Local app framework
- [Compass](https://github.com/marctjones/compass) - Privacy browser
