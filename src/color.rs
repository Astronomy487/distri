use crate::globals;
use crate::url;

#[derive(Clone, Debug)]
pub struct Color(pub u8, pub u8, pub u8);
impl Color {
	pub fn from(hex_code: &str) -> Color {
		assert!(
			hex_code.starts_with('#'),
			"Invalid color string \"{}\" from JSON",
			hex_code
		);
		assert!(
			hex_code.len() == 7,
			"Invalid color string \"{}\" from JSON",
			hex_code
		);
		let red = u8::from_str_radix(&hex_code[1..3], 16)
			.unwrap_or_else(|_| panic!("Invalid color string \"{}\" from JSON", hex_code));
		let green = u8::from_str_radix(&hex_code[3..5], 16)
			.unwrap_or_else(|_| panic!("Invalid color string \"{}\" from JSON", hex_code));
		let blue = u8::from_str_radix(&hex_code[5..7], 16)
			.unwrap_or_else(|_| panic!("Invalid color string \"{}\" from JSON", hex_code));
		assert!(
			hex_code == hex_code.to_lowercase(),
			"Color string \"{}\" must be lowercase",
			hex_code
		);
		Color(red, green, blue)
	}
	fn lightness(&self) -> f32 {
		fn convert(integer: u8) -> f32 {
			let float = f32::from(integer) / 255.0;
			if float <= 0.03928 {
				float / 12.92
			} else {
				((float + 0.055) / 1.055).powf(2.4)
			}
		}
		let red = convert(self.0);
		let green = convert(self.1);
		let blue = convert(self.2);
		red * 0.2126 + green * 0.7152 + blue * 0.0722
	}
	pub fn contrast(&self, other: &Color) -> f32 {
		let lum1 = self.lightness();
		let lum2 = other.lightness();
		(lum1.max(lum2) + 0.05) / (lum1.min(lum2) + 0.05)
	}
	fn lerp(&self, amount: f32, other: &Color) -> Color {
		fn clamp(value: f32) -> u8 {
			value.clamp(0.0, 255.0) as u8
		}
		let red = f32::from(self.0) * (1.0 - amount) + f32::from(other.0) * amount;
		let green = f32::from(self.1) * (1.0 - amount) + f32::from(other.1) * amount;
		let blue = f32::from(self.2) * (1.0 - amount) + f32::from(other.2) * amount;
		Color(clamp(red), clamp(green), clamp(blue))
	}
	fn find_min_towards(&self, other: &Color, required_contrast: f32) -> Color {
		let mut low = 0.0;
		let mut high = 1.0;
		assert!(
			self.contrast(other) >= required_contrast,
			"Could not determine a gray color for the palette; indicates that the foreground is already invalid?"
		);
		while high - low > 0.01 {
			let mid = f32::midpoint(low, high);
			let candidate = self.lerp(mid, other);
			let contrast = self.contrast(&candidate);
			if contrast >= required_contrast {
				high = mid;
			} else {
				low = mid;
			}
		}
		self.lerp(high, other)
	}
}
impl std::fmt::Display for Color {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
	}
}
#[derive(PartialEq, Clone, Debug)]
pub enum PaletteMode {
	Normal,
	White,
	Black
}
#[derive(Clone, Debug)]
pub struct Palette {
	palette_mode: PaletteMode,
	foreground: Color,
	background: Color,
	gray: Color,
	accent: Color
}
impl Palette {
	pub fn home_page_album_needs_borders(&self) -> bool {
		self.background.lightness() < 0.002
	}
	pub fn style_tag(&self) -> String {
		format!(":root{{{}}}", self.style_text())
	}
	pub fn style_text(&self) -> String {
		format!(
			"--bg:{};--fg:{};--acc:{};--gray:{}",
			self.background, self.foreground, self.accent, self.gray
		)
	}
	pub fn html_theme_color(&self) -> String {
		self.background.to_string()
	}
	pub fn palette_mode_as_css_class_name(&self) -> Option<&'static str> {
		match self.palette_mode {
			PaletteMode::Normal => None,
			PaletteMode::White => Some("mode-white"),
			PaletteMode::Black => Some("mode-black")
		}
	}
	pub fn from(val: &serde_json::Value, url_set: &url::UrlSet) -> Palette {
		let obj = globals::map_with_only_these_keys(
			val,
			"Color",
			&["foreground", "background", "accent", "mode"]
		);
		let fg_val = obj.get("foreground").unwrap_or_else(|| {
			panic!(
				"\"color\" from JSON has no \"foreground\" attribute: {}",
				val
			)
		});
		let fg_str = fg_val.as_str().unwrap_or_else(|| {
			panic!(
				"\"foreground\" (\"color\") from JSON is not a string: {}",
				fg_val
			)
		});
		let bg_val = obj.get("background").unwrap_or_else(|| {
			panic!(
				"\"color\" from JSON has no \"background\" attribute: {}",
				val
			)
		});
		let bg_str = bg_val.as_str().unwrap_or_else(|| {
			panic!(
				"\"background\" (\"color\") from JSON is not a string: {}",
				bg_val
			)
		});
		let acc_val = obj
			.get("accent")
			.unwrap_or_else(|| panic!("\"color\" from JSON has no \"accent\" attribute: {}", val));
		let acc_str = acc_val.as_str().unwrap_or_else(|| {
			panic!(
				"\"accent\" (\"color\") from JSON is not a string: {}",
				acc_val
			)
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
		let foreground = Color::from(fg_str);
		let background = Color::from(bg_str);
		let palette = Palette {
			gray: background.find_min_towards(&foreground, 4.5),
			foreground,
			background,
			accent: Color::from(acc_str),
			palette_mode
		};
		assert!(
			palette.foreground.contrast(&palette.background) >= 4.5,
			"Foreground color {} has insufficient contrast with background color {}",
			palette.foreground,
			palette.background
		);
		assert!(
			palette.accent.contrast(&palette.background) >= 3.0,
			"Accent color {} has insufficient contrast with background color {}",
			palette.accent,
			palette.background
		);

		let mut logo_colors_that_dont_pass: Vec<&'static str> = Vec::new();
		for (platform, the_color) in url_set.logo_colors_used() {
			let contrast = the_color.contrast(&palette.background);
			if contrast < 3.0 {
				logo_colors_that_dont_pass.push(platform);
			}
		}
		if logo_colors_that_dont_pass.is_empty() && palette.palette_mode != PaletteMode::Normal {
			// panic!("Your palette ({}, {}, {}) could use normal mode but does not", palette.background, palette.accent, palette.foreground);
		}
		assert!(
			logo_colors_that_dont_pass.is_empty() || palette.palette_mode != PaletteMode::Normal,
			"Background color {} has insufficient contrast for {}",
			palette.background,
			logo_colors_that_dont_pass.join(", ")
		);

		match &palette.palette_mode {
			PaletteMode::Normal => {}
			PaletteMode::White => {
				assert!(
					Color(255, 255, 255).contrast(&palette.background) >= 3.0,
					"Background {} is too bright for white-mode palette",
					palette.background
				);
			}
			PaletteMode::Black => {
				assert!(
					Color(0, 0, 0).contrast(&palette.background) >= 3.0,
					"Background {} is too dark for black-mode palette",
					palette.background
				);
			}
		}
		palette
	}
}
