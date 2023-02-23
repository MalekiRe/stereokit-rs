use std::marker::PhantomData;
use crate::pose::Pose;
use crate::values::{matrix_from, matrix_to, MMatrix, MQuat, MVec3, pose_from, pose_to, quat_from, quat_to, vec3_from, vec3_to};

pub fn hierarchy(transform: MMatrix, h: impl FnOnce(&Hierarchy)) {
    _hierarchy_push(transform);
    h(&Hierarchy::new());
    _hierarchy_pop();
}

pub struct Hierarchy(PhantomData<*const ()>);

impl Hierarchy {
    fn new() -> Self {
        Self(Default::default())
    }
    pub fn to_world(&self) -> MMatrix {
        _hierarchy_to_world()
    }
    pub fn to_local(&self) -> MMatrix {
        _hierarchy_to_local()
    }
    pub fn to_local_point(&self, world_pt: MVec3) -> MVec3 {
        _hierarchy_to_local_point(world_pt)
    }
    pub fn to_local_direction(&self, world_dir: MVec3) -> MVec3 {
        _hierarchy_to_local_direction(world_dir)
    }
    pub fn to_local_rotation(&self, world_orientation: MQuat) -> MQuat {
        _hierarchy_to_local_rotation(world_orientation)
    }
    pub fn to_local_pose(&self, world_pose: Pose) -> Pose {
        _hierarchy_to_local_pose(world_pose)
    }
    pub fn to_world_point(&self, local_pt: MVec3) -> MVec3 {
        _hierarchy_to_world_point(local_pt)
    }
    pub fn to_world_direction(&self, local_dir: MVec3) -> MVec3 {
        _hierarchy_to_world_direction(local_dir)
    }
    pub fn to_world_rotation(&self, local_orientation: MQuat) -> MQuat {
        _hierarchy_to_world_rotation(local_orientation)
    }
    pub fn to_world_pose(&self, local_pose: Pose) -> Pose {
        _hierarchy_to_world_pose(local_pose)
    }
}

fn _hierarchy_push(transform: MMatrix) {
    unsafe {
        stereokit_sys::hierarchy_push(&matrix_from(transform))
    }
}
fn _hierarchy_pop() {
    unsafe {
        stereokit_sys::hierarchy_pop()
    }
}
fn _hierarchy_to_world() -> MMatrix {
    matrix_to(unsafe {
        *stereokit_sys::hierarchy_to_world()
    })
}
fn _hierarchy_to_local() -> MMatrix {
    matrix_to(
        unsafe {
            *stereokit_sys::hierarchy_to_local()
        }
    )
}
fn _hierarchy_to_local_point(world_pt: MVec3) -> MVec3 {
    vec3_to(unsafe {
        stereokit_sys::hierarchy_to_local_point(&vec3_from(world_pt))
    })
}
fn _hierarchy_to_local_direction(world_dir: MVec3) -> MVec3 {
    vec3_to(unsafe {
        stereokit_sys::hierarchy_to_local_direction(&vec3_from(world_dir))
    })
}
fn _hierarchy_to_local_rotation(world_orientation: MQuat) -> MQuat {
    quat_to(
        unsafe {
            stereokit_sys::hierarchy_to_local_rotation(&quat_from(world_orientation))
        }
    )
}
fn _hierarchy_to_local_pose(world_pose: Pose) -> Pose {
    pose_to(
        unsafe {
            stereokit_sys::hierarchy_to_local_pose(&pose_from(world_pose))
        }
    )
}
fn _hierarchy_to_world_point(local_pt: MVec3) -> MVec3 {
    vec3_to(unsafe {
        stereokit_sys::hierarchy_to_world_point(&vec3_from(local_pt))
    })
}
fn _hierarchy_to_world_direction(local_dir: MVec3) -> MVec3 {
    vec3_to(unsafe {
        stereokit_sys::hierarchy_to_world_direction(&vec3_from(local_dir))
    })
}
fn _hierarchy_to_world_rotation(local_orientation: MQuat) -> MQuat {
    quat_to(
        unsafe {
            stereokit_sys::hierarchy_to_world_rotation(&quat_from(local_orientation))
        }
    )
}
fn _hierarchy_to_world_pose(local_pose: Pose) -> Pose {
    pose_to(
        unsafe {
            stereokit_sys::hierarchy_to_world_pose(&pose_from(local_pose))
        }
    )
}

