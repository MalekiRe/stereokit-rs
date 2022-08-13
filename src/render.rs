use crate::texture::Texture;

pub struct SphericalHarmonics {
	pub(crate) spherical_harmonics: stereokit_sys::spherical_harmonics_t,
}

pub fn set_skylight(light: &SphericalHarmonics) {
	unsafe {
		stereokit_sys::render_set_skylight(&light.spherical_harmonics);
	}
}

pub fn set_skytex(tex: &Texture) {
	unsafe {
		stereokit_sys::render_set_skytex(tex.tex.as_ptr());
	}
}
