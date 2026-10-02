use crate::TransformDto;

pub type Vec3 = glam::Vec3;

pub struct Transform(pub glam::Mat4);

impl From<TransformDto> for Transform {
    fn from(t: TransformDto) -> Self {
        let mut matrix = glam::Mat4::IDENTITY;

        if let Some(p) = t.position {
            matrix *= glam::Mat4::from_translation(p.into());
        }

        if let Some(s) = t.scale {
            matrix *= glam::Mat4::from_scale((s, s, s).into());
        }

        if let Some(r) = t.rotation {
            let rotation_quat = glam::Quat::from_euler(glam::EulerRot::XYZ, r[0], r[1], r[2]);
            matrix *= glam::Mat4::from_quat(rotation_quat);
        }

        Transform(matrix)
    }
}

impl Transform {
    pub fn scale_position(self: &Self, vector: glam::Vec3) -> glam::Vec3 {
        self.0.transform_point3(vector)
    }
    pub fn scale_direction(self: &Self, vector: glam::Vec3) -> glam::Vec3 {
        self.0.transform_vector3(vector)
    }
}
