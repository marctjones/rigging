/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Transport-aware URL parsing
//!
//! Extends standard URLs with explicit transport specification:
//!
//! ```text
//! http::unix///tmp/app.sock/api/data    # Unix socket
//! http::tcp//localhost:8080             # Explicit TCP
//! http::tor//example.onion              # Tor network
//! ```

use crate::types::{Transport, TransportError};
use url::Url;

/// A URL with explicit transport information
#[derive(Debug, Clone)]
pub struct TransportUrl {
    /// The underlying URL (normalized)
    url: Url,
    /// The transport to use
    transport: Transport,
    /// Original scheme before normalization
    original_scheme: String,
    /// Whether transport was explicitly specified
    explicit_transport: bool,
    /// Unix socket path (if applicable)
    unix_socket_path: Option<String>,
    /// Named pipe path (if applicable, Windows)
    named_pipe_path: Option<String>,
}

impl TransportUrl {
    /// Parse a transport-aware URL
    ///
    /// # Examples
    ///
    /// ```
    /// use rigging::TransportUrl;
    ///
    /// // Standard URL (implicit TCP)
    /// let url = TransportUrl::parse("https://example.com/").unwrap();
    ///
    /// // Unix socket URL
    /// let url = TransportUrl::parse("http::unix///tmp/app.sock/api").unwrap();
    ///
    /// // Tor URL
    /// let url = TransportUrl::parse("http::tor//example.onion/").unwrap();
    /// ```
    pub fn parse(url_str: &str) -> Result<Self, TransportError> {
        // Check for transport specification: scheme::transport//...
        if let Some((scheme_transport, rest)) = url_str.split_once("//") {
            if let Some((scheme, transport_str)) = scheme_transport.split_once("::") {
                // Explicit transport specified
                let transport = Transport::from_str(transport_str)
                    .ok_or_else(|| TransportError::InvalidTransport(transport_str.to_string()))?;

                return Self::parse_with_transport(scheme, transport, rest);
            }
        }

        // Standard URL - parse normally
        let url = Url::parse(url_str)
            .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        // Check for .onion addresses (always use Tor)
        let transport = if url.host_str().map(|h| h.ends_with(".onion")).unwrap_or(false) {
            Transport::Tor
        } else {
            Transport::Tcp
        };

        Ok(Self {
            original_scheme: url.scheme().to_string(),
            url,
            transport,
            explicit_transport: false,
            unix_socket_path: None,
            named_pipe_path: None,
        })
    }

    fn parse_with_transport(
        scheme: &str,
        transport: Transport,
        rest: &str,
    ) -> Result<Self, TransportError> {
        match transport {
            Transport::Unix => Self::parse_unix_url(scheme, rest),
            Transport::NamedPipe => Self::parse_named_pipe_url(scheme, rest),
            Transport::Tor => Self::parse_tor_url(scheme, rest),
            Transport::Tcp | Transport::Ssh | Transport::Quic => {
                // Standard URL format
                let full_url = format!("{}://{}", scheme, rest);
                let url = Url::parse(&full_url)
                    .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

                Ok(Self {
                    original_scheme: scheme.to_string(),
                    url,
                    transport,
                    explicit_transport: true,
                    unix_socket_path: None,
                    named_pipe_path: None,
                })
            }
        }
    }

    fn parse_unix_url(scheme: &str, rest: &str) -> Result<Self, TransportError> {
        // Unix socket URL format:
        // http::unix//relative/path.sock         -> relative path
        // http::unix///absolute/path.sock        -> absolute path (note 3 slashes)
        // http::unix///tmp/app.sock/api/data     -> socket path + URL path

        let (socket_path, url_path) = if rest.starts_with('/') {
            // Absolute path: ///tmp/app.sock or ///tmp/app.sock/api
            // rest is "/tmp/app.sock/..." - keep the leading slash for absolute paths
            Self::extract_socket_path(rest)
        } else {
            // Relative path: //relative/path.sock
            Self::extract_socket_path(rest)
        };

        // Downgrade HTTPS to HTTP for local sockets (TLS not needed)
        let effective_scheme = match scheme {
            "https" => "http",
            "wss" => "ws",
            other => other,
        };

        // Create a localhost URL for the URL parsing
        let url_string = format!("{}://localhost{}", effective_scheme, url_path);
        let url = Url::parse(&url_string)
            .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        Ok(Self {
            original_scheme: scheme.to_string(),
            url,
            transport: Transport::Unix,
            explicit_transport: true,
            unix_socket_path: Some(socket_path),
            named_pipe_path: None,
        })
    }

