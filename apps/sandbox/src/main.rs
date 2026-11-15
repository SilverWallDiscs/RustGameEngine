//! Native sandbox application wiring windowing, input, time, and rendering.

use std::{sync::Arc, time::{Duration, Instant}};
use anyhow::{Context, Result};
use engine_core::{init_logging, EngineConfig, GameLoop};
use engine_input::InputState;
use engine_renderer::{ClearColor, Renderer};
use tracing::{error, info};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct SandboxApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: InputState,
    game_loop: GameLoop,
    failure: Option<anyhow::Error>,
}

impl SandboxApp {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            input: InputState::default(),
            game_loop: GameLoop::new(EngineConfig::default())
                .expect("the built-in engine timing configuration is valid"),
            failure: None,
        }
    }
}

impl ApplicationHandler for SandboxApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = Window::default_attributes()
            .with_title("Rust Game Engine | Sandbox")
            .with_inner_size(PhysicalSize::new(1280, 720));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.failure = Some(error.into());
                event_loop.exit();
                return;
            }
        };
        match pollster::block_on(Renderer::new(window.clone())) {
            Ok(renderer) => {
                info!(adapter = renderer.adapter_name(), "sandbox renderer ready");
                self.renderer = Some(renderer);
                self.window = Some(window);
            }
            Err(error) => {
                self.failure = Some(error.into());
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else { return; };
        if window.id() != id { return; }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.game_loop.tick(|_fixed_delta| {}, |_frame| {});
                if let Some(renderer) = self.renderer.as_mut() {
                    if let Err(error) = renderer.render(ClearColor::default()) {
                        error!(%error, "rendering failed");
                        self.failure = Some(error.into());
                        event_loop.exit();
                    }
                }
                self.input.end_frame();
            }
            other => self.input.process_window_event(&other),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(16)
            ));
        }
    }
}

fn main() -> Result<()> {
    init_logging();
    let event_loop = EventLoop::new().context("failed to create winit event loop")?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = SandboxApp::new();
    event_loop.run_app(&mut app).context("winit event loop failed")?;
    if let Some(error) = app.failure { return Err(error); }
    Ok(())
}
