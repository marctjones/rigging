/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Browser builder
//!
//! This module provides the `BrowserBuilder` for constructing and running
//! browser instances. This is the main entry point for embedding Servo.

use super::backend;
use super::config::BrowserConfig;
use super::events::{BrowserEvent, EventCallback};
use super::EmbedError;
use log::{debug, info};

/// Builder for creating and running browser instances
///
/// This is the main entry point for embedding Servo in applications.
/// Use the builder pattern to configure the browser, then call `run()`
/// to start the event loop.
///
/// # Example
///
/// ```rust,ignore
/// use rigging::embed::{BrowserBuilder, BrowserConfig};
///
/// BrowserBuilder::new()
///     .config(BrowserConfig::new("http://localhost/"))
///     .on_event(|event| println!("Event: {:?}", event))
///     .run()?;
/// ```
pub struct BrowserBuilder {
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
}

impl Default for BrowserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserBuilder {
    /// Create a new browser builder with default configuration
    pub fn new() -> Self {
        Self {
            config: BrowserConfig::default(),
            event_callback: None,
        }
    }

    /// Set the browser configuration
    pub fn config(mut self, config: BrowserConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the URL to load
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.config.url = url.into();
        self
    }

    /// Set the window title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    /// Set the window size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Enable headless mode
    pub fn headless(mut self) -> Self {
        self.config.headless = true;
        self
    }

    /// Set event callback
    ///
    /// The callback will be invoked for all browser events.
    /// Only one callback can be set; calling this multiple times
    /// will replace the previous callback.
    pub fn on_event<F>(mut self, callback: F) -> Self
    where
        F: Fn(BrowserEvent) + Send + 'static,
    {
        self.event_callback = Some(Box::new(callback));
        self
    }

    /// Build and run the browser
    ///
    /// This method blocks until the browser window is closed.
    /// It will:
    /// 1. Initialize the Servo engine
    /// 2. Create a window with the specified configuration
    /// 3. Load the specified URL
    /// 4. Run the event loop
    /// 5. Return when the window is closed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Servo is not available (feature not enabled)
    /// - Window creation fails
    /// - URL loading fails
    /// - Event loop encounters an error
    pub fn run(self) -> Result<(), EmbedError> {
        info!("Starting browser with URL: {}", self.config.url);
        debug!("Browser config: {:?}", self.config);

        // Validate configuration
        self.validate_config()?;

        // Run the backend implementation
        backend::run_browser(self.config, self.event_callback)
    }

    /// Validate the configuration before running
    fn validate_config(&self) -> Result<(), EmbedError> {
        // Check URL is not empty
        if self.config.url.is_empty() {
            return Err(EmbedError::InvalidUrl("URL cannot be empty".into()));
        }

        // Check window size is reasonable
        if self.config.width == 0 || self.config.height == 0 {
            return Err(EmbedError::WindowFailed(
                "Window size cannot be zero".into(),
            ));
        }

        // Validate min/max size constraints
        if let (Some((min_w, min_h)), Some((max_w, max_h))) =
            (self.config.min_size, self.config.max_size)
        {
            if min_w > max_w || min_h > max_h {
                return Err(EmbedError::WindowFailed(
                    "Minimum size cannot exceed maximum size".into(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = BrowserBuilder::new();
        assert_eq!(builder.config.url, "about:blank");
        assert_eq!(builder.config.width, 1024);
        assert_eq!(builder.config.height, 768);
    }

    #[test]
    fn test_builder_chain() {
        let builder = BrowserBuilder::new()
            .url("http://localhost/")
            .title("Test")
            .size(800, 600)
            .headless();

        assert_eq!(builder.config.url, "http://localhost/");
        assert_eq!(builder.config.title, "Test");
        assert_eq!(builder.config.width, 800);
        assert_eq!(builder.config.height, 600);
        assert!(builder.config.headless);
    }

    #[test]
    fn test_validate_empty_url() {
        let builder = BrowserBuilder::new().url("");
        let result = builder.validate_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_zero_size() {
        let builder = BrowserBuilder::new().size(0, 600);
        let result = builder.validate_config();
        assert!(result.is_err());
    }
}
