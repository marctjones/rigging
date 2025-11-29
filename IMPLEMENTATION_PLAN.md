# Rigging Implementation Plan

## Overview

Rigging is a **fork of servoshell's core embedding code** with browser chrome stripped out and a pluggable networking layer added.

**Critical to understand:**
- **servoshell** = Servo's reference shell (browser UI + embedding code)
- **Rigging** = servoshell's embedding code only (NO browser UI) + pluggable Connector trait
- **Harbor** = Rigging + backend management + UDS-only connector (Electron alternative)
- **Compass** = Rigging + browser UI + Tor connector (privacy browser)

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATIONS                                  │
├─────────────────────────────┬───────────────────────────────────┤
│         HARBOR              │           COMPASS                  │
│  - No browser chrome        │  - Full browser chrome             │
│  - Backend management       │  - Tor integration                 │
│  - UdsConnector only        │  - TorConnector + TcpConnector     │
└─────────────────────────────┴───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RIGGING (this repo)                           │
│            (forked from servoshell core)                         │
│  - WebView API                                                   │
│  - Window management (winit/surfman)                             │
│  - Pluggable Connector trait                                     │
│  - NO browser chrome                                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SERVO (upstream)                              │
│  - WebRender, Stylo, Layout, Script, etc.                       │
│  - Minimal patches to net component                              │
└─────────────────────────────────────────────────────────────────┘
```

## Current Status

**Phase 1 (Transport Layer)**: COMPLETE
- Transport-aware URL parsing works
- TCP and Unix connectors work
- Tor connector structure exists

**Phase 2 (Servo Embedding)**: NOT STARTED - THIS IS THE CRITICAL WORK

Harbor and Compass are BLOCKED waiting for this phase.

---

## Phase 1: Transport Layer ✓ COMPLETE

### 1.1 Basic Types ✓
- [x] `Transport` enum
- [x] `TransportChain` struct
- [x] `TransportError` error types

### 1.2 URL Parsing ✓
- [x] `TransportUrl` struct
- [x] Parse transport from URL scheme
- [x] Extract socket path for Unix/Pipe
- [x] Standard URL component extraction

### 1.3 Connectors ✓
- [x] TCP connector (basic)
- [x] Unix socket connector (basic)
- [x] Tor connector structure (via Corsair)

---

## Phase 2: Fork servoshell into Rigging - CURRENT PRIORITY

> **THIS IS THE CRITICAL WORK** that Harbor and Compass are waiting for.

### Analysis Complete (2024-11-29)

Servoshell has been analyzed. Key findings:

**Files to copy from `/home/marc/servo/ports/servoshell/`:**

| File | Lines | Purpose | Action |
|------|-------|---------|--------|
| `running_app_state.rs` | 760 | Core state, WebViewDelegate impl | COPY |
| `window.rs` | 371 | ServoShellWindow, PlatformWindow trait | COPY |
| `desktop/app.rs` | 250 | App struct, ServoBuilder setup | COPY |
| `desktop/event_loop.rs` | ~174 | Event loop waking | COPY |
| `desktop/keyutils.rs` | ~600 | Keyboard event translation | COPY |
| `desktop/headed_window.rs` | ~1,350 | Window impl (extract ~300 lines) | EXTRACT |
| `desktop/headless_window.rs` | ~163 | Headless rendering | COPY |

**Files to NOT copy (browser chrome):**

| File | Lines | Why Remove |
|------|-------|------------|
| `desktop/gui.rs` | 686 | egui toolbar, URL bar |
| `desktop/dialog.rs` | 711 | egui dialogs |
| Most of `headed_window.rs` | ~1,050 | egui integration |

### 2.1 Set Up Directory Structure
- [ ] Create `src/servoshell/` directory in Rigging
- [ ] Create `src/servoshell/desktop/` subdirectory
- [ ] Set up module declarations

### 2.2 Copy Core Files (Minimal Changes)
- [ ] Copy `running_app_state.rs` → `src/servoshell/running_app_state.rs`
- [ ] Copy `window.rs` → `src/servoshell/window.rs`
- [ ] Copy `desktop/app.rs` → `src/servoshell/desktop/app.rs`
- [ ] Copy `desktop/event_loop.rs` → `src/servoshell/desktop/event_loop.rs`
- [ ] Copy `desktop/keyutils.rs` → `src/servoshell/desktop/keyutils.rs`
- [ ] Copy `desktop/headless_window.rs` → `src/servoshell/desktop/headless_window.rs`
- [ ] Fix imports and module paths
- [ ] Verify it compiles (will need servo dependency)

### 2.3 Extract Core from headed_window.rs
- [ ] Read `desktop/headed_window.rs` (1,350 lines)
- [ ] Identify PlatformWindow trait implementation (~200 lines)
- [ ] Identify winit/surfman setup code (~100 lines)
- [ ] Identify event handling code (~100 lines)
- [ ] Remove ALL egui code (~700+ lines)
- [ ] Remove dialog display logic
- [ ] Create minimal `src/servoshell/desktop/headed_window.rs`

### 2.4 Add Connector Trait
- [ ] Define `Connector` trait in `src/connector/mod.rs`:
  ```rust
  pub trait Connector: Send + Sync {
      fn allows_url(&self, url: &TransportUrl) -> bool;
      fn connect(&self, url: &TransportUrl) -> Result<Box<dyn AsyncReadWrite>, ConnectError>;
  }
  ```
- [ ] Implement `UdsConnector` (blocks non-UDS URLs)
- [ ] Implement `TcpConnector` (standard TCP)
- [ ] Implement `TorConnector` (via Corsair)

### 2.5 Wire Connector into Servo Initialization
- [ ] Modify `desktop/app.rs` to accept a `Connector`
- [ ] Pass Connector to ServoBuilder setup
- [ ] This may require patching Servo's `net` component

### 2.6 Patch Servo's net Component
- [ ] Fork servo or create patch files
- [ ] Add Connector hook to `components/net/http_loader.rs`
- [ ] Route HTTP requests through injected Connector
- [ ] Test that UdsConnector blocks TCP URLs
- [ ] Test that TcpConnector allows normal browsing

### 2.7 Create Rigging's Public API
- [ ] Create `src/webview.rs` with public `WebView` struct
- [ ] Create `src/webview_config.rs` with `WebViewConfig`
- [ ] Create `src/webview_event.rs` with `WebViewEvent` enum
- [ ] Hide servoshell internals from public API
- [ ] Export in `src/lib.rs`:
  ```rust
  pub use webview::{WebView, WebViewConfig, WebViewEvent};
  pub use connector::{Connector, UdsConnector, TcpConnector, TorConnector};
  ```

### 2.8 Testing
- [ ] Unit tests for Connector implementations
- [ ] Unit tests for WebView API
- [ ] Integration test: Render static HTML
- [ ] Integration test: UDS connection works
- [ ] Integration test: TCP blocked when using UdsConnector

---

## Phase 3: Windows Support

### 3.1 Named Pipe Connector
- [ ] Implement `PipeConnector` for Windows
- [ ] Support `http::pipe//pipename/` URLs
- [ ] Test on Windows

