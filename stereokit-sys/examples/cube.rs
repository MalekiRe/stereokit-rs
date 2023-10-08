	use std::ptr;

use stereokit_sys::*;
fn main() {
	unsafe {
		if sk_init(sk_settings_t {
			app_name: ptr::null(),
			assets_folder: ptr::null(),
			display_preference: 0,
			blend_preference: 0,
			no_flatscreen_fallback: 0,
			depth_mode: 0,
			log_filter: 0,
			overlay_app: 0,
			overlay_priority: 0,
			flatscreen_pos_x: 0,
			flatscreen_pos_y: 0,
			flatscreen_width: 0,
			flatscreen_height: 0,
			disable_flatscreen_mr_sim: 0,
			disable_desktop_input_window: 0,
			disable_unfocused_sleep: 0,
			render_scaling: 1.0,
			render_multisample: 0,
			android_java_vm: ptr::null_mut(),
			android_activity: ptr::null_mut(),
    		origin: origin_mode__origin_mode_floor,
		}) == 0
		{
			panic!("Unable to initialize StereoKit");
		}

		sk_run(Some(step), None);
	}
}

unsafe extern "C" fn step() {
	mesh_draw(
		mesh_find(std::mem::transmute(default_id_mesh_cube)),
		material_find(std::mem::transmute(default_id_material_ui_box)),
		matrix_ts(
			vec3 {
				x: 0.0,
				y: 0.0,
				z: -0.5,
			},
			vec3 {
				x: 0.1,
				y: 0.1,
				z: 0.1,
			},
		),
		color128 {
			r: 1.0,
			g: 1.0,
			b: 1.0,
			a: 1.0,
		},
		1,
	);
}
