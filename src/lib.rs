pub use lifecycle::{Settings, StereoKit};
pub use stereokit_sys as sys;

#[allow(unused)]
pub mod constants;
#[allow(unused)]
pub mod enums;
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
#[allow(unused)]
pub mod ui;
#[allow(unused)]
pub mod values;

#[test]
fn test() {
	let stereokit = Settings::default()
		.init()
		.expect("StereoKit failed to initialize");

	let mut window_pose = pose::IDENTITY;
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
	let cube_material = material::Material::copy_from_id(&stereokit, "default")
		.expect("Could not copy default material from id");

	let cube_model = model::Model::from_mesh(&stereokit, cube_mesh, cube_material)
		.expect("Could not make model out of mesh and material");
	stereokit.run(
		|_sk, ctx| {
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

			// cube_model.draw(_sk, Matrix::IDENTITY, Color::, enums::RenderLayer::Layer0);
		},
		|| {},
	);
}
