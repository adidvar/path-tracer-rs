use std::collections::HashMap;

use crate::{ASSETS_DIR, CameraDto, MaterialDto, SceneDto, Transform, Vec3};

pub type Triangle = ([Vec3; 3], [Vec3; 3], u32);
pub type Sphere = (Vec3, f32, u32);

pub type Material = MaterialDto;
pub type Camera = CameraDto;

pub struct Scene {
    pub materials: Vec<Material>,
    pub triangles: Vec<Triangle>,
    pub spheres: Vec<Sphere>,
}

pub fn load_scene(name: &str) -> (Scene, Camera) {
    let path = format!("scenes/{}.json", name);
    let buffer = ASSETS_DIR.get_file(path).unwrap().contents_utf8().unwrap();
    let scene_dto: SceneDto = serde_json::from_str(buffer).unwrap();
    (bake_scene(scene_dto.clone()), bake_camera(scene_dto.camera))
}

fn bake_scene(dto: SceneDto) -> Scene {
    let mut scene = Scene {
        materials: vec![],
        triangles: vec![],
        spheres: vec![],
    };

    let mut material_map: HashMap<String, u32> = HashMap::new();
    for (id, mat_dto) in dto.materials {
        material_map.insert(id, scene.materials.len() as u32);
        scene.materials.push(mat_dto);
    }

    let get_mat_idx = |id: &str| -> u32 { *material_map.get(id).unwrap_or(&0) };

    for sphere in dto.spheres {
        let mat_idx = get_mat_idx(&sphere.material_id);
        let transform = sphere
            .transform
            .map(Transform::from)
            .unwrap_or(Transform(glam::Mat4::IDENTITY));

        let center = transform.scale_position(glam::Vec3::ZERO);
        let radius = transform.scale_direction(glam::Vec3::X).length();

        scene.spheres.push((center, radius, mat_idx));
    }

    for mesh in dto.meshes {
        let mat_idx = get_mat_idx(&mesh.material_id);
        let transform = mesh
            .transform
            .map(Transform::from)
            .unwrap_or(Transform(glam::Mat4::IDENTITY));

        for tri in mesh.triangles {
            let v0 = transform.scale_position(tri.vertex[0].into());
            let v1 = transform.scale_position(tri.vertex[1].into());
            let v2 = transform.scale_position(tri.vertex[2].into());

            let n = transform
                .scale_direction(tri.normal.into())
                .normalize_or_zero();

            scene.triangles.push(([v0, v1, v2], [n, n, n], mat_idx));
        }
    }

    for file_mesh in dto.file_meshes {
        let mat_idx = get_mat_idx(&file_mesh.material_id);
        let transform = file_mesh
            .transform
            .map(Transform::from)
            .unwrap_or(Transform(glam::Mat4::IDENTITY));

        let path = format!("objects/{}.glb", file_mesh.file_name);

        let buffer = ASSETS_DIR
            .get_file(&path)
            .unwrap_or_else(|| panic!("Failed to find file {}", path))
            .contents();

        let (document, buffers, _) = gltf::import_slice(buffer).expect("GLB parsing error");

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();

                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .expect("Normals should be present")
                    .collect();

                if let Some(indices_reader) = reader.read_indices() {
                    let indices: Vec<u32> = indices_reader.into_u32().collect();

                    for chunk in indices.chunks_exact(3) {
                        let i0 = chunk[0] as usize;
                        let i1 = chunk[1] as usize;
                        let i2 = chunk[2] as usize;

                        let v0 = transform.scale_position(positions[i0].into());
                        let v1 = transform.scale_position(positions[i1].into());
                        let v2 = transform.scale_position(positions[i2].into());

                        let n0 = transform
                            .scale_direction(normals[i0].into())
                            .normalize_or_zero();
                        let n1 = transform
                            .scale_direction(normals[i1].into())
                            .normalize_or_zero();
                        let n2 = transform
                            .scale_direction(normals[i2].into())
                            .normalize_or_zero();

                        scene.triangles.push(([v0, v1, v2], [n0, n1, n2], mat_idx));
                    }
                }
            }
        }
    }

    scene
}

fn bake_camera(dto: CameraDto) -> Camera {
    dto
}
