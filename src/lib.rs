#[allow(unused)]
pub mod constants;
#[allow(unused)]
pub mod enums;
#[allow(unused)]
pub mod font;
#[allow(unused)]
pub mod functions;
#[allow(unused)]
pub mod input;
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

pub use stereokit_sys as sys;

#[test]
fn test() {
	functions::SKSettings::default().init();
	functions::sk_run(
		{
			let window_pose = std::rc::Rc::from(std::cell::RefCell::from(pose::IDENTITY));

			move || {
				ui::window::begin(
					"StereoKit Test",
					&mut *window_pose.borrow_mut(),
					mint::Vector2 { x: 0., y: 0. },
					ui::window::WindowType::WindowNormal,
					ui::window::MoveType::MoveExact,
				);

				ui::ui::label("Test Label", true);
				ui::ui::label("Test Text", true);

				ui::window::end();
			}
		},
		|| {},
	);
}
