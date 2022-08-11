pub use lifecycle::{Settings, StereoKit};
pub use stereokit_sys;

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
	stereokit.run(
		|| {
			ui::window::begin(
				"StereoKit Test",
				&mut window_pose,
				mint::Vector2 { x: 0., y: 0. },
				ui::window::WindowType::WindowNormal,
				ui::window::MoveType::MoveExact,
			);

			ui::ui::label("Test Label", true);
			ui::ui::label("Test Text", true);

			ui::window::end();
		},
		|| {},
	);
}
