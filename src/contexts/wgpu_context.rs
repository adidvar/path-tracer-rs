use anyhow::Context;

use crate::get_limits;

pub struct WGPUApplicationContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub context: egui::Context,
}

impl WGPUApplicationContext {
    pub async fn new() -> anyhow::Result<WGPUApplicationContext> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .context("Failed to init adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Rendering device"),
                required_features: wgpu::Features::empty(),
                required_limits: get_limits(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .context("Failed to init device and queue")?;

        let context = egui::Context::default();

        Ok(WGPUApplicationContext {
            instance,
            adapter,
            device,
            queue,
            context,
        })
    }
}
