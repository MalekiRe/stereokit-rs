pub use lifecycle::{StereoKitSettings, StereoKit};
pub use stereokit_sys as sys;
use crate::info::Display;
use crate::lifecycle::DisplayMode;
use crate::shader::Shader;
use color_eyre::Result;
use crate::color_named::WHITE;

#[macro_use]
pub mod macros;
#[allow(unused)]
pub mod bounds;
#[allow(unused)]
pub mod font;
#[allow(unused)]
pub mod info;
#[allow(unused)]
pub mod input;
#[allow(unused)]
pub mod lifecycle;
#[allow(unused)]
pub mod lines;
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
pub mod sprite;
#[allow(unused)]
pub mod structs;
#[allow(unused)]
pub mod text;
#[allow(unused)]
pub mod texture;
pub mod time;
#[allow(unused)]
pub mod ui;
#[allow(unused)]
pub mod values;

#[allow(unused)]
#[cfg(feature = "high-level")]
pub mod high_level;
pub mod sound;
pub mod world;
pub mod color_named;

#[test]
fn basic() {
	let stereokit = StereoKitSettings::default().init().unwrap();
	stereokit.run(|_| {}, |_| {});
}

/*
#[test]
fn shader_error() -> color_eyre::Result<()> {
	let sk = StereoKitSettings::default().init().unwrap();
	let shader = Shader::from_name(&sk, "yeet nerds")?;
	Ok(())
}

#[test]
fn init_error() -> color_eyre::Result<()> {
	let sk = StereoKitSettings::default().display_preference(DisplayMode::MixedReality).no_flatscreen_fallback(true).init()?;
	Ok(())
}
 */

#[test]
fn test() -> Result<()> {
	use glam::{vec3, Mat4, Quat};
	use prisma::{Rgb, Rgba};

	let stereokit = StereoKitSettings::default().init()?;

	let mut window_pose = pose::Pose::IDENTITY;
	let cube_mesh = mesh::Mesh::gen_cube(
		&stereokit,
		mint::Vector3 {
			x: 1_f32,
			y: 1_f32,
			z: 1_f32,
		},
		1,
	)?;
	let cube_material = material::Material::copy_from_id(&stereokit, material::DEFAULT_ID_MATERIAL)?;

	let cube_model = model::Model::from_mesh(&stereokit, &cube_mesh, &cube_material)?;
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
				color_named::STEEL_BLUE,
				render::RenderLayer::Layer0,
			);
		},
		|_| {},
	);
	Ok(())
}
