use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};
use prisma::Rgb;
use crate::lifecycle::DrawContext;
use crate::{StereoKit, text};
use crate::high_level::math_traits::MatrixContainer;
use crate::text::{TextAlign, TextFit, TextStyle};
use crate::values::Color128;

pub struct Text {
    pub text_style: crate::text::TextStyle,
    pub text_align: crate::text::TextAlign,
    pub text_align_pos: text::TextAlign,
    pub text_fit: crate::text::TextFit,
    pub text: String,
    pub size: Vec2,
    pub tint: Color128,
    pub offset: Vec3,
    matrix: MatrixContainer,
}

impl Text {
    pub fn new(sk: &StereoKit, text: impl AsRef<str>, pos: impl Into<crate::values::Vec3>, rot: impl Into<crate::values::Vec3>, scale: impl Into<crate::values::Vec3>) -> Self {
        Self {
            text_style: TextStyle::default(sk),
            text_align: TextAlign::Center,
            text_align_pos: TextAlign::Center,
            text_fit: TextFit::Overflow,
            text: text.as_ref().to_string(),
            size: Default::default(),
            tint: Color128::new(Rgb::new(1.0, 1.0, 1.0), 1.0),
            offset: Default::default(),
            matrix: MatrixContainer::new(pos, rot, scale)
        }
    }
    pub fn draw_in(&self, ctx: &DrawContext) {
        text::draw_in(ctx, &self.text, self.matrix.get_matrix(), self.size, self.text_fit, &self.text_style, self.text_align_pos, self.text_align, self.offset, self.tint);
    }
    pub fn draw_at(&self, ctx: &DrawContext) {
        text::draw_at(ctx, &self.text, self.matrix.get_matrix(), &self.text_style, self.text_align_pos, self.text_align, self.offset, self.tint);
    }
}