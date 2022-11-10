// use crate::model::Model;

use std::cell::RefCell;
use std::ops::{Add, AddAssign, Mul};
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
use crate::input::Handed::Right;

pub struct Model {
    pub model: crate::model::Model,
    matrix: Mat4,
    scale: Vec3,
    position: Vec3,
    rotation: Quat,
    pub tint: Color128,
    pub render_layer: RenderLayer,
}

impl Model {
    fn sync_matrix(&mut self) {
        self.matrix = Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
    }
    fn sync_matrix_subfields(&mut self) {
        (self.scale, self.rotation, self.position) = self.matrix.to_scale_rotation_translation();
    }

    pub fn from_mesh(sk: &StereoKit, mesh: &Mesh, material: &Material) -> Result<Self> {
        let model = crate::model::Model::from_mesh(sk, mesh, material).context("Unable to create model from mesh")?;
        Ok(Self {
            model,
            matrix: Default::default(),
            scale: Vec3::new(1f32, 1f32, 1f32),
            position: Vec3::default(),
            rotation: Quat::from_euler(EulerRot::XYZ, 0f32, 0f32, 0f32),
            tint: Rgba::new(Rgb::new(1.0, 1.0, 1.0), 1.0),
            render_layer: RenderLayer::Layer0
        })
    }

    pub fn draw(&self, ctx: &DrawContext) {
        self.model.draw(ctx, self.matrix.into(), self.tint, self.render_layer)
    }


    pub fn get_pos(&self) -> Vec3 {
        self.position
    }
    pub fn get_rotation(&self) -> Quat {
        self.rotation
    }
    pub fn get_scale(&self) -> Vec3 {
        self.scale
    }
    pub fn get_matrix(&self) -> Mat4 {
        self.matrix
    }

    pub fn set_rotation(&mut self, rotation: (f32, f32, f32)) {
        self.rotation = quat_from_angles(rotation.0, rotation.1, rotation.2);
        self.sync_matrix();
    }
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.sync_matrix();
    }
    pub fn set_matrix(&mut self, matrix: Mat4) {
        self.matrix = matrix;
        self.sync_matrix_subfields();
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = self.rotation.mul_quat(quat_from_angles(x, y, z));
        self.sync_matrix();
    }
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.translate_vec(Vec3::new(x, y, z));
    }
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale_vec(Vec3::new(x, y, z));
    }

    pub fn translate_vec(&mut self, translation: Vec3) {
        self.position.add_assign(translation);
        self.sync_matrix();
    }
    pub fn scale_vec(&mut self, scale: Vec3) {
        self.scale.add_assign(scale);
        self.sync_matrix();
    }

    pub fn contains(&self, sk: &StereoKit, point: Vec3) -> bool {
        let inverted_matrix = self.matrix.inverse();
        let new_point = inverted_matrix.transform_point3(point);
        self.model.get_bounds(sk).bounds_point_contains(new_point.into())
    }
}
#[test]
fn bound_test() {
    let sk = Settings::default().init().unwrap();
    let mesh = &Mesh::gen_cube(&sk, Vec3::new(1f32, 1f32, 1f32).into(), 1).unwrap();
    let material = &Material::copy_from_id(&sk, DEFAULT_ID_MATERIAL).unwrap();
    let mut model = Model::from_mesh(&sk, mesh, material).unwrap();
    model.set_pos((1, 0, 0));
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
        model.set_pos((0.1, 0.0, 0.0));
        let mut red_val = 1f32;
        sk.run(|sk, ctx| {
            model.draw(ctx);
            model.rotate(0f32, 0.1f32, 0f32);
            model.scale(0.001f32, 0f32, 0f32);
            model.translate(0f32, 0f32, 0.01f32);
            model.tint.set_red(red_val);
            red_val += 0.0005f32;
        }, |_| {});
}

trait SetPos<T> {
    fn set_pos(&mut self, new_pos: T);
}

impl SetPos<Vec3> for Model {
    fn set_pos(&mut self, new_pos: Vec3) {
        self.position = new_pos;
        self.sync_matrix();
    }
}

impl SetPos<(f32, f32, f32)> for Model {
    fn set_pos(&mut self, new_pos: (f32, f32, f32)) {
        self.set_pos(Vec3::new(new_pos.0, new_pos.1, new_pos.2));
    }
}

impl SetPos<(i32, i32, i32)> for Model {
    fn set_pos(&mut self, new_pos: (i32, i32, i32)) {
        self.set_pos(Vec3::new(new_pos.0 as f32, new_pos.1 as f32, new_pos.2 as f32));
    }
}