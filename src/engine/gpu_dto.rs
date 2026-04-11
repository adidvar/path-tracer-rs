use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Quat, Vec3};
use std::collections::HashMap;

use crate::{CameraDto, SceneDto};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GlobalParamsGpu {
    pub resolution: [u32; 2],
    pub frame_count: u32,
    pub max_bounces: u32,
    pub global_light_intensity: f32,
    pub rays_per_pixel: u32,
    pub use_msaa: u32,
    pub _padding: [u32; 1],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraGpu {
    pub position: [f32; 3],
    pub fov: f32,
    pub forward: [f32; 3],
    pub aperture: f32,
    pub right: [f32; 3],
    pub focus_dist: f32,
    pub up: [f32; 3],
    pub _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MaterialGpu {
    pub diffuse: [f32; 3],
    pub light_power: f32,
    pub glossiness: f32,
    pub specular: f32,
    pub _padding: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SphereGpu {
    pub position: [f32; 3],
    pub radius: f32,
    pub material_index: u32,
    pub _padding: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PlaneGpu {
    pub position: [f32; 3],
    pub _padding1: u32,
    pub normal: [f32; 3],
    pub material_index: u32,
}

pub struct GpuSceneData {
    pub global_params: GlobalParamsGpu,
    pub camera: CameraGpu,
    pub materials: Vec<MaterialGpu>,
    pub spheres: Vec<SphereGpu>,
    pub planes: Vec<PlaneGpu>,
}

impl GpuSceneData {
    pub fn from_dto(dto: &SceneDto, width: u32, height: u32) -> Self {
        let mut materials = Vec::new();
        let mut mat_indices = HashMap::new();

        for (name, mat_dto) in &dto.materials {
            mat_indices.insert(name.clone(), materials.len() as u32);
            materials.push(MaterialGpu {
                diffuse: mat_dto.diffuse,
                light_power: mat_dto.light_power,
                glossiness: mat_dto.glossiness,
                specular: mat_dto.specular,
                _padding: [0.0; 2],
            });
        }

        if materials.is_empty() {
            materials.push(MaterialGpu {
                diffuse: [1.0, 1.0, 1.0],
                light_power: 0.0,
                glossiness: 0.0,
                specular: 0.0,
                _padding: [0.0; 2],
            });
        }

        let spheres = dto
            .spheres
            .iter()
            .map(|s| {
                let mat_idx = *mat_indices.get(&s.material_id).unwrap_or(&0);
                SphereGpu {
                    position: s.position,
                    radius: s.radius,
                    material_index: mat_idx,
                    _padding: [0; 3],
                }
            })
            .collect();

        let camera = Self::build_camera_gpu(&dto.camera);

        let global_params = GlobalParamsGpu {
            resolution: [width, height],
            frame_count: 0,
            max_bounces: 4,
            global_light_intensity: 1.0,
            rays_per_pixel: 1,
            use_msaa: 0,
            _padding: [0; 1],
        };

        let planes = dto
            .planes
            .iter()
            .map(|p| {
                let mat_idx = *mat_indices.get(&p.material_id).unwrap_or(&0);
                PlaneGpu {
                    position: p.position,
                    _padding1: 0,
                    normal: p.normal,
                    material_index: mat_idx,
                }
            })
            .collect();

        Self {
            global_params,
            camera,
            materials,
            spheres,
            planes,
        }
    }

    fn build_camera_gpu(cam_dto: &CameraDto) -> CameraGpu {
        Self::camera_gpu_from_dto(cam_dto)
    }

    pub fn camera_gpu_from_dto(cam: &CameraDto) -> CameraGpu {
        let pitch = cam.rotation_angles[0].to_radians();
        let yaw = cam.rotation_angles[1].to_radians();
        let roll = cam.rotation_angles[2].to_radians();

        let rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

        let forward = (rotation * Vec3::NEG_Z).normalize();
        let right = (rotation * Vec3::X).normalize();
        let up = (rotation * Vec3::Y).normalize();

        CameraGpu {
            position: cam.position,
            fov: cam.fov,
            forward: forward.to_array(),
            aperture: cam.aperture,
            right: right.to_array(),
            focus_dist: cam.focus_distance,
            up: up.to_array(),
            _padding: 0.0,
        }
    }
}
