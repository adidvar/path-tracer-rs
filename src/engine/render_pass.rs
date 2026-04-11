use crate::{AppSettings, WGPUApplicationContext, WGPUPassContext};

pub trait RenderPass {
    fn resize(&mut self, ctx: &WGPUApplicationContext, width: u32, height: u32);
    fn render(&mut self, ctx: &mut WGPUPassContext, state: &AppSettings);
}
