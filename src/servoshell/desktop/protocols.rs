/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stub for protocol registration (TODO: port from servoshell)

use servo::protocol_handler::ProtocolRegistry;

// TODO: Port these protocol handlers from servoshell

pub mod urlinfo {
    use servo::protocol_handler::ProtocolHandler;

    #[derive(Default)]
    pub struct UrlInfoProtocolHander;

    impl ProtocolHandler for UrlInfoProtocolHander {
        // Stub implementation
    }
}

pub mod servo {
    use servo::protocol_handler::ProtocolHandler;

    #[derive(Default)]
    pub struct ServoProtocolHandler;

    impl ProtocolHandler for ServoProtocolHandler {
        // Stub implementation
    }
}

pub mod resource {
    use servo::protocol_handler::ProtocolHandler;

    #[derive(Default)]
    pub struct ResourceProtocolHandler;

    impl ProtocolHandler for ResourceProtocolHandler {
        // Stub implementation
    }
}

/// Register custom protocols (stub implementation)
pub fn register_custom_protocols(_registry: &mut ProtocolRegistry) {
    // TODO: Port protocol registration from servoshell
}
