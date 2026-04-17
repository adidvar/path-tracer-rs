#![allow(clippy::needless_return)]

use wgpu_engine::new_application;

#[cfg(not(target_arch = "wasm32"))]
use wgpu_engine::start_application;

#[cfg(target_arch = "wasm32")]
use wgpu_engine::spawn_application;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let application =
        pollster::block_on(new_application()).context("Failed to create application")?;

    start_application(application)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

    wasm_bindgen_futures::spawn_local(async {
        let application = new_application()
            .await
            .expect("Failed to create application");

        spawn_application(application).expect("Failed to spawn application");
    });
}
