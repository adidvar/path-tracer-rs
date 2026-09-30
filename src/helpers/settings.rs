use crate::CameraDto;

#[derive(Debug, PartialEq)]
pub enum SurfaceMode {
    UNorm,
    Linear,
    HDR,
}

pub struct AppSettings {
    pub surface_mode: SurfaceMode,
    pub surface_dirty: bool,

    pub time: f32,

    pub render_scene: String,
    pub camera: CameraDto,
    pub camera_dirty: bool,
    pub max_bounces: u32,
    pub global_light_intensity: f32,
    pub rays_per_pixel: u32,
    pub use_msaa: bool,
    pub render_time_ms: f32,
}

impl AppSettings {
    pub fn default() -> AppSettings {
        AppSettings {
            time: 0.0,
            render_scene: "scene".to_owned(),
            camera: CameraDto::default(),
            camera_dirty: false,
            max_bounces: 4,
            global_light_intensity: 1.0,
            rays_per_pixel: 1,
            use_msaa: true,
            render_time_ms: 0.0,
            surface_mode: SurfaceMode::UNorm,
            surface_dirty: false,
        }
    }
}
