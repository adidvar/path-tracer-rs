use anyhow::Context;
use egui_wgpu::RendererOptions;
use log::info;
use std::sync::Arc;
use winit::window::Window;

use crate::interface::wgpu_context::WGPUApplicationContext;

pub struct WGPUWindowContext {
    pub surface: wgpu::Surface<'static>,
    pub window: Arc<Window>,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
    pub surface_configuration: wgpu::SurfaceConfiguration,
}

impl WGPUWindowContext {
    pub fn new(
        context: &WGPUApplicationContext,
        window: Window,
    ) -> anyhow::Result<WGPUWindowContext> {
        let window = Arc::new(window);
        let surface = context
            .instance
            .create_surface(window.clone())
            .context("Failed to create surface")?;

        let viewport_id = context.context.viewport_id();

        let state = egui_winit::State::new(
            context.context.clone(),
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let surface_caps = surface.get_capabilities(&context.adapter);

        let texture_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        info!("Selected surface format: {:?}", texture_format);

        let size = window.inner_size();

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,

            present_mode: wgpu::PresentMode::AutoVsync,

            alpha_mode: surface_caps.alpha_modes[0],

            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&context.device, &surface_configuration);

        let renderer =
            egui_wgpu::Renderer::new(&context.device, texture_format, RendererOptions::default());

        Ok(WGPUWindowContext {
            window,
            surface,
            state,
            renderer,
            surface_configuration,
        })
    }
}
