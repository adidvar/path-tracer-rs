use anyhow::Context;

pub struct WGPUApplicationContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub context: egui::Context,
}

impl WGPUApplicationContext {
    pub fn new() -> anyhow::Result<WGPUApplicationContext> {
        let instance = wgpu::Instance::default();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,

            force_fallback_adapter: false,

            compatible_surface: None,
        }))
        .context("Failed to init adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Rendering device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::default(),
        }))
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
