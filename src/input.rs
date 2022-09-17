use crate::{
	input::ButtonState::{Active, Any, Changed, Inactive, JustActive, JustInactive},
	pose::Pose,
	values::{Quat, Vec2, Vec3},
	StereoKit,
};
use derive_is_enum_variant::is_enum_variant;
use num_derive::FromPrimitive;
use std::ops::Deref;
use std::slice::Iter;
use std::{fmt::Pointer, mem::transmute};
use stereokit_sys::{button_state_, input_key, key_, pose_t};

pub enum Key {
	None = 0,
	MouseLeft = 1,
	MouseRight = 2,
	MouseCenter = 4,
	MouseForward = 5,
	MouseBack = 6,
	Backspace = 8,
	Tab = 9,
	Return = 13,
	Shift = 16,
	Ctrl = 17,
	Alt = 18,
	CapsLock = 20,
	Esc = 27,
	Space = 32,
	End = 35,
	Home = 36,
	Left = 37,
	Right = 39,
	Up = 38,
	Down = 40,
	PageUp = 33,
	PageDown = 34,
	Printscreen = 42,
	Insert = 45,
	Del = 46,
	Key0 = 48,
	Key1 = 49,
	Key2 = 50,
	Key3 = 51,
	Key4 = 52,
	Key5 = 53,
	Key6 = 54,
	Key7 = 55,
	Key8 = 56,
	Key9 = 57,
	KeyA = 65,
	KeyB = 66,
	KeyC = 67,
	KeyD = 68,
	KeyE = 69,
	KeyF = 70,
	KeyG = 71,
	KeyH = 72,
	KeyI = 73,
	KeyJ = 74,
	KeyK = 75,
	KeyL = 76,
	KeyM = 77,
	KeyN = 78,
	KeyO = 79,
	KeyP = 80,
	KeyQ = 81,
	KeyR = 82,
	KeyS = 83,
	KeyT = 84,
	KeyU = 85,
	KeyV = 86,
	KeyW = 87,
	KeyX = 88,
	KeyY = 89,
	KeyZ = 90,
	KeyNum0 = 96,
	KeyNum1 = 97,
	KeyNum2 = 98,
	KeyNum3 = 99,
	KeyNum4 = 100,
	KeyNum5 = 101,
	KeyNum6 = 102,
	KeyNum7 = 103,
	KeyNum8 = 104,
	KeyNum9 = 105,
	KeyF1 = 112,
	KeyF2 = 113,
	KeyF3 = 114,
	KeyF4 = 115,
	KeyF5 = 116,
	KeyF6 = 117,
	KeyF7 = 118,
	KeyF8 = 119,
	KeyF9 = 120,
	KeyF10 = 121,
	KeyF11 = 122,
	KeyF12 = 123,
	Comma = 188,
	Period = 190,
	SlashFwd = 191,
	SlashBack = 220,
	Semicolon = 186,
	Apostrophe = 222,
	BracketOpen = 219,
	BracketClose = 221,
	Minus = 189,
	Equals = 187,
	Backtick = 192,
	LCmd = 91,
	RCmd = 92,
	Multiply = 106,
	Add = 107,
	Subtract = 109,
	Decimal = 110,
	Divide = 111,
	MAX = 255,
}
#[derive(Debug, FromPrimitive, Copy, Clone, is_enum_variant)]
pub enum ButtonState {
	Inactive = 0,
	Active = 1,
	JustInactive = 2,
	JustActive = 4,
	Changed = 6,
	Any = 2147483647,
}
impl ButtonState {
	pub fn iterator() -> Iter<'static, ButtonState> {
		static BUTTON_STATES: [ButtonState; 6] =
			[Inactive, Active, JustInactive, JustActive, Changed, Any];
		BUTTON_STATES.iter()
	}
}
pub fn key(key: Key) -> Vec<ButtonState> {
	let mut button_states: Vec<ButtonState> = vec![];
	let input = unsafe { input_key(key as key_) } as usize;
	ButtonState::iterator().for_each(|button_state| {
		if (input & (*button_state as usize)) != 0 {
			match button_state {
				Inactive => button_states.push(Inactive),
				Active => button_states.push(Active),
				JustInactive => button_states.push(JustInactive),
				JustActive => button_states.push(JustActive),
				Changed => button_states.push(Changed),
				Any => button_states.push(Any),
			}
		}
	});
	button_states
}
pub fn key_test(key: Key) {
	println!("{}", unsafe { input_key(key as key_) });
}

pub struct Ray {
	pub pos: Vec3,
	pub dir: Vec3,
}
impl Ray {
	pub fn from_mouse(mouse: &Mouse) -> Option<Self> {
		let mut ray = Ray {
			pos: Vec3::from([0.0, 0.0, 0.0]),
			dir: Vec3::from([0.0, 0.0, 0.0]),
		};

		unsafe { stereokit_sys::ray_from_mouse(transmute(mouse.pos), transmute(&mut ray)) != 0 }
			.then_some(ray)
	}
}

pub struct Mouse {
	available: i32,
	pub pos: Vec2,
	pub pos_change: Vec2,
	pub scroll: f32,
	pub scroll_change: f32,
}
impl Mouse {
	pub fn available(&self) -> bool {
		self.available != 0
	}
}

impl StereoKit {
	pub fn input_head(&self) -> &Pose {
		unsafe { transmute(&*stereokit_sys::input_head()) }
	}

	pub fn input_mouse(&self) -> &Mouse {
		unsafe { transmute(&*stereokit_sys::input_mouse()) }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Handed {
	Left = 0,
	Right = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct Joint {
	pub position: Vec3,
	pub orientation: Quat,
	pub radius: f32,
}

/// The fingers go thumb to little, metacarpal to tip
#[derive(Debug, Clone, Copy)]
pub struct Hand {
	pub fingers: [[Joint; 5]; 5],
	pub wrist: Pose,
	pub palm: Pose,
	pub pinch_point: f32,
	pub handedness: Handed,
	pub tracked_state: ButtonState,
	pub pinch_state: ButtonState,
	pub grip_state: ButtonState,
	pub size: f32,
	pub pinch_activation: f32,
	pub grip_activation: f32,
}

impl StereoKit {
	pub fn input_hand(&self, handed: Handed) -> &Hand {
		unsafe { std::mem::transmute(&*stereokit_sys::input_hand(handed as u32)) }
	}
}
