/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stable Servo Embedding API
//!
//! This module provides a stable, simplified API for embedding Servo in
//! applications like Harbor and Compass. It isolates application code from
//! Servo's internal APIs, making it easier to upgrade Servo versions.
//!
//! # Design Principles
//!
//! 1. **Stability**: This API should remain stable across Servo upgrades
//! 2. **Simplicity**: Expose only what applications need, hide complexity
//! 3. **Isolation**: All Servo-specific types stay within this module
//! 4. **Documentation**: Every change to this API should be documented
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │           Applications (Harbor, Compass, etc.)              │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │ Uses stable API
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Rigging Library                           │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │  embed/ (THIS MODULE) - Stable Embedding API        │    │
//! │  │  - BrowserBuilder                                   │    │
//! │  │  - BrowserConfig                                    │    │
//! │  │  - BrowserEvent                                     │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │  Transport Layer                                    │    │
//! │  │  - TransportUrl parsing                             │    │
//! │  │  - Unix/TCP/Tor connectors                          │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │ Internal implementation
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              Servo Engine (marctjones/servo fork)            │
//! │              with Rigging transport patches applied          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Upgrading Servo
//!
//! When upgrading Servo:
//! 1. Update the servo git dependency in Rigging's Cargo.toml
//! 2. Fix any compilation errors in the `backend` submodule ONLY
//! 3. Do NOT change the public API unless absolutely necessary
//! 4. If API changes are needed, document them in CHANGELOG.md
//! 5. Applications (Harbor, Compass) should not need changes
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use rigging::embed::{BrowserBuilder, BrowserConfig};
//!
//! let config = BrowserConfig::new("http::unix///tmp/app.sock/")
//!     .with_title("My App")
//!     .with_size(1200, 800);
//!
//! BrowserBuilder::new()
//!     .config(config)
//!     .on_event(|event| println!("Event: {:?}", event))
//!     .run()?;
//! ```

mod config;
mod events;
mod builder;
mod backend;
#[cfg(feature = "servo")]
mod servo_backend;

pub use config::BrowserConfig;
pub use events::{BrowserEvent, NavigationEvent, LoadState};
pub use builder::BrowserBuilder;

use thiserror::Error;

/// Errors from the browser embedding API
#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("Failed to initialize browser engine: {0}")]
    InitFailed(String),

    #[error("Failed to create window: {0}")]
    WindowFailed(String),

    #[error("Failed to load URL: {0}")]
    LoadFailed(String),

    #[error("Invalid transport URL: {0}")]
    InvalidUrl(String),

    #[error("Event loop error: {0}")]
    EventLoopError(String),

    #[error("Servo engine not available (feature not enabled)")]
    ServoNotAvailable,
}

/// Check if any browser engine is available
///
/// Returns true if Rigging was compiled with browser support (webview or servo).
/// Applications should check this before attempting to create browser windows.
pub fn is_browser_available() -> bool {
    cfg!(any(feature = "webview", feature = "servo"))
}

/// Check if the webview backend is available
pub fn is_webview_available() -> bool {
    cfg!(feature = "webview")
}

/// Check if the Servo browser engine is available
pub fn is_servo_available() -> bool {
    cfg!(feature = "servo")
}

/// Get the Rigging version
pub fn rigging_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the Servo version (if available)
pub fn servo_version() -> Option<&'static str> {
    if cfg!(feature = "servo") {
        // TODO: Get actual version from Servo when integrated
        Some("0.0.1-dev")
    } else {
        None
    }
}
