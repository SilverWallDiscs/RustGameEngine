#![forbid(unsafe_code)]
//! wgpu graphics device and presentation surface abstraction.
//!
//! Phase 1 selects an adapter, configures a surface, handles resizing and transient
//! surface states, then clears each frame. Render graphs, pipelines, PBR, and post-effects
//! are subsequent renderer phases.
//!
//! # Examples
//!
//! ~~~rust,no_run
//! use std::sync::Arc;
//! use engine_renderer::Renderer;
//! use winit::{event_loop::OwnedDisplayHandle, window::Window};
//! async fn init(window: Arc<Window>, display: OwnedDisplayHandle) {
//!     let renderer = Renderer::new(window, display).await.unwrap();
//!     println!("{}", renderer.adapter_name());
//! }
//! ~~~

use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};
use winit::{event_loop::OwnedDisplayHandle, window::Window};

/// Linear RGBA clear color.
///
/// # Examples
///
/// ~~~rust
/// use engine_renderer::ClearColor;
/// let color = ClearColor::new(0.02, 0.03, 0.06, 1.0);
/// assert_eq!(color.a, 1.0);
/// ~~~
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClearColor {
    /// Red component from 0.0 to 1.0.
    pub r: f64,
    /// Green component from 0.0 to 1.0.
    pub g: f64,
    /// Blue component from 0.0 to 1.0.
    pub b: f64,
    /// Alpha component from 0.0 to 1.0.
    pub a: f64,
}
impl ClearColor {
    /// Constructs an RGBA clear color.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_renderer::ClearColor;
    /// let white = ClearColor::new(1.0, 1.0, 1.0, 1.0);
    /// assert_eq!(white.r, 1.0);
    /// ~~~
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self { Self { r, g, b, a } }
}
impl Default for ClearColor {
    fn default() -> Self { Self::new(0.035, 0.045, 0.075, 1.0) }
}

/// Errors returned by renderer initialization or presentation.
///
/// # Examples
///
/// ~~~rust
/// use engine_renderer::RendererError;
/// fn message(error: RendererError) -> String { error.to_string() }
/// ~~~
#[derive(Debug, Error)]
pub enum RendererError {
    /// Surface creation failed.
    #[error("failed to create wgpu surface: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    /// No adapter supports the surface.
    #[error("no compatible graphics adapter was found: {0}")]
    Adapter(String),
    /// Device creation failed.
    #[error("failed to create wgpu device: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    /// Adapter cannot present to the selected surface.
    #[error("selected graphics adapter does not support this surface")]
    UnsupportedSurface,
    /// Surface acquisition reported a validation error.
    #[error("wgpu reported a validation error while acquiring the surface texture")]
    SurfaceValidation,
}

/// Result of trying to render and present one frame.
///
/// # Examples
///
/// ~~~rust
/// use engine_renderer::FrameStatus;
/// assert_eq!(FrameStatus::Presented, FrameStatus::Presented);
/// ~~~
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameStatus {
    /// Frame cleared, submitted, and presented.
    Presented,
    /// Frame skipped because the surface is occluded or timed out.
    Skipped,
    /// Surface reconfigured; a later frame can proceed.
    Reconfigured,
}

