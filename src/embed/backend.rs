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
use log::{info, warn};

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

    // Check for transport-aware URLs - webview backend does not support them
    if config.url.contains("::unix//") || config.url.contains("::tor//") || config.url.contains("::pipe//") {
        return Err(EmbedError::InvalidUrl(
            format!("WebView backend does not support transport-aware URLs (found: {}). Use Servo backend with 'servo' feature.", config.url)
        ));
    }

    let url = &config.url;
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
        .with_url(url)
        .with_devtools(config.devtools)
        .build(&window)
        .map_err(|e| EmbedError::InitFailed(e.to_string()))?;

    info!("Browser window created, entering event loop");

    // Emit load started
    emit_event(
        &event_callback,
        BrowserEvent::LoadStarted {
            url: url.to_string(),
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

/// Helper to emit events if a callback is registered
fn emit_event(callback: &Option<EventCallback>, event: BrowserEvent) {
    if let Some(ref cb) = callback {
        cb(event);
    }
}

