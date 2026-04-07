use anyhow::Context;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::*;
use winit::window::Window;
use winit::window::WindowId;

use crate::interface::renderer::render_frame;
use crate::interface::wgpu_context::WGPUApplicationContext;
use crate::interface::wgpu_surface::WGPUWindowContext;

const WINDOW_START_SIZE: LogicalSize<u32> = LogicalSize::new(800, 800);
const WINDOW_START_TITLE: &str = "Path tracer application";

pub fn start_application() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = ApplicationWindowHandler::new()?;

    event_loop.run_app(&mut app)?;

    Ok(())
}

struct ApplicationWindowHandler {
    window: Option<WGPUWindowContext>,
    context: WGPUApplicationContext,
}

impl ApplicationWindowHandler {
    pub fn new() -> anyhow::Result<ApplicationWindowHandler> {
        Ok(ApplicationWindowHandler {
            window: None,
            context: WGPUApplicationContext::new().context("Failed to init wgpu context")?,
        })
    }
}

impl ApplicationHandler for ApplicationWindowHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            WGPUWindowContext::new(
                &self.context,
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_inner_size(WINDOW_START_SIZE)
                            .with_title(WINDOW_START_TITLE),
                    )
                    .unwrap(),
            )
            .unwrap(),
        );
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut wstate = self.window.as_mut().unwrap();

        let responce = wstate.state.on_window_event(&wstate.window, &event);

        if responce.repaint {
            wstate.window.request_redraw();
        }

        if responce.consumed {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                render_frame(&mut self.context, &mut wstate);
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    wstate.surface_configuration.width = new_size.width;
                    wstate.surface_configuration.height = new_size.height;

                    wstate
                        .surface
                        .configure(&self.context.device, &wstate.surface_configuration);

                    wstate.window.request_redraw();
                }
            }
            _ => (),
        }
    }
}
