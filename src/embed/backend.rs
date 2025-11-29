/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Browser backend implementation
//!
//! **INTERNAL MODULE** - This module contains the browser engine implementation.
//! When upgrading browser engines, changes should be isolated to this file.
//!
//! The public API in the parent module should remain stable.
//!
//! # Backend Options
//!
//! - `webview` feature: Uses wry/tao for system webview (WebKitGTK on Linux)
//! - `servo` feature: Will use embedded Servo engine (future)

use super::config::BrowserConfig;
use super::events::{BrowserEvent, EventCallback};
use super::EmbedError;
use log::{debug, info, warn};

/// Run the browser with the given configuration
///
/// This is the main entry point for the browser backend.
#[cfg(feature = "webview")]
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
) -> Result<(), EmbedError> {
    use tao::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    use wry::WebViewBuilder;

    // Emit initialization event
    emit_event(&event_callback, BrowserEvent::Initialized);

    info!("Initializing browser (wry/webview backend)...");
    debug!("Window: {}x{}", config.width, config.height);
    debug!("URL: {}", config.url);

    // Convert transport URL to standard URL for webview
    let url = convert_transport_url(&config.url)?;
    info!("Loading URL: {}", url);

    // Create event loop
    let event_loop = EventLoop::new();

    // Build window
    let window = WindowBuilder::new()
        .with_title(&config.title)
        .with_inner_size(tao::dpi::LogicalSize::new(config.width as f64, config.height as f64))
        .with_resizable(config.resizable)
        .with_decorations(config.decorated)
        .build(&event_loop)
        .map_err(|e| EmbedError::WindowFailed(e.to_string()))?;

    // Emit window created event
    emit_event(
        &event_callback,
        BrowserEvent::WindowCreated { window_id: 1 },
    );

    // Build webview
    let _webview = WebViewBuilder::new()
        .with_url(&url)
        .with_devtools(config.devtools)
        .build(&window)
        .map_err(|e| EmbedError::InitFailed(e.to_string()))?;

    info!("Browser window created, entering event loop");

    // Emit load started
    emit_event(
        &event_callback,
        BrowserEvent::LoadStarted {
            url: url.clone(),
        },
    );

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                emit_event(&event_callback, BrowserEvent::Shutdown);
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                debug!("Window resized to {}x{}", size.width, size.height);
            }
            _ => {}
        }
    });
}

/// Run browser with Servo engine
#[cfg(all(feature = "servo", not(feature = "webview")))]
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
) -> Result<(), EmbedError> {
    super::servo_backend::run_browser(config, event_callback)
}

/// Run browser - fallback when no backend is available
#[cfg(not(any(feature = "webview", feature = "servo")))]
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
) -> Result<(), EmbedError> {
    emit_event(&event_callback, BrowserEvent::Initialized);

    warn!("No browser backend available");
    warn!("Enable 'webview' or 'servo' feature in Cargo.toml");
    info!("URL would be loaded: {}", config.url);

    Err(EmbedError::ServoNotAvailable)
}

/// Convert transport-aware URL to standard URL for webview
///
/// Transport URLs like `http::unix///tmp/app.sock/` need to be converted
/// to standard HTTP URLs that the webview can understand.
///
/// For Unix socket URLs, we start a local proxy server that forwards
/// requests to the socket. For now, we just convert to localhost.
fn convert_transport_url(url: &str) -> Result<String, EmbedError> {
    if url.is_empty() {
        return Err(EmbedError::InvalidUrl("URL cannot be empty".into()));
    }

    // Handle transport-aware URLs
    if url.starts_with("http::unix//") {
        // Unix socket URL - extract socket path and URL path
        // Format: http::unix///path/to/socket.sock/url/path
        let rest = &url["http::unix//".len()..];

        // For now, Unix socket URLs need a proxy.
        // Check if it starts with localhost (already converted) or needs conversion
        if rest.starts_with("/") {
            // Absolute socket path - we need to proxy this
            // For the demo, assume a proxy is running on localhost:9999
            // In production, we'd start a proxy automatically
            warn!("Unix socket URL detected: {}", url);
            warn!("Note: System webview cannot directly access Unix sockets");
            warn!("Starting local proxy would be needed for production use");

            // Extract the URL path after the socket
            let socket_and_path = &rest[1..]; // Remove leading /
            if let Some(sock_end) = socket_and_path.find(".sock") {
                let after_sock = &socket_and_path[sock_end + 5..];
                let path = if after_sock.is_empty() { "/" } else { after_sock };
                // Return localhost URL - assumes proxy is running
                return Ok(format!("http://localhost:5000{}", path));
            }
        }

        // Fallback - just use localhost
        Ok("http://localhost:5000/".to_string())
    } else if url.starts_with("http::tcp//") {
        // Explicit TCP - convert to standard URL
        let rest = &url["http::tcp//".len()..];
        Ok(format!("http://{}", rest))
    } else if url.starts_with("https::tcp//") {
        let rest = &url["https::tcp//".len()..];
        Ok(format!("https://{}", rest))
    } else if url.starts_with("http://") || url.starts_with("https://") {
        // Already standard URL
        Ok(url.to_string())
    } else if url.starts_with("about:") {
        // about: URLs - convert to data URL or blank
        if url == "about:blank" {
            Ok("about:blank".to_string())
        } else {
            Ok("about:blank".to_string())
        }
    } else if url.starts_with("file://") {
        // File URLs pass through
        Ok(url.to_string())
    } else {
        // Unknown scheme - try to use as-is
        warn!("Unknown URL scheme: {}", url);
        Ok(url.to_string())
    }
}

/// Helper to emit events if a callback is registered
fn emit_event(callback: &Option<EventCallback>, event: BrowserEvent) {
    if let Some(ref cb) = callback {
        cb(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_transport_url_empty() {
        assert!(convert_transport_url("").is_err());
    }

    #[test]
    fn test_convert_transport_url_http() {
        let result = convert_transport_url("http://localhost/").unwrap();
        assert_eq!(result, "http://localhost/");
    }

    #[test]
    fn test_convert_transport_url_explicit_tcp() {
        let result = convert_transport_url("http::tcp//localhost:8080/").unwrap();
        assert_eq!(result, "http://localhost:8080/");
    }

    #[test]
    fn test_convert_transport_url_about() {
        let result = convert_transport_url("about:blank").unwrap();
        assert_eq!(result, "about:blank");
    }
}
