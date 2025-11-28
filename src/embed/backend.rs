/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo backend implementation
//!
//! **INTERNAL MODULE** - This module contains the Servo-specific implementation.
//! When upgrading Servo, changes should be isolated to this file.
//!
//! The public API in the parent module should remain stable.

use super::config::BrowserConfig;
use super::events::{BrowserEvent, EventCallback};
use super::EmbedError;
use log::{debug, info, warn};

/// Run the browser with the given configuration
///
/// This is the main entry point for the Servo backend. It handles:
/// 1. Creating the event loop
/// 2. Initializing Servo
/// 3. Creating the window
/// 4. Loading the URL
/// 5. Running the event loop until exit
///
/// # Implementation Notes
///
/// When Servo is fully integrated, this function will:
/// - Use `winit::event_loop::EventLoop` for the main loop
/// - Use `servo::ServoBuilder` to create the Servo instance
/// - Create a `PlatformWindow` for rendering
/// - Handle window events and forward them to Servo
///
/// The current implementation is a placeholder that demonstrates
/// the expected behavior.
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
) -> Result<(), EmbedError> {
    // Emit initialization event
    emit_event(&event_callback, BrowserEvent::Initialized);

    info!("Initializing Servo browser engine...");
    debug!("Window: {}x{}", config.width, config.height);
    debug!("URL: {}", config.url);

    // Validate transport URL
    validate_transport_url(&config.url)?;

    // TODO: Full Servo implementation
    //
    // When Servo is added as a dependency, this will use code like:
    //
    // ```rust
    // use servo::config::opts::Opts;
    // use servo::config::prefs::Preferences;
    // use servo::ServoBuilder;
    // use winit::event_loop::EventLoop;
    // use winit::window::WindowBuilder;
    //
    // // Create event loop
    // let event_loop = EventLoop::new()
    //     .map_err(|e| EmbedError::EventLoopError(e.to_string()))?;
    //
    // // Configure Servo
    // let opts = Opts::default();
    // let prefs = Preferences::default();
    //
    // // Build Servo
    // let servo = ServoBuilder::default()
    //     .opts(opts)
    //     .preferences(prefs)
    //     .build();
    //
    // // Create window
    // let window = WindowBuilder::new()
    //     .with_title(&config.title)
    //     .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height))
    //     .with_resizable(config.resizable)
    //     .with_decorations(config.decorated)
    //     .build(&event_loop)
    //     .map_err(|e| EmbedError::WindowFailed(e.to_string()))?;
    //
    // // Run event loop
    // event_loop.run(move |event, target| {
    //     // Handle events...
    // });
    // ```

    // For now, emit window created event and return error
    emit_event(
        &event_callback,
        BrowserEvent::WindowCreated { window_id: 1 },
    );

    warn!("Servo integration pending - browser window not yet implemented");
    info!("URL would be loaded: {}", config.url);

    // Return error indicating implementation is pending
    Err(EmbedError::ServoNotAvailable)
}

/// Validate a transport-aware URL
fn validate_transport_url(url: &str) -> Result<(), EmbedError> {
    // Basic validation - detailed parsing happens when Servo loads the URL

    if url.is_empty() {
        return Err(EmbedError::InvalidUrl("URL cannot be empty".into()));
    }

    // Log transport type for debugging
    if url.starts_with("http::unix//") {
        debug!("Transport: Unix Domain Socket");
    } else if url.starts_with("http::tcp//") {
        debug!("Transport: TCP (explicit)");
    } else if url.starts_with("http::tor//") {
        debug!("Transport: Tor");
    } else if url.starts_with("http://") || url.starts_with("https://") {
        debug!("Transport: TCP (implicit)");
    } else if url.starts_with("about:") || url.starts_with("file://") {
        debug!("Transport: Local");
    } else {
        warn!("Unknown URL scheme, Servo will attempt to parse: {}", url);
    }

    Ok(())
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
    fn test_validate_transport_url_empty() {
        assert!(validate_transport_url("").is_err());
    }

    #[test]
    fn test_validate_transport_url_http() {
        assert!(validate_transport_url("http://localhost/").is_ok());
    }

    #[test]
    fn test_validate_transport_url_unix() {
        assert!(validate_transport_url("http::unix///tmp/app.sock/").is_ok());
    }

    #[test]
    fn test_validate_transport_url_about() {
        assert!(validate_transport_url("about:blank").is_ok());
    }
}