    fn parse_named_pipe_url(scheme: &str, rest: &str) -> Result<Self, TransportError> {
        // Named pipe URL format (Windows):
        // http::pipe//\\.\pipe\myapp           -> named pipe
        // http::pipe//myapp                    -> shorthand for \\.\pipe\myapp

        let pipe_path = if rest.starts_with(r"\\.\pipe\") {
            rest.to_string()
        } else {
            format!(r"\\.\pipe\{}", rest.split('/').next().unwrap_or(rest))
        };

        let url_path: String = if let Some(idx) = rest.find('/') {
            if !rest.starts_with(r"\\") {
                rest[idx..].to_string()
            } else {
                // For full pipe paths, find the path after pipe name
                rest.splitn(2, '/').nth(1).map(|s| format!("/{}", s)).unwrap_or_else(|| "/".to_string())
            }
        } else {
            "/".to_string()
        };

        let effective_scheme = match scheme {
            "https" => "http",
            "wss" => "ws",
            other => other,
        };

        let url_string = format!("{}://localhost{}", effective_scheme, url_path);
        let url = Url::parse(&url_string)
            .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        Ok(Self {
            original_scheme: scheme.to_string(),
            url,
            transport: Transport::NamedPipe,
            explicit_transport: true,
            unix_socket_path: None,
            named_pipe_path: Some(pipe_path),
        })
    }

    fn parse_tor_url(scheme: &str, rest: &str) -> Result<Self, TransportError> {
        let full_url = format!("{}://{}", scheme, rest);
        let url = Url::parse(&full_url)
            .map_err(|e| TransportError::InvalidUrl(e.to_string()))?;

        Ok(Self {
            original_scheme: scheme.to_string(),
            url,
            transport: Transport::Tor,
            explicit_transport: true,
            unix_socket_path: None,
            named_pipe_path: None,
        })
    }

    /// Extract socket path from URL path, separating socket file from URL path
    fn extract_socket_path(path: &str) -> (String, String) {
        // Look for common socket file extensions
        for ext in &[".sock", ".socket", ".sk"] {
            if let Some(idx) = path.find(ext) {
                let end_idx = idx + ext.len();
                let socket_path = &path[..end_idx];
                let url_path = if end_idx < path.len() {
                    &path[end_idx..]
                } else {
                    "/"
                };
                return (socket_path.to_string(), url_path.to_string());
            }
        }

        // No extension found - assume entire path is socket
        (path.to_string(), "/".to_string())
    }

    /// Get the transport type
    pub fn transport(&self) -> Transport {
        self.transport
    }

    /// Check if transport was explicitly specified
    pub fn is_explicit_transport(&self) -> bool {
        self.explicit_transport
    }

    /// Get the underlying URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the URL scheme
    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    /// Get the original scheme (before any downgrading)
    pub fn original_scheme(&self) -> &str {
        &self.original_scheme
    }

    /// Get the host string
    pub fn host_str(&self) -> Option<&str> {
        self.url.host_str()
    }

    /// Get the port
    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    /// Get the port or default for scheme
    pub fn port_or_default(&self) -> u16 {
        self.url.port().unwrap_or_else(|| {
            match self.url.scheme() {
                "https" | "wss" => 443,
                "http" | "ws" => 80,
                _ => 80,
            }
        })
    }

    /// Get the URL path
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// Get the full URL as string
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    /// Get Unix socket path (if applicable)
    pub fn unix_socket_path(&self) -> Option<&str> {
        self.unix_socket_path.as_deref()
    }

