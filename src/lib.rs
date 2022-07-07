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
	let settings = functions::SKSettingBuilder::default().build().unwrap();
	functions::sk_init(settings);

	// functions::sk_run_data(on_update: &mut Box<&mut dyn FnMut()>, on_close: &mut Box<&mut dyn FnMut()>);
}
