use anyhow::Context;
use egui_wgpu::RendererOptions;
use log::info;
use std::sync::Arc;
use wgpu::TextureFormat;
use winit::window::Window;

use crate::{SurfaceMode, WGPUApplicationContext};

#[derive(Clone)]
pub struct WGPUSupportedSufraces {
    pub unorm: Option<TextureFormat>,
    pub srgb: Option<TextureFormat>,
    pub hdr: Option<TextureFormat>,
}

impl WGPUSupportedSufraces {
    pub fn get_texture_format(self, mode: &SurfaceMode) -> TextureFormat {
        match mode {
            SurfaceMode::UNorm => self.unorm.unwrap(),
            SurfaceMode::Linear => self.srgb.unwrap(),
            SurfaceMode::HDR => self.hdr.unwrap(),
        }
    }
}

pub struct WGPUWindowContext {
    pub surface: wgpu::Surface<'static>,
    pub window: Arc<Window>,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
    pub surface_configuration: wgpu::SurfaceConfiguration,
    pub surface_formates: WGPUSupportedSufraces,
}

impl WGPUWindowContext {
    pub fn new(
        context: &WGPUApplicationContext,
        window: Window,
        prefered_surface: Option<TextureFormat>,
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
            .collect::<Vec<TextureFormat>>();

        let supported_sufraces = WGPUSupportedSufraces {
            unorm: texture_format.iter().find(|e| !e.is_srgb()).copied(),
            srgb: texture_format.iter().find(|e| e.is_srgb()).copied(),
            hdr: texture_format
                .iter()
                .find(|&&e| e == TextureFormat::Rgba16Float)
                .copied(),
        };

        let texture_format = prefered_surface.unwrap_or(supported_sufraces.unorm.unwrap());

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
            surface_formates: supported_sufraces,
        })
    }
}
