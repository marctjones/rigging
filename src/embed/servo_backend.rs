/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo browser backend implementation
//!
//! This module provides the Servo-powered browser backend for Rigging.
//! It launches the servo binary as a subprocess to display web content.
//!
//! # Note on Architecture
//!
//! Full Servo embedding requires building Servo's script_bindings which has
//! complex Python/WebIDL dependencies. This implementation uses a subprocess
//! approach as an interim solution while full embedding is developed.

use super::config::BrowserConfig;
use super::events::{BrowserEvent, EventCallback};
use super::EmbedError;
use log::{debug, info, warn};
use std::process::{Command, Stdio};
use std::path::PathBuf;

/// Find the servo binary
fn find_servo_binary() -> Option<PathBuf> {
    // Check common locations
    let candidates = [
        // Development builds
        PathBuf::from("/home/marc/servo/target/release/servo"),
        PathBuf::from("/home/marc/servo/target/debug/servo"),
        // System-wide
        PathBuf::from("/usr/local/bin/servo"),
        PathBuf::from("/usr/bin/servo"),
    ];

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }

    // Try PATH
    if let Ok(output) = Command::new("which").arg("servo").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = PathBuf::from(path_str.trim());
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

/// Run the browser with Servo engine (subprocess approach)
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<EventCallback>,
) -> Result<(), EmbedError> {
    // Emit initialization event
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::Initialized);
    }

    info!("Initializing browser (Servo subprocess backend)...");
    debug!("Window: {}x{}", config.width, config.height);
    debug!("URL: {}", config.url);

    // Find the servo binary
    let servo_path = find_servo_binary()
        .ok_or_else(|| EmbedError::ServoNotAvailable)?;

    info!("Using Servo binary: {}", servo_path.display());

    // Emit window created (we're about to launch)
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::WindowCreated { window_id: 1 });
    }

    // Build the command
    let mut cmd = Command::new(&servo_path);

    // Set screen size (WIDTHxHEIGHT format)
    cmd.arg(format!("--screen-size={}x{}", config.width, config.height));

    // Add the URL
    cmd.arg(&config.url);

    // Inherit stdout/stderr for debugging
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    // Emit load started
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::LoadStateChanged {
            state: super::events::LoadState::Loading,
            url: config.url.clone(),
        });
    }

    info!("Launching Servo with URL: {}", config.url);

    // Run Servo and wait for it to exit
    let status = cmd.status()
        .map_err(|e| EmbedError::InitFailed(format!("Failed to run Servo: {}", e)))?;

    // Emit close/shutdown
    if let Some(ref cb) = event_callback {
        cb(BrowserEvent::CloseRequested);
    }

    if status.success() {
        info!("Servo exited successfully");
        Ok(())
    } else {
        warn!("Servo exited with status: {:?}", status.code());
        // Don't treat non-zero exit as an error - user might have closed the window
        Ok(())
    }
}
