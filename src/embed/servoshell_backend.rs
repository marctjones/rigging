/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servoshell-based browser backend
//!
//! This module provides direct Servo embedding using the servoshell code.
//! Unlike servo_backend.rs (subprocess approach), this embeds Servo in-process.

use super::config::BrowserConfig;
use super::events::{BrowserEvent, EventCallback};
use super::EmbedError;
use crate::servoshell::desktop::app::App;
use crate::servoshell::desktop::event_loop::ServoShellEventLoop;
use crate::servoshell::stubs::ServoShellPreferences;
use euclid::Size2D;
use log::{debug, info};
use servo::{Opts, Preferences, ServoUrl};

/// Run the browser using direct servoshell embedding
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
) -> Result<(), EmbedError> {
    // Emit initialization event
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::Initialized);
    }

    info!("Initializing browser (Servoshell direct embedding)...");
    info!("Window: {}x{}", config.width, config.height);
    info!("URL: {}", config.url);

    // Convert transport-aware URL to standard URL for Servo's URL parser only
    // The actual HTTP requests will use the original transport URL via the connector
    let servo_url_str = convert_transport_url_for_parser(&config.url);
    info!("URL for Servo parser: {} -> {}", config.url, servo_url_str);

    // Parse URL using Servo's parser
    let _url = ServoUrl::parse(&servo_url_str)
        .map_err(|e| EmbedError::InitFailed(format!("Invalid URL '{}': {}", servo_url_str, e)))?;

    info!("Parsed URL successfully, proceeding with Servo initialization...");

    // Create servoshell preferences
    let prefs = ServoShellPreferences {
        url: Some(config.url.clone()),
        width: Some(config.width),
        height: Some(config.height),
        headless: config.headless,
        webdriver_port: None,
        exit_after_stable_image: false,
        output_image_path: None,
        initial_window_size: Size2D::new(config.width, config.height),
        device_pixel_ratio_override: None,
        screen_size_override: None,
        userscripts_directory: None,
        use_custom_connector: true, // Enable transport-aware URLs
    };

    // Install default crypto provider (required for TLS/HTTPS)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok(); // Ignore error if already installed

    // Create Servo opts
    let opts = Opts::default();
    let preferences = Preferences::default();

    // Emit window created
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::WindowCreated { window_id: 1 });
    }

    // Create event loop
    let event_loop = if config.headless {
        ServoShellEventLoop::headless()
    } else {
        ServoShellEventLoop::headed()
    };

    // Create app
    info!("Launching servoshell with URL: {}", config.url);
    let mut app = App::new(opts, preferences, prefs, &event_loop);

    info!("Starting event loop...");

    // Run the event loop (this blocks until the window is closed)
    // Note: app.init() will be called automatically by winit via the resumed() callback
    event_loop.run_app(&mut app);

    // Emit close event
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::CloseRequested);
    }

    info!("Servoshell exited");
    Ok(())
}

/// Convert transport-aware URL to standard URL for Servo's parser
///
/// This is only used for Servo's URL parser which doesn't understand transport prefixes.
/// The actual HTTP connector receives the original transport URL and handles it correctly.
///
/// Examples:
/// - `http::unix///tmp/app.sock/` -> `http://localhost/`
/// - `http::tcp//example.com/` -> `http://example.com/`
/// - `http://example.com/` -> `http://example.com/` (unchanged)
fn convert_transport_url_for_parser(url: &str) -> String {
    if url.contains("::unix") {
        // Unix socket URL - use localhost as placeholder for parser
        // The connector will parse the actual socket path from the original URL
        "http://localhost/".to_string()
    } else if url.contains("::tor") {
        // Tor URL - convert to standard URL format
        url.replace("::tor:", "://")
    } else if url.contains("::tcp") {
        // TCP URL - remove transport prefix
        url.replace("::tcp:", "://")
    } else {
        // Standard URL - return as-is
        url.to_string()
    }
}
