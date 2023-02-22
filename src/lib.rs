pub use lifecycle::{Settings, StereoKit};
pub use stereokit_sys as sys;
use crate::ui::{layout, MoveType, window, WindowType};
use crate::ui::layout::Side;

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

pub mod color_named;
#[allow(unused)]
#[cfg(feature = "high-level")]
pub mod high_level;
pub mod sound;
pub mod world;

pub mod microphone;

#[test]
fn basic() {
	let stereokit = Settings::default().init().unwrap();
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
fn rect_cut() -> color_eyre::eyre::Result<()> {
	color_eyre::install()?;
	let sk = Settings::default().init()?;
	let mut window_pose = pose::Pose::IDENTITY;

	sk.run(|sk| {
		window(sk, "yooo", &mut window_pose, [0.3, 0.3].into(), WindowType::WindowBody, MoveType::MoveExact, |ui| {
			layout::layout_cut(ui, Side::Right, 0.0, |layout| {
				layout.ui(|ui| {
					ui.label("hi", false);
				})
			})
		});
	}, |_| {});
	Ok(())
}

#[test]
fn test() -> color_eyre::eyre::Result<()> {
	color_eyre::install()?;
	tracing_subscriber::fmt()
		.with_max_level(tracing::Level::DEBUG)
		.init();

	use glam::{vec3, Mat4, Quat};

	let stereokit = Settings::default()
		.log_filter(crate::lifecycle::LogFilter::None)
		.init()?;

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
	let cube_material =
		material::Material::copy_from_id(&stereokit, material::DEFAULT_ID_MATERIAL)?;

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
