/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Minimal headed window implementation for Rigging
//!
//! Extracted from servoshell's headed_window.rs with browser chrome removed.
//! No egui, no dialogs, no toolbar - just window + webrender + input handling.

use std::cell::Cell;
use std::rc::Rc;

use euclid::{Point2D, Scale, Size2D};
use log::{info, trace};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use servo::{
    Cursor, DeviceIndependentIntRect, DeviceIndependentPixel, DeviceIntPoint, DeviceIntRect,
    DeviceIntSize, DevicePixel, OffscreenRenderingContext, RenderingContext, ScreenGeometry,
    WebView, WindowRenderingContext, convert_rect_to_css_pixel,
};
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;

use crate::servoshell::stubs::ServoShellPreferences;
use crate::servoshell::running_app_state::RunningAppState;
use crate::servoshell::window::{MIN_WINDOW_INNER_SIZE, PlatformWindow, ServoShellWindow, ServoShellWindowId};

// Helper functions for coordinate conversion (from servoshell's geometry module)
fn winit_position_to_euclid_point(
    position: PhysicalPosition<i32>,
) -> Point2D<i32, DevicePixel> {
    Point2D::new(position.x, position.y)
}

fn winit_size_to_euclid_size(size: PhysicalSize<u32>) -> Size2D<u32, DevicePixel> {
    Size2D::new(size.width, size.height)
}

const INITIAL_WINDOW_TITLE: &str = "Rigging";

#[allow(dead_code)] // Some fields unused but preserved from servoshell API
pub struct Window {
    winit_window: winit::window::Window,
    screen_size: Size2D<u32, DeviceIndependentPixel>,
    monitor: winit::monitor::MonitorHandle,
    inner_size: Cell<PhysicalSize<u32>>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<f32>,
    modifiers_state: Cell<ModifiersState>,
    /// Servo's offscreen rendering context
    rendering_context: Rc<OffscreenRenderingContext>,
    /// Window's direct rendering context
    window_rendering_context: Rc<WindowRenderingContext>,
    webview_relative_mouse_point: Cell<Point2D<f32, DevicePixel>>,
}

#[allow(dead_code)] // Some methods unused but preserved from servoshell API
impl Window {
    pub(crate) fn new(
        servoshell_preferences: &ServoShellPreferences,
        event_loop: &ActiveEventLoop,
    ) -> Rc<Self> {
        info!("Creating headed window");

        let inner_size = servoshell_preferences.initial_window_size;
        trace!("Window size: {}x{}", inner_size.width, inner_size.height);

        let window_attr = winit::window::Window::default_attributes()
            .with_title(INITIAL_WINDOW_TITLE.to_string())
            .with_decorations(true) // Always show decorations in Rigging
            .with_inner_size(LogicalSize::new(inner_size.width, inner_size.height))
            .with_min_inner_size(LogicalSize::new(
                MIN_WINDOW_INNER_SIZE.width,
                MIN_WINDOW_INNER_SIZE.height,
            ))
            .with_position(winit::dpi::LogicalPosition::new(100, 100)) // Explicit position
            .with_visible(true); // Start visible immediately (no accessibility setup needed)

        // Set app ID for Wayland - CRITICAL for proper window identification on GNOME/Wayland
        #[cfg(target_os = "linux")]
        let window_attr = window_attr.with_name("org.rigging.Rigging", "Rigging");

        #[allow(deprecated)]
        let winit_window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window.");

        trace!("Winit window created with ID: {:?}", winit_window.id());

        // Force window to be visible and focused
        winit_window.set_visible(true);
        winit_window.focus_window();

        let monitor = winit_window
            .current_monitor()
            .or_else(|| winit_window.available_monitors().nth(0))
            .expect("No monitor detected");

        trace!("Monitor: {:?}, size: {:?}", monitor.name(), monitor.size());

        let (screen_size, screen_scale) = servoshell_preferences
            .screen_size_override
            .map_or_else(
                || (monitor.size(), winit_window.scale_factor()),
                |size| (PhysicalSize::new(size.width, size.height), 1.0),
            );

        let screen_scale: Scale<f64, DeviceIndependentPixel, DevicePixel> = Scale::new(screen_scale);
        let screen_size = (winit_size_to_euclid_size(screen_size).to_f64() / screen_scale).to_u32();
        let inner_size = winit_window.inner_size();

        // Create rendering contexts
        let display_handle = event_loop
            .display_handle()
            .expect("could not get display handle");
        let window_handle = winit_window
            .window_handle()
            .expect("could not get window handle");

        let window_rendering_context = Rc::new(
            WindowRenderingContext::new(display_handle, window_handle, inner_size)
                .expect("Could not create WindowRenderingContext"),
        );

        let rendering_context = Rc::new(window_rendering_context.offscreen_context(inner_size));

        info!("Window created successfully");

        let window = Rc::new(Window {
            winit_window,
            screen_size,
            monitor,
            inner_size: Cell::new(inner_size),
            fullscreen: Cell::new(false),
            device_pixel_ratio_override: servoshell_preferences.device_pixel_ratio_override,
            modifiers_state: Cell::new(ModifiersState::empty()),
            rendering_context,
            window_rendering_context,
            webview_relative_mouse_point: Cell::new(Point2D::zero()),
        });

        // Show window after all setup is complete
        window.winit_window.set_visible(true);
        window.winit_window.set_minimized(false);
        window.winit_window.set_maximized(true);
        window.winit_window.focus_window();
        window.winit_window.request_redraw();

        window
    }

