/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Composed transport connector
//!
//! Allows chaining multiple transports together, e.g., Tor → Unix socket.

use crate::types::{Transport, TransportChain, TransportError};
use crate::TransportUrl;
use std::path::PathBuf;

#[cfg(feature = "unix")]
use crate::unix_connector::UnixConnector;

#[cfg(feature = "tcp")]
use crate::tcp_connector::TcpConnector;

#[cfg(feature = "tor")]
use crate::tor_connector::TorConnector;

/// Configuration for composed transports
#[derive(Debug, Clone)]
pub struct ComposedConfig {
    /// Default socket directory for Unix sockets
    pub socket_dir: Option<PathBuf>,
    /// Path to Tor SOCKS proxy socket
    pub tor_socket: Option<PathBuf>,
}

impl Default for ComposedConfig {
    fn default() -> Self {
        Self {
            socket_dir: Some(PathBuf::from("/tmp/servo-sockets")),
            tor_socket: Some(PathBuf::from("/tmp/servo-sockets/tor.sock")),
        }
    }
}

/// A composed connector that routes based on transport type
pub struct ComposedConnector {
    config: ComposedConfig,
}

impl ComposedConnector {
    /// Create a new composed connector with default config
    pub fn new() -> Self {
        Self {
            config: ComposedConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ComposedConfig) -> Self {
        Self { config }
    }

    /// Create a Unix-only connector
    #[cfg(feature = "unix")]
    pub fn unix<P: Into<PathBuf>>(socket_path: P) -> Self {
        Self {
            config: ComposedConfig {
                socket_dir: Some(socket_path.into()),
                tor_socket: None,
            },
        }
    }

    /// Create a Tor connector
    #[cfg(feature = "tor")]
    pub fn tor() -> Self {
        Self {
            config: ComposedConfig {
                socket_dir: None,
                tor_socket: Some(PathBuf::from("/tmp/servo-sockets/tor.sock")),
            },
        }
    }

    /// Get the appropriate connector for a URL
    pub fn connector_for_url(&self, url: &TransportUrl) -> Result<ConnectorType, TransportError> {
        match url.transport() {
            Transport::Unix => {
                #[cfg(feature = "unix")]
                {
                    let socket_path = url.unix_socket_path()
                        .map(PathBuf::from)
                        .or_else(|| {
                            self.config.socket_dir.as_ref().and_then(|dir| {
                                url.host_str().map(|h| dir.join(format!("{}.sock", h)))
                            })
                        })
                        .ok_or(TransportError::SocketPathNotFound)?;

                    Ok(ConnectorType::Unix(UnixConnector::new(socket_path)))
                }
                #[cfg(not(feature = "unix"))]
                {
                    Err(TransportError::NotAvailable("Unix sockets not compiled".to_string()))
                }
            }
            Transport::Tcp => {
                #[cfg(feature = "tcp")]
                {
                    Ok(ConnectorType::Tcp(TcpConnector::new()))
                }
                #[cfg(not(feature = "tcp"))]
                {
                    Err(TransportError::NotAvailable("TCP not compiled".to_string()))
                }
            }
            Transport::Tor => {
                #[cfg(feature = "tor")]
                {
                    let socket_path = self.config.tor_socket.clone()
                        .ok_or(TransportError::TorNotAvailable)?;
                    Ok(ConnectorType::Tor(TorConnector::with_socket(socket_path)))
                }
                #[cfg(not(feature = "tor"))]
                {
                    Err(TransportError::NotAvailable("Tor not compiled".to_string()))
                }
            }
            Transport::NamedPipe => {
                Err(TransportError::NotAvailable("Named pipes not yet implemented".to_string()))
            }
            Transport::Ssh => {
                Err(TransportError::NotAvailable("SSH tunnels not yet implemented".to_string()))
            }
            Transport::Quic => {
                Err(TransportError::NotAvailable("QUIC not yet implemented".to_string()))
            }
        }
    }

    /// Connect to a URL using the appropriate transport
    pub async fn connect(&self, url_str: &str) -> Result<Connection, TransportError> {
        let url = TransportUrl::parse(url_str)?;
        self.connect_url(&url).await
    }

    /// Connect to a parsed URL
    pub async fn connect_url(&self, url: &TransportUrl) -> Result<Connection, TransportError> {
        let connector = self.connector_for_url(url)?;

        match connector {
            #[cfg(feature = "unix")]
            ConnectorType::Unix(c) => {
                let conn = c.connect().await?;
                Ok(Connection::Unix(conn))
            }
            #[cfg(feature = "tcp")]
            ConnectorType::Tcp(c) => {
                let host = url.host_str().ok_or_else(|| {
                    TransportError::InvalidUrl("No host".to_string())
                })?;
                let port = url.port_or_default();
                let conn = c.connect(host, port).await?;
                Ok(Connection::Tcp(conn))
            }
            #[cfg(feature = "tor")]
            ConnectorType::Tor(c) => {
                let host = url.host_str().ok_or_else(|| {
                    TransportError::InvalidUrl("No host".to_string())
                })?;
                let port = url.port_or_default();
                let conn = c.connect(host, port).await?;
                Ok(Connection::Tor(conn))
            }
            #[allow(unreachable_patterns)]
            _ => Err(TransportError::NotAvailable("Transport not available".to_string())),
        }
    }
}

impl Default for ComposedConnector {
    fn default() -> Self {
        Self::new()
    }
}

/// Enum of connector types
pub enum ConnectorType {
    #[cfg(feature = "unix")]
    Unix(UnixConnector),
    #[cfg(feature = "tcp")]
    Tcp(TcpConnector),
    #[cfg(feature = "tor")]
    Tor(TorConnector),
}

/// Enum of connection types
pub enum Connection {
    #[cfg(feature = "unix")]
    Unix(crate::unix_connector::UnixConnection),
    #[cfg(feature = "tcp")]
    Tcp(crate::tcp_connector::TcpConnection),
    #[cfg(feature = "tor")]
    Tor(crate::tor_connector::TorConnection),
}

/// Builder for transport chains
pub struct TransportChainBuilder {
    transports: Vec<Transport>,
    config: ComposedConfig,
}

impl TransportChainBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            transports: Vec::new(),
            config: ComposedConfig::default(),
        }
    }

    /// Add a transport to the chain
    pub fn add(mut self, transport: Transport) -> Self {
        self.transports.push(transport);
        self
    }

    /// Add TCP transport
    pub fn tcp(self) -> Self {
        self.add(Transport::Tcp)
    }

    /// Add Unix socket transport
    pub fn unix(self) -> Self {
        self.add(Transport::Unix)
    }

    /// Add Tor transport
    pub fn tor(self) -> Self {
        self.add(Transport::Tor)
    }

    /// Set socket directory
    pub fn socket_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config.socket_dir = Some(dir.into());
        self
    }

    /// Set Tor socket path
    pub fn tor_socket<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.tor_socket = Some(path.into());
        self
    }

    /// Build the transport chain
    pub fn build(self) -> (TransportChain, ComposedConfig) {
        (TransportChain::new(self.transports), self.config)
    }
}

impl Default for TransportChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composed_connector_default() {
        let connector = ComposedConnector::new();
        assert!(connector.config.socket_dir.is_some());
    }

    #[test]
    fn test_transport_chain_builder() {
        let (chain, _config) = TransportChainBuilder::new()
            .tor()
            .unix()
            .socket_dir("/tmp/sockets")
            .build();

        assert_eq!(chain.len(), 2);
        assert_eq!(chain.first(), Some(&Transport::Tor));
        assert_eq!(chain.last(), Some(&Transport::Unix));
    }
}
