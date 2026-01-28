/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Rigging - Transport layer and embedding API for Servo-based applications
//!
//! Rigging provides two main capabilities:
//!
//! 1. **Transport Layer** - Connect to servers over various protocols
//! 2. **Embedding API** - Stable interface for embedding Servo browser engine
//!
//! # Transport Layer
//!
//! Rigging extends standard URLs with transport specifications:
//!
//! - **TCP** - Standard TCP/IP connections
//! - **Unix Domain Sockets** - Local IPC on Unix systems
//! - **Named Pipes** - Local IPC on Windows
//! - **Tor** - Anonymous connections via Corsair daemon
//!
//! ## Transport URL Syntax
//!
//! ```text
//! http::unix///tmp/app.sock/api/data    # Unix socket (absolute path)
//! http::unix//var/run/app.sock          # Unix socket (relative path)
//! http::tcp//localhost:8080             # Explicit TCP
//! http::tor//example.onion              # Tor network
//! ```
//!
//! ## Transport Example
//!
//! ```rust,ignore
//! use rigging::{TransportUrl, Transport};
//!
//! let url = TransportUrl::parse("http::unix///tmp/app.sock/api")?;
//! assert_eq!(url.transport(), Transport::Unix);
//! assert_eq!(url.unix_socket_path(), Some("/tmp/app.sock"));
//! ```
//!
//! # Embedding API
//!
//! Rigging provides a stable API for embedding the Servo browser engine.
//! This isolates applications from Servo's internal APIs, making upgrades easier.
//!
//! ## Embedding Example
//!
//! ```rust,ignore
//! use rigging::embed::{BrowserBuilder, BrowserConfig};
//!
//! // Simple usage
//! BrowserBuilder::new()
//!     .url("http::unix///tmp/app.sock/")
//!     .title("My App")
//!     .size(1200, 800)
//!     .run()?;
//!
//! // With full configuration
//! let config = BrowserConfig::new("http://localhost/")
//!     .with_title("My App")
//!     .with_size(1200, 800)
//!     .with_devtools(true);
//!
//! BrowserBuilder::new()
//!     .config(config)
//!     .on_event(|event| println!("Event: {:?}", event))
//!     .run()?;
//! ```
//!
//! # Feature Flags
//!
//! - `unix` - Unix Domain Socket support (default)
//! - `tcp` - TCP transport support (default)
//! - `tor` - Tor transport via Corsair daemon
//! - `named-pipe` - Windows Named Pipe support
//! - `servo` - Enable embedded Servo browser engine

// Transport layer modules
pub mod transport_url;
pub mod types;

#[cfg(feature = "unix")]
pub mod unix_connector;

#[cfg(feature = "tcp")]
pub mod tcp_connector;

#[cfg(feature = "tor")]
pub mod tor_connector;

pub mod composed;

// Embedding API module
pub mod embed;

// Servoshell embedding code (forked from servo/ports/servoshell)
#[cfg(feature = "servo")]
pub mod servoshell;

// Transport layer re-exports
pub use transport_url::TransportUrl;
pub use types::{Transport, TransportChain, TransportError};

#[cfg(feature = "unix")]
pub use unix_connector::UnixConnector;

// Embedding API re-exports (for convenience)
pub use embed::{BrowserBuilder, BrowserConfig, BrowserEvent, EmbedError, is_browser_available};
