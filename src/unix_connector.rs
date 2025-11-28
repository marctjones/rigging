/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Unix Domain Socket connector for HTTP clients
//!
//! Provides a Hyper-compatible connector for making HTTP requests
//! over Unix domain sockets.

use crate::types::TransportError;
use futures::future::BoxFuture;
use hyper::Uri;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UnixStream;
use tower_service::Service;

/// A stream type that wraps Unix socket connections
pub struct UnixConnection {
    stream: UnixStream,
}

impl UnixConnection {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
    }
}

impl AsyncRead for UnixConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl hyper::rt::Read for UnixConnection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        let mut read_buf = tokio::io::ReadBuf::uninit(unsafe { buf.as_mut() });
        match Pin::new(&mut self.get_mut().stream).poll_read(cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let filled = read_buf.filled().len();
                unsafe { buf.advance(filled) };
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl hyper::rt::Write for UnixConnection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

/// Unix socket connector for Hyper HTTP clients
///
/// # Example
///
/// ```rust,ignore
/// use rigging::UnixConnector;
///
/// let connector = UnixConnector::new("/tmp/app.sock");
/// // Use with hyper client...
/// ```
#[derive(Clone)]
pub struct UnixConnector {
    /// Path to the Unix socket
    socket_path: PathBuf,
}

impl UnixConnector {
    /// Create a new Unix connector for the given socket path
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Connect to the Unix socket
    pub async fn connect(&self) -> Result<UnixConnection, TransportError> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(TransportError::Io)?;

        Ok(UnixConnection::new(stream))
    }
}

impl Service<Uri> for UnixConnector {
    type Response = UnixConnection;
    type Error = TransportError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _uri: Uri) -> Self::Future {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            let stream = UnixStream::connect(&socket_path)
                .await
                .map_err(TransportError::Io)?;

            Ok(UnixConnection::new(stream))
        })
    }
}

/// Socket path mapping configuration
///
/// Maps hostnames to Unix socket paths for transparent routing.
#[derive(Debug, Clone, Default)]
pub struct SocketMapping {
    /// Default socket directory
    pub socket_dir: Option<PathBuf>,
    /// Explicit hostname to socket path mappings
    mappings: std::collections::HashMap<String, PathBuf>,
}

impl SocketMapping {
    /// Create a new socket mapping
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default socket directory
    pub fn with_socket_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.socket_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Add a hostname to socket path mapping
    pub fn add_mapping<S: Into<String>, P: AsRef<Path>>(&mut self, host: S, path: P) {
        self.mappings.insert(host.into(), path.as_ref().to_path_buf());
    }

    /// Get socket path for a hostname
    pub fn get_socket_path(&self, host: &str) -> Option<PathBuf> {
        // Check explicit mappings first
        if let Some(path) = self.mappings.get(host) {
            return Some(path.clone());
        }

        // Fall back to default directory + hostname.sock
        self.socket_dir.as_ref().map(|dir| dir.join(format!("{}.sock", host)))
    }

    /// Parse mappings from environment variable format
    ///
    /// Format: "host1:/path1,host2:/path2"
    pub fn from_env_string(s: &str) -> Self {
        let mut mapping = Self::new();
        for pair in s.split(',') {
            if let Some((host, path)) = pair.split_once(':') {
                mapping.add_mapping(host.trim(), path.trim());
            }
        }
        mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_mapping() {
        let mut mapping = SocketMapping::new()
            .with_socket_dir("/tmp/sockets");

        mapping.add_mapping("myapp", "/custom/path.sock");

        // Explicit mapping
        assert_eq!(
            mapping.get_socket_path("myapp"),
            Some(PathBuf::from("/custom/path.sock"))
        );

        // Default directory
        assert_eq!(
            mapping.get_socket_path("other"),
            Some(PathBuf::from("/tmp/sockets/other.sock"))
        );
    }

    #[test]
    fn test_socket_mapping_from_env() {
        let mapping = SocketMapping::from_env_string("app1:/tmp/app1.sock,app2:/var/run/app2.sock");

        assert_eq!(
            mapping.get_socket_path("app1"),
            Some(PathBuf::from("/tmp/app1.sock"))
        );
        assert_eq!(
            mapping.get_socket_path("app2"),
            Some(PathBuf::from("/var/run/app2.sock"))
        );
    }
}