/// Minimal wgpu renderer owning a safe static-lifetime surface.
///
/// Passing an Arc window into wgpu allows the surface to retain its native handle source
/// without a self-referential struct or unsafe raw-window-handle code.
///
/// # Examples
///
/// ~~~rust,no_run
/// use std::sync::Arc;
/// use engine_renderer::Renderer;
/// use winit::window::Window;
/// async fn create(window: Arc<Window>) {
///     let renderer = Renderer::new(window).await.unwrap();
///     println!("GPU: {}", renderer.adapter_name());
/// }
/// ~~~
pub struct Renderer {
    instance: wgpu::Instance,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    adapter_name: String,
}
impl Renderer {
    /// Creates a compatible adapter, device, and configured presentation surface.
    ///
    /// # Examples
    ///
    /// ~~~rust,no_run
    /// use std::sync::Arc;
    /// use engine_renderer::Renderer;
    /// use winit::window::Window;
    /// async fn init(window: Arc<Window>) {
    ///     let _renderer = Renderer::new(window).await.unwrap();
    /// }
    /// ~~~
    pub async fn new(
        window: Arc<Window>,
        display: OwnedDisplayHandle,
    ) -> Result<Self, RendererError> {
        let size = window.inner_size();
        let descriptor = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display));
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.map_err(|error| RendererError::Adapter(error.to_string()))?;
        let info = adapter.get_info();
        info!(adapter = %info.name, backend = ?info.backend, "graphics adapter selected");

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Rust Game Engine device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }).await?;

        let config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or(RendererError::UnsupportedSurface)?;
        surface.configure(&device, &config);
        Ok(Self {
            instance, window, surface, device, queue, config,
            adapter_name: format!("{} ({:?})", info.name, info.backend),
        })
    }

    /// Resizes the surface using physical pixel dimensions; zero dimensions are ignored.
    ///
    /// # Examples
    ///
    /// ~~~rust,no_run
    /// # use std::sync::Arc;
    /// # use engine_renderer::Renderer;
    /// # use winit::window::Window;
    /// # async fn example(window: Arc<Window>, display: OwnedDisplayHandle) {
    /// let mut renderer = Renderer::new(window, display).await.unwrap();
    /// renderer.resize(1280, 720);
    /// # }
    /// ~~~
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }
        if self.config.width == width && self.config.height == height { return; }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Clears the acquired swapchain image and presents it.
    ///
    /// # Examples
    ///
    /// ~~~rust,no_run
    /// # use std::sync::Arc;
    /// # use engine_renderer::{ClearColor, Renderer};
    /// # use winit::{event_loop::OwnedDisplayHandle, window::Window};
    /// # async fn example(window: Arc<Window>) {
    /// let mut renderer = Renderer::new(window).await.unwrap();
    /// let _status = renderer.render(ClearColor::default());
    /// # }
    /// ~~~
    pub fn render(&mut self, color: ClearColor) -> Result<FrameStatus, RendererError> {
        let (frame, suboptimal) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Outdated => {
                warn!("surface configuration outdated; reconfiguring");
                self.surface.configure(&self.device, &self.config);
                return Ok(FrameStatus::Reconfigured);
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                warn!("surface lost; recreating from the retained window handle");
                let surface = self.instance.create_surface(self.window.clone())?;
                surface.configure(&self.device, &self.config);
                self.surface = surface;
                return Ok(FrameStatus::Reconfigured);
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(FrameStatus::Skipped);
            }
            wgpu::CurrentSurfaceTexture::Validation => return Err(RendererError::SurfaceValidation),
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Rust Game Engine clear pass"),
        });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Rust Game Engine clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: color.r, g: color.g, b: color.b, a: color.a,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);

        if suboptimal {
            self.surface.configure(&self.device, &self.config);
            Ok(FrameStatus::Reconfigured)
        } else {
            Ok(FrameStatus::Presented)
        }
    }

    /// Returns the selected graphics adapter name and backend.
    ///
    /// # Examples
    ///
    /// ~~~rust,no_run
    /// # use std::sync::Arc;
    /// # use engine_renderer::Renderer;
    /// # use winit::window::Window;
    /// # async fn example(window: Arc<Window>, display: OwnedDisplayHandle) {
    /// let renderer = Renderer::new(window, display).await.unwrap();
    /// println!("{}", renderer.adapter_name());
    /// # }
    /// ~~~
    pub fn adapter_name(&self) -> &str { &self.adapter_name }

    /// Returns the surface pixel format selected by wgpu.
    ///
    /// # Examples
    ///
    /// ~~~rust,no_run
    /// # use std::sync::Arc;
    /// # use engine_renderer::Renderer;
    /// # use winit::window::Window;
    /// # async fn example(window: Arc<Window>) {
    /// let renderer = Renderer::new(window).await.unwrap();
    /// let _format = renderer.surface_format();
    /// # }
    /// ~~~
    pub fn surface_format(&self) -> wgpu::TextureFormat { self.config.format }
}
