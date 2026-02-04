use crate::icons;
use crate::color;

macro_rules! define_urlset {
	(
		$(
			$label:expr,
			$field:ident,
			$short:expr,
			$required_substring:expr,
			[ $( $color:expr ),* $(,)? ];
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

			pub fn entries(&self) -> Vec<(&'static str, &String, &'static str, &'static str)> {
				let mut out = Vec::new();
				$(
					if let Some(v) = &self.$field {
						let mut need_a_color = true;
						$(
							#[allow(unused_assignments)] // TODO there has to be a better way to select the first
							if need_a_color {
								out.push(($label, v, $short, { // doing this at runtime for every .entries() call feels excessive. precompilation where r u
									let white = color::Color(255, 255, 255);
									if color::Color::from($color).contrast(&white) > 3.0 {
										"withwhite"
									} else {
										"withblack"
									}
								}));
								need_a_color = false;
							}
						)*
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
						$(
							out.push(($label, color::Color::from($color)));
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
				let mut out = Vec::new();
				$(
					#[allow(unused_assignments)] // TODO there has to be a better way to select the first
					{
						let mut need_a_color = true;
						$(
							if need_a_color {
								out.push(($short, color::Color::from($color)));
								need_a_color = false;
							}
						)*
					}
				)*
				out
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
	"Bandcamp",         bandcamp,         "bandcamp",       "bandcamp.com",     ["#1da0c3"];
	"YouTube",          youtube,          "youtube",        "youtube.com",      ["#ff0000"];
	"YouTube Full Mix", youtube_full_mix, "youtubefullmix", "youtube.com",      ["#ff0000"];
	"Apple Music",      apple_music,      "applemusic",     "music.apple.com",  ["#ff2950", "#ff4e6b", "#ff0335"];
	"Spotify",          spotify,          "spotify",        "open.spotify.com", ["#1ed760"];
	"Soundcloud",       soundcloud,       "soundcloud",     "soundcloud.com",   ["#ff5001", "#fe7500", "#ff3701"];
	"Amazon Music",     amazon_music,     "amazonmusic",    "music.amazon.com", ["#25d3da"];
	"iHeartRadio",      iheartradio,      "iheartradio",    "iheart.com",       ["#c6002b"];
	"Deezer",           deezer,           "deezer",         "deezer.com",       ["#a238ff"];
	"Pandora",          pandora,          "pandora",        "pandora.com",      ["#1b86f6", "#3160f9", "#00a0ee"];
	"Tidal",            tidal,            "tidal",          "tidal.com",        ["#33ffee"];
	//"Tencent Music", tencent_music, "tencentmusic", "tencentmusic.com", [];
}

// ^ These icons are a subset of those provided by icons.rs
// These have additional information such as color, url patterns, and display names

// Don't forget to add new platforms to linkpage-style.css. WISHLIST
// ^^ check the hover states for those url buttons. they might not be optimal