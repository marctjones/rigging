# AI Agent Development Guide for Rigging

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on Rigging.

**IMPORTANT**: Read this ENTIRE document before writing any code. Pay special attention to the "Common Mistakes to Avoid" section.

## Before Starting Any Work

**ALWAYS read `IMPLEMENTATION_PLAN.md` first** to understand:
- Current project status (what's complete, what's in progress)
- What phases are blocked and why
- The specific next tasks to work on
- Detailed step-by-step implementation plans

The implementation plan has checkboxes showing exactly where we left off.

## Project Overview

**Rigging is a fork of servoshell's core embedding code**, stripped of browser chrome, with a pluggable networking layer added.

This is critical to understand:
- **servoshell** = Servo's reference shell (browser UI + embedding code)
- **Rigging** = servoshell's embedding code only (NO browser UI) + pluggable Connector trait
- **Harbor** = Rigging + backend management + UDS-only connector (Electron alternative)
- **Compass** = Rigging + browser UI + Tor connector (privacy browser)

## What Rigging IS vs IS NOT

| Rigging IS | Rigging IS NOT |
|------------|----------------|
| A fork of servoshell's core | A wrapper around Servo |
| Embedding code only | A browser with UI |
| Pluggable networking (Connector trait) | Hardcoded to TCP or UDS |
| Shared by Harbor and Compass | Specific to one application |
| Tracking upstream Servo | A complete Servo fork |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATIONS                                  │
├─────────────────────────────┬───────────────────────────────────┤
│         HARBOR              │           COMPASS                  │
│  - No browser chrome        │  - Full browser chrome             │
│  - Backend management       │  - Tor integration                 │
│  - UdsConnector only        │  - TorConnector + TcpConnector     │
│  - app.toml config          │  - Privacy features                │
└─────────────────────────────┴───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RIGGING                                       │
│            (forked from servoshell core)                         │
├─────────────────────────────────────────────────────────────────┤
│  FROM SERVOSHELL (kept):           STRIPPED (removed):          │
│  - headed_window.rs (window)       - Toolbar/URL bar            │
│  - webview.rs (Servo integration)  - Tabs                        │
│  - app.rs (event loop)             - Bookmarks                   │
│  - Compositor integration          - History UI                  │
│  - WebRender setup                 - Preferences UI              │
│  - EmbedderMethods impl            - Download manager            │
│  - WindowMethods impl              - Minibrowser UI              │
├─────────────────────────────────────────────────────────────────┤
│  ADDED BY RIGGING:                                               │
│  - Pluggable Connector trait                                     │
│  - Transport-aware URL parsing (http::unix:, http::pipe:, etc.) │
│  - UdsConnector (for Harbor)                                     │
│  - TcpConnector (for Compass standard browsing)                  │
│  - TorConnector (for Compass .onion sites, via Corsair)          │
│  - WebView public API                                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SERVO (upstream)                              │
│              (minimal patches, track upstream)                   │
├─────────────────────────────────────────────────────────────────┤
│  USED AS-IS:                       PATCHED (minimal):           │
│  - WebRender                       - net component:              │
│  - Stylo (CSS)                       - Connector hook            │
│  - Layout                            - Transport URL support     │
│  - Script (DOM + SpiderMonkey)                                   │
│  - html5ever                                                     │
│  - Fonts, Canvas, Compositing                                    │
└─────────────────────────────────────────────────────────────────┘
```

## Repository Structure

```
rigging/
├── src/
│   ├── lib.rs                   # Main library entry point
│   ├── webview.rs               # WebView public API (from servoshell)
│   ├── window.rs                # Window management (from servoshell)
│   ├── app.rs                   # Event loop (from servoshell)
│   ├── compositor.rs            # Compositor integration (from servoshell)
│   ├── connector/
│   │   ├── mod.rs               # Connector trait definition
│   │   ├── uds.rs               # Unix Domain Socket connector
│   │   ├── tcp.rs               # TCP connector
│   │   └── tor.rs               # Tor connector (via Corsair)
│   └── transport/
│       ├── mod.rs               # Transport types
│       └── url.rs               # Transport-aware URL parsing
├── patches/                     # Git patches for Servo net component
├── apply-patches.sh             # Apply patches to Servo
├── regenerate-patches.sh        # Regenerate patches from fork
├── README.md
├── DESIGN.md
└── Cargo.toml
```

## Key Design Principles

### Rigging is a servoshell Fork

We forked servoshell's embedding code because:
1. **servoshell already solved the hard problems** - EmbedderMethods, WindowMethods, winit+surfman integration
2. **We're replacing the shell, not wrapping it** - Harbor and Compass ARE shells
3. **Easier to track upstream Servo** - Servo stays upstream, we only patch `net`

### Stable API Boundary

Rigging provides a **stable API** for Harbor and Compass:

- **DO NOT** change public types without version bump
- **DO NOT** expose Servo internal types
- **DO** add new fields with defaults via builder pattern
- **DO** keep all Servo-specific code internal

### Public API (STABLE)

```rust
// The Connector trait - key abstraction for pluggable networking
pub trait Connector: Send + Sync {
    /// Check if this connector allows the given URL
    fn allows_url(&self, url: &TransportUrl) -> bool;

    /// Connect to the given URL
    fn connect(&self, url: &TransportUrl) -> Result<Box<dyn AsyncReadWrite>, ConnectError>;
}

