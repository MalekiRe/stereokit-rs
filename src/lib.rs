pub use lifecycle::{Settings, StereoKit};
pub use stereokit_sys as sys;

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

pub mod hierarchy;
pub mod microphone;
#[cfg(feature = "physics")]
pub mod physics;

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

	sk.run(
		|sk| {
			ui::window(
				sk,
				"yooo",
				&mut window_pose,
				[0.3, 0.3].into(),
				ui::WindowType::WindowBody,
				ui::MoveType::MoveExact,
				|ui| {
					ui::layout::layout_cut(ui, ui::layout::Side::Right, 0.0, |layout| {
						layout.ui(|ui| {
							ui.label("hi", false);
						})
					})
				},
			);
		},
		|_| {},
	);
	Ok(())
}

#[cfg(feature = "physics")]
#[test]
fn physics_test() -> color_eyre::eyre::Result<()> {
	color_eyre::install()?;
	let sk = Settings::default().init()?;
	let cube_mesh = mesh::Mesh::gen_cube(
		&sk,
		mint::Vector3 {
			x: 0.2_f32,
			y: 0.2_f32,
			z: 0.2_f32,
		},
		1,
	)?;
	let cube_material = material::Material::copy_from_id(&sk, material::DEFAULT_ID_MATERIAL)?;
	let cube_model = model::Model::from_mesh(&sk, &cube_mesh, &cube_material)?;
	let solid = crate::physics::Solid::new(
		&sk,
		[0.0, 0.4, 0.0].into(),
		Quat::IDENTITY.into(),
		crate::physics::SolidType::Normal,
	)
	.unwrap();
	solid.add_box(&sk, [0.2, 0.2, 0.2].into(), 0.1, [0.0, 0.0, 0.0].into());
	let platform = crate::physics::Solid::new(
		&sk,
		[0.0, 0.0, 0.0].into(),
		Quat::IDENTITY.into(),
		crate::physics::SolidType::Immovable,
	)
	.unwrap();
	platform.add_box(&sk, [20.0, 0.2, 20.0].into(), 0.0, [0.0, 0.0, 0.0].into());
	sk.run(
		|sk| {
			let pose = solid.get_pose(sk);
			cube_model.draw(
				sk,
				Mat4::from_scale_rotation_translation(
					glam::Vec3::new(1.0, 1.0, 1.0),
					pose.orientation.into(),
					pose.position.into(),
				)
				.into(),
				color_named::BLUE,
				RenderLayer::Layer0,
			);
		},
		|_| {},
	);
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
