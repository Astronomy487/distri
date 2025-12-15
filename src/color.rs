use crate::globals;

struct Color(u8, u8, u8);
impl Color {
	fn from(s: &str) -> Color {
		if !s.starts_with('#') {
			panic!("Invalid color string \"{}\" from JSON", s);
		}
		if s.len() != 7 {
			panic!("Invalid color string \"{}\" from JSON", s);
		}
		let r = u8::from_str_radix(&s[1..3], 16)
			.unwrap_or_else(|_| panic!("Invalid color string \"{}\" from JSON", s));
		let g = u8::from_str_radix(&s[3..5], 16)
			.unwrap_or_else(|_| panic!("Invalid color string \"{}\" from JSON", s));
		let b = u8::from_str_radix(&s[5..7], 16)
			.unwrap_or_else(|_| panic!("Invalid color string \"{}\" from JSON", s));
		Color(r, g, b)
	}
	fn lightness(&self) -> f32 {
		let mut r = self.0 as f32;
		let mut g = self.1 as f32;
		let mut b = self.2 as f32;
		r = if r < 0.03928 {
			r / 12.92
		} else {
			((r + 0.055) / 1.055).powf(2.4)
		};
		g = if g < 0.03928 {
			g / 12.92
		} else {
			((g + 0.055) / 1.055).powf(2.4)
		};
		b = if b < 0.03928 {
			b / 12.92
		} else {
			((b + 0.055) / 1.055).powf(2.4)
		};
		r * 0.2126 + g * 0.7152 + b * 0.0722
	}
	fn contrast(&self, other: &Color) -> f32 {
		let lum1 = self.lightness();
		let lum2 = other.lightness();
		(lum1.max(lum2) + 0.05) / (lum1.min(lum2) + 0.05)
	}
}
impl std::fmt::Display for Color {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
	}
}
pub enum PaletteMode {
	Normal,
	White,
	Black
}
pub struct Palette {
	palette_mode: PaletteMode,
	foreground: Color,
	background: Color,
	accent: Color
}
impl Palette {
	pub fn style_tag(&self) -> String {
		format!(
			":root{{--bg:{};--fg:{};--acc:{}}}",
			self.background, self.foreground, self.accent
		)
	}
	pub fn html_theme_color(&self) -> String {
		self.background.to_string()
	}
	pub fn palette_mode_as_css_class_name(&self) -> &'static str {
		match self.palette_mode {
			PaletteMode::Normal => "",
			PaletteMode::White => "mode-white",
			PaletteMode::Black => "mode-black"
		}
	}
	pub fn from(v: &serde_json::Value) -> Palette {
		let obj = globals::map_with_only_these_keys(v, "Color", &["fg", "bg", "acc", "mode"]);
		let fg_val = obj
			.get("fg")
			.unwrap_or_else(|| panic!("\"color\" from JSON has no \"fg\" attribute: {}", v));
		let fg_str = fg_val
			.as_str()
			.unwrap_or_else(|| panic!("\"fg\" (\"color\") from JSON is not a string: {}", fg_val));
		let bg_val = obj
			.get("bg")
			.unwrap_or_else(|| panic!("\"color\" from JSON has no \"bg\" attribute: {}", v));
		let bg_str = bg_val
			.as_str()
			.unwrap_or_else(|| panic!("\"bg\" (\"color\") from JSON is not a string: {}", bg_val));
		let acc_val = obj
			.get("acc")
			.unwrap_or_else(|| panic!("\"color\" from JSON has no \"acc\" attribute: {}", v));
		let acc_str = acc_val.as_str().unwrap_or_else(|| {
			panic!("\"acc\" (\"color\") from JSON is not a string: {}", acc_val)
		});
		let palette_mode = match obj.get("mode") {
			None => PaletteMode::Normal,
			Some(mode_val) => {
				let mode_str = mode_val.as_str().unwrap_or_else(|| {
					panic!(
						"\"mode\" (\"color\") from JSON is not a string: {}",
						mode_val
					)
				});
				match mode_str {
					"white" => PaletteMode::White,
					"black" => PaletteMode::Black,
					other => panic!(
						"\"mode\" (\"color\") from JSON is an invalid string: {}",
						other
					)
				}
			}
		};
		let palette = Palette {
			foreground: Color::from(fg_str),
			background: Color::from(bg_str),
			accent: Color::from(acc_str),
			palette_mode
		};
		if palette.foreground.contrast(&palette.background) < 4.5 {
			panic!(
				"Foreground color {} has insufficient contrast with background color {}",
				palette.foreground, palette.background
			);
		}
		if palette.accent.contrast(&palette.background) < 3.0 {
			panic!(
				"Accent color {} has insufficient contrast with background color {}",
				palette.accent, palette.background
			);
		}
		let logo_colors_of_concern: Vec<(&'static str, Color)> = match &palette.palette_mode {
			PaletteMode::Normal => {
				vec![
					("Amazon Music", Color(37, 211, 218)),
					("Apple Music (lightest)", Color(255, 78, 107)),
					("Apple Music (darkest)", Color(255, 3, 53)),
					("Bandcamp", Color(29, 160, 195)),
					("iHeartRadio", Color(198, 0, 43)),
					("Soundcloud (lightest)", Color(254, 117, 0)),
					("Soundcloud (darkest)", Color(255, 55, 1)),
					("Spotify", Color(30, 215, 96)),
					("Tencent Music", Color(23, 114, 249)),
					("YouTube", Color(255, 0, 0)),
				]
			}
			PaletteMode::White => {
				vec![("Pure white", Color(255, 255, 255))]
			}
			PaletteMode::Black => {
				vec![("Pure black", Color(0, 0, 0))]
			}
		};
		for (concern_source, logo_color_of_concern) in logo_colors_of_concern {
			if logo_color_of_concern.contrast(&palette.background) < 3.0 {
				panic!(
					"{} color {} has insufficient contrast with background color {}",
					concern_source, logo_color_of_concern, palette.background
				);
			}
		}
		palette
	}
}