// Built-in connectors
pub struct UdsConnector;      // Unix Domain Sockets (Harbor)
pub struct TcpConnector;      // Standard TCP (Compass)
pub struct TorConnector;      // Tor via Corsair (Compass)

// WebView API
pub struct WebViewConfig {
    pub initial_url: String,
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f32,
}

pub struct WebView { /* contains Servo internals */ }

impl WebView {
    pub fn new<C: Connector>(config: WebViewConfig, connector: C, window: &Window) -> Result<Self>;
    pub fn navigate(&mut self, url: &str) -> Result<()>;
    pub fn tick(&mut self) -> Vec<WebViewEvent>;
    pub fn resize(&mut self, width: u32, height: u32);
    pub fn handle_input(&mut self, event: InputEvent);
    pub fn render(&mut self);
    pub fn shutdown(self);
}

pub enum WebViewEvent {
    TitleChanged(String),
    UrlChanged(String),
    LoadStarted,
    LoadComplete,
    NavigationRequest { url: String, is_external: bool },
    Error(String),
}
```

### Internal Implementation (MAY CHANGE)

These may change between Servo/servoshell versions:
- Window management internals (from servoshell)
- Compositor integration
- Servo trait implementations
- Private helper functions

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

1. **Rigging is a servoshell fork**: Not a wrapper, an actual fork of embedding code
2. **Stable API Contract**: Never break WebView, WebViewConfig, Connector, or WebViewEvent
3. **Servo stays upstream**: Only patch the `net` component, nothing else
4. **Patch Maintenance**: When updating Servo, regenerate patches
5. **No SOCKS5**: Tor uses binary IPC via Corsair, not SOCKS5

## Tracking Upstream

```
Servo (upstream)
    ↓ submodule in Rigging, minimal patches to net component
Rigging (fork of servoshell core + Connector abstraction)
    ↓ cargo dependency
Harbor / Compass (applications)
```

**When Servo updates:**
1. Update Rigging's Servo submodule
2. Rebase minimal net patches
3. Test Rigging
4. Harbor/Compass get updates via `cargo update`

**When servoshell updates:**
1. Cherry-pick relevant embedding improvements into Rigging
2. Ignore browser chrome changes (we don't have that code)

## Common Mistakes to Avoid

**READ THIS SECTION CAREFULLY.** These are mistakes AI assistants keep making:

### 1. DO NOT Add Browser Chrome to Rigging

**WRONG:**
- Adding toolbar/URL bar code
- Implementing tabs
- Adding bookmarks functionality

**WHY:** Rigging is embedding code only. Compass adds its own chrome on top.

### 2. DO NOT Hardcode Networking

**WRONG:**
```rust
// Hardcoding TCP
let stream = TcpStream::connect(url)?;
```

**RIGHT:**
```rust
// Use the Connector trait
let stream = connector.connect(&transport_url)?;
```

**WHY:** Harbor uses UDS, Compass uses TCP/Tor. The Connector trait abstracts this.

### 3. DO NOT Expose Servo Types in Public API

**WRONG:**
```rust
pub fn get_servo_webview(&self) -> &servo::WebView { ... }
```

**RIGHT:**
```rust
pub fn navigate(&mut self, url: &str) -> Result<(), WebViewError> { ... }
```

**WHY:** Servo internals change frequently. Our API must be stable.

### 4. DO NOT Fork All of Servo

**WRONG:**
- Copying all of Servo into Rigging
- Modifying WebRender, Stylo, Layout, etc.

**RIGHT:**
- Keep Servo as upstream submodule
- Only patch the `net` component
- Fork servoshell embedding code only

### 5. DO NOT Skip the Connector Trait

**WRONG:**
- Adding a new transport by modifying Servo directly
- Bypassing the Connector abstraction

**RIGHT:**
- Implement a new Connector (e.g., `QuicConnector`)
- Register it with WebView

### 6. DO NOT Forget Transport URL Format

**WRONG:**
```
http://localhost:8000
unix:///tmp/app.sock
```

**RIGHT:**
```
http::unix///tmp/app.sock/path    # Absolute socket path
http::tcp//example.com/path       # Explicit TCP
http::tor//example.onion/path     # Tor hidden service
```

### 7. DO NOT Break Harbor or Compass

Before any change, verify:
- Harbor can still create a WebView with UdsConnector
- Compass can still create a WebView with TcpConnector/TorConnector
- The public API hasn't changed incompatibly

## Development Workflow: TDD and Commits

### Test-Driven Development

**Write tests BEFORE or ALONGSIDE implementation code.**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_uds_connector_blocks_tcp() {
        let connector = UdsConnector;
        let tcp_url = TransportUrl::parse("http::tcp//example.com/").unwrap();
        assert!(!connector.allows_url(&tcp_url));
    }

    #[test]
    fn test_uds_connector_allows_unix() {
        let connector = UdsConnector;
        let uds_url = TransportUrl::parse("http::unix///tmp/app.sock/").unwrap();
        assert!(connector.allows_url(&uds_url));
    }
}
```

### Commit Frequently

**Commit after every successful test run.**

```bash
cargo test && git add -A && git commit -m "feat: add UdsConnector"
```

## Related Projects

- [Servo](https://github.com/servo/servo) - Upstream browser engine
- [servoshell](https://github.com/servo/servo/tree/main/ports/servoshell) - Reference shell (we forked from this)
- [Harbor](https://github.com/marctjones/harbor) - Local app framework (Electron alternative)
- [Compass](https://github.com/marctjones/compass) - Privacy browser
- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
