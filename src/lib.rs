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
    functions::SKSettings::default().init();

    functions::sk_run_data(
        &mut Box::new(&mut move || {}),
        &mut Box::new(&mut || {
            println!("Shutting down StereoKit");
        }),
    );
}
