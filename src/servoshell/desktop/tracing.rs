/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stub for event tracing (TODO: port from servoshell)

use winit::event::WindowEvent;
use winit::window::WindowId;

/// Trace winit event (stub implementation)
pub fn trace_winit_event(_event: &WindowEvent, _window_id: WindowId) {
    // TODO: Port tracing from servoshell if needed
}
