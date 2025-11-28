/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Browser window configuration
//!
//! This module defines the configuration options for browser windows.
//! The `BrowserConfig` struct is part of the stable API.

use std::path::PathBuf;

/// Configuration for a browser window
///
/// This struct is part of the **stable API**. Fields should not be removed,
/// only added with appropriate defaults via `#[serde(default)]`.
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// URL to load (supports transport-aware URLs like `http::unix:///path/`)
    pub url: String,

    /// Window title
    pub title: String,

    /// Window width in pixels
    pub width: u32,

    /// Window height in pixels
    pub height: u32,

    /// Minimum window size (width, height)
    pub min_size: Option<(u32, u32)>,

    /// Maximum window size (width, height)
    pub max_size: Option<(u32, u32)>,

    /// Whether the window can be resized
    pub resizable: bool,

    /// Whether to show window decorations (title bar, borders)
    pub decorated: bool,

    /// Whether to start in fullscreen mode
    pub fullscreen: bool,

    /// Whether to enable developer tools (F12)
    pub devtools: bool,

    /// Custom user agent string (None = default Servo user agent)
    pub user_agent: Option<String>,

    /// Path to userscripts directory (optional)
    pub userscripts_dir: Option<PathBuf>,

    /// Whether to run in headless mode (no visible window)
    pub headless: bool,

    /// Path to output screenshot on exit (headless mode)
    pub screenshot_path: Option<PathBuf>,

    /// Homepage URL (for new tabs, etc.)
    pub homepage: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            url: "about:blank".to_string(),
            title: "Rigging Browser".to_string(),
            width: 1024,
            height: 768,
            min_size: None,
            max_size: None,
            resizable: true,
            decorated: true,
            fullscreen: false,
            devtools: false,
            user_agent: None,
            userscripts_dir: None,
            headless: false,
            screenshot_path: None,
            homepage: None,
        }
    }
}

impl BrowserConfig {
    /// Create a new browser config with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set minimum window size
    pub fn with_min_size(mut self, width: u32, height: u32) -> Self {
        self.min_size = Some((width, height));
        self
    }

    /// Set maximum window size
    pub fn with_max_size(mut self, width: u32, height: u32) -> Self {
        self.max_size = Some((width, height));
        self
    }

    /// Enable/disable resizing
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Enable/disable window decorations
    pub fn with_decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Enable/disable fullscreen
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Enable/disable devtools
    pub fn with_devtools(mut self, devtools: bool) -> Self {
        self.devtools = devtools;
        self
    }

    /// Set custom user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set userscripts directory
    pub fn with_userscripts_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.userscripts_dir = Some(path.into());
        self
    }

    /// Enable headless mode
    pub fn with_headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    /// Set screenshot output path (for headless mode)
    pub fn with_screenshot(mut self, path: impl Into<PathBuf>) -> Self {
        self.screenshot_path = Some(path.into());
        self.headless = true; // Screenshots imply headless
        self
    }

    /// Set homepage URL
    pub fn with_homepage(mut self, url: impl Into<String>) -> Self {
        self.homepage = Some(url.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = BrowserConfig::new("http://localhost/")
            .with_title("Test")
            .with_size(800, 600)
            .with_resizable(false)
            .with_devtools(true);

        assert_eq!(config.url, "http://localhost/");
        assert_eq!(config.title, "Test");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(!config.resizable);
        assert!(config.devtools);
    }

    #[test]
    fn test_config_defaults() {
        let config = BrowserConfig::default();
        assert_eq!(config.url, "about:blank");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert!(config.resizable);
        assert!(config.decorated);
        assert!(!config.fullscreen);
        assert!(!config.devtools);
        assert!(!config.headless);
    }
}
