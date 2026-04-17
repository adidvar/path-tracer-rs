use anyhow::Context;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::*;
use winit::window::WindowId;

use crate::*;

const WINDOW_START_SIZE: LogicalSize<u32> = LogicalSize::new(800, 800);
const WINDOW_START_TITLE: &str = "Path tracer application";

pub struct ApplicationWindowHandler {
    context: WGPUApplicationContext,
    window: Option<WGPUWindowContext>,

pub fn start_application_with_context(context: WGPUApplicationContext) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

pub async fn new_application() -> anyhow::Result<ApplicationWindowHandler> {
    Ok(ApplicationWindowHandler {
        context: WGPUApplicationContext::new().await?,
        window: None,
        ui_manager: None,
        compute_pass: None,
        post_process_pass: None,
        settings: AppSettings::default(),
    })
}

pub fn start_application(mut application: ApplicationWindowHandler) -> anyhow::Result<()> {
    let event_loop =
        EventLoop::new().context("Failed to create event loop for starting the application")?;
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
        .run_app(&mut application)
        .context("Failed while running the application event loop")?;

    Ok(())
}

pub fn spawn_application(mut application: ApplicationWindowHandler) -> anyhow::Result<()> {
    let event_loop =
        EventLoop::new().context("Failed to create event loop for spawning the application")?;
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop
        .run_app(&mut application)
        .context("Failed while spawning the application event loop")?;

    Ok(())
}

impl ApplicationHandler for ApplicationWindowHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            #[allow(unused_mut)]
            let mut attributes = window_attributes(WINDOW_START_SIZE, WINDOW_START_TITLE);
            let window_obj = event_loop.create_window(attributes).unwrap();

            let w_ctx = WGPUWindowContext::new(&self.context, window_obj).unwrap();
            let ui_m = UIManager::new(&self.context, &w_ctx);

            let width = w_ctx.surface_configuration.width;
            let height = w_ctx.surface_configuration.height;
            let format = w_ctx.surface_configuration.format;

            let compute_p = ComputePathTracePass::new(
                &self.context,
                width,
                height,
                &GpuSceneData::from_dto(
                    &load_scene_into_settings(&mut self.settings),
                    width,
                    height,
                ),
            );
            let mut post_p = PostProcessPass::new(&self.context, format);

            post_p.update_bind_group(&self.context.device, compute_p.get_view());

            self.window = Some(w_ctx);
            self.ui_manager = Some(ui_m);
            self.compute_pass = Some(compute_p);
            self.post_process_pass = Some(post_p);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.window.is_none() {
            return;
        }

        let w_ctx = self.window.as_mut().unwrap();
        let ui_m = self.ui_manager.as_mut().unwrap();

        let ui_resp = ui_m.handle_event(&w_ctx.window, &event);
        if ui_resp.needs_repaint {
            w_ctx.window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested => {
                self.compute_pass = None;
                self.post_process_pass = None;
                self.ui_manager = None;
                self.window = None;

                event_loop.exit();
            }

            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    w_ctx.surface_configuration.width = new_size.width;
                    w_ctx.surface_configuration.height = new_size.height;
                    w_ctx
                        .surface
                        .configure(&self.context.device, &w_ctx.surface_configuration);

                    if let Some(compute) = &mut self.compute_pass {
                        compute.resize(&self.context, new_size.width, new_size.height);
                    }

                    if let (Some(compute), Some(post)) =
                        (&self.compute_pass, &mut self.post_process_pass)
                    {
                        post.update_bind_group(&self.context.device, compute.get_view());
                    }

                    ui_m.resize(&self.context, new_size.width, new_size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                let start_time = web_time::Instant::now();

                ui_m.prepare(&mut self.context, w_ctx, |ctx| {
                    generate_window_interface(ctx, &mut self.settings);
                });

                let Some(mut pass_ctx) = WGPUPassContext::new(&self.context, w_ctx) else {
                    return;
                };

                if let Some(compute) = &mut self.compute_pass {
                    compute.render(&mut pass_ctx, &self.settings);
                }

                self.settings.camera_dirty = false;

                if let Some(post) = &mut self.post_process_pass {
                    post.render(&mut pass_ctx, &self.settings);
                }

                ui_m.render(&mut pass_ctx, &self.settings);

                pass_ctx.finish();

                self.settings.render_time_ms = start_time.elapsed().as_secs_f32() * 1000.0;

                w_ctx.window.request_redraw();
            }
            _ => (),
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.compute_pass = None;
        self.post_process_pass = None;
        self.ui_manager = None;
        self.window = None;
    }
}
