// use crate::model::Model;

use std::cell::RefCell;
use std::ops::{Add, AddAssign, Mul};
use glam::{DQuat, EulerRot, Mat4, Quat, Vec3};
use mint::{EulerAngles, Quaternion};
use prisma::{Color, Rgb};
use stereokit_sys::{matrix, render_layer_};
use crate::lifecycle::DrawContext;
use crate::material::{DEFAULT_ID_MATERIAL, Material};
use crate::mesh::Mesh;
use crate::render::RenderLayer;
use crate::{Settings, StereoKit};
use crate::values::{Color128};
use anyhow::{Context, Result};
use crate::high_level::quat_from_angles;

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

    pub fn from_mesh(sk: &StereoKit, mesh: &Mesh, material: &Material, position: Vec3, rotation: (f32, f32, f32)) -> Result<Self> {
        Model::from_mesh_scale(sk, mesh, material, position, rotation, Vec3::new(1f32, 1f32, 1f32))
    }

    pub fn from_mesh_scale(sk: &StereoKit, mesh: &Mesh, material: &Material, position: Vec3, rotation: (f32, f32, f32), scale: Vec3) -> Result<Self> {
        Model::from_mesh_scale_tint(sk, mesh, material, position, rotation, scale, Color128::new(Rgb::new(1f32, 1f32, 1f32), 1f32))
    }

    pub fn from_mesh_scale_tint(sk: &StereoKit, mesh: &Mesh, material: &Material, position: Vec3, rotation: (f32, f32, f32), scale: Vec3, tint: Color128) -> Result<Self> {
        Model::from_mesh_scale_tint_render_layer(sk, mesh, material, position, rotation, scale, tint, RenderLayer::Layer0)
    }
    pub fn from_mesh_scale_tint_render_layer(sk: &StereoKit, mesh: &Mesh, material: &Material, position: Vec3, rotation: (f32, f32, f32), scale: Vec3, tint: Color128, render_layer: RenderLayer) -> Result<Self> {
        let model = crate::model::Model::from_mesh(sk, mesh, material).context("Unable to create model from mesh")?;
        Ok(Self {
            model,
            matrix: Default::default(),
            scale,
            position,
            rotation: Quat::from_euler(EulerRot::XYZ, rotation.0, rotation.1, rotation.2),
            tint,
            render_layer
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

    pub fn set_pos(&mut self, pos: Vec3) {
        self.position = pos;
        self.sync_matrix();
    }
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
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
        self.position.add_assign(Vec3::new(x, y, z));
        self.sync_matrix();
    }
    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale.add_assign(Vec3::new(x, y, z));
        self.sync_matrix();
    }
}
#[test]
fn model_test() {
        let sk = Settings::default().init().unwrap();
        let mesh = &Mesh::gen_cube(&sk, Vec3::new(1f32, 1f32, 1f32).into(), 1).unwrap();
        let material = &Material::copy_from_id(&sk, DEFAULT_ID_MATERIAL).unwrap();
        let mut model = Model::from_mesh(&sk, mesh, material, Default::default(), (0.0, 0.0, 0.0)).unwrap();

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