use crate::interface::{
    wgpu_context::WGPUApplicationContext, wgpu_surface::WGPUWindowContext,
    window_gen::generate_window_interface,
};

pub fn render_frame(context: &mut WGPUApplicationContext, wcontext: &mut WGPUWindowContext) {
    let raw_input = wcontext.state.take_egui_input(&wcontext.window);
    context.context.begin_pass(raw_input);

    generate_window_interface(&context.context);

    let full_output = context.context.end_pass();

    wcontext
        .state
        .handle_platform_output(&wcontext.window, full_output.platform_output);

    for (id, image_delta) in &full_output.textures_delta.set {
        wcontext
            .renderer
            .update_texture(&context.device, &context.queue, *id, image_delta);
    }

    let paint_jobs = context
        .context
        .tessellate(full_output.shapes, full_output.pixels_per_point);

    let surface_texture = wcontext.surface.get_current_texture();

    let wgpu::CurrentSurfaceTexture::Success(frame) = surface_texture else {
        return;
    };

    let output_view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [
            wcontext.surface_configuration.width,
            wcontext.surface_configuration.height,
        ],
        pixels_per_point: wcontext.window.scale_factor() as f32,
    };

    wcontext.renderer.update_buffers(
        &context.device,
        &context.queue,
        &mut encoder,
        &paint_jobs,
        &screen_descriptor,
    );

    {
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EGUI Render Pass"),
            multiview_mask: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.1,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let mut static_render_pass = render_pass.forget_lifetime();

        wcontext
            .renderer
            .render(&mut static_render_pass, &paint_jobs, &screen_descriptor);
    }

    context.queue.submit(std::iter::once(encoder.finish()));
    frame.present();

    for id in &full_output.textures_delta.free {
        wcontext.renderer.free_texture(id);
    }
}
