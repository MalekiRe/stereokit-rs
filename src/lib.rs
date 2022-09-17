pub use lifecycle::{Settings, StereoKit};
pub use stereokit_sys as sys;

#[allow(unused)]
pub mod font;
#[allow(unused)]
pub mod input;
#[allow(unused)]
pub mod lifecycle;
#[allow(unused)]
pub mod material;
#[allow(unused)]
pub mod mesh;
#[allow(unused)]
pub mod model;
#[allow(unused)]
pub mod pose;
#[allow(unused)]
pub mod render;
#[allow(unused)]
pub mod richtext;
#[allow(unused)]
pub mod shader;
#[allow(unused)]
pub mod structs;
#[allow(unused)]
pub mod textstyle;
#[allow(unused)]
pub mod texture;
mod time;
#[allow(unused)]
pub mod ui;
#[allow(unused)]
pub mod values;

#[test]
fn basic() {
	let stereokit = Settings::default().init().unwrap();
	stereokit.run(|_| {}, || {});
}

#[test]
fn test() {
	use glam::{vec3, Mat4, Quat};
	use prisma::{Rgb, Rgba};

	let stereokit = Settings::default()
		.init()
		.expect("StereoKit failed to initialize");

	let mut window_pose = pose::Pose::IDENTITY;
	let cube_mesh = mesh::Mesh::gen_cube(
		&stereokit,
		mint::Vector3 {
			x: 1_f32,
			y: 1_f32,
			z: 1_f32,
		},
		1,
	)
	.expect("Failed to generate cube");
	let cube_material = material::Material::copy_from_id(&stereokit, material::DEFAULT_ID_MATERIAL)
		.expect("Could not copy default material from id");

	let cube_model = model::Model::from_mesh(&stereokit, &cube_mesh, &cube_material)
		.expect("Could not make model out of mesh and material");
	stereokit.run(
		|ctx| {
			ui::window(
				ctx,
				"StereoKit Test",
				&mut window_pose,
				mint::Vector2 { x: 0., y: 0. },
				ui::window::WindowType::WindowNormal,
				ui::window::MoveType::MoveExact,
				|ui| {
					ui.label("Test Label", true);
					ui.sameline();
					ui.label("Test Text", true);
				},
			);

			cube_model.draw(
				ctx,
				Mat4::from_scale_rotation_translation(
					vec3(0.1, 0.1, 0.1),
					Quat::IDENTITY,
					vec3(0., 0., 0.),
				)
				.into(),
				Rgba::new(Rgb::new(1_f32, 1_f32, 1_f32), 1_f32).into(),
				render::RenderLayer::Layer0,
			);
		},
		|| {},
	);
}
