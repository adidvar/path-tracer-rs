use crate::{ASSETS_DIR, RenderPass, RenderSettings, WGPUApplicationContext, WGPUPassContext};

pub struct ComputeNoisePass {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl ComputeNoisePass {
    pub fn new(context: &WGPUApplicationContext, width: u32, height: u32) -> Self {
        let device = &context.device;

        let (texture, texture_view) = Self::create_texture(device, width, height);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Noise Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, &texture_view);

        let shader_file = ASSETS_DIR.get_file("shaders/compute_noise.wgsl").unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Noise Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_file.contents_utf8().unwrap().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Noise Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Noise Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            texture,
            texture_view,
            compute_pipeline,
            bind_group_layout,
            bind_group,
            width,
            height,
        }
    }

    fn create_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("HDR Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Noise Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
        })
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }
}

impl RenderPass for ComputeNoisePass {
    fn resize(&mut self, ctx: &WGPUApplicationContext, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        let (new_tex, new_view) = Self::create_texture(&ctx.device, width, height);
        self.texture = new_tex;
        self.texture_view = new_view;
        self.bind_group =
            Self::create_bind_group(&ctx.device, &self.bind_group_layout, &self.texture_view);
    }

    fn render(&mut self, ctx: &mut WGPUPassContext, _state: &RenderSettings) {
        let mut cpass = ctx
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Noise Pass"),
                timestamp_writes: None,
            });

        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);

        cpass.dispatch_workgroups((self.width + 15) / 16, (self.height + 15) / 16, 1);
    }
}
