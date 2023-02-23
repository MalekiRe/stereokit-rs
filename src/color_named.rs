use crate::values::Color128;

macro_rules! named_color {
	($name:ident, $r:expr, $g:expr, $b:expr) => {
		pub const $name: Color128 =
			Color128::new_rgb($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0);
	};
}

named_color!(WHITE, 255, 255, 255);
named_color!(ALICE_BLUE, 240, 248, 255);
named_color!(AQUA, 0, 255, 255);
named_color!(AQUAMARINE, 127, 255, 212);
named_color!(AZURE, 240, 255, 255);
named_color!(BEIGE, 255, 228, 196);
named_color!(BISQUE, 255, 288, 196);
named_color!(BLACK, 0, 0, 0);
named_color!(BLANCHED_ALMOND, 255, 235, 205);
named_color!(BLUE, 0, 0, 255);
named_color!(BLUE_VIOLET, 138, 42, 226);
named_color!(BROWN, 165, 42, 42);
named_color!(BURLY_WOOD, 222, 184, 135);
named_color!(CADET_BLUE, 95, 158, 160);
named_color!(CHARTREUSE, 127, 255, 0);
named_color!(CHOCOLATE, 210, 105, 30);
named_color!(CORAL, 255, 127, 80);
named_color!(CORNFLOWER_BLUE, 100, 149, 237);
named_color!(CORN_SILK, 255, 248, 220);
named_color!(CRIMSON, 220, 20, 60);
named_color!(CYAN, 0, 255, 255);
named_color!(DARK_BLUE, 0, 0, 139);
named_color!(DARK_CYAN, 0, 139, 139);
named_color!(DARK_GOLDEN_ROD, 184, 134, 11);
named_color!(DARK_GRAY, 169, 169, 169);
named_color!(DARK_GREEN, 0, 100, 0);
named_color!(DARK_GREY, 169, 169, 169);
named_color!(DARK_KHAKI, 189, 183, 107);
named_color!(DARK_MAGENTA, 139, 0, 139);
named_color!(DARK_OLIVE_GREEN, 85, 107, 47);
named_color!(DARK_ORANGE, 255, 140, 0);
named_color!(DARK_ORCHID, 153, 50, 204);
named_color!(DARK_RED, 139, 0, 0);
named_color!(DARK_SALMON, 233, 150, 122);
named_color!(DARK_SEA_GREEN, 143, 188, 143);
named_color!(DARK_SLATE_BLUE, 72, 61, 139);
named_color!(DARK_SLATE_GRAY, 47, 79, 79);
named_color!(DARK_TURQUOISE, 0, 206, 209);
named_color!(DARK_VIOLET, 148, 0, 211);
named_color!(DEEP_PINK, 255, 20, 147);
named_color!(DEEP_SKY_BLUE, 0, 191, 255);
named_color!(DIM_GRAY, 105, 105, 105);
named_color!(DOGER_BLUE, 30, 144, 255);
named_color!(FIRE_BRICK, 178, 34, 34);
named_color!(FLORAL_WHITE, 255, 250, 240);
named_color!(FOREST_GREEN, 34, 139, 34);
named_color!(FUCHSIA, 255, 0, 255);
named_color!(GAINSBORO, 220, 220, 220);
named_color!(GHOST_WHITE, 248, 248, 255);
named_color!(GOLD, 255, 215, 0);
named_color!(GOLDENROD, 218, 165, 32);
named_color!(GRAY, 128, 128, 128);
named_color!(GREY, 128, 128, 128);
named_color!(GREEN, 0, 128, 0);
named_color!(GREEN_YELLOW, 173, 255, 47);
named_color!(HONEYDEW, 240, 255, 240);
named_color!(HOT_PINK, 255, 105, 180);
named_color!(INDIAN_RED, 205, 92, 92);
named_color!(INDIGO, 75, 0, 130);
named_color!(IVORY, 255, 255, 240);
named_color!(KHAKI, 240, 230, 140);
named_color!(LAVENDER, 230, 230, 250);
named_color!(LAVENDER_BLUSH, 255, 240, 245);
named_color!(LAWN_GREEN, 124, 242, 0);
named_color!(LEMON_CHIFFON, 255, 250, 205);
named_color!(LIGHT_BLUE, 173, 216, 230);
named_color!(LIGHT_CORAL, 173, 216, 230);
named_color!(LIGHT_CYAN, 224, 255, 255);
named_color!(LIGHT_GOLDENROD_YELLOW, 250, 250, 210);
named_color!(LIGHT_GRAY, 211, 211, 211);
named_color!(LIGHT_GREEN, 144, 238, 144);
named_color!(LIGHT_GREY, 211, 211, 211);
named_color!(LIGHT_PINK, 255, 182, 193);
named_color!(LIGHT_SALMON, 255, 160, 122);
named_color!(LIGHT_SEA_GREEN, 32, 178, 170);
named_color!(LIGHT_SKY_BLUE, 135, 206, 250);
named_color!(LIGHT_SLATE_GRAY, 119, 136, 153);
named_color!(LIGHT_STEEL_BLUE, 176, 196, 222);
named_color!(LIGHT_YELLOW, 255, 255, 224);
named_color!(LIME, 0, 255, 0);
named_color!(LIME_GREEN, 50, 205, 50);
named_color!(LINEN, 250, 240, 230);
named_color!(MAGENTA, 255, 0, 255);
named_color!(MAROON, 128, 0, 0);
named_color!(MEDIUM_AQUAMARINE, 102, 205, 170);
named_color!(MEDIUM_BLUE, 0, 0, 205);
named_color!(MEDIUM_ORCHID, 186, 85, 211);
named_color!(MEDIUM_PURPLE, 186, 85, 211);
named_color!(MEDIUM_SEA_GREEN, 60, 179, 113);
named_color!(MEDIUM_SLATE_BLUE, 123, 104, 238);
named_color!(MEDIUM_SPRING_GREEN, 0, 250, 154);
named_color!(MEDIUM_TURQUOISE, 72, 209, 204);
named_color!(MEDIUM_VIOLET_RED, 199, 21, 133);
named_color!(MIDNIGHT_BLUE, 25, 25, 112);
named_color!(MINT_CREAM, 245, 255, 250);
named_color!(MISTY_ROSE, 255, 228, 225);
named_color!(MOCCASIN, 255, 228, 181);
named_color!(NAVAJO_WHITE, 255, 222, 173);
named_color!(NAVY, 0, 0, 128);
named_color!(OLD_LACE, 253, 245, 230);
named_color!(OLIVE, 128, 128, 0);
named_color!(OLIVE_DRAB, 107, 142, 35);
named_color!(ORANGE, 255, 165, 0);
named_color!(ORANGE_RED, 255, 69, 0);
named_color!(ORCHID, 218, 112, 214);
named_color!(PALE_GOLDEN_ROD, 238, 232, 170);
named_color!(PALE_GREEN, 152, 251, 152);
named_color!(PALE_TURQUOISE, 175, 238, 238);
named_color!(PALE_VIOLET_RED, 219, 112, 147);
named_color!(PAPAYAWHIP, 255, 239, 213);
named_color!(PEACH_PUFF, 255, 218, 185);
named_color!(PERU, 205, 133, 63);
named_color!(PINK, 255, 192, 203);
named_color!(PLUM, 221, 160, 221);
named_color!(POWDER_BLUE, 176, 224, 230);
named_color!(PURPLE, 128, 0, 128);
named_color!(REBECCA_PURPLE, 102, 51, 153);
named_color!(RED, 255, 0, 0);
named_color!(ROSY_BROWN, 188, 143, 143);
named_color!(ROYAL_BLUE, 65, 105, 225);
named_color!(SADDLE_BROWN, 139, 69, 19);
named_color!(SALMON, 250, 128, 114);
named_color!(SANDY_BROWN, 244, 164, 96);
named_color!(SEA_GREEN, 46, 139, 87);
named_color!(SEA_SHELL, 255, 245, 238);
named_color!(SIENNA, 160, 82, 45);
named_color!(SILVER, 192, 192, 192);
named_color!(SKY_BLUE, 135, 206, 235);
named_color!(SLATE_BLUE, 106, 90, 205);
named_color!(SLATE_GRAY, 112, 128, 144);
named_color!(SNOW, 255, 250, 250);
named_color!(SPRING_GREEN, 0, 255, 127);
named_color!(STEEL_BLUE, 70, 130, 180);
named_color!(TAN, 210, 180, 140);
named_color!(TEAL, 0, 128, 128);
named_color!(THISTLE, 216, 191, 216);
named_color!(TOMATO, 255, 99, 71);
named_color!(TURQUOISE, 64, 224, 208);
named_color!(VIOLET, 238, 130, 238);
named_color!(WHEAT, 245, 222, 179);
named_color!(WHITE_SMOKE, 245, 245, 245);
named_color!(YELLOW, 255, 255, 0);
named_color!(YELLOW_GREEN, 154, 205, 50);
