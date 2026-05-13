/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Resource reader for Servo - loads fonts, UA stylesheets, etc.
//!
//! This is critical for rendering. Without it, Servo can't load:
//! - Default fonts
//! - User agent stylesheets (CSS)
//! - Other built-in resources

use std::path::PathBuf;
use std::sync::Mutex;
use std::{env, fs};

use servo::resources::{self, Resource};

static CMD_RESOURCE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

struct ResourceReader;

/// Initialize the resource reader - MUST be called before creating Servo
pub fn init() {
    eprintln!("[RESOURCES] Initializing resource reader...");
    resources::set(Box::new(ResourceReader));
    // Test that resources can be found
    let dir = resources_dir_path();
    eprintln!("[RESOURCES] ✓ Resource directory: {}", dir.display());
}

pub(crate) fn resources_dir_path() -> PathBuf {
    // This needs to be called before the process is sandboxed
    // as we only give permission to read inside the resources directory,
    // not the permissions the "search" for the resources directory.
    let mut dir = CMD_RESOURCE_DIR.lock().unwrap();
    if let Some(ref path) = *dir {
        return PathBuf::from(path);
    }

    // Try ./resources and ./Resources relative to the directory containing the
    // canonicalised executable path, then each of its ancestors.
    let mut path = env::current_exe().unwrap().canonicalize().unwrap();
    while path.pop() {
        path.push("resources");
        if path.is_dir() {
            *dir = Some(path);
            return dir.clone().unwrap();
        }
        path.pop();

        // Check for Resources on mac when using a case sensitive filesystem.
        path.push("Resources");
        if path.is_dir() {
            *dir = Some(path);
            return dir.clone().unwrap();
        }
        path.pop();
    }

    // Try ./resources in the current directory, then each of its ancestors.
    // This is for development builds - Rigging is not intended for production distribution yet
    let mut path = std::env::current_dir().unwrap();
    loop {
        path.push("resources");
        if path.is_dir() {
            eprintln!("[RESOURCES] Found resources at: {}", path.display());
            *dir = Some(path);
            return dir.clone().unwrap();
        }
        path.pop();

        if !path.pop() {
            panic!("Can't find resources directory. Please create a symlink: ln -s /path/to/servo/resources ./resources")
        }
    }
}

impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let mut path = resources_dir_path();
        path.push(file.filename());
        eprintln!("[RESOURCES] Reading resource: {}", path.display());
        fs::read(&path).unwrap_or_else(|e| {
            eprintln!("[RESOURCES] ⚠️  Failed to read {}: {}", path.display(), e);
            vec![]
        })
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![resources_dir_path()]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}
