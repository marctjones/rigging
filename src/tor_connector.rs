/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tor connector via Corsair daemon
//!
//! This connector communicates with the Corsair Tor daemon over a Unix
//! domain socket using a simple binary IPC protocol (not SOCKS5).
//!
//! # Protocol
//!
//! 1. Client sends ConnectRequest (host, port) - bincode serialized, length-prefixed
//! 2. Server responds with ConnectResponse (success/error)
//! 3. If successful, bidirectional data relay begins

use crate::types::TransportError;
use futures::future::BoxFuture;
use hyper::Uri;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UnixStream;
use tower_service::Service;

/// Default path to the Corsair (Tor daemon) socket
pub const DEFAULT_TOR_SOCKET: &str = "/tmp/servo-sockets/corsair.sock";

/// Request to connect to a remote host through Tor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    /// Target hostname (can be .onion)
    pub host: String,
    /// Target port
    pub port: u16,
}

/// Response to a connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResponse {
    /// Whether the connection succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// A connection through the Tor network
pub struct TorConnection {
    stream: UnixStream,
}

impl TorConnection {
    fn new(stream: UnixStream) -> Self {
        Self { stream }
    }
}

impl AsyncRead for TorConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TorConnection {
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

impl hyper::rt::Read for TorConnection {
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

impl hyper::rt::Write for TorConnection {
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

/// Tor connector that communicates with Corsair daemon via binary IPC
#[derive(Clone)]
pub struct TorConnector {
    /// Path to the Corsair socket
    socket_path: PathBuf,
}

impl TorConnector {
    /// Create a new Tor connector with default socket path
    pub fn new() -> Self {
        Self {
            socket_path: PathBuf::from(DEFAULT_TOR_SOCKET),
        }
    }

    /// Create a Tor connector with custom socket path
    pub fn with_socket<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Check if the Tor daemon is available
    pub async fn is_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Connect to a host through Tor
    pub async fn connect(&self, host: &str, port: u16) -> Result<TorConnection, TransportError> {
        // Connect to Corsair daemon
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|_| TransportError::TorNotAvailable)?;

        // Send connection request using binary protocol
        self.send_connect_request(&mut stream, host, port).await?;

        // Read response
        let response = self.read_connect_response(&mut stream).await?;

        if !response.success {
            return Err(TransportError::ConnectionFailed(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        log::debug!("Tor connection established to {}:{}", host, port);
        Ok(TorConnection::new(stream))
    }

    /// Send a connection request to Corsair
    async fn send_connect_request(
        &self,
        stream: &mut UnixStream,
        host: &str,
        port: u16,
    ) -> Result<(), TransportError> {
        let request = ConnectRequest {
            host: host.to_string(),
            port,
        };

        let data = bincode::serialize(&request)
            .map_err(|e| TransportError::ConnectionFailed(format!("Serialize error: {}", e)))?;

        let len = (data.len() as u32).to_be_bytes();

        stream.write_all(&len).await
            .map_err(|e| TransportError::Io(e))?;
        stream.write_all(&data).await
            .map_err(|e| TransportError::Io(e))?;
        stream.flush().await
            .map_err(|e| TransportError::Io(e))?;

        Ok(())
    }

    /// Read a connection response from Corsair
    async fn read_connect_response(
        &self,
        stream: &mut UnixStream,
    ) -> Result<ConnectResponse, TransportError> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await
            .map_err(|e| TransportError::Io(e))?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 1024 * 1024 {
            return Err(TransportError::ConnectionFailed("Response too large".to_string()));
        }

        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await
            .map_err(|e| TransportError::Io(e))?;

        let response: ConnectResponse = bincode::deserialize(&data)
            .map_err(|e| TransportError::ConnectionFailed(format!("Deserialize error: {}", e)))?;

        Ok(response)
    }
}

impl Default for TorConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl Service<Uri> for TorConnector {
    type Response = TorConnection;
    type Error = TransportError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let socket_path = self.socket_path.clone();
        Box::pin(async move {
            let host = uri.host().ok_or_else(|| {
                TransportError::InvalidUrl("No host in URI".to_string())
            })?;

            let port = uri.port_u16().unwrap_or_else(|| {
                match uri.scheme_str() {
                    Some("https") => 443,
                    Some("http") => 80,
                    _ => 80,
                }
            });

            let connector = TorConnector { socket_path };
            connector.connect(host, port).await
        })
    }
}