    fn toolbar_height(&self) -> f32 {
        // No toolbar in Rigging
        0.0
    }

    pub(crate) fn rendering_context(&self) -> Rc<OffscreenRenderingContext> {
        self.rendering_context.clone()
    }

    pub(crate) fn window_rendering_context(&self) -> Rc<WindowRenderingContext> {
        self.window_rendering_context.clone()
    }
}

impl PlatformWindow for Window {
    fn screen_geometry(&self) -> ScreenGeometry {
        let hidpi_factor = self.hidpi_scale_factor();
        let toolbar_size = Size2D::new(0.0, 0.0); // No toolbar
        let screen_size = self.screen_size.to_f32() * hidpi_factor;
        let available_screen_size = screen_size - toolbar_size;

        let window_rect = DeviceIntRect::from_origin_and_size(
            winit_position_to_euclid_point(self.winit_window.outer_position().unwrap_or_default()),
            winit_size_to_euclid_size(self.winit_window.outer_size()).to_i32(),
        );

        ScreenGeometry {
            size: screen_size.to_i32(),
            available_size: available_screen_size.to_i32(),
            window_rect,
        }
    }

    fn device_hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(self.winit_window.scale_factor() as f32)
    }

    fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.device_pixel_ratio_override
            .map(Scale::new)
            .unwrap_or_else(|| self.device_hidpi_scale_factor())
    }

    fn rebuild_user_interface(&self, _: &RunningAppState, _: &ServoShellWindow) {
        // No browser UI to rebuild in Rigging
    }

    fn update_user_interface_state(&self, _: &RunningAppState, window: &ServoShellWindow) -> bool {
        // Update window title from active webview
        if let Some(webview) = window.active_webview() {
            let title = webview
                .page_title()
                .filter(|title| !title.is_empty())
                .map(|title| title.to_string())
                .or_else(|| webview.url().map(|url| url.to_string()))
                .unwrap_or_else(|| INITIAL_WINDOW_TITLE.to_string());

            self.winit_window.set_title(&title);
        }
        false // No UI state changed
    }

    fn request_repaint(&self, _: &ServoShellWindow) {
        trace!("📢 Requesting window redraw from winit");
        self.winit_window.request_redraw();
    }

    fn request_resize(&self, _: &WebView, new_outer_size: DeviceIntSize) -> Option<DeviceIntSize> {
        let inner_size = self.winit_window.inner_size();
        let outer_size = self.winit_window.outer_size();
        let decoration_size: DeviceIntSize = Size2D::new(
            outer_size.height - inner_size.height,
            outer_size.width - inner_size.width,
        )
        .cast();

        let screen_size = (self.screen_size.to_f32() * self.hidpi_scale_factor()).to_i32();
        let new_outer_size =
            new_outer_size.clamp(MIN_WINDOW_INNER_SIZE + decoration_size, screen_size * 2);

        if outer_size.width == new_outer_size.width as u32
            && outer_size.height == new_outer_size.height as u32
        {
            return Some(new_outer_size);
        }

        let new_inner_size = new_outer_size - decoration_size;
        self.winit_window
            .request_inner_size(PhysicalSize::new(
                new_inner_size.width,
                new_inner_size.height,
            ))
            .map(|resulting_size| {
                DeviceIntSize::new(
                    resulting_size.width as i32 + decoration_size.width,
                    resulting_size.height as i32 + decoration_size.height,
                )
            })
    }

    fn window_rect(&self) -> DeviceIndependentIntRect {
        let outer_size = self.winit_window.outer_size();
        let scale = self.hidpi_scale_factor();
        let outer_size = winit_size_to_euclid_size(outer_size).to_i32();

        let origin = self
            .winit_window
            .outer_position()
            .map(winit_position_to_euclid_point)
            .unwrap_or_default();

        convert_rect_to_css_pixel(
            DeviceIntRect::from_origin_and_size(origin, outer_size),
            scale,
        )
    }

    fn set_position(&self, point: DeviceIntPoint) {
        self.winit_window
            .set_outer_position::<PhysicalPosition<i32>>(PhysicalPosition::new(point.x, point.y))
    }

    fn set_fullscreen(&self, state: bool) {
        if self.fullscreen.get() != state {
            self.winit_window.set_fullscreen(if state {
                Some(winit::window::Fullscreen::Borderless(Some(
                    self.monitor.clone(),
                )))
            } else {
                None
            });
        }
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        self.fullscreen.get()
    }

    fn set_cursor(&self, cursor: Cursor) {
        use winit::window::CursorIcon;

        let winit_cursor = match cursor {
            Cursor::Default => CursorIcon::Default,
            Cursor::Pointer => CursorIcon::Pointer,
            Cursor::ContextMenu => CursorIcon::ContextMenu,
            Cursor::Help => CursorIcon::Help,
            Cursor::Progress => CursorIcon::Progress,
            Cursor::Wait => CursorIcon::Wait,
            Cursor::Cell => CursorIcon::Cell,
            Cursor::Crosshair => CursorIcon::Crosshair,
            Cursor::Text => CursorIcon::Text,
            Cursor::VerticalText => CursorIcon::VerticalText,
            Cursor::Alias => CursorIcon::Alias,
            Cursor::Copy => CursorIcon::Copy,
            Cursor::Move => CursorIcon::Move,
            Cursor::NoDrop => CursorIcon::NoDrop,
            Cursor::NotAllowed => CursorIcon::NotAllowed,
            Cursor::Grab => CursorIcon::Grab,
            Cursor::Grabbing => CursorIcon::Grabbing,
            Cursor::EResize => CursorIcon::EResize,
            Cursor::NResize => CursorIcon::NResize,
            Cursor::NeResize => CursorIcon::NeResize,
            Cursor::NwResize => CursorIcon::NwResize,
            Cursor::SResize => CursorIcon::SResize,
            Cursor::SeResize => CursorIcon::SeResize,
            Cursor::SwResize => CursorIcon::SwResize,
            Cursor::WResize => CursorIcon::WResize,
            Cursor::EwResize => CursorIcon::EwResize,
            Cursor::NsResize => CursorIcon::NsResize,
            Cursor::NeswResize => CursorIcon::NeswResize,
            Cursor::NwseResize => CursorIcon::NwseResize,
            Cursor::ColResize => CursorIcon::ColResize,
            Cursor::RowResize => CursorIcon::RowResize,
            Cursor::AllScroll => CursorIcon::AllScroll,
            Cursor::ZoomIn => CursorIcon::ZoomIn,
            Cursor::ZoomOut => CursorIcon::ZoomOut,
            Cursor::None => {
                self.winit_window.set_cursor_visible(false);
                return;
            },
        };
        self.winit_window.set_cursor(winit_cursor);
        self.winit_window.set_cursor_visible(true);
    }

    fn id(&self) -> ServoShellWindowId {
        let id: u64 = self.winit_window.id().into();
        id.into()
    }

    fn maximize(&self, _: &WebView) {
        self.winit_window.set_maximized(true);
    }

    fn rendering_context(&self) -> Rc<dyn RenderingContext> {
        self.rendering_context.clone() as Rc<dyn RenderingContext>
    }

    fn focused(&self) -> bool {
        self.winit_window.has_focus()
    }

    // Override trait's empty default implementation
    fn handle_winit_window_event(
        &self,
        _state: Rc<RunningAppState>,
        window: &ServoShellWindow,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                info!("Window resized to {:?}", new_size);
                self.inner_size.set(new_size);
            }
            WindowEvent::CloseRequested => {
                info!("Window close requested");
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                trace!("Modifiers changed: {:?}", new_modifiers.state());
                self.modifiers_state.set(new_modifiers.state());
            }
            WindowEvent::RedrawRequested => {
                // Paint webview to window
                // WARNING: do not defer painting or presenting to some later tick of the event
                // loop or the window may become unresponsive! (servo#30312)
                trace!("Repainting webviews");
                window.repaint_webviews();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                info!("Scale factor changed to {}", scale_factor);
            }
            WindowEvent::Focused(focused) => {
                info!("Window focus changed: {}", focused);
            }
            WindowEvent::Occluded(occluded) => {
                trace!("Window occluded: {}", occluded);
            }
            ref e => {
                trace!("Window event: {:?}", e);
            }
        }
    }

    fn handle_winit_app_event(
        &self,
        _state: Rc<RunningAppState>,
        _window: &ServoShellWindow,
        _app_event: super::event_loop::AppEvent,
    ) {
        // No custom app event handling needed for Rigging (no egui/accessibility)
    }
}
