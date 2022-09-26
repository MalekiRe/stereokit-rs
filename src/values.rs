use crate::pose::Pose;
use prisma::{FromTuple, Rgba};
use std::fmt::Pointer;
use stereokit_sys::{color128, color32, matrix, quat, text_style_t, vec2, vec3, vec4};

pub(crate) type Vec2 = mint::Vector2<f32>;
pub(crate) type Vec3 = mint::Vector3<f32>;
pub(crate) type Vec4 = mint::Vector4<f32>;
pub(crate) type Quat = mint::Quaternion<f32>;
pub(crate) type Matrix = mint::ColumnMatrix4<f32>;
pub(crate) type Color32 = Rgba<u8>;
pub(crate) type Color128 = Rgba<f32>;

pub(crate) fn vec2_from(var: Vec2) -> vec2 {
	vec2 { x: var.x, y: var.y }
}
pub(crate) fn vec2_to(var: vec2) -> Vec2 {
	Vec2 { x: var.x, y: var.y }
}

pub(crate) fn vec3_from(var: Vec3) -> vec3 {
	vec3 {
		x: var.x,
		y: var.y,
		z: var.z,
	}
}
pub(crate) fn vec3_to(var: vec3) -> Vec3 {
	Vec3 {
		x: var.x,
		y: var.y,
		z: var.z,
	}
}

pub(crate) fn vec4_from(var: Vec4) -> vec4 {
	vec4 {
		x: var.x,
		y: var.y,
		z: var.z,
		w: var.w,
	}
}
pub(crate) fn vec4_to(var: vec4) -> Vec4 {
	Vec4 {
		x: var.x,
		y: var.y,
		z: var.z,
		w: var.w,
	}
}

pub(crate) fn color32_from(var: Color32) -> color32 {
	color32 {
		r: var.red(),
		g: var.green(),
		b: var.blue(),
		a: var.alpha(),
	}
}
pub(crate) fn color32_to(color: color32) -> Color32 {
	Color32::from_tuple(((color.r, color.g, color.b), color.a))
}

pub(crate) fn color128_from(var: Color128) -> color128 {
	color128 {
		r: var.red(),
		g: var.green(),
		b: var.blue(),
		a: var.alpha(),
	}
}
pub(crate) fn color128_to(c: color128) -> Color128 {
	Color128::from_tuple(((c.r, c.g, c.b), c.a))
}

//TODO: Get someone really smart to figure out why this doesn't work
pub(crate) fn matrix_from(m: Matrix) -> matrix {
	matrix {
		row: [
			vec4_from(m.x),
			vec4_from(m.y),
			vec4_from(m.z),
			vec4_from(m.w),
		],
	}
}
pub(crate) fn matrix_to(m: matrix) -> Matrix {
	unsafe {
		match m {
			matrix { m: ma } => Matrix::from(ma),
			matrix { row: r } => Matrix::from([
				r[0].x, r[0].y, r[0].z, r[0].w, r[1].x, r[1].y, r[1].z, r[1].w, r[2].x, r[2].y,
				r[2].z, r[2].w, r[3].x, r[3].y, r[3].z, r[3].w,
			]),
		}
	}
}

pub(crate) fn quat_from(q: Quat) -> quat {
	quat {
		x: q.v.x,
		y: q.v.y,
		z: q.v.z,
		w: q.s,
	}
}
pub(crate) fn quat_to(q: quat) -> Quat {
	Quat::from([q.x, q.y, q.z, q.w])
}
