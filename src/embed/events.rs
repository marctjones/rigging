/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Browser events
//!
//! This module defines events that can occur during browser operation.
//! These types are part of the stable API.

/// Events emitted by the browser during operation
///
/// This enum is part of the **stable API**. Variants should not be removed,
/// only added. Applications should handle unknown variants gracefully.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum BrowserEvent {
    /// Browser engine initialized successfully
    Initialized,

    /// Window was created
    WindowCreated {
        /// Window ID (for multi-window support)
        window_id: u64,
    },

    /// Window was closed
    WindowClosed {
        /// Window ID
        window_id: u64,
    },

    /// Navigation event occurred
    Navigation(NavigationEvent),

    /// Page load state changed
    LoadStateChanged {
        /// Current load state
        state: LoadState,
        /// URL being loaded
        url: String,
    },

    /// Page title changed
    TitleChanged {
        /// New title
        title: String,
    },

    /// Favicon changed
    FaviconChanged {
        /// URL of the new favicon
        url: Option<String>,
    },

    /// User requested to close the browser
    CloseRequested,

    /// Browser encountered an error
    Error {
        /// Error message
        message: String,
        /// Whether the error is recoverable
        recoverable: bool,
    },

    /// Console message from the page
    ConsoleMessage {
        /// Log level
        level: ConsoleLevel,
        /// Message text
        message: String,
        /// Source file (if available)
        source: Option<String>,
        /// Line number (if available)
        line: Option<u32>,
    },

    /// JavaScript alert/confirm/prompt
    Alert {
        /// Alert message
        message: String,
    },
}

/// Navigation events
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum NavigationEvent {
    /// Navigation started
    Started {
        /// Target URL
        url: String,
    },

    /// Navigation completed successfully
    Completed {
        /// Final URL (may differ from started URL due to redirects)
        url: String,
    },

    /// Navigation failed
    Failed {
        /// URL that failed to load
        url: String,
        /// Error message
        error: String,
    },

    /// Navigation was cancelled
    Cancelled {
        /// URL that was being loaded
        url: String,
    },
}

/// Page load states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    /// Initial state, nothing loaded
    Initial,
    /// Loading started
    Loading,
    /// Main content loaded (DOMContentLoaded equivalent)
    Interactive,
    /// All resources loaded
    Complete,
    /// Load failed
    Failed,
}

/// Console message log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleLevel {
    /// Debug message
    Debug,
    /// Informational message
    Info,
    /// Warning message
    Warn,
    /// Error message
    Error,
}

/// Callback type for browser events
pub type EventCallback = Box<dyn Fn(BrowserEvent) + Send + 'static>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_event_debug() {
        let event = BrowserEvent::TitleChanged {
            title: "Test Page".to_string(),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("TitleChanged"));
        assert!(debug_str.contains("Test Page"));
    }

    #[test]
    fn test_load_state_equality() {
        assert_eq!(LoadState::Loading, LoadState::Loading);
        assert_ne!(LoadState::Loading, LoadState::Complete);
    }
}
