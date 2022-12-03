use std::ops::Mul;
use glam::{Mat4, Vec3};
use crate::mesh::Mesh;
use crate::{StereoKit, values};
use crate::high_level::math_traits::{MatrixContainer, MatrixTrait};
use crate::high_level::model::Model;
use crate::lifecycle::StereoKitContext;

#[derive(Debug, Clone, Copy)]
pub enum Collider {
    CapsuleCollider(CapsuleCollider),
    //add Sphere Collider later
}
impl Collider {
    pub fn get_type(&self) -> ColliderType {
        match self {
            Collider::CapsuleCollider(_) => {
                ColliderType::CapsuleCollider
            }
        }
    }
}
pub enum ColliderType {
    CapsuleCollider
}
#[derive(Debug, Clone, Copy)]
pub struct CapsuleCollider {
    pub point1: Vec3,
    pub point2: Vec3,
    pub radius: f32,
}

impl CapsuleCollider {
    pub fn new(point1: impl Into<values::MVec3>, point2: impl Into<values::MVec3>, radius: f32) -> Self {
        Self {
            point1: point1.into().into(),
            point2: point2.into().into(),
            radius
        }
    }
    pub fn from(sk: &impl StereoKitContext, model: &Model) -> Self {
        //this scales the box size to the proper scale
        let dims = model.get_bounds(sk).dimensions;
        //this scales the center as well
        //let center = Vec3::from(mesh.get_bounds(sk).center).mul(matrix.scale);

        let mut radius = 0.0f32;
        let mut point1 = Vec3::default();
        let mut point2 = Vec3::default();
        //now we find the two closest dimensions, so it's most like a capsule shape

        //x and z are the closest 2
        if dims.y > dims.x && dims.y > dims.z {
            //we wanna make it's bounding box *larger* then the object all else fails
            //so we should use the larger one for the radius
            if dims.x > dims.z {
                radius = dims.x / 2.0;
            } else {
                radius = dims.z / 2.0;
            }
            point1 = Vec3::new(0.0, dims.y/2.0, 0.0);
            point2 = Vec3::new(0.0, -dims.y/2.0, 0.0);
        }
        //y and z
        else if dims.x > dims.y && dims.x > dims.z {
            if dims.y > dims.z {
                radius = dims.y / 2.0;
            } else {
                radius = dims.z / 2.0;
            }
            point1 = Vec3::new(0.0, dims.x/2.0, 0.0);
            point2 = Vec3::new(0.0, -dims.x/2.0, 0.0);
        }
        //y and x
        else {
            if dims.y > dims.x {
                radius = dims.y / 2.0;
            } else {
                radius = dims.x / 2.0;
            }
            point1 = Vec3::new(0.0, dims.z/2.0, 0.0);
            point2 = Vec3::new(0.0, -dims.z/2.0, 0.0);
        }

        //now we gotta transform the points based on the rotation and translation of the matrix
        let trans_rot_mat = Mat4::from_rotation_translation(model.get_matrix().to_scale_rotation_translation().1, model.get_matrix().to_scale_rotation_translation().2);

        point1 = trans_rot_mat.transform_point3(point1);
        point2 = trans_rot_mat.transform_point3(point2);

        Self {
            point1,
            point2,
            radius
        }
    }
}