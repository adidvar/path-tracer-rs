use log::warn;
use wgpu::{SurfaceTexture, TextureView};

use crate::{WGPUApplicationContext, WGPUWindowContext};

pub struct WGPUPassContext<'a> {
    pub app: &'a WGPUApplicationContext,
    _window: &'a mut WGPUWindowContext,

    surface_texture: SurfaceTexture,

    pub encoder: wgpu::CommandEncoder,
    pub surface_view: TextureView,
}

impl<'a> WGPUPassContext<'a> {
    pub fn new(app: &'a WGPUApplicationContext, window: &'a mut WGPUWindowContext) -> Option<Self> {
        let surface_object = window.surface.get_current_texture();

        let wgpu::CurrentSurfaceTexture::Success(surface_texture) = surface_object else {
            warn!("Failed to get surface texture to get view");
            return None;
        };

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("WGPU Pass Encoder"),
            });

        Some(Self {
            encoder,
            surface_texture,
            surface_view,
            app,
            _window: window,
        })
    }

    pub fn finish(self) {
        self.app
            .queue
            .submit(std::iter::once(self.encoder.finish()));
        self.surface_texture.present();
    }
}
