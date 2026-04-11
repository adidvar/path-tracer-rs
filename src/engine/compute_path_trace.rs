use wgpu::util::DeviceExt;

use crate::{
    ASSETS_DIR, AppSettings, GlobalParamsGpu, GpuSceneData, RenderPass, WGPUApplicationContext,
    WGPUPassContext,
};

pub struct ComputePathTracePass {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout_targets: wgpu::BindGroupLayout,
    bind_group_layout_scene: wgpu::BindGroupLayout,
    bind_group_targets: wgpu::BindGroup,
    bind_group_scene: wgpu::BindGroup,
    width: u32,
    height: u32,

    frame_count: u32,
    global_params_buf: wgpu::Buffer,
    camera_buf: wgpu::Buffer,
    materials_buf: wgpu::Buffer,
    spheres_buf: wgpu::Buffer,
    planes_buf: wgpu::Buffer,
    accum_buf: wgpu::Buffer,
}

impl ComputePathTracePass {
    pub fn new(
        context: &WGPUApplicationContext,
        width: u32,
        height: u32,
        gpu_scene: &GpuSceneData,
    ) -> Self {
        let device = &context.device;

        let (texture, texture_view) = Self::create_texture(device, width, height);

        let global_params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Params Buffer"),
            contents: bytemuck::cast_slice(&[gpu_scene.global_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[gpu_scene.camera]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let materials_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Materials Buffer"),
            contents: bytemuck::cast_slice(&gpu_scene.materials),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let spheres_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spheres Buffer"),
            contents: bytemuck::cast_slice(&gpu_scene.spheres),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let planes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Planes Buffer"),
            contents: bytemuck::cast_slice(&gpu_scene.planes),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let accum_buf_size = (width * height).max(1) * 16;
        let accum_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Accumulation Buffer"),
            size: accum_buf_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout_targets = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Targets Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group_layout_scene = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Scene Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group_targets = Self::create_bind_group_targets(
            device,
            &bind_group_layout_targets,
            &texture_view,
            &accum_buf,
        );

        let bind_group_scene = Self::create_bind_group_scene(
            device,
            &bind_group_layout_scene,
            &global_params_buf,
            &camera_buf,
            &materials_buf,
            &spheres_buf,
            &planes_buf,
        );

        let shader_file = ASSETS_DIR
            .get_file("shaders/compute_path_trace.wgsl")
            .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Path Trace Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_file.contents_utf8().unwrap().into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Path Trace Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout_targets), Some(&bind_group_layout_scene)],
            immediate_size: 0,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Path Trace Pipeline"),
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
            bind_group_layout_targets,
            bind_group_layout_scene,
            bind_group_targets,
            bind_group_scene,
            width,
            height,
            frame_count: 0,
            global_params_buf,
            camera_buf,
            materials_buf,
            spheres_buf,
            planes_buf,
            accum_buf,
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

    fn create_bind_group_targets(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        accum_buf: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Path Trace Targets Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: accum_buf.as_entire_binding(),
                },
            ],
        })
    }

    fn create_bind_group_scene(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        globals_buf: &wgpu::Buffer,
        camera_buf: &wgpu::Buffer,
        materials_buf: &wgpu::Buffer,
        spheres_buf: &wgpu::Buffer,
        planes_buf: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Path Trace Scene Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: materials_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: spheres_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: planes_buf.as_entire_binding(),
                },
            ],
        })
    }

    pub fn get_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }
}

impl RenderPass for ComputePathTracePass {
    fn resize(&mut self, ctx: &WGPUApplicationContext, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        let (new_tex, new_view) = Self::create_texture(&ctx.device, width, height);
        self.texture = new_tex;
        self.texture_view = new_view;

        let accum_buf_size = (width * height).max(1) * 16;
        self.accum_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Accumulation Buffer"),
            size: accum_buf_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.bind_group_targets = Self::create_bind_group_targets(
            &ctx.device,
            &self.bind_group_layout_targets,
            &self.texture_view,
            &self.accum_buf,
        );

        self.frame_count = 0;
    }

    fn render(&mut self, ctx: &mut WGPUPassContext, state: &AppSettings) {
        if state.camera_dirty {
            let new_cam = GpuSceneData::camera_gpu_from_dto(&state.camera);
            ctx.app
                .queue
                .write_buffer(&self.camera_buf, 0, bytemuck::cast_slice(&[new_cam]));
            self.frame_count = 0;
        }

        self.frame_count += 1;

        let globals = GlobalParamsGpu {
            resolution: [self.width, self.height],
            frame_count: self.frame_count,
            max_bounces: state.max_bounces,
            global_light_intensity: state.global_light_intensity,
            rays_per_pixel: state.rays_per_pixel,
            use_msaa: if state.use_msaa { 1 } else { 0 },
            _padding: [0; 1],
        };

        ctx.app
            .queue
            .write_buffer(&self.global_params_buf, 0, bytemuck::cast_slice(&[globals]));

        let mut cpass = ctx
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Path Trace Pass"),
                timestamp_writes: None,
            });

        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.bind_group_targets, &[]);
        cpass.set_bind_group(1, &self.bind_group_scene, &[]);

        cpass.dispatch_workgroups((self.width + 15) / 16, (self.height + 15) / 16, 1);
    }
}
