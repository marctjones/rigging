/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Minimal headed window implementation for Rigging
//!
//! TODO: Extract core ~300 lines from servoshell's headed_window.rs:
//! - PlatformWindow trait implementation (~200 lines)
//! - winit/surfman setup code (~100 lines)
//! - Event handling code (~100 lines)
//!
//! REMOVED from servoshell version:
//! - ALL egui code (~700+ lines) - No browser chrome in Rigging
//! - Dialog display logic - No browser dialogs
//! - Toolbar/URL bar integration - Applications provide their own UI
//!
//! See servoshell/desktop/headed_window.rs (1,350 lines) for full implementation.

// Placeholder - to be implemented
pub struct Window {
    // TODO: Add minimal window fields
}

impl Window {
    pub fn new() -> Self {
        todo!("Extract headed_window implementation from servoshell")
    }
}