    /// Get named pipe path (if applicable, Windows)
    pub fn named_pipe_path(&self) -> Option<&str> {
        self.named_pipe_path.as_deref()
    }

    /// Check if this is a local-only URL (Unix socket or named pipe)
    pub fn is_local(&self) -> bool {
        self.transport.is_local()
    }

    /// Check if this URL requires Tor
    pub fn requires_tor(&self) -> bool {
        self.transport == Transport::Tor ||
            self.url.host_str().map(|h| h.ends_with(".onion")).unwrap_or(false)
    }
}

impl std::fmt::Display for TransportUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.explicit_transport {
            match self.transport {
                Transport::Unix => {
                    if let Some(ref socket) = self.unix_socket_path {
                        write!(f, "{}::unix//{}{}", self.original_scheme, socket, self.url.path())
                    } else {
                        write!(f, "{}", self.url)
                    }
                }
                Transport::NamedPipe => {
                    if let Some(ref pipe) = self.named_pipe_path {
                        write!(f, "{}::pipe//{}{}", self.original_scheme, pipe, self.url.path())
                    } else {
                        write!(f, "{}", self.url)
                    }
                }
                _ => {
                    write!(f, "{}::{}//{}", self.original_scheme, self.transport, &self.url.as_str()[self.url.scheme().len() + 3..])
                }
            }
        } else {
            write!(f, "{}", self.url)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_url() {
        let url = TransportUrl::parse("https://example.com/path").unwrap();
        assert_eq!(url.transport(), Transport::Tcp);
        assert!(!url.is_explicit_transport());
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_unix_socket_absolute() {
        let url = TransportUrl::parse("http::unix///tmp/app.sock/api/data").unwrap();
        assert_eq!(url.transport(), Transport::Unix);
        assert!(url.is_explicit_transport());
        assert_eq!(url.unix_socket_path(), Some("/tmp/app.sock"));
        assert_eq!(url.path(), "/api/data");
    }

    #[test]
    fn test_unix_socket_relative() {
        let url = TransportUrl::parse("http::unix//var/run/app.sock").unwrap();
        assert_eq!(url.transport(), Transport::Unix);
        assert_eq!(url.unix_socket_path(), Some("var/run/app.sock"));
    }

    #[test]
    fn test_https_downgrade_for_unix() {
        let url = TransportUrl::parse("https::unix///tmp/app.sock").unwrap();
        assert_eq!(url.scheme(), "http"); // Downgraded
        assert_eq!(url.original_scheme(), "https");
    }

    #[test]
    fn test_onion_auto_tor() {
        let url = TransportUrl::parse("http://example.onion/").unwrap();
        assert_eq!(url.transport(), Transport::Tor);
        assert!(!url.is_explicit_transport()); // Auto-detected
    }

    #[test]
    fn test_explicit_tor() {
        let url = TransportUrl::parse("http::tor//example.com/").unwrap();
        assert_eq!(url.transport(), Transport::Tor);
        assert!(url.is_explicit_transport());
    }

    #[test]
    fn test_explicit_tcp() {
        let url = TransportUrl::parse("http::tcp//localhost:8080/").unwrap();
        assert_eq!(url.transport(), Transport::Tcp);
        assert!(url.is_explicit_transport());
    }

    #[test]
    fn test_is_local() {
        let unix = TransportUrl::parse("http::unix///tmp/app.sock").unwrap();
        assert!(unix.is_local());

        let tcp = TransportUrl::parse("http://example.com/").unwrap();
        assert!(!tcp.is_local());
    }

    #[test]
    fn test_requires_tor() {
        let onion = TransportUrl::parse("http://example.onion/").unwrap();
        assert!(onion.requires_tor());

        let explicit_tor = TransportUrl::parse("http::tor//example.com/").unwrap();
        assert!(explicit_tor.requires_tor());

        let normal = TransportUrl::parse("http://example.com/").unwrap();
        assert!(!normal.requires_tor());
    }
}
