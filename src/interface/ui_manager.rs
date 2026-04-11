use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;

use crate::{AppSettings, RenderPass, WGPUApplicationContext, WGPUPassContext, WGPUWindowContext};

pub struct UIEventResponse {
    pub consumed: bool,
    pub needs_repaint: bool,
}

pub struct UIManager {
    pub context: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    current_paint_jobs: Vec<egui::ClippedPrimitive>,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
}

impl UIManager {
    pub fn new(app: &WGPUApplicationContext, win: &WGPUWindowContext) -> Self {
        let context = egui::Context::default();

        let viewport_id = app.context.viewport_id();

        let state = State::new(
            context.clone(),
            viewport_id,
            &win.window,
            Some(win.window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = Renderer::new(
            &app.device,
            win.surface_configuration.format,
            RendererOptions::default(),
        );

        Self {
            context,
            state,
            renderer,
            current_paint_jobs: Vec::new(),
            screen_descriptor: egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    win.surface_configuration.width,
                    win.surface_configuration.height,
                ],
                pixels_per_point: win.window.scale_factor() as f32,
            },
        }
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> UIEventResponse {
        let response = self.state.on_window_event(window, event);
        UIEventResponse {
            consumed: response.consumed,
            needs_repaint: response.repaint,
        }
    }

    pub fn prepare(
        &mut self,
        ctx: &WGPUApplicationContext,
        wctx: &mut WGPUWindowContext,
        ui_logic: impl FnOnce(&egui::Context),
    ) {
        let raw_input = self.state.take_egui_input(&wctx.window);
        self.context.begin_pass(raw_input);
        ui_logic(&self.context);
        let output = self.context.end_pass();

        for (id, delta) in output.textures_delta.set {
            self.renderer
                .update_texture(&ctx.device, &ctx.queue, id, &delta);
        }
        for id in output.textures_delta.free {
            self.renderer.free_texture(&id);
        }

        self.screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                wctx.surface_configuration.width,
                wctx.surface_configuration.height,
            ],
            pixels_per_point: wctx.window.scale_factor() as f32,
        };

        self.current_paint_jobs = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("EGUI Upload/Update Encoder"),
            });

        self.renderer.update_buffers(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            &self.current_paint_jobs,
            &self.screen_descriptor,
        );

        ctx.queue.submit(std::iter::once(encoder.finish()));

        self.state
            .handle_platform_output(&wctx.window, output.platform_output);
    }
}

impl RenderPass for UIManager {
    fn resize(&mut self, _ctx: &WGPUApplicationContext, width: u32, height: u32) {
        self.screen_descriptor.size_in_pixels = [width, height];
    }

    fn render(&mut self, ctx: &mut WGPUPassContext, _state: &AppSettings) {
        let rpass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EGUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &ctx.surface_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            multiview_mask: None,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.renderer.render(
            &mut rpass.forget_lifetime(),
            &self.current_paint_jobs,
            &self.screen_descriptor,
        );
    }
}
