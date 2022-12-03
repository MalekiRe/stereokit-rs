use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};
use prisma::Rgb;
use crate::lifecycle::{StereoKitContext, StereoKitDraw};
use crate::{StereoKit, text};
use crate::high_level::math_traits::{MatrixContainer, MatrixTrait, PosTrait, RotationTrait, ScaleTrait};
use crate::high_level::WHITE;
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
    pub fn new(sk: &impl StereoKitContext, text: impl AsRef<str>, pos: impl Into<crate::values::MVec3>, rot: impl Into<crate::values::MVec3>, scale: impl Into<crate::values::MVec3>) -> Self {
        Self {
            text_style: TextStyle::default(sk),
            text_align: TextAlign::Center,
            text_align_pos: TextAlign::Center,
            text_fit: TextFit::Overflow,
            text: text.as_ref().to_string(),
            size: Vec2::new(1.0, 1.0),
            tint: WHITE,
            offset: Default::default(),
            matrix: MatrixContainer::new(pos, rot, scale)
        }
    }
    pub fn from(sk: &impl StereoKitContext, text: impl AsRef<str>) -> Self {
        Self::new(sk, text, Vec3::default(), Vec3::new(0.0, 180.0, 0.0), Vec3::new(1.0, 1.0, 1.0))
    }
    pub fn draw_in(&self, ctx: &StereoKitDraw) {
        text::draw_in(ctx, &self.text, self.matrix.get_matrix(), self.size, self.text_fit, &self.text_style, self.text_align_pos, self.text_align, self.offset, self.tint);
    }
    pub fn draw_at(&self, ctx: &StereoKitDraw) {
        text::draw_at(ctx, &self.text, self.matrix.get_matrix(), &self.text_style, self.text_align_pos, self.text_align, self.offset, self.tint);
    }
}
impl PosTrait for Text {
    fn get_pos_vec(&self) -> Vec3 {
        self.matrix.get_pos_vec()
    }

    fn set_pos_vec(&mut self, pos: impl Into<crate::values::MVec3>) {
        self.matrix.set_pos_vec(pos)
    }

    fn translate_vec(&mut self, translation: impl Into<crate::values::MVec3>) {
        self.matrix.translate_vec(translation)
    }
}
impl RotationTrait for Text {
    fn get_rotation_vec(&self) -> Vec3 {
        self.matrix.get_rotation_vec()
    }

    fn set_rotation_vec(&mut self, rotation: impl Into<crate::values::MVec3>) {
        self.matrix.set_rotation_vec(rotation)
    }

    fn rotate_vec(&mut self, rotation: impl Into<crate::values::MVec3>) {
        self.matrix.rotate_vec(rotation)
    }
}
impl ScaleTrait for Text {
    fn get_scale_vec(&self) -> Vec3 {
        self.matrix.get_scale_vec()
    }

    fn set_scale_vec(&mut self, scale: impl Into<crate::values::MVec3>) {
        self.matrix.set_scale_vec(scale)
    }

    fn scale_vec(&mut self, scale: impl Into<crate::values::MVec3>) {
        self.matrix.scale_vec(scale)
    }
}
impl MatrixTrait for Text {
    fn get_matrix(&self) -> Mat4 {
        self.matrix.get_matrix()
    }

    fn set_matrix(&mut self, matrix: Mat4) {
        self.matrix.set_matrix(matrix)
    }
}