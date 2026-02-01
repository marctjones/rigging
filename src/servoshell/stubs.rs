/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Temporary stubs for servoshell features not yet ported

use euclid::Size2D;
use servo::DeviceIndependentPixel;

/// Stub for gamepad support
#[derive(Default)]
pub struct GamepadSupport;

/// Stub for WebDriver embedder controls
#[derive(Default)]
pub struct WebDriverEmbedderControls;

impl WebDriverEmbedderControls {
    pub fn show_embedder_control(
        &mut self,
        _webview_id: servo::WebViewId,
        _embedder_control: servo::EmbedderControl,
    ) {
        // Stub: no-op
    }

    pub fn hide_embedder_control(
        &mut self,
        _webview_id: servo::WebViewId,
        _embedder_control_id: servo::EmbedderControlId,
    ) {
        // Stub: no-op
    }
}

/// Stub for servoshell preferences
#[derive(Clone)]
pub struct ServoShellPreferences {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub headless: bool,
    pub webdriver_port: Option<u16>,
    pub exit_after_stable_image: bool,
    pub output_image_path: Option<std::path::PathBuf>,
    pub initial_window_size: Size2D<u32, DeviceIndependentPixel>,
    pub device_pixel_ratio_override: Option<f32>,
    pub screen_size_override: Option<Size2D<u32, DeviceIndependentPixel>>,
    pub userscripts_directory: Option<std::path::PathBuf>,
    /// Enable custom connector (for transport-aware URLs)
    pub use_custom_connector: bool,
}

impl Default for ServoShellPreferences {
    fn default() -> Self {
        Self {
            url: None,
            width: Some(800),
            height: Some(600),
            headless: false,
            webdriver_port: None,
            exit_after_stable_image: false,
            output_image_path: None,
            initial_window_size: Size2D::new(800, 600),
            device_pixel_ratio_override: None,
            screen_size_override: None,
            userscripts_directory: None,
            use_custom_connector: false,
        }
    }
}

impl ServoShellPreferences {
    pub fn new() -> Self {
        Self::default()
    }
}

impl GamepadSupport {
    pub fn maybe_new() -> Option<Self> {
        None // Stub: no gamepad support
    }

    pub fn handle_gamepad_events(&mut self) {
        // Stub: no-op
    }

    #[allow(dead_code)]
    pub fn play_haptic_effect(&mut self, _index: usize) {
        // Stub: no-op (GamepadHapticEffectType not exported, so simplified signature)
    }
}
