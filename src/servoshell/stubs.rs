/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Temporary stubs for servoshell features not yet ported

/// Stub for gamepad support
#[derive(Default)]
pub struct GamepadSupport;

/// Stub for WebDriver embedder controls
#[derive(Default)]
pub struct WebDriverEmbedderControls;

/// Stub for servoshell preferences
#[derive(Default, Clone)]
pub struct ServoShellPreferences {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub headless: bool,
}

impl ServoShellPreferences {
    pub fn new() -> Self {
        Self::default()
    }
}
