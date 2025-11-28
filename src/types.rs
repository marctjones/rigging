/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Transport types and error definitions

use std::fmt;
use thiserror::Error;

/// Supported transport protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transport {
    /// Standard TCP/IP connection
    Tcp,
    /// Unix Domain Socket (Linux/macOS)
    Unix,
    /// Named Pipe (Windows)
    NamedPipe,
    /// Tor anonymity network (via SOCKS5)
    Tor,
    /// SSH tunnel
    Ssh,
    /// QUIC/HTTP3
    Quic,
}

impl Transport {
    /// Parse transport from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tcp" => Some(Transport::Tcp),
            "unix" | "uds" => Some(Transport::Unix),
            "pipe" | "namedpipe" => Some(Transport::NamedPipe),
            "tor" | "onion" => Some(Transport::Tor),
            "ssh" => Some(Transport::Ssh),
            "quic" | "http3" => Some(Transport::Quic),
            _ => None,
        }
    }

    /// Get the transport name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Transport::Tcp => "tcp",
            Transport::Unix => "unix",
            Transport::NamedPipe => "pipe",
            Transport::Tor => "tor",
            Transport::Ssh => "ssh",
            Transport::Quic => "quic",
        }
    }

    /// Check if this transport is local-only (no network)
    pub fn is_local(&self) -> bool {
        matches!(self, Transport::Unix | Transport::NamedPipe)
    }

    /// Check if this transport provides anonymity
    pub fn is_anonymous(&self) -> bool {
        matches!(self, Transport::Tor)
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            Transport::Tcp => "TCP/IP",
            Transport::Unix => "Unix Socket",
            Transport::NamedPipe => "Named Pipe",
            Transport::Tor => "Tor Network",
            Transport::Ssh => "SSH Tunnel",
            Transport::Quic => "QUIC/HTTP3",
        }
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Transport {
    fn default() -> Self {
        Transport::Tcp
    }
}

/// A chain of transports (for composed connections)
///
/// Example: `[Tor, Unix]` means connect through Tor, then to a Unix socket
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportChain {
    transports: Vec<Transport>,
}

impl TransportChain {
    /// Create a new transport chain
    pub fn new(transports: Vec<Transport>) -> Self {
        Self { transports }
    }

    /// Create a single-transport chain
    pub fn single(transport: Transport) -> Self {
        Self {
            transports: vec![transport],
        }
    }

    /// Get the transports in the chain
    pub fn transports(&self) -> &[Transport] {
        &self.transports
    }

    /// Get the first (outermost) transport
    pub fn first(&self) -> Option<&Transport> {
        self.transports.first()
    }

    /// Get the last (innermost) transport
    pub fn last(&self) -> Option<&Transport> {
        self.transports.last()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.transports.is_empty()
    }

    /// Get the length of the chain
    pub fn len(&self) -> usize {
        self.transports.len()
    }

    /// Parse a chain from a string like "tor+unix" or "ssh+tcp"
    pub fn parse(s: &str) -> Result<Self, TransportError> {
        let transports: Result<Vec<_>, _> = s
            .split('+')
            .map(|part| {
                Transport::from_str(part.trim())
                    .ok_or_else(|| TransportError::InvalidTransport(part.to_string()))
            })
            .collect();

        Ok(Self {
            transports: transports?,
        })
    }
}

impl Default for TransportChain {
    fn default() -> Self {
        Self::single(Transport::Tcp)
    }
}

impl fmt::Display for TransportChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<_> = self.transports.iter().map(|t| t.as_str()).collect();
        write!(f, "{}", names.join("+"))
    }
}

/// Errors that can occur during transport operations
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Invalid transport: {0}")]
    InvalidTransport(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Transport not available: {0}")]
    NotAvailable(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Socket path not found")]
    SocketPathNotFound,

    #[error("Named pipe not found: {0}")]
    NamedPipeNotFound(String),

    #[error("Tor proxy not available")]
    TorNotAvailable,

    #[error("SOCKS5 error: {0}")]
    Socks5Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_parse() {
        assert_eq!(Transport::from_str("tcp"), Some(Transport::Tcp));
        assert_eq!(Transport::from_str("unix"), Some(Transport::Unix));
        assert_eq!(Transport::from_str("UDS"), Some(Transport::Unix));
        assert_eq!(Transport::from_str("tor"), Some(Transport::Tor));
        assert_eq!(Transport::from_str("invalid"), None);
    }

    #[test]
    fn test_transport_chain_parse() {
        let chain = TransportChain::parse("tor+unix").unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.first(), Some(&Transport::Tor));
        assert_eq!(chain.last(), Some(&Transport::Unix));
    }

    #[test]
    fn test_transport_chain_display() {
        let chain = TransportChain::new(vec![Transport::Tor, Transport::Unix]);
        assert_eq!(chain.to_string(), "tor+unix");
    }

    #[test]
    fn test_transport_is_local() {
        assert!(Transport::Unix.is_local());
        assert!(Transport::NamedPipe.is_local());
        assert!(!Transport::Tcp.is_local());
        assert!(!Transport::Tor.is_local());
    }
}
