pub struct RenderSettings {
    pub enable_tonemapping: bool,
    pub enable_gamma: bool,
    pub time: f32,
}
impl RenderSettings {
    pub fn default() -> RenderSettings {
        RenderSettings {
            enable_tonemapping: true,
            enable_gamma: true,
            time: 0.0,
        }
    }
}
