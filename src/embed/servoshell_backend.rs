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
use log::{debug, info, warn};
use servo::{DeviceIndependentPixel, Opts, Preferences, ServoUrl};

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
    debug!("Window: {}x{}", config.width, config.height);
    debug!("URL: {}", config.url);

    // Parse URL
    let url = ServoUrl::parse(&config.url)
        .map_err(|e| EmbedError::InitFailed(format!("Invalid URL: {}", e)))?;

    // Create servoshell preferences
    let mut prefs = ServoShellPreferences {
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

    // TODO: Properly integrate event_callback into servoshell event loop
    // For now, just run the app (this blocks until shutdown)
    // Note: run_app() doesn't exist yet - we need to implement it
    // For now, just initialize and return
    app.init(None);

    // TODO: Run the event loop
    warn!("Servoshell event loop not yet implemented - initialization only");

    // Emit close
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::CloseRequested);
    }

    info!("Servoshell exited");
    Ok(())
}
