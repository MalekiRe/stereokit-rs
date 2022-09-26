use crate::input::key;
use crate::lifecycle::DrawContext;
use crate::pose::Pose;
use crate::text::{self, TextAlign, TextStyle};
use crate::values::{color128_from, matrix_from, Color128, Matrix, Vec3};
use crate::StereoKit;
use prisma::FromTuple;
use std::ffi::CString;
use std::rc::{Rc, Weak};
use stereokit_sys::{text_add_at, text_align_, text_size};

pub struct RichText {
	text_modules: Vec<TextModule>,
	transform: Matrix,
	padding: f32,
}
pub struct TextModule {
	pub text: String,
	pub text_style: TextStyle,
}
impl RichText {
	pub fn new(sk: &StereoKit, transform: Matrix, padding: f32) -> Self {
		RichText {
			text_modules: vec![],
			transform,
			padding,
		}
	}
	pub fn clear(&mut self) {
		self.text_modules.clear();
	}
	pub fn push(&mut self, text_module: TextModule) {
		self.text_modules.push(text_module);
	}
	pub fn pop(&mut self) {
		self.text_modules.pop();
	}
	pub fn remove(&mut self, pos: usize) {
		self.text_modules.remove(pos);
	}
	pub fn iterator(&mut self) {
		self.text_modules.iter();
	}
	pub fn get_modules(&mut self) -> &Vec<TextModule> {
		&self.text_modules
	}

	pub fn draw(&mut self, ctx: &DrawContext) {
		let white = Color128::from_tuple(((100., 100., 100.), 100.));
		let mut last: Option<&TextModule> = None;
		let mut total_offset = 0.0f32;
		for text_module in &self.text_modules {
			text::draw_at(
				ctx,
				&text_module.text,
				self.transform,
				&text_module.text_style,
				TextAlign::TopLeft,
				TextAlign::TopLeft,
				Vec3::from([-total_offset, 0.0, 0.0]),
				white,
			);
			total_offset += text::size(&text_module.text, &text_module.text_style).x;
		}
	}
}