### 3.2 Platform Abstraction
- [ ] Ensure WebView API works on Windows
- [ ] Cross-platform CI

---

## Phase 4: Advanced Features

### 4.1 Connection Management
- [ ] Optional connection pooling
- [ ] Health checking
- [ ] Automatic reconnection

### 4.2 Observability
- [ ] Tracing integration
- [ ] Metrics collection
- [ ] Debug logging

---

## Milestones

### v0.1.0 - Transport Layer ✓ COMPLETE
- [x] Transport-aware URL parsing
- [x] TCP and Unix connectors
- [x] Tor connector structure

### v0.2.0 - Servo Embedding - CURRENT TARGET
- [ ] Fork servoshell core into Rigging
- [ ] Strip browser chrome
- [ ] Add Connector trait
- [ ] Patch Servo's net component
- [ ] Create WebView public API
- [ ] Test with Harbor (UDS) and Compass (TCP)

### v0.3.0 - Windows Support
- [ ] Named pipe connector
- [ ] Cross-platform WebView

### v1.0.0 - Stable Release
- [ ] Full documentation
- [ ] Comprehensive tests
- [ ] API stability guarantee

---

## Key Files in Servo to Understand

### servoshell (what we're forking)
Location: `/home/marc/servo/ports/servoshell/`

- `running_app_state.rs` - Core state, WebViewDelegate, WebViewCollection
- `window.rs` - PlatformWindow trait, ServoShellWindow
- `desktop/app.rs` - App struct, ApplicationHandler, ServoBuilder
- `desktop/event_loop.rs` - Event loop waking mechanism
- `desktop/headed_window.rs` - Winit window (extract ~300 lines, remove egui)

### Servo net component (where we add Connector hook)
Location: `/home/marc/servo/components/net/`

- `http_loader.rs` - Where HTTP requests are made
- `connector.rs` - Network connection abstraction

### Key Servo Types
```rust
// From servo crate
use servo::{Servo, ServoBuilder, ServoDelegate};
use servo::{WebView, WebViewBuilder, WebViewDelegate, WebViewId};
use servo::{EventLoopWaker, RenderingContext, ScreenGeometry};

// From euclid
use euclid::Scale;

// From winit
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::WindowId;
```

---

## Dependencies

### Current (Transport Layer)
| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.x | Async runtime |
| url | 2.x | URL parsing |
| thiserror | 1.x | Error handling |
| serde | 1.x | Serialization |
| bincode | 1.x | Corsair IPC |

### Phase 2 (Servo Embedding)
| Crate | Version | Purpose |
|-------|---------|---------|
| servo | submodule | Browser engine |
| winit | 0.30+ | Window management |
| surfman | 0.9+ | GPU surface |
| euclid | 0.22 | Geometry types |
| raw-window-handle | 0.6 | Window handles |

---

## Open Questions

1. **Servo as submodule or patches?** - Probably patches applied to upstream
2. **How to inject Connector into Servo?** - Need to investigate net component
3. **Feature flags for servo?** - May need separate features for servo vs transport-only

---

## Contributing

See AGENTS.md for AI assistant guidelines and coding standards.
