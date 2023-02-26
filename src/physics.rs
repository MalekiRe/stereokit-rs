use std::ptr::NonNull;
use color_eyre::Report;
use color_eyre::Result;
use stereokit_sys::{_solid_t, solid_type_};
use crate::lifecycle::StereoKitContext;
use crate::pose::Pose;
use crate::values::{MQuat, MVec3, pose_from, pose_to, quat_from, vec3_from};

pub enum SolidType {
    Normal = 0,
    Immovable = 1,
    Unaffected = 2,
}

pub struct Solid(pub(crate) NonNull<_solid_t>);

impl Solid {
    pub fn new(_sk: &impl StereoKitContext, position: MVec3, rotation: MQuat, solid_type: SolidType) -> Result<Self> {
        Ok(_solid_create(position, rotation, solid_type).ok_or(Report::msg("unable to create solid"))?)
    }
    pub fn add_box(&self, _sk: &impl StereoKitContext, dimensions: MVec3, kilograms: f32, offset: MVec3) {
        _solid_add_box(self, dimensions, kilograms, offset);
    }
    pub fn add_capsule(&self, _sk: &impl StereoKitContext, diameter: f32, height: f32, kilograms: f32, offset: MVec3) {
        _solid_add_capsule(self, diameter, height, kilograms, offset);
    }
    pub fn get_pose(&self, _sk: &impl StereoKitContext) -> Pose {
        _solid_get_pose(self)
    }
    pub fn move_to(&self, pose: Pose) {
        _solid_move(self, pose.position, pose.orientation);
    }
    pub fn teleport_to(&self, pose: Pose) {
        _solid_teleport(self, pose.position, pose.orientation);
    }
}
impl Drop for Solid {
    fn drop(&mut self) {
        unsafe { stereokit_sys::solid_release(self.0.as_ptr()) }
    }
}
fn _solid_create(position: MVec3, rotation: MQuat, solid_type: SolidType) -> Option<Solid> {
    Some(Solid(NonNull::new(unsafe {
        stereokit_sys::solid_create(&vec3_from(position), &quat_from(rotation), solid_type as solid_type_)
    })?))
}
fn _solid_add_box(solid: &Solid, dimensions: MVec3, kilograms: f32, offset: MVec3) {
    unsafe {
        stereokit_sys::solid_add_box(solid.0.as_ptr(), &vec3_from(dimensions), kilograms, &vec3_from(offset))
    }
}
fn _solid_get_pose(solid: &Solid) -> Pose {
    let mut temp_pose = pose_from(Pose::IDENTITY);
    unsafe {
        stereokit_sys::solid_get_pose(solid.0.as_ptr(), &mut temp_pose);
    }
    pose_to(temp_pose)
}
fn _solid_move(solid: &Solid, position: MVec3, rotation: MQuat) {
    unsafe {
        stereokit_sys::solid_move(solid.0.as_ptr(), &vec3_from(position), &quat_from(rotation));
    }
}
fn _solid_teleport(solid: &Solid, position: MVec3, rotation: MQuat) {
    unsafe {
        stereokit_sys::solid_teleport(solid.0.as_ptr(), &vec3_from(position), &quat_from(rotation));
    }
}
fn _solid_add_capsule(solid: &Solid, diameter: f32, height: f32, kilograms: f32, offset: MVec3) {
    unsafe {
        stereokit_sys::solid_add_capsule(solid.0.as_ptr(), diameter, height, kilograms, &vec3_from(offset));
    }
}