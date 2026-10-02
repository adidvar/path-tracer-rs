use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Quat, Vec3};

use crate::{CameraDto, Scene};

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
pub struct TriangleGpu {
    pub v0: [f32; 3],
    pub n0_x: f32,
    pub v1: [f32; 3],
    pub n0_y: f32,
    pub v2: [f32; 3],
    pub n0_z: f32,
    pub n1: [f32; 3],
    pub material_index: u32,
    pub n2: [f32; 3],
    pub _padding: u32,
}

pub struct GpuSceneData {
    pub global_params: GlobalParamsGpu,
    pub camera: CameraGpu,
    pub materials: Vec<MaterialGpu>,
    pub spheres: Vec<SphereGpu>,
    pub triangles: Vec<TriangleGpu>,
}

impl GpuSceneData {
    pub fn from_dto(dto: &Scene, width: u32, height: u32) -> Self {
        let mut materials: Vec<MaterialGpu> = dto
            .materials
            .iter()
            .map(|m| MaterialGpu {
                diffuse: m.diffuse,
                light_power: m.light_power,
                glossiness: m.glossiness,
                specular: m.specular,
                _padding: [0.0; 2],
            })
            .collect();

        if materials.is_empty() {
            materials.push(MaterialGpu {
                diffuse: [1.0, 1.0, 1.0],
                light_power: 0.0,
                glossiness: 0.0,
                specular: 0.0,
                _padding: [0.0; 2],
            });
        }

        let spheres: Vec<SphereGpu> = dto
            .spheres
            .iter()
            .map(|s| SphereGpu {
                position: s.0.into(),
                radius: s.1,
                material_index: s.2,
                _padding: [0; 3],
            })
            .collect();

        let triangles: Vec<TriangleGpu> = dto
            .triangles
            .iter()
            .map(|t| TriangleGpu {
                v0: t.0[0].into(),
                n0_x: t.1[0].x,
                v1: t.0[1].into(),
                n0_y: t.1[0].y,
                v2: t.0[2].into(),
                n0_z: t.1[0].z,
                n1: t.1[1].into(),
                material_index: t.2,
                n2: t.1[2].into(),
                _padding: 0,
            })
            .collect();

        let camera = Self::camera_gpu_from_dto(&CameraDto::default());

        let global_params = GlobalParamsGpu {
            resolution: [width, height],
            frame_count: 0,
            max_bounces: 4,
            global_light_intensity: 1.0,
            rays_per_pixel: 1,
            use_msaa: 0,
            _padding: [0; 1],
        };

        Self {
            global_params,
            camera,
            materials,
            spheres,
            triangles,
        }
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
