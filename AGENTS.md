# AI Agent Development Guide for Rigging

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on Rigging.

## Project Overview

**Rigging** provides two main capabilities for Servo-based applications:

1. **Stable Embedding API** - A simple, stable interface for embedding Servo browser engine
2. **Transport Layer** - Extended network transport support (Unix sockets, Named pipes, Tor)

## Repository Structure

```
rigging/
├── src/
│   ├── lib.rs                   # Main library entry point
│   ├── transport_url.rs         # Transport-aware URL parsing
│   ├── types.rs                 # Transport enum and types
│   ├── unix_connector.rs        # Unix socket connector
│   ├── tcp_connector.rs         # TCP connector
│   ├── tor_connector.rs         # Tor connector (via Corsair)
│   ├── composed.rs              # Transport composition
│   └── embed/                   # Servo embedding API
│       ├── mod.rs               # Embedding module entry
│       ├── config.rs            # BrowserConfig
│       ├── builder.rs           # BrowserBuilder
│       ├── events.rs            # BrowserEvent types
│       └── backend.rs           # Servo backend (internal)
├── patches/                     # Git patches for Servo transport layer
├── apply-patches.sh             # Apply patches to Servo
├── regenerate-patches.sh        # Regenerate patches from fork
├── README.md
├── DESIGN.md
└── Cargo.toml
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

## Key Design Principles

### Stable API Boundary

The `embed` module provides a **stable API** for embedding Servo:

- **DO NOT** change public types without version bump
- **DO NOT** expose Servo internal types
- **DO** add new fields with defaults via builder pattern
- **DO** keep all Servo-specific code in `backend.rs`

When upgrading Servo, only `src/embed/backend.rs` should need changes.

### Public API (STABLE)

These are stable and should not change:

```rust
// From src/embed/
pub struct BrowserConfig { ... }      // Configuration
pub struct BrowserBuilder { ... }     // Builder pattern entry point
pub enum BrowserEvent { ... }         // Events during operation
pub enum EmbedError { ... }           // Error types
```

### Internal Implementation (MAY CHANGE)

These may change between Servo versions:

- `src/embed/backend.rs` - Servo integration code
- Private helper functions
- Internal error handling

## Development Workflow

### Working on Embedding API

1. Make changes to `src/embed/*.rs`
2. Run tests: `cargo test`
3. Build: `cargo build`

### Working on Transport Layer

1. **For Servo patches**: Work in the Servo fork (marctjones/servo)
   ```bash
   cd /path/to/servo-fork
   git checkout transport-layer
   # Make changes to components/net/
   git commit -m "Description"
   ```

2. **Regenerate patches**:
   ```bash
   ./regenerate-patches.sh /path/to/servo-fork
   git add patches/
   git commit -m "Update patches"
   ```

3. **For Rigging transport code**: Work directly in `src/`
   ```bash
   # Edit src/unix_connector.rs, etc.
   cargo test
   cargo build
   ```

## Transport-Aware URLs

```
scheme::transport//path/url_path

Examples:
http::unix///tmp/app.sock/api/v1     # Unix socket (absolute path)
http::unix//var/run/app.sock/        # Unix socket (relative path)
http::pipe//myapp/api                # Windows named pipe
http::tor//example.onion/            # Tor hidden service
https::tcp//example.com/             # Standard TCP (explicit)
```

## Common Commands

```bash
# Build
cargo build

# Run tests
cargo test

# Check (faster than build)
cargo check

# Apply patches to Servo
./apply-patches.sh /path/to/servo

# Regenerate patches from fork
./regenerate-patches.sh /path/to/servo-fork
```

## Adding a New Feature

### To Embedding API

1. Add field to `BrowserConfig` with default value
2. Add builder method in `config.rs`
3. Update `backend.rs` to use new field
4. Add tests
5. Update documentation

### To Transport Layer

1. Add transport variant in `types.rs` if needed
2. Create connector in `src/{transport}_connector.rs`
3. Update `composed.rs` for composition support
4. Add tests
5. If patching Servo, regenerate patches

## Integration Points

### With Corsair (Tor)

- `TorConnector` communicates with Corsair daemon
- Uses binary IPC protocol over Unix socket
- Protocol: length-prefixed bincode messages

### With Harbor (Local Apps)

- Harbor depends on Rigging for browser embedding
- Uses `BrowserBuilder` to create windows
- URL format: `http::unix///tmp/app.sock/`

### With Compass (Browser)

- Compass depends on Rigging for browser embedding
- Full browser UI with Tor support
- Uses Corsair for Tor connections

## Important Notes

1. **Stable API Contract**: Never break `BrowserConfig`, `BrowserBuilder`, or `BrowserEvent`
2. **Servo Isolation**: All Servo types stay in `backend.rs`
3. **Patch Maintenance**: When updating Servo fork, regenerate patches
4. **No SOCKS5**: Tor uses binary IPC, not SOCKS5

## Related Projects

- [Servo](https://github.com/servo/servo) - Upstream browser engine
- [marctjones/servo](https://github.com/marctjones/servo) - Fork with transport patches
- [Harbor](https://github.com/marctjones/harbor) - Local app framework
- [Compass](https://github.com/marctjones/compass) - Privacy browser
- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
