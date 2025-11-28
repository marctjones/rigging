/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Rigging - Transport layer library for Servo-based applications
//!
//! This crate provides transport abstractions for connecting to web servers
//! over various protocols:
//!
//! - **TCP** - Standard TCP/IP connections
//! - **Unix Domain Sockets** - Local IPC on Unix systems
//! - **Named Pipes** - Local IPC on Windows
//! - **Tor** - Anonymous connections via SOCKS5 proxy
//!
//! # Transport URL Syntax
//!
//! Rigging extends standard URLs with transport specifications:
//!
//! ```text
//! http::unix///tmp/app.sock/api/data    # Unix socket (absolute path)
//! http::unix//var/run/app.sock          # Unix socket (relative path)
//! http::tcp//localhost:8080             # Explicit TCP
//! http::tor//example.onion              # Tor network
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use rigging::{TransportUrl, Transport};
//!
//! let url = TransportUrl::parse("http::unix///tmp/app.sock/api")?;
//! assert_eq!(url.transport(), Transport::Unix);
//! assert_eq!(url.unix_socket_path(), Some("/tmp/app.sock"));
//! ```

pub mod transport_url;
pub mod types;

#[cfg(feature = "unix")]
pub mod unix_connector;

#[cfg(feature = "tcp")]
pub mod tcp_connector;

#[cfg(feature = "tor")]
pub mod tor_connector;

pub mod composed;

// Re-exports
pub use transport_url::TransportUrl;
pub use types::{Transport, TransportChain, TransportError};

#[cfg(feature = "unix")]
pub use unix_connector::UnixConnector;
