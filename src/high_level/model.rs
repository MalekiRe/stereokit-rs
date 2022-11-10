// use crate::model::Model;

use std::cell::RefCell;
use std::ops::{Add, AddAssign, Deref, Mul};
use glam::{DQuat, EulerRot, Mat4, Quat, Vec3};
use mint::{EulerAngles, Quaternion};
use prisma::{Color, Rgb, Rgba};
use stereokit_sys::{matrix, render_layer_};
use crate::lifecycle::DrawContext;
use crate::material::{DEFAULT_ID_MATERIAL, Material};
use crate::mesh::Mesh;
use crate::render::RenderLayer;
use crate::{Settings, StereoKit};
use crate::values::{Color128};
use anyhow::{Context, Result};
use crate::high_level::{Pos, quat_from_angles, Scale};
use crate::high_level::math_traits::{MatrixContainer, PosTrait, RotationTrait, ScaleTrait};
use crate::input::Handed::Right;

pub struct Model {
    pub model: crate::model::Model,
    matrix: MatrixContainer,
    pub tint: Color128,
    pub render_layer: RenderLayer,
}

impl Model {

    pub fn from_mesh(sk: &StereoKit, mesh: &Mesh, material: &Material) -> Result<Self> {
        let model = crate::model::Model::from_mesh(sk, mesh, material).context("Unable to create model from mesh")?;
        Ok(Self {
            model,
            matrix: MatrixContainer::new(Vec3::default(), Vec3::new(0f32, 0f32, 0f32), [1f32, 1f32, 1f32]),
            tint: Rgba::new(Rgb::new(1.0, 1.0, 1.0), 1.0),
            render_layer: RenderLayer::Layer0
        })
    }

    pub fn draw(&self, ctx: &DrawContext) {
        self.model.draw(ctx, self.matrix.get_matrix().into(), self.tint, self.render_layer)
    }

    pub fn contains(&self, sk: &StereoKit, point: Vec3) -> bool {
        let inverted_matrix = self.matrix.get_matrix().inverse();
        let new_point = inverted_matrix.transform_point3(point);
        self.model.get_bounds(sk).bounds_point_contains(new_point.into())
    }
}

impl PosTrait for Model {
    fn get_pos_vec(&self) -> Vec3 {
        self.matrix.get_pos_vec()
    }

    fn set_pos_vec(&mut self, pos: impl Into<crate::values::Vec3>) {
        self.matrix.set_pos_vec(pos)
    }

    fn translate_vec(&mut self, translation: impl Into<crate::values::Vec3>) {
        self.matrix.translate_vec(translation)
    }
}

impl ScaleTrait for Model {
    fn get_scale_vec(&self) -> Vec3 {
        self.matrix.get_scale_vec()
    }

    fn set_scale_vec(&mut self, scale: impl Into<crate::values::Vec3>) {
        self.matrix.set_scale_vec(scale)
    }

    fn scale_vec(&mut self, scale: impl Into<crate::values::Vec3>) {
        self.matrix.scale_vec(scale)
    }
}

impl RotationTrait for Model {
    fn get_rotation_vec(&self) -> Vec3 {
        self.matrix.get_rotation_vec()
    }

    fn set_rotation_vec(&mut self, rotation: impl Into<crate::values::Vec3>) {
        self.matrix.set_rotation_vec(rotation)
    }

    fn rotate_vec(&mut self, rotation: impl Into<crate::values::Vec3>) {
        self.matrix.rotate_vec(rotation)
    }
}

#[test]
fn bound_test() {
    let sk = Settings::default().init().unwrap();
    let mesh = &Mesh::gen_cube(&sk, Vec3::new(1f32, 1f32, 1f32).into(), 1).unwrap();
    let material = &Material::copy_from_id(&sk, DEFAULT_ID_MATERIAL).unwrap();
    let mut model = Model::from_mesh(&sk, mesh, material).unwrap();
    model.set_pos(1.1, 0.0, 0.0);
    sk.run(|sk, ctx| {
        model.draw(ctx);
        let palm_pos = sk.input_hand(Right).palm.position;
        if model.contains(sk, palm_pos.into()) {
            model.translate(1f32, 0f32, 0f32);
        }
    }, |_| {});
}

#[test]
fn model_test() {
        let sk = Settings::default().init().unwrap();
        let mesh = &Mesh::gen_cube(&sk, Vec3::new(1f32, 1f32, 1f32).into(), 1).unwrap();
        let material = &Material::copy_from_id(&sk, DEFAULT_ID_MATERIAL).unwrap();
        let mut model = Model::from_mesh(&sk, mesh, material).unwrap();
        model.set_pos_vec([0.1, 0.0, 0.0]);
        let mut red_val = 1f32;
        sk.run(|sk, ctx| {
            model.draw(ctx);
            model.rotate(0.0f32, 0.05f32, 0f32);
            model.scale(0.001f32, 0f32, 0f32);
            model.translate(0f32, 0f32, 0.01f32);
            model.tint.set_red(red_val);
            red_val += 0.0005f32;
        }, |_| {});
}

