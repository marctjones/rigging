/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application entry point, runs the event loop.

use std::path::Path;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use log::{info, debug, trace};
use servo::protocol_handler::ProtocolRegistry;
use servo::Opts;
use servo::Preferences;
use servo::ServoUrl;
use servo::{UserContentManager, UserScript};
use servo::{EventLoopWaker, ServoBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use winit::window::WindowId;

use super::event_loop::AppEvent;
use super::{headed_window, headless_window};
use crate::servoshell::desktop::event_loop::ServoShellEventLoop;
use crate::servoshell::desktop::protocols;
use crate::servoshell::desktop::tracing::trace_winit_event;
use crate::servoshell::parser::get_default_url;
use crate::servoshell::stubs::ServoShellPreferences;
use crate::servoshell::running_app_state::RunningAppState;
use crate::servoshell::window::PlatformWindow;

pub(crate) enum AppState {
    Initializing,
    Running(Rc<RunningAppState>),
    ShuttingDown,
}

pub struct App {
    opts: Opts,
    preferences: Preferences,
    servoshell_preferences: ServoShellPreferences,
    waker: Box<dyn EventLoopWaker>,
    event_loop_proxy: Option<EventLoopProxy<AppEvent>>,
    initial_url: ServoUrl,
    t_start: Instant,
    t: Instant,
    state: AppState,
}

impl App {
    pub fn new(
        opts: Opts,
        preferences: Preferences,
        servo_shell_preferences: ServoShellPreferences,
        event_loop: &ServoShellEventLoop,
    ) -> Self {
        let initial_url = get_default_url(servo_shell_preferences.url.as_deref());

        let t = Instant::now();
        App {
            opts,
            preferences,
            servoshell_preferences: servo_shell_preferences,
            waker: event_loop.create_event_loop_waker(),
            event_loop_proxy: event_loop.event_loop_proxy(),
            initial_url: initial_url.clone(),
            t_start: t,
            t,
            state: AppState::Initializing,
        }
    }

    /// Initialize Application once event loop start running.
    pub fn init(&mut self, active_event_loop: Option<&ActiveEventLoop>) {
        // Use println! because logging isn't set up yet
        println!("\n🚀 Initializing application...");
        println!("   Mode: {}", if active_event_loop.is_some() { "HEADED (GUI)" } else { "HEADLESS" });
        println!("   Initial URL: {}", self.initial_url);

        // TODO: Port userscripts support once API is available
        // let mut user_content_manager = UserContentManager::new();
        // for script in load_userscripts(self.servoshell_preferences.userscripts_directory.as_deref())
        //     .expect("Loading userscripts failed")
        // {
        //     user_content_manager.add_script(script);
        // }

        // TODO: Port protocol handlers once properly exposed
        // let mut protocol_registry = ProtocolRegistry::default();
        // let _ = protocol_registry.register(
        //     "urlinfo",
        //     protocols::urlinfo::UrlInfoProtocolHander::default(),
        // );
        // let _ =
        //     protocol_registry.register("servo", protocols::servo::ServoProtocolHandler::default());
        // let _ = protocol_registry.register(
        //     "resource",
        //     protocols::resource::ResourceProtocolHandler::default(),
        // );

        info!("   Building Servo...");
        let mut servo_builder = ServoBuilder::default()
            .opts(self.opts.clone())
            .preferences(self.preferences.clone())
            // .user_content_manager(user_content_manager)  // TODO: API changed
            // .protocol_registry(protocol_registry)  // TODO: Add back once protocols work
            .event_loop_waker(self.waker.clone());

        // Inject custom connector if requested (for transport-aware URLs)
        #[cfg(feature = "unix")]
        if self.servoshell_preferences.use_custom_connector {
            info!("   Enabling custom Unix connector");
            use crate::unix_connector::UnixConnector;
            let connector = UnixConnector::dynamic();
            servo_builder = servo_builder.with_connector(connector);
        }

        info!("   Creating platform window...");
        let platform_window =
            self.create_platform_window(self.initial_url.clone(), active_event_loop);
        info!("   ✓ Platform window created");

        #[cfg(feature = "webxr")]
        let servo_builder =
            servo_builder.webxr_registry(super::webxr::XrDiscoveryWebXrRegistry::new_boxed(
                platform_window.clone(),
                active_event_loop,
                &self.preferences,
            ));

        info!("   Building Servo instance...");
        let servo = servo_builder.build();
        servo.setup_logging();
        info!("   ✓ Servo built and logging configured");

        info!("   Creating running state and initial window...");
        let running_state = Rc::new(RunningAppState::new(
            servo,
            self.servoshell_preferences.clone(),
            self.waker.clone(),
        ));
        running_state.create_window(platform_window, self.initial_url.as_url().clone());
        info!("   ✓ Window created with initial URL");

        self.state = AppState::Running(running_state);
        info!("✅ Application initialization complete!");
    }

    fn create_platform_window(
        &self,
        url: ServoUrl,
        active_event_loop: Option<&ActiveEventLoop>,
    ) -> Rc<dyn PlatformWindow> {
        assert_eq!(
            self.servoshell_preferences.headless,
            active_event_loop.is_none()
        );

        if self.servoshell_preferences.headless {
            // Headless mode - no window
            headless_window::Window::new(&self.servoshell_preferences)
        } else {
            // Headed mode - create actual window
            let event_loop = active_event_loop.expect("Headed mode requires ActiveEventLoop");
            headed_window::Window::new(&self.servoshell_preferences, event_loop)
        }
    }

    pub fn pump_servo_event_loop(&mut self) -> bool {
        let AppState::Running(state) = &self.state else {
            trace!("⚠️  Pump called but not in Running state");
            return false;
        };

        trace!("🔄 Pumping Servo event loop...");
        if !state.spin_event_loop() {
            info!("🛑 Servo event loop returned false, shutting down");
            self.state = AppState::ShuttingDown;
            return false;
        }
        trace!("   ✓ Event loop pump complete");
        true
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("▶️  Event loop resumed, initializing...");
        self.init(Some(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        window_event: WindowEvent,
    ) {
        let now = Instant::now();
        trace!("⚡ window_event received: {:?} for window {:?}", window_event, window_id);
        trace_winit_event(&window_event, window_id);
        self.t = now;

        {
            let AppState::Running(state) = &self.state else {
                trace!("   Ignoring: App not in Running state");
                return;
            };
            let window_id: u64 = window_id.into();
            if let Some(window) = state.window(window_id.into()) {
                trace!("   Dispatching to window handler");
                window.platform_window().handle_winit_window_event(
                    state.clone(),
                    &window,
                    window_event,
                );
            } else {
                trace!("   ⚠️  No window found for ID: {}", window_id);
            }
        }

        if !self.pump_servo_event_loop() {
            event_loop.exit();
        }
        // Block until the window gets an event
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, app_event: AppEvent) {
        {
            let AppState::Running(state) = &self.state else {
                return;
            };
            if let Some(window_id) = app_event.window_id() {
                let window_id: u64 = window_id.into();
                if let Some(window) = state.window(window_id.into()) {
                    window.platform_window().handle_winit_app_event(
                        state.clone(),
                        &window,
                        app_event,
                    );
                }
            }
        }

        if !self.pump_servo_event_loop() {
            event_loop.exit();
        }

        // Block until the window gets an event
        event_loop.set_control_flow(ControlFlow::Wait);
    }
}

fn load_userscripts(userscripts_directory: Option<&Path>) -> std::io::Result<Vec<UserScript>> {
    let mut userscripts = Vec::new();
    if let Some(userscripts_directory) = &userscripts_directory {
        let mut files = std::fs::read_dir(userscripts_directory)?
            .map(|e| e.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;
        files.sort_unstable();
        for file in files {
            userscripts.push(UserScript {
                script: std::fs::read_to_string(&file)?,
                source_file: Some(file),
            });
        }
    }
    Ok(userscripts)
}
