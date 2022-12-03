use crate::{
	lifecycle::StereoKitDraw,
	pose::Pose,
	text::TextAlign,
	values::{vec3_from, Color128, Color32, MMatrix, MVec3},
};
use stereokit_sys::line_point_t;

#[derive(Debug, Clone)]
pub struct LinePoint {
	pub point: MVec3,
	pub thickness: f32,
	pub color: Color32,
}

pub fn line_add(
    _draw_ctx: &StereoKitDraw,
    start: MVec3,
    end: MVec3,
    color_start: impl Into<Color32>,
    color_end: impl Into<Color32>,
    thickness: f32,
) {
	unsafe {
		stereokit_sys::line_add(
			vec3_from(start),
			vec3_from(end),
			color_start.into(),
			color_end.into(),
			thickness,
		);
	}
}
pub fn line_addv(_draw_ctx: &StereoKitDraw, start: &LinePoint, end: &LinePoint) {
	let start = line_point_t {
		pt: vec3_from(start.point),
		thickness: start.thickness,
		color: start.color,
	};
	let end = line_point_t {
		pt: vec3_from(end.point),
		thickness: end.thickness,
		color: end.color,
	};
	unsafe {
		stereokit_sys::line_addv(start, end);
	}
}
pub fn line_add_axis(_draw_ctx: &StereoKitDraw, pose: Pose, size: f32) {
	unsafe {
		stereokit_sys::line_add_axis(std::mem::transmute(pose), size);
	}
}
pub fn line_add_list(points: &[MVec3], color: impl Into<Color32>, thickness: f32) {
	unsafe {
		stereokit_sys::line_add_list(
			points.as_ptr() as *const _,
			points.len() as i32,
			color.into(),
			thickness,
		);
	}
}
pub fn line_add_listv(_draw_ctx: &StereoKitDraw, points: &[LinePoint]) {
	let points: Vec<line_point_t> = points
		.into_iter()
		.map(|p| line_point_t {
			pt: vec3_from(p.point),
			thickness: p.thickness,
			color: p.color,
		})
		.collect();
	unsafe {
		stereokit_sys::line_add_listv(points.as_ptr(), points.len() as i32);
	}
}
