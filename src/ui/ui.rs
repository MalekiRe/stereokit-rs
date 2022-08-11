use stereokit_sys::{bool32_t, text_align_, ui_label, ui_sameline, ui_settings, ui_space, ui_text};

pub type UISettings = stereokit_sys::ui_settings_t;
pub fn settings(settings: UISettings) {
	unsafe {
		ui_settings(settings);
	}
}
