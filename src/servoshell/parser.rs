/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stub for URL parsing (TODO: port from servoshell)

use servo::ServoUrl;

/// Get default URL (stub implementation)
pub fn get_default_url() -> ServoUrl {
    // TODO: Port URL parsing from servoshell
    ServoUrl::parse("about:blank").unwrap()
}
