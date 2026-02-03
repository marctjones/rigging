/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stub for URL parsing (TODO: port from servoshell)

use servo::ServoUrl;

/// Get default URL from optional string
///
/// Handles transport-aware URLs by converting them for Servo's parser.
/// The actual HTTP requests will use the original transport URL via the connector.
pub fn get_default_url(url: Option<&str>) -> ServoUrl {
    match url {
        Some(url_str) => {
            // Convert transport-aware URLs for Servo's parser
            let converted_url = convert_transport_url_for_parser(url_str);

            eprintln!("[Parser] Original URL: {}", url_str);
            eprintln!("[Parser] Converted URL: {}", converted_url);

            ServoUrl::parse(&converted_url).unwrap_or_else(|e| {
                eprintln!("[Parser] Failed to parse URL '{}': {}", converted_url, e);
                ServoUrl::parse("about:blank").unwrap()
            })
        }
        None => ServoUrl::parse("about:blank").unwrap(),
    }
}

/// Convert transport-aware URL to standard URL for Servo's parser
///
/// This is only used for Servo's URL parser which doesn't understand transport prefixes.
/// The actual HTTP connector receives the original transport URL and handles it correctly.
fn convert_transport_url_for_parser(url: &str) -> String {
    if url.contains("::unix") {
        // Unix socket URL - use localhost as placeholder for parser
        // The connector will parse the actual socket path from the original URL
        "http://localhost/".to_string()
    } else if url.contains("::tor") {
        // Tor URL - convert to standard URL format
        url.replace("::tor:", "://")
    } else if url.contains("::tcp") {
        // TCP URL - remove transport prefix
        url.replace("::tcp:", "://")
    } else {
        // Standard URL - return as-is
        url.to_string()
    }
}
