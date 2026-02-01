/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stub for URL parsing (TODO: port from servoshell)

use servo::ServoUrl;

/// Get default URL from optional string
pub fn get_default_url(url: Option<&str>) -> ServoUrl {
    match url {
        Some(url_str) => ServoUrl::parse(url_str).unwrap_or_else(|_| {
            ServoUrl::parse("about:blank").unwrap()
        }),
        None => ServoUrl::parse("about:blank").unwrap(),
    }
}
