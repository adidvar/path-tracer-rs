use crate::{ASSETS_DIR, AppSettings, SceneDto};

pub fn load_scene(settings: &AppSettings) -> SceneDto {
    let path = format!("scenes/{}.json", settings.render_scene);
    let buffer = ASSETS_DIR.get_file(path).unwrap().contents_utf8().unwrap();
    serde_json::from_str(buffer).unwrap()
}

pub fn load_scene_into_settings(settings: &mut AppSettings) -> SceneDto {
    let scene = load_scene(settings);
    settings.camera = scene.camera.clone();
    settings.camera_dirty = false;
    scene
}
