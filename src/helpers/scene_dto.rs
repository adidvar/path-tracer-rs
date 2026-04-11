use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Vec3 = [f32; 3];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialDto {
    pub diffuse: Vec3,
    #[serde(default)]
    pub light_power: f32,
    #[serde(default)]
    pub glossiness: f32,
    #[serde(default)]
    pub specular: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SphereDto {
    pub position: Vec3,
    pub radius: f32,
    pub material_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaneDto {
    pub position: Vec3,
    pub normal: Vec3,
    pub material_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriangleDto {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub normal: Vec3,
    pub material_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshDto {
    pub position: Vec3,
    pub scale: f32,
    pub file_name: String,
    pub material_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct CameraDto {
    pub position: Vec3,
    pub rotation_angles: Vec3,
    pub fov: f32, 

    pub aperture: f32,
    pub focus_distance: f32,
}

impl Default for CameraDto {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation_angles: [0.0, 0.0, 0.0],
            fov: 90.0,
            aperture: 0.0,
            focus_distance: 10.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SceneDto {
    pub camera: CameraDto,
    pub materials: HashMap<String, MaterialDto>,
    pub spheres: Vec<SphereDto>,
    pub planes: Vec<PlaneDto>,
    pub triangles: Vec<TriangleDto>,
    pub meshes: Vec<MeshDto>,
}

impl Default for SceneDto {
    fn default() -> Self {
        Self {
            camera: CameraDto::default(),
            materials: HashMap::new(),
            spheres: Vec::new(),
            planes: Vec::new(),
            triangles: Vec::new(),
            meshes: Vec::new(),
        }
    }
}
