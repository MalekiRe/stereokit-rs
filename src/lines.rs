use crate::{
	lifecycle::DrawContext,
	pose::Pose,
	text::TextAlign,
	values::{color32_from, vec3_from, Color128, Color32, Matrix, Vec3},
};
use stereokit_sys::line_point_t;

#[derive(Debug, Clone)]
pub struct LinePoint {
	pub point: Vec3,
	pub thickness: f32,
	pub color: Color32,
}

pub fn line_add(
	_draw_ctx: &DrawContext,
	start: Vec3,
	end: Vec3,
	color_start: Color32,
	color_end: Color32,
	thickness: f32,
) {
	unsafe {
		stereokit_sys::line_add(
			vec3_from(start),
			vec3_from(end),
			color32_from(color_start),
			color32_from(color_end),
			thickness,
		);
	}
}
pub fn line_addv(_draw_ctx: &DrawContext, start: &LinePoint, end: &LinePoint) {
	let start = line_point_t {
		pt: vec3_from(start.point),
		thickness: start.thickness,
		color: color32_from(start.color),
	};
	let end = line_point_t {
		pt: vec3_from(end.point),
		thickness: end.thickness,
		color: color32_from(end.color),
	};
	unsafe {
		stereokit_sys::line_addv(start, end);
	}
}
pub fn line_add_axis(_draw_ctx: &DrawContext, pose: Pose, size: f32) {
	unsafe {
		stereokit_sys::line_add_axis(std::mem::transmute(pose), size);
	}
}
pub fn line_add_list(points: &[Vec3], color: Color32, thickness: f32) {
	unsafe {
		stereokit_sys::line_add_list(
			points.as_ptr() as *const _,
			points.len() as i32,
			color32_from(color),
			thickness,
		);
	}
}
pub fn line_add_listv(_draw_ctx: &DrawContext, points: &[LinePoint]) {
	let points: Vec<line_point_t> = points
		.into_iter()
		.map(|p| line_point_t {
			pt: vec3_from(p.point),
			thickness: p.thickness,
			color: color32_from(p.color),
		})
		.collect();
	unsafe {
		stereokit_sys::line_add_listv(points.as_ptr(), points.len() as i32);
	}
}
