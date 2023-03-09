use crate::pose::Pose;
use mint::{ColumnMatrix4, RowMatrix4};
use std::fmt::Pointer;
use stereokit_sys::{
	color128, color32, matrix, pose_t, quat, ray_t, text_style_t, vec2, vec3, vec4,
};

pub type MVec4 = mint::Vector4<f32>;
pub type MVec3 = mint::Vector3<f32>;
pub type MVec2 = mint::Vector2<f32>;
pub type MQuat = mint::Quaternion<f32>;
pub type MMatrix = mint::ColumnMatrix4<f32>;
pub type Color32 = color32;
pub type Color128 = color128;

#[cfg(target_os = "linux")]
pub type IntegerType = u32;

#[cfg(target_os = "windows")]
pub type IntegerType = i32;

pub struct Ray {
	pub pos: MVec3,
	pub dir: MVec3,
}

pub struct SKMatrix {
	matrix: MMatrix,
	inverse: Option<MMatrix>,
}
impl SKMatrix {
	pub fn new(matrix: MMatrix) -> Self {
		Self {
			matrix,
			inverse: None,
		}
	}
	pub fn transform_point(&mut self, pt: MVec3) -> MVec3 {
		if self.inverse.is_none() {
			self.inverse = Some(matrix_to(unsafe {
				stereokit_sys::matrix_invert(&matrix_from(self.matrix))
			}))
		}
		let inverse = self.inverse.unwrap();
		vec3_to(unsafe { stereokit_sys::matrix_transform_pt(matrix_from(inverse), vec3_from(pt)) })
	}
}

pub(crate) fn vec2_from(var: MVec2) -> vec2 {
	vec2 { x: var.x, y: var.y }
}
pub(crate) fn vec2_to(var: vec2) -> MVec2 {
	MVec2 { x: var.x, y: var.y }
}

pub fn vec3_from(var: MVec3) -> vec3 {
	vec3 {
		x: var.x,
		y: var.y,
		z: var.z,
	}
}
pub fn vec3_to(var: vec3) -> MVec3 {
	MVec3 {
		x: var.x,
		y: var.y,
		z: var.z,
	}
}

pub(crate) fn vec4_from(var: MVec4) -> vec4 {
	vec4 {
		x: var.x,
		y: var.y,
		z: var.z,
		w: var.w,
	}
}
pub(crate) fn vec4_to(var: vec4) -> MVec4 {
	MVec4 {
		x: var.x,
		y: var.y,
		z: var.z,
		w: var.w,
	}
}

//TODO: Get someone really smart to figure out why this doesn't work
pub fn matrix_from(m: MMatrix) -> matrix {
	matrix {
		row: [
			vec4_from(m.x),
			vec4_from(m.y),
			vec4_from(m.z),
			vec4_from(m.w),
		],
	}
}
pub fn matrix_to(m: matrix) -> MMatrix {
	unsafe {
		match m {
			matrix { m: ma } => MMatrix::from(ma),
			matrix { row: r } => ColumnMatrix4::from(RowMatrix4::from([
				r[0].x, r[0].y, r[0].z, r[0].w, r[1].x, r[1].y, r[1].z, r[1].w, r[2].x, r[2].y,
				r[2].z, r[2].w, r[3].x, r[3].y, r[3].z, r[3].w,
			])),
		}
	}
}

pub(crate) fn quat_from(q: MQuat) -> quat {
	quat {
		x: q.v.x,
		y: q.v.y,
		z: q.v.z,
		w: q.s,
	}
}
pub(crate) fn quat_to(q: quat) -> MQuat {
	MQuat::from([q.x, q.y, q.z, q.w])
}
pub(crate) fn pose_to(pose: pose_t) -> Pose {
	let pose_t {
		position,
		orientation,
	} = pose;
	Pose {
		position: vec3_to(position),
		orientation: quat_to(orientation),
	}
}
pub(crate) fn pose_from(pose: Pose) -> pose_t {
	let Pose {
		position,
		orientation,
	} = pose;
	pose_t {
		position: vec3_from(position),
		orientation: quat_from(orientation),
	}
}

pub fn ray_to(ray: ray_t) -> Ray {
	Ray {
		dir: vec3_to(ray.dir),
		pos: vec3_to(ray.pos),
	}
}
