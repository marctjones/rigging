/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! TCP connector for HTTP clients
//!
//! Standard TCP/IP connector, mainly for completeness in the transport abstraction.

use crate::types::TransportError;
use futures::future::BoxFuture;
use hyper::Uri;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tower_service::Service;

/// A stream type that wraps TCP connections
pub struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl AsyncRead for TcpConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpConnection {
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

impl hyper::rt::Read for TcpConnection {
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

impl hyper::rt::Write for TcpConnection {
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

/// TCP connector for Hyper HTTP clients
#[derive(Clone, Default)]
pub struct TcpConnector;

impl TcpConnector {
    /// Create a new TCP connector
    pub fn new() -> Self {
        Self
    }

    /// Connect to a host:port
    pub async fn connect(&self, host: &str, port: u16) -> Result<TcpConnection, TransportError> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(TransportError::Io)?;

        Ok(TcpConnection::new(stream))
    }
}

impl Service<Uri> for TcpConnector {
    type Response = TcpConnection;
    type Error = TransportError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
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

            let addr = format!("{}:{}", host, port);
            let stream = TcpStream::connect(&addr)
                .await
                .map_err(TransportError::Io)?;

            Ok(TcpConnection::new(stream))
        })
    }
}
