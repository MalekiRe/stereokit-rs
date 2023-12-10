use glam::{vec3, Mat4};
use stereokit::{Material, RenderLayer, SettingsBuilder, StereoKitDraw, StereoKitMultiThread};
use stereokit_sys::color128;

fn main() {
	let sk = SettingsBuilder::new().init().unwrap();
	let cube_mesh = sk.mesh_gen_cube(vec3(0.1, 0.1, 0.1), 1);
	let cube_material = sk.material_copy(Material::UI_BOX);
	sk.run(
		|d| {
			d.mesh_draw(
				&cube_mesh,
				&cube_material,
				Mat4::IDENTITY,
				color128 {
					r: 1.0,
					g: 1.0,
					b: 1.0,
					a: 1.0,
				},
				RenderLayer::all(),
			)
		},
		|_| {},
	);
}
