use crate::color;
use crate::icons;

macro_rules! define_urlset {
	(
		$(
			$label:expr,
			$field:ident,
			$short:expr,
			$required_substring:expr,
			$main_color:expr,
			[ $( $extra_color:expr ),* $(,)? ];
		)*
	) => {
		fn assert_is_url(s: &str) {
			assert!(s.starts_with("https://"), "Invalid URL \"{}\"", s);
			assert!(
				!s.contains(char::is_whitespace),
				"Invalid URL \"{}\"", s
			);
		}

		#[derive(Debug)]
		pub struct UrlSet {
			$( $field: Option<String>, )*
		}
		impl UrlSet {
			pub fn empty() -> UrlSet {
				UrlSet {
					$( $field: None, )*
				}
			}
			pub fn from(val: &serde_json::Value) -> UrlSet {
				fn get_str(
					obj: &serde_json::Map<String, serde_json::Value>,
					key: &'static str,
					required_substring: &'static str
				) -> Option<String> {
					obj.get(key).map(|v| {
						let s = v.as_str()
							.unwrap_or_else(|| panic!("\"url\" attribute \"{}\" from JSON is not a string", key))
							.to_string();
						assert_is_url(&s);
						assert!(
							s.contains(required_substring),
							"URL \"{}\" is not valid for platform {}; must contain {}",
							s, key, required_substring
						);
						s
					})
				}
				let obj = crate::globals::map_with_only_these_keys(
					val,
					"UrlSet",
					&[
						$( $label, )*
					]
				);
				UrlSet {
					$( $field: get_str(&obj, $label, $required_substring), )*
				}
			}
			pub fn try_to_get_at_least_one_link(&self) -> Option<&str> {
				$(
					if let Some(url) = &self.$field {
						return Some(&url);
					}
				)*
				None
			}
			pub fn entries(&self) -> Vec<(&'static str, &String, &'static str, &'static str)> {
				let mut out = Vec::new();
				$(
					if let Some(v) = &self.$field {
						out.push((
							$label,
							v,
							$short,
							{
								let white = color::Color(255, 255, 255);
								if color::Color::from($main_color).contrast(&white) > 3.0 {
									"withwhite"
								} else {
									"withblack"
								}
							}
						))
					}
				)*
				out
			}

			pub fn combine(a: Option<&UrlSet>, b: Option<&UrlSet>) -> UrlSet {
				fn pick(a: Option<&String>, b: Option<&String>) -> Option<String> {
					a.cloned().or_else(|| b.cloned())
				}
				UrlSet {
					$(
						$field: pick(
							a.and_then(|x| x.$field.as_ref()),
							b.and_then(|x| x.$field.as_ref())
						),
					)*
				}
			}
			pub fn logo_colors_used(&self) -> Vec<(&'static str, color::Color)> {
				let mut out = Vec::new();
				$(
					if self.$field.is_some() {
						out.push(($label, color::Color::from($main_color)));
						$(
							out.push(($label, color::Color::from($extra_color)));
						)*
					}
				)*
				out
			}
			pub fn check_valid_icons() {
				$(
					if !icons::valid_icon($short) {
						panic!("URL icon {} is not found in icon set; requires recompilation", $short)
					}
				)*
			}
			pub fn platform_shorts_and_main_colors() -> Vec<(&'static str, color::Color)> {
				vec![
					$(
						($short, color::Color::from($main_color)),
					)*
				]
			}
			pub fn linkpage_css_for_platforms() -> String {
				let mut the_string = String::new();
				for (platform_short, main_color) in UrlSet::platform_shorts_and_main_colors() {
					the_string += &format!(".{}{{--acc:{}}}", platform_short, main_color);
				}
				the_string
			}
		}
	};
}

#[rustfmt::skip]
define_urlset! {
	// Name             struct field      svg + css name    url substring       color      other colors to check for contrast (namely gradients)
	"Bandcamp",         bandcamp,         "bandcamp",       "bandcamp.com",     "#1da0c3", [];
	"YouTube",          youtube,          "youtube",        "youtube.com",      "#ff0000", [];
	"YouTube Full Mix", youtube_full_mix, "youtubefullmix", "youtube.com",      "#ff0000", [];
	"Apple Music",      apple_music,      "applemusic",     "music.apple.com",  "#ff2950", ["#ff4e6b", "#ff0335"];
	"Spotify",          spotify,          "spotify",        "open.spotify.com", "#1ed760", [];
	"Soundcloud",       soundcloud,       "soundcloud",     "soundcloud.com",   "#ff5001", ["#fe7500", "#ff3701"];
	"Amazon Music",     amazon_music,     "amazonmusic",    "music.amazon.com", "#25d3da", [];
	"iHeartRadio",      iheartradio,      "iheartradio",    "iheart.com",       "#c6002b", [];
	"Deezer",           deezer,           "deezer",         "deezer.com",       "#a238ff", [];
	"Pandora",          pandora,          "pandora",        "pandora.com",      "#1b86f6", ["#3160f9", "#00a0ee"];
	"Tidal",            tidal,            "tidal",          "tidal.com",        "#33ffee", [];
	"Tencent Music",    tencent_music,    "tencentmusic",   "tencentmusic.com", "#1772f9", [];
}

// ^ These icons are a subset of those provided by icons.rs
// These have additional information such as color, url patterns, and display names
