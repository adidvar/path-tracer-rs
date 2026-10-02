use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Vec3Dto = [f32; 3];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransformDto {
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub rotation: Option<Vec3Dto>,
    #[serde(default)]
    pub position: Option<Vec3Dto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialDto {
    pub diffuse: Vec3Dto,
    #[serde(default)]
    pub light_power: f32,
    #[serde(default)]
    pub glossiness: f32,
    #[serde(default)]
    pub specular: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SphereDto {
    pub transform: Option<TransformDto>,
    pub material_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriangleDto {
    pub vertex: [Vec3Dto; 3],
    pub normal: Vec3Dto,
    pub material_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeshDto {
    pub triangles: Vec<TriangleDto>,
    pub material_id: String,
    pub transform: Option<TransformDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMeshDto {
    pub file_name: String,
    pub material_id: String,
    pub transform: Option<TransformDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct CameraDto {
    pub position: Vec3Dto,
    pub rotation_angles: Vec3Dto,
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
    pub meshes: Vec<MeshDto>,
    pub file_meshes: Vec<FileMeshDto>,
}

impl Default for SceneDto {
    fn default() -> Self {
        Self {
            camera: CameraDto::default(),
            materials: HashMap::new(),
            spheres: Vec::new(),
            meshes: Vec::new(),
            file_meshes: Vec::new(),
        }
    }
}
