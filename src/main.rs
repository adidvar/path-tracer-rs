use wgpu_engine::interface::window::start_application;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    start_application()
}
