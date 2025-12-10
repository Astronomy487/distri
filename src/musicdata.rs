use crate::globals;
use crate::imagedeal;
use crate::zipper;

pub struct Date {
	year: u32,
	month: u32, // 1 = january
	day: u32
}
impl Date {
	fn from(yyyy_mm_dd: &str) -> Date {
		fn bad(yyyy_mm_dd: &str) -> ! {
			panic!("Date must be in YYYY-MM-DD format: \"{}\"", yyyy_mm_dd);
		}
		if yyyy_mm_dd.len() != 10 {
			bad(yyyy_mm_dd);
		}
		let year = yyyy_mm_dd[0..4].parse().unwrap_or_else(|_| bad(yyyy_mm_dd));
		let month = yyyy_mm_dd[5..7].parse().unwrap_or_else(|_| bad(yyyy_mm_dd));
		let day = yyyy_mm_dd[8..10]
			.parse()
			.unwrap_or_else(|_| bad(yyyy_mm_dd));
		if !(1..=12).contains(&month) {
			bad(yyyy_mm_dd);
		}
		Date { year, month, day }
	}
	pub fn weekday_name(&self) -> &'static str {
		const LEADING_VALUES: [u32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
		let year = if (self.month as i32) < 3 {
			self.year - 1
		} else {
			self.year
		};
		match (year + year / 4 - year / 100
			+ year / 400
			+ LEADING_VALUES[(self.month - 1) as usize]
			+ self.day)
			% 7
		{
			0 => "Sun",
			1 => "Mon",
			2 => "Tue",
			3 => "Wed",
			4 => "Thu",
			5 => "Fri",
			6 => "Sat",
			_ => unreachable!()
		}
	}
	pub fn to_rfc822(&self) -> String {
		format!(
			"{}, {:02} {} {} 17:00:00 GMT",
			self.weekday_name(),
			self.day,
			match self.month {
				1 => "Jan",
				2 => "Feb",
				3 => "Mar",
				4 => "Apr",
				5 => "May",
				6 => "Jun",
				7 => "Jul",
				8 => "Aug",
				9 => "Sep",
				10 => "Oct",
				11 => "Nov",
				12 => "Dec",
				_ => unreachable!()
			},
			self.year
		)
	}

	pub fn now_rfc822() -> String {
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_else(|_| std::time::Duration::from_secs(0))
			.as_secs();
		let secs = now % 60;
		let mins = (now / 60) % 60;
		let hours = (now / 3600) % 24;
		let days = now / 86400;
		let weekday = (4 + days % 7) % 7;
		let weekday_name = match weekday {
			0 => "Sun",
			1 => "Mon",
			2 => "Tue",
			3 => "Wed",
			4 => "Thu",
			5 => "Fri",
			6 => "Sat",
			_ => unreachable!()
		};
		let z_value = days as i64 + 719468;
		let era = (if z_value >= 0 {
			z_value
		} else {
			z_value - 146096
		}) / 146097;
		let doe = z_value - era * 146097;
		let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
		let year = yoe + era * 400;
		let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
		let mp = (5 * doy + 2) / 153;
		let date = doy - (153 * mp + 2) / 5 + 1;
		let month = mp + if mp < 10 { 3 } else { -9 };
		let year = year + (month <= 2) as i64;
		let month_name = match month {
			1 => "Jan",
			2 => "Feb",
			3 => "Mar",
			4 => "Apr",
			5 => "May",
			6 => "Jun",
			7 => "Jul",
			8 => "Aug",
			9 => "Sep",
			10 => "Oct",
			11 => "Nov",
			12 => "Dec",
			_ => unreachable!()
		};
		format!(
			"{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
			weekday_name, date, month_name, year, hours, mins, secs
		)
	}
}
fn format_duration(seconds: u32) -> String {
	let minutes = (seconds / 60) % 60;
	let hours = (seconds / 3600) % 60;
	let seconds = seconds % 60;
	let mut parts = Vec::new();
	if hours > 0 {
		parts.push(format!("{}h", hours));
	}
	if minutes > 0 {
		parts.push(format!("{}m", minutes));
	}
	if seconds > 0 {
		parts.push(format!("{}s", seconds));
	}
	parts.join(" ")
}
pub fn format_file_size(bytes: u64) -> String {
	const KB: f64 = 1024.0;
	const MB: f64 = 1024.0 * 1024.0;
	const GB: f64 = 1024.0 * 1024.0 * 1024.0;
	let bytes = bytes as f64;
	fn sig3(amount: f64) -> String {
		if amount >= 100.0 {
			format!("{:.0}", amount)
		} else if amount >= 10.0 {
			format!("{:.1}", amount)
		} else {
			format!("{:.2}", amount)
		}
	}
	if bytes >= GB {
		format!("{} GB", sig3(bytes / GB))
	} else if bytes >= MB {
		format!("{} MB", sig3(bytes / MB))
	} else if bytes >= KB {
		format!("{} kB", sig3(bytes / KB))
	} else {
		format!("{:.0} B", bytes)
	}
}
impl std::fmt::Display for Date {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			fmt,
			"{} {}, {}",
			match self.month {
				1 => "Jan",
				2 => "Feb",
				3 => "Mar",
				4 => "Apr",
				5 => "May",
				6 => "Jun",
				7 => "Jul",
				8 => "Aug",
				9 => "Sep",
				10 => "Oct",
				11 => "Nov",
				12 => "Dec",
				_ => unreachable!()
			},
			self.day,
			self.year
		)
	}
}

struct UrlSet {
	spotify: Option<String>,
	apple_music: Option<String>,
	youtube_video: Option<String>,
	youtube_playlist: Option<String>,
	youtube_full_mix: Option<String>,
	bandcamp: Option<String>,
	amazon_music: Option<String>,
	iheartradio: Option<String>,
	soundcloud: Option<String>,
	tencent_music: Option<String>
}
impl UrlSet {
	fn from(val: &serde_json::Value, is_album: bool) -> UrlSet {
		let obj = val.as_object().expect("\"url\" from JSON is not an object");
		let allowed = [
			"Spotify",
			"Apple Music",
			"YouTube",
			"YouTube Full Mix",
			"Bandcamp",
			"Amazon Music",
			"iHeartRadio",
			"Soundcloud",
			"Tencent Music"
		];
		for key in obj.keys() {
			if !allowed.contains(&key.as_str()) {
				panic!("\"url\" object from JSON has unknown key \"{}\"", key);
			}
		}
		UrlSet {
			spotify: obj.get("Spotify").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"Spotify\" from JSON is not a string")
					.to_string()
			}),
			apple_music: obj.get("Apple Music").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"Apple Music\" from JSON is not a string")
					.to_string()
			}),
			youtube_video: if !is_album {
				obj.get("YouTube").map(|v| {
					v.as_str()
						.expect("\"url\" attribute \"YouTube\" from JSON is not a string")
						.to_string()
				})
			} else {
				None
			},
			youtube_playlist: if is_album {
				obj.get("YouTube").map(|v| {
					v.as_str()
						.expect("\"url\" attribute \"YouTube\" from JSON is not a string")
						.to_string()
				})
			} else {
				None
			},
			youtube_full_mix: obj.get("YouTube Full Mix").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"YouTube Full Mix\" from JSON is not a string")
					.to_string()
			}),
			bandcamp: obj.get("Bandcamp").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"Bandcamp\" from JSON is not a string")
					.to_string()
			}),
			amazon_music: obj.get("Amazon Music").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"Amazon Music\" from JSON is not a string")
					.to_string()
			}),
			iheartradio: obj.get("iHeartRadio").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"iHeartRadio\" from JSON is not a string")
					.to_string()
			}),
			soundcloud: obj.get("Soundcloud").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"Soundcloud\" from JSON is not a string")
					.to_string()
			}),
			tencent_music: obj.get("Tencent Music").map(|v| {
				v.as_str()
					.expect("\"url\" attribute \"Tencent Music\" from JSON is not a string")
					.to_string()
			})
		}
	}
	fn iter(&self) -> Vec<(&'static str, &String)> {
		let mut vec = Vec::new();
		if let Some(x) = &self.bandcamp {
			vec.push(("Bandcamp", x));
		}
		if let Some(x) = &self.youtube_video {
			vec.push(("YouTube", x));
		}
		if let Some(x) = &self.youtube_playlist {
			vec.push(("YouTube", x));
		}
		if let Some(x) = &self.youtube_full_mix {
			vec.push(("YouTube Full Mix", x));
		}
		if let Some(x) = &self.apple_music {
			vec.push(("Apple Music", x));
		}
		if let Some(x) = &self.spotify {
			vec.push(("Spotify", x));
		}
		if let Some(x) = &self.soundcloud {
			vec.push(("Soundcloud", x));
		}
		if let Some(x) = &self.amazon_music {
			vec.push(("Amazon Music", x));
		}
		if let Some(x) = &self.iheartradio {
			vec.push(("iHeartRadio", x));
		}
		if let Some(x) = &self.tencent_music {
			vec.push(("Tencent Music", x));
		}
		vec
	}
	fn combine<'a>(a: Option<&'a UrlSet>, b: Option<&'a UrlSet>) -> UrlSet {
		let a_video = a.and_then(|x| x.youtube_video.as_ref());
		let a_playlist = a.and_then(|x| x.youtube_playlist.as_ref());
		UrlSet {
			spotify: a
				.and_then(|x| x.spotify.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.spotify.as_ref().cloned())),
			apple_music: a
				.and_then(|x| x.apple_music.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.apple_music.as_ref().cloned())),
			youtube_video: a
				.and_then(|x| x.youtube_video.as_ref().cloned())
				.or_else(|| {
					if a_playlist.is_some() {
						None
					} else {
						b.and_then(|x| x.youtube_video.as_ref().cloned())
					}
				}),
			youtube_playlist: a
				.and_then(|x| x.youtube_playlist.as_ref().cloned())
				.or_else(|| {
					if a_video.is_some() {
						None
					} else {
						b.and_then(|x| x.youtube_playlist.as_ref().cloned())
					}
				}),
			youtube_full_mix: a
				.and_then(|x| x.youtube_full_mix.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.youtube_full_mix.as_ref().cloned())),
			bandcamp: a
				.and_then(|x| x.bandcamp.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.bandcamp.as_ref().cloned())),
			amazon_music: a
				.and_then(|x| x.amazon_music.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.amazon_music.as_ref().cloned())),
			iheartradio: a
				.and_then(|x| x.iheartradio.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.iheartradio.as_ref().cloned())),
			soundcloud: a
				.and_then(|x| x.soundcloud.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.soundcloud.as_ref().cloned())),
			tencent_music: a
				.and_then(|x| x.tencent_music.as_ref().cloned())
				.or_else(|| b.and_then(|x| x.tencent_music.as_ref().cloned()))
		}
	}
}

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
}
impl std::fmt::Display for Color {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
	}
}
enum PaletteMode {
	Normal,
	White,
	Black
}
struct Palette {
	palette_mode: PaletteMode,
	foreground: Color,
	background: Color,
	accent: Color
}
impl Palette {
	fn style_tag(&self) -> String {
		format!(
			":root{{--bg:{};--fg:{};--acc:{}}}",
			self.background, self.foreground, self.accent
		)
	}
	fn from(v: &serde_json::Value) -> Palette {
		let obj = v
			.as_object()
			.unwrap_or_else(|| panic!("\"color\" from JSON is not an object: {}", v));
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
		Palette {
			foreground: Color::from(fg_str),
			background: Color::from(bg_str),
			accent: Color::from(acc_str),
			palette_mode
		}
	}
}

#[derive(Copy, Clone)]
enum Language {
	English,
	Japanese,
	French,
	Latin
}
impl Language {
	fn iso_639_2(&self) -> &'static str {
		match self {
			Language::English => "eng",
			Language::Japanese => "jpn",
			Language::French => "fra",
			Language::Latin => "lat"
		}
	}
	fn iso_639_1(&self) -> &'static str {
		match self {
			Language::English => "en",
			Language::Japanese => "ja",
			Language::French => "fr",
			Language::Latin => "la"
		}
	}
	fn from(s: &str) -> Language {
		match s {
			"en" => Language::English,
			"eng" => Language::English,
			"ja" => Language::Japanese,
			"jpn" => Language::Japanese,
			"fr" => Language::French,
			"fra" => Language::French,
			"la" => Language::Latin,
			"lat" => Language::Latin,
			_ => {
				panic!("Unrecognized ISO 639 language code \"{}\"", s);
			}
		}
	}
}
#[derive(Clone)]
struct LyricLine {
	start: f32,
	end: f32,
	text: String,
	language: Language,
	vocalist: String
}
impl LyricLine {
	fn from(text: &str) -> Vec<Vec<LyricLine>> {
		let mut last_lang: Option<Language> = None;
		let mut last_vocalist: Option<String> = None;
		let vvll: Vec<Vec<LyricLine>> = text
			.split("\n\n")
			.map(|stanza| {
				stanza
					.lines()
					.filter_map(|line| {
						if line.is_empty() {
							return None;
						}
						let parts: Vec<&str> = line.split('\t').collect();
						if parts.len() < 3 {
							panic!("Invalid lyric line \"{}\"", line);
						}
						let start = parts[0].parse::<f32>().ok()?;
						let end = parts[1].parse::<f32>().ok()?;
						let text = parts[2].to_string();

						let mut lang_override: Option<Language> = None;
						let mut vocalist_override: Option<String> = None;

						for kv in &parts[3..] {
							if let Some((key, value)) = kv.split_once(":") {
								match key {
									"language" => lang_override = Some(Language::from(value)),
									"vocalist" => vocalist_override = Some(value.to_string()),
									_ => panic!("Invalid lyric tag \"{}\"", key)
								}
							}
						}
						let language = if let Some(lang) = lang_override {
							last_lang = Some(lang);
							lang
						} else if let Some(lang) = last_lang {
							lang
						} else {
							panic!("First lyric line (\"{}\") has no language tag", line);
						};
						let vocalist = if let Some(v) = vocalist_override {
							last_vocalist = Some(v.clone());
							v
						} else if let Some(v) = &last_vocalist {
							v.clone()
						} else {
							panic!("First lyric line (\"{}\") has no vocalist tag", line);
						};
						Some(LyricLine {
							start,
							end,
							text,
							language,
							vocalist
						})
					})
					.collect()
			})
			.collect();
		for vll in &vvll {
			if vll.is_empty() {
				panic!("Stanza has no lines. Lyric text:\n```\n{}\n```", text);
			}
		}
		if vvll.is_empty() {
			panic!("Stanza has no lines. Lyric text:\n```\n{}\n```", text);
		}
		vvll
	}
	fn as_plaintext(lyrics: &[Vec<LyricLine>]) -> String {
		lyrics
			.iter()
			.map(|stanza| {
				stanza
					.iter()
					.map(|line| line.to_unsynced())
					.collect::<Vec<_>>()
					.join("\n")
			})
			.collect::<Vec<_>>()
			.join("\n\n")
	}
	fn as_lrc(lyrics: &[Vec<LyricLine>]) -> String {
		// TODO - why do empty lines still have a timestamp? .join("\n\n") is not doing its job
		lyrics
			.iter()
			.map(|stanza| {
				let stanza_lines: Vec<String> =
					stanza.iter().map(|line| line.to_synced_text()).collect();
				stanza_lines.join("\n")
			})
			.collect::<Vec<_>>()
			.join("\n\n")
	}
	fn as_sylt_data(lyrics: &Vec<Vec<LyricLine>>) -> Vec<(u32, String)> {
		let mut synced_content: Vec<(u32, String)> = Vec::new();
		for stanza in lyrics {
			for line in stanza {
				synced_content.push(line.to_synced_pair());
			}
			if let Some(last_line) = stanza.last() {
				synced_content.push((last_line.end_ms(), String::new()));
			}
		}
		synced_content
	}
	fn start_ms(&self) -> u32 {
		(self.start * 1000.0).round() as u32
	}
	fn end_ms(&self) -> u32 {
		(self.end * 1000.0).round() as u32
	}
	fn to_unsynced(&self) -> String {
		self.text.clone()
	}
	fn to_synced_pair(&self) -> (u32, String) {
		(self.start_ms(), self.text.clone())
	}
	fn to_synced_text(&self) -> String {
		let total_ms = self.start_ms();
		let minutes = total_ms / 60000;
		let seconds = (total_ms % 60000) / 1000;
		let hundredths = (total_ms % 1000) / 10; // two‑digit fraction
		format!(
			"[{:02}:{:02}.{:02}] {}",
			minutes, seconds, hundredths, self.text
		)
	}
}

enum Codec {
	Mp3,
	Flac
}
impl Codec {
	fn ext(&self) -> &'static str {
		match self {
			Codec::Mp3 => "mp3",
			Codec::Flac => "flac"
		}
	}
	fn ffmpeg_args(&self, input: &str, output: &str) -> Vec<String> {
		match self {
			Codec::Mp3 => vec![
				"-y".into(),
				"-i".into(),
				input.into(),
				"-codec:a".into(),
				"libmp3lame".into(),
				"-b:a".into(),
				"320k".into(),
				"-map_metadata".into(),
				"-1".into(),
				"-loglevel".into(),
				"panic".into(),
				output.into(),
			],
			Codec::Flac => vec![
				"-y".into(),
				"-i".into(),
				input.into(),
				"-codec:a".into(),
				"flac".into(),
				"-compression_level".into(),
				"8".into(),
				"-map_metadata".into(),
				"-1".into(),
				"-loglevel".into(),
				"panic".into(),
				output.into(),
			]
		}
	}
}

pub struct Album {
	pub songs: Vec<Song>,
	title: String,
	artist: String,
	pub released: Date,
	length: u32,
	pub temporary: bool,
	upc: Option<String>,
	bcid: Option<String>,
	pub about: Option<String>,
	palette: Option<Palette>,
	single: bool,
	url: UrlSet
}

pub struct Song {
	parent_album: Option<(usize, usize)>, /* album-index, position in tracklist. todo rename to parent_album_indeces or something */
	title: String,
	artist: String,
	released: Option<Date>,
	pub bonus: bool,
	event: bool,
	single_artwork: Option<String>,
	length: u32,
	isrc: Option<String>,
	lyrics: Option<Vec<Vec<LyricLine>>>,
	palette: Option<Palette>,
	url: Option<UrlSet>,
	samples: Option<Vec<String>>, // report as "Mix tracklist" if event
	about: Option<String>
}

pub enum Titlable<'a> {
	Song(&'a Song, Option<&'a Album>),
	Album(&'a Album)
}
impl Titlable<'_> {
	fn artist(&self) -> &str {
		match self {
			Titlable::Song(song, _) => &song.artist,
			Titlable::Album(album) => &album.artist
		}
	}
	fn palette(&self) -> &Option<Palette> {
		match self {
			Titlable::Song(song, maybe_parent_album) => {
				if song.palette.is_some() {
					&song.palette
				} else if let Some(album) = maybe_parent_album {
					&album.palette
				} else {
					&song.palette
				}
			}
			Titlable::Album(album) => &album.palette
		}
	}
	fn title(&self) -> &str {
		match self {
			Titlable::Song(song, _) => &song.title,
			Titlable::Album(album) => &album.title
		}
	}
	fn dash(&self) -> &'static str {
		match self {
			Titlable::Song(song, _) => {
				if song.event {
					"@"
				} else {
					"–" // en dash btw
				}
			}
			Titlable::Album(_) => "–" // en dash btw
		}
	}
	fn audio_download_url(&self, codec: &Codec) -> String {
		match self {
			Titlable::Song(song, _) => format!(
				"https://audio.astronomy487.com/{}/{}.{}",
				codec.ext(),
				song.slug(),
				codec.ext()
			),
			Titlable::Album(album) => format!(
				"https://audio.astronomy487.com/{}/{}.zip",
				codec.ext(),
				album.slug()
			)
		}
	}
	fn audio_download_local_location(&self, codec: &Codec) -> std::path::PathBuf {
		match self {
			Titlable::Song(song, _) => std::path::Path::new(globals::FILEZONE)
				.join("audio.astronomy487.com")
				.join(codec.ext())
				.join(song.slug())
				.with_extension(codec.ext()),
			Titlable::Album(album) => std::path::Path::new(globals::FILEZONE)
				.join("audio.astronomy487.com")
				.join(codec.ext())
				.join(album.slug())
				.with_extension("zip")
		}
	}
	fn audio_download_size(&self, codec: &Codec) -> Option<u64> {
		let path = self.audio_download_local_location(codec);
		std::fs::metadata(&path).ok().map(|m| m.len())
	}
	fn format_title(&self) -> String {
		format!("{} {} {}", self.artist(), self.dash(), self.title())
	}
	fn format_title_short(&self) -> String {
		if self.artist() == "Astro"
			&& match self {
				Titlable::Album(_) => true,
				Titlable::Song(song, _) => !song.event
			} {
			self.title().to_string()
		} else {
			format!("{} {} {}", self.artist(), self.dash(), self.title())
		}
	}
	fn slug(&self) -> String {
		let mut slug = if self.artist() != "Astro" {
			format!("{} {}", self.artist(), self.title())
		} else {
			self.title().to_owned()
		};
		slug = slug.to_lowercase();
		slug = unicode_normalization::UnicodeNormalization::nfd(slug.chars())
			.filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
			.collect();
		slug = slug.replace("ke$ha", "kesha").replace("a$tro", "astro");
		let re_punct =
			regex::Regex::new(r#"[()\[\],.?!'"*\$]"#).expect("re_punct is invalid regex");
		let re_sep = regex::Regex::new(r#"[_/&+:;\s]+"#).expect("re_sep is invalid regex");
		let re_dash = regex::Regex::new(r#"-+"#).expect("re_dash is invalid regex");
		slug = re_punct.replace_all(&slug, "").into_owned();
		slug = re_sep.replace_all(&slug, "-").into_owned();
		slug = re_dash.replace_all(&slug, "-").into_owned();
		slug = slug.chars().filter(|c| c.is_ascii()).collect();
		while slug.starts_with('-') {
			slug = slug[1..].to_string();
		}
		while slug.ends_with('-') {
			slug = slug[..slug.len() - 1].to_string();
		}
		slug = slug.replace("--", "-");
		slug
	}
	pub fn make_link_page(&self) {
		let destination_folder = std::path::Path::new(globals::FILEZONE)
			.join("music.astronomy487.com")
			.join(self.slug());
		if destination_folder.exists() {
			std::fs::remove_dir_all(&destination_folder).unwrap_or_else(|_| {
				panic!("Couldn't remove directory {}", destination_folder.display())
			});
		}
		std::fs::create_dir(&destination_folder).unwrap_or_else(|_| {
			panic!("Couldn't create directory {}", destination_folder.display())
		});

		// lyrics files come first. link pages need to report their size
		if let Titlable::Song(song, _) = self
			&& let Some(lxs) = &song.lyrics
		{
			let lyrics_txt_location = destination_folder.join("lyrics.txt");
			let mut file = std::fs::File::create(&lyrics_txt_location).unwrap_or_else(|_| {
				panic!("Couldn't create file {}", lyrics_txt_location.display())
			});
			let _ = std::io::Write::write(&mut file, LyricLine::as_plaintext(lxs).as_bytes())
				.unwrap_or_else(|_| {
					panic!("Couldn't write to file {}", lyrics_txt_location.display())
				});
			let lyrics_lrc_location = destination_folder.join("lyrics.lrc");
			let mut file = std::fs::File::create(&lyrics_lrc_location).unwrap_or_else(|_| {
				panic!("Couldn't create file {}", lyrics_lrc_location.display())
			});
			let _ = std::io::Write::write(&mut file, LyricLine::as_lrc(lxs).as_bytes())
				.unwrap_or_else(|_| {
					panic!("Couldn't write to file {}", lyrics_lrc_location.display())
				});
		}

		let url = format!("https://music.astronomy487.com/{}", self.slug());
		let title = self.format_title();
		let title_short = self.format_title_short();
		let description = self.description();

		let (length, released) = match self {
			Titlable::Album(a) => (a.length, &a.released),
			Titlable::Song(song, parent) => {
				let released = if let Some(date) = &song.released {
					date
				} else if let Some(album) = parent {
					&album.released
				} else {
					panic!(
						"Song {} has no release date and no parent album",
						song.slug()
					)
				};
				(song.length, released)
			}
		};

		let length_str = format_duration(length);

		let artwork = match self {
			Titlable::Album(a) => Some(format!("../{}.jpg", a.slug())),
			Titlable::Song(s, parent) => {
				if let Some(sa) = &s.single_artwork {
					Some(format!("../{}.jpg", sa))
				} else {
					parent
						.as_ref()
						.map(|album| format!("../{}.jpg", album.slug()))
				}
			}
		};

		let palette = self.palette();
		let url_set = match self {
			Titlable::Album(a) => &a.url,
			Titlable::Song(s, parent) => &UrlSet::combine(s.url.as_ref(), parent.map(|a| &a.url))
		};

		let thing_to_offer_as_download = match self {
			Titlable::Album(album) => Titlable::Album(album),
			Titlable::Song(song, maybe_parent_album) => match *maybe_parent_album {
				None => Titlable::Song(song, *maybe_parent_album),
				Some(parent_album) => {
					if parent_album.single {
						Titlable::Album(parent_album)
					} else {
						Titlable::Song(song, *maybe_parent_album)
					}
				}
			}
		};

		let html: maud::Markup = maud::html! {
			(maud::DOCTYPE)
			html lang="en" {
				head {
					meta charset="utf-8";
					link rel="stylesheet" href="https://rsms.me/inter/inter.css";
					link rel="icon" href="../favicon.png" type="image/png";
					link rel="canonical" href=(url);
					link rel="stylesheet" href="../style.css";
					title { (title) }
					@if let Some(p) = palette {
						style { (p.style_tag()) }
						meta name="theme-color" content=(p.background.to_string());
					}
					meta name="description" content=(description);
					meta name="keywords" content="electronic, dance, music, astro, artist, indie, edm";
					meta name="author" content="Astro, astronomy487";
					meta name="robots" content="index, follow";
					meta property="og:site_name" content="astronomy487.com";
					meta property="og:title" content=(title);
					meta property="og:description" content=(description);
					@if let Some(art) = &artwork {
						meta property="og:image" content=(art);
						link rel="apple-touch-icon" href=(art);
					} @else {
						meta property="og:image" content="../squarelogo.png";
						link rel="apple-touch-icon" href="../squarelogo.png";
					}
					meta property="og:url" content=(url);
					meta property="music:musician" content="https://www.astronomy487.com";
					meta property="music:release_date" content=(released);
					meta property="music:duration" content=(length.to_string());

					@match self {
						Titlable::Album(album) => {
							meta property="og:type" content="music.album";
							@for (i, song) in album.songs.iter().enumerate() {
								@if !song.bonus {
									meta property="music:song" content=(format!("https://music.astronomy487.com/{}", song.slug()));
									meta property="music:song:track" content=((i+1).to_string());
								}
							}
						}
						Titlable::Song(song, parent_album) => {
							meta property="og:type" content="music.song";
							@if let Some(album) = parent_album {
								meta property="music:album" content=(format!("https://music.astronomy487.com/{}", album.slug()));
								@let track_num = album.songs.iter().position(|s| {
									match self {
										Titlable::Song(me, _) => std::ptr::eq(*me, s),
										_ => false
									}
								}).unwrap_or_else(|| panic!("Song {} cannot be found in its parent album {}", song.slug(), album.slug())) + 1;
								meta property="music:album:track" content=(track_num.to_string());
							}
						}
					}

					script src="../talktalk/talktalk.min.js" data-talktalk="../talktalk" {}
				}
				body class = {
					@match palette {
						None => "",
						Some(palette) => {
							@match palette.palette_mode {
								PaletteMode::Normal => "",
								PaletteMode::White => "mode-white",
								PaletteMode::Black => "mode-black"
							}
						}
					}
				} {
					@if let Some(a) = &artwork {
						img src=(a) {}
					} @else {
						style { "body { margin-top: 16rem; }" }
					}

					h1 { (title_short) }

					table {
						tr {
							td { (released) }
							@if let Titlable::Album(a) = self {
								td { (format!("{} tracks", a.songs.iter().filter(|s| !s.bonus).count())) }
							}
							td { (length_str) }
						}
					}

					table {
						@for chunk in url_set.iter().chunks(2).into_iter() {
							@let vec = chunk.iter().collect::<Vec<_>>();
							tr {
								@for (key, value) in vec.iter() {
									@let short = key.to_lowercase().replace(' ', "");
									td {
										a class=(format!("{} streamlink", short)) href=(value) {
											img src=(format!("../icons/{}.svg", short)) {}
											span { (key) }
										}
									}
								}
								@if vec.len() == 1 {
									td { }
								}
							}
						}
					}

					table class="bottomlinks" {
						@if let Some(mp3_size) = thing_to_offer_as_download.audio_download_size(&Codec::Mp3) {
							@let flac_size = thing_to_offer_as_download.audio_download_size(&Codec::Flac).expect("mp3 has size but not flac? get serious");
							tr {
								td {
									span data-talktalk="download" { "Download" }
								}
								td {
									a href=(thing_to_offer_as_download.audio_download_url(&Codec::Mp3)) download {
										(Codec::Mp3.ext())
										@if let Titlable::Album(_) = thing_to_offer_as_download {
											" zip"
										}
										", "
										(format_file_size(mp3_size))
									}
								}
								td {
									a href=(thing_to_offer_as_download.audio_download_url(&Codec::Flac)) download {
										(Codec::Flac.ext())
										@if let Titlable::Album(_) = thing_to_offer_as_download {
											" zip"
										}
										", "
										(format_file_size(flac_size))
									}
								}
							}
						}
						@if let Titlable::Song(song, _) = self {
							@if let Some(_) = &song.lyrics {
								tr {
									td {
										span data-talktalk="lyrics" { "Lyrics" }
									}
									td {
										a href="lyrics.txt" download=(&format!("{}-lyrics.txt", song.slug())) {
											"txt, "
											(format_file_size( //TODO this is messy. in general
												std::fs::metadata(std::path::Path::new(globals::FILEZONE).join("music.astronomy487.com").join(song.slug()).join("lyrics").with_extension("txt")).unwrap_or_else(|_|
													panic!("lyrics.txt does not exist for {}", song)
												).len()
											))
										}
									}
									td {
										a href="lyrics.lrc" download=(&format!("{}-lyrics.lrc", song.slug())) {
											"lrc, "
											(format_file_size( //TODO this is messy. in general
												std::fs::metadata(std::path::Path::new(globals::FILEZONE).join("music.astronomy487.com").join(song.slug()).join("lyrics").with_extension("lrc")).unwrap_or_else(|_|
													panic!("lyrics.lrc does not exist for {}", song)
												).len()
											))
										}
									}
								}
							}
						}
					}

					script src="../localizelinkpage.js" {}
				}
			}
		};

		let html_location = destination_folder.join("index.html");
		let mut file = std::fs::File::create(&html_location)
			.unwrap_or_else(|_| panic!("Couldn't create file {}", html_location.display()));
		let _ = std::io::Write::write(&mut file, html.into_string().as_bytes())
			.unwrap_or_else(|_| panic!("Couldn't write to file {}", html_location.display()));
	}
	pub fn description(&self) -> String {
		match self {
			Titlable::Album(album) => {
				let released = album.released.to_string();
				let length = format_duration(album.length);
				let track_count = album.songs.iter().filter(|s| !s.bonus).count();
				format!(
					"Album released {}, {} tracks, {}",
					released, track_count, length
				)
			}
			Titlable::Song(song, parent_opt) => {
				let released = song
					.released
					.as_ref()
					.map(|d| d.to_string())
					.or_else(|| parent_opt.map(|parent| parent.released.to_string()))
					.unwrap_or_else(|| {
						panic!("Song {} has no release date or parent album", song.slug())
					});
				let length = format_duration(song.length);
				match parent_opt {
					None => {
						if song.event {
							format!("DJ set for {} on {}, {}", song.title, released, length)
						} else if !song.artist.is_empty() {
							format!("Remix released {}, {}", released, length)
						} else {
							format!("Mix released {}, {}", released, length)
						}
					}
					Some(parent) => match parent.single {
						false => {
							let track_number = song
								.parent_album
								.expect("Titlable has parent album but also doesn't")
								.1 + 1;
							format!(
								"Track {} on {}, released {}, {}",
								track_number, parent.title, released, length
							)
						}
						true => {
							format!("Song released {}, {}", released, length)
						}
					}
				}
			}
		}
	}
}

impl std::fmt::Display for Album {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{} – {}", self.artist, self.title)
	}
}
impl std::fmt::Display for Song {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{} – {}", self.artist, self.title)
	}
}
impl std::fmt::Debug for Album {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{} – {}", self.artist, self.title)
	}
}
impl std::fmt::Debug for Song {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{} – {}", self.artist, self.title)
	}
}

impl Album {
	pub fn slug(&self) -> String {
		Titlable::Album(self).slug()
	}
	fn from_json(val: &serde_json::Value) -> Album {
		Album {
			songs: {
				let songs_val = val
					.get("songs")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"songs\"", val));

				let songs_arr = songs_val.as_array().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"songs\" is not an array: {}",
						songs_val
					)
				});

				songs_arr.iter().map(Song::from_json).collect()
			},
			artist: match val.get("artist") {
				None => "Astro".to_string(),
				Some(artist_val) => artist_val
					.as_str()
					.unwrap_or_else(|| {
						panic!(
							"Album JSON has non-string \"artist\" attribute: {}",
							artist_val
						)
					})
					.to_string()
			},
			title: {
				let title_val = val
					.get("title")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"title\"", val));

				title_val
					.as_str()
					.unwrap_or_else(|| {
						panic!(
							"Album JSON attribute \"title\" is not a string: {}",
							title_val
						)
					})
					.to_string()
			},
			released: {
				let rel_val = val
					.get("released")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"released\"", val));
				let rel_str = rel_val.as_str().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"released\" is not a string: {}",
						rel_val
					)
				});
				Date::from(rel_str)
			},
			length: {
				let len_val = val
					.get("length")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"length\"", val));
				len_val.as_i64().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"length\" is not an integer: {}",
						len_val
					)
				}) as u32
			},
			temporary: match val.get("temporary") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Album JSON attribute \"temporary\" is not a boolean: {}", v)
				})
			},
			single: match val.get("single") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Album JSON attribute \"single\" is not a boolean: {}", v)
				})
			},
			upc: val.get("upc").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"upc\" is not a string: {}", v)
					})
					.to_owned()
			}),
			bcid: val.get("bcid").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"bcid\" is not a string: {}", v)
					})
					.to_owned()
			}),
			about: val.get("about").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned()
			}),
			palette: val.get("color").map(Palette::from),
			url: {
				let url_val = val
					.get("url")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"url\"", val));

				UrlSet::from(url_val, true)
			}
		}
	}
	pub fn try_encode(&self, all_albums: &[Album]) {
		if !self.temporary {
			for song in &self.songs {
				song.try_encode(all_albums);
			}
			self.zip(&Codec::Mp3);
			self.zip(&Codec::Flac);
		}
	}
	fn zip(&self, codec: &Codec) {
		let destination = std::path::Path::new(globals::FILEZONE)
			.join("audio.astronomy487.com")
			.join(codec.ext())
			.join(self.slug())
			.with_extension("zip");
		if destination.exists() {
			return;
		}
		println!(
			"{}Zipping    {:4}{}   {}",
			globals::ANSI_MAGENTA,
			codec.ext(),
			globals::ANSI_RESET,
			self
		);
		let mut zipper = zipper::Zipper::new(&destination);
		for (song_index, song) in self.songs.iter().enumerate() {
			println!(
				"           {}+{}      {}",
				globals::ANSI_MAGENTA,
				globals::ANSI_RESET,
				song
			);
			zipper.add_file(
				&song.destination_location(codec),
				std::path::Path::new(&format!(
					"{}{:03}-{}.{}",
					if song.bonus { "bonus/" } else { "" },
					song_index + 1,
					song.slug(),
					codec.ext()
				))
			);
			if let Some(lyrics) = &song.lyrics {
				zipper.add_text_file(
					&LyricLine::as_plaintext(lyrics),
					std::path::Path::new(&format!(
						"lyrics/txt/{:03}-{}.txt",
						song_index,
						song.slug()
					))
				);
				zipper.add_text_file(
					&LyricLine::as_lrc(lyrics),
					std::path::Path::new(&format!(
						"lyrics/lrc/{:03}-{}.lrc",
						song_index,
						song.slug()
					))
				);
			}
		}
		zipper.add_text_file(&self.description(), std::path::Path::new("about.txt"));
		zipper.add_file(
			&std::path::Path::new(globals::FILEZONE)
				.join("source")
				.join("image")
				.join(self.slug())
				.with_extension("png"),
			std::path::Path::new("artwork.png")
		);
		zipper.finish();
	}
	fn non_bonus_song_count(&self) -> usize {
		self.songs.iter().take_while(|song| !song.bonus).count()
	}
	fn copyright_message(&self) -> String {
		format!(
			"© {} {}",
			self.released.year,
			self.artist.replace("Astro", "Astro \"astronomy487\"")
		)
	}
	fn description(&self) -> String {
		// utf-8, but use ascii ' " and -
		let mut text = Vec::new();

		text.push(format!("{} - {}", self.artist, self.title));
		text.push(self.released.to_string());

		if let Some(about) = &self.about {
			text.push("".to_string());
			text.push(about.to_string());
		}

		text.push("".to_string());
		for (song_index, song) in self.songs.iter().enumerate() {
			if !song.bonus {
				text.push(format!("{}. {}", song_index + 1, song.title));
			}
		}

		let bonus_song_count = self.songs.len() - self.non_bonus_song_count();
		if bonus_song_count > 0 {
			text.push("".to_string());
			text.push(format!(
				"Bonus track{} included with digital download:",
				if bonus_song_count == 1 { "" } else { "s" }
			));
			text.push("".to_string());
			for (song_index, song) in self.songs.iter().enumerate() {
				if song.bonus {
					text.push(format!("{}. {}", song_index + 1, song.title));
				}
			}
		}

		text.push("".to_string());
		text.push(self.copyright_message());
		if self.artist == "Astro" {
			text.push("Shared under CC BY-NC-SA 4.0 license".to_string())
		}
		text.push("Thank you for downloading!".to_string());
		text.push(format!("https://music.astronomy487.com/{}", self.slug()));

		text.join("\n")
	}
}

impl Song {
	fn slug(&self) -> String {
		Titlable::Song(self, None).slug()
	}
	fn from_json(val: &serde_json::Value) -> Song {
		let mut song = Song {
			parent_album: None,
			artist: match val.get("artist") {
				None => "Astro".to_string(),
				Some(v) => v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"artist\" is not a string: {}", v)
					})
					.to_string()
			},
			title: {
				let t = val
					.get("title")
					.unwrap_or_else(|| panic!("Song JSON {} has no attribute \"title\"", val));
				t.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"title\" is not a string: {}", t)
					})
					.to_string()
			},
			released: val.get("released").map(|v| {
				let s = v.as_str().unwrap_or_else(|| {
					panic!("Song JSON attribute \"released\" is not a string: {}", v)
				});
				Date::from(s)
			}),
			bonus: match val.get("bonus") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Song JSON attribute \"bonus\" is not a boolean: {}", v)
				})
			},
			event: match val.get("event") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Song JSON attribute \"event\" is not a boolean: {}", v)
				})
			},
			single_artwork: None, // single_artwork determined right after construction
			length: {
				let l = val
					.get("length")
					.unwrap_or_else(|| panic!("Song JSON {} has no attribute \"length\"", val));

				l.as_i64().unwrap_or_else(|| {
					panic!("Song JSON attribute \"length\" is not an integer: {}", l)
				}) as u32
			},
			isrc: val.get("isrc").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"isrc\" is not a string: {}", v)
					})
					.to_owned()
			}),
			lyrics: val.get("lyrics").map(|v| {
				let s = v.as_str().unwrap_or_else(|| {
					panic!("Song JSON attribute \"lyrics\" is not a string: {}", v)
				});
				LyricLine::from(s)
			}),
			palette: val.get("color").map(Palette::from),
			url: val.get("url").map(|v| UrlSet::from(v, false)),
			samples: val.get("samples").map(|v| {
				let arr = v.as_array().unwrap_or_else(|| {
					panic!("Song JSON attribute \"samples\" is not an array: {}", v)
				});
				arr.iter()
					.map(|s| {
						s.as_str()
							.unwrap_or_else(|| {
								panic!("Song JSON `samples` element is not a string: {}", s)
							})
							.to_owned()
					})
					.collect()
			}),
			about: val.get("about").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned()
			})
		};

		// artwork handling
		match val.get("artwork") {
			None => {}
			Some(v) => match v {
				serde_json::Value::String(s) => {
					song.single_artwork = Some(s.to_string());
				}
				serde_json::Value::Bool(_) => {
					song.single_artwork = Some(song.slug());
				}
				_ => panic!(
					"Song JSON attribute \"artwork\" must be a string or boolean: {}",
					v
				)
			}
		}

		song
	}

	fn destination_location(&self, codec: &Codec) -> std::path::PathBuf {
		std::path::Path::new(globals::FILEZONE)
			.join(if self.bonus {
				"private"
			} else {
				"audio.astronomy487.com"
			})
			.join(codec.ext())
			.join(self.slug())
			.with_extension(codec.ext())
	}
	fn do_encode(&self, codec: &Codec, all_albums: &[Album]) {
		let input_file_name = match self.parent_album {
			// input_file_name includes album directory where we expect it
			Some((album_index, _)) => all_albums[album_index].slug() + "/" + &self.slug(),
			None => self.slug()
		} + ".flac";
		let input_file = std::path::Path::new(globals::FILEZONE)
			.join("source")
			.join("audio")
			.join(&input_file_name);
		if !input_file.exists() {
			panic!("Could not find audio source {}", input_file_name)
		}
		let destination = self.destination_location(codec);
		if !destination.exists() {
			let args = codec.ffmpeg_args(
				input_file.to_str().unwrap_or_else(|| {
					panic!(
						"FFmpeg input file {} could not be made into a string",
						input_file.display()
					)
				}),
				destination.to_str().unwrap_or_else(|| {
					panic!(
						"FFmpeg output file {} could not be made into a string",
						destination.display()
					)
				})
			);
			println!(
				"{}Encoding   {:4}{}   {}",
				globals::ANSI_YELLOW,
				codec.ext(),
				globals::ANSI_RESET,
				self
			);
			let ffmpeg_status = std::process::Command::new("ffmpeg").args(&args).status();
			if !matches!(ffmpeg_status, std::result::Result::Ok(s) if s.success()) {
				panic!("FFmpeg encoding failed");
			}
		} else {
			// return;
			// TODO ^^^ if runtime is ever too much from re re re re rewriting metadata, return early here
		}

		let f = std::fs::File::open(&input_file).expect("Suddenly the input file doesn't exist");
		let mss = symphonia::core::io::MediaSourceStream::new(Box::new(f), Default::default());
		let hint = symphonia::core::probe::Hint::new();
		let probed = symphonia::default::get_probe()
			.format(
				&hint,
				mss,
				&symphonia::core::formats::FormatOptions::default(),
				&symphonia::core::meta::MetadataOptions::default()
			)
			.expect("Symphonia cannot handle flac audio");
		let track = probed
			.format
			.tracks()
			.iter()
			.find(|t| t.codec_params.sample_rate.is_some())
			.expect("Symphonia couldn't identify audio length");
		let sr = track
			.codec_params
			.sample_rate
			.expect("Symphonia couldn't identify audio length");
		let frames = track
			.codec_params
			.n_frames
			.expect("Symphonia couldn't identify audio length");
		let dur = (frames as f64 / sr as f64).floor() as u32;
		assert!(
			dur == self.length,
			"JSON reports that {} has length {}, but it has length {}",
			self,
			self.length,
			dur
		);
		// woaf is for a public-facing song page; woas is for parent album if it exists, else the song
		let (woaf_string, woas_string) = match (self.bonus, self.parent_album) {
			(true, None) => {
				panic!("Bonus track {} has no parent album", self)
			}
			(true, Some((album_index, _))) => (None, Some(all_albums[album_index].slug())),
			(false, None) => (Some(self.slug()), Some(self.slug())),
			(false, Some((album_index, _))) => {
				(Some(self.slug()), Some(all_albums[album_index].slug()))
			}
		};
		match codec {
			Codec::Mp3 => {
				let mut tag = id3::Tag::new();
				id3::TagLike::set_title(&mut tag, &self.title);
				id3::TagLike::set_artist(&mut tag, &self.artist);
				match self.parent_album {
					Some((album_index, song_index)) => {
						let parent_album = &all_albums[album_index];
						id3::TagLike::set_album(&mut tag, &parent_album.title);
						id3::TagLike::set_album_artist(&mut tag, &parent_album.artist);
						id3::TagLike::set_track(&mut tag, (song_index + 1) as u32);
						id3::TagLike::set_total_tracks(
							&mut tag,
							parent_album.non_bonus_song_count() as u32
						);
					}
					None => {
						id3::TagLike::set_album(&mut tag, &self.title);
						id3::TagLike::set_album_artist(&mut tag, &self.artist);
					}
				}
				id3::TagLike::set_duration(&mut tag, self.length);
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Picture {
						mime_type: "image/jpeg".to_string(),
						picture_type: id3::frame::PictureType::Other,
						description: "".to_string(),
						data: self.grab_artwork(all_albums)
					}
				);
				let release_date = self.release_date(all_albums);
				id3::TagLike::set_year(&mut tag, release_date.year as i32);
				id3::TagLike::set_date_released(
					&mut tag,
					id3::Timestamp {
						year: release_date.year as i32,
						month: Some(release_date.month as u8),
						day: Some(release_date.day as u8),
						hour: None,
						minute: None,
						second: None
					}
				);
				if let Some(isrc) = &self.isrc {
					let _ =
						id3::TagLike::add_frame(&mut tag, id3::frame::Frame::text("TSRC", isrc));
				}
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Frame::link("WOAR", "https://astronomy487.com")
				);
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Frame::text("TENC", globals::ENCODER)
				);
				let _ = id3::TagLike::add_frame(&mut tag, id3::frame::Frame::text("TFLT", "mp3"));
				if let Some((album_index, _)) = self.parent_album {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text(
							"TCOP",
							all_albums[album_index].copyright_message()
						)
					);
				}
				if let Some(lyrics) = &self.lyrics {
					let first_lang = lyrics[0][0].language;
					let lang_code = first_lang.iso_639_2().to_string();
					let mut all_lines = Vec::new();
					for stanza in lyrics {
						let mut collected: Vec<LyricLine> = Vec::new();
						for line in stanza {
							collected.push(line.clone());
						}
						all_lines.push(collected);
					}
					let uslt = id3::frame::Lyrics {
						lang: lang_code.clone(),
						description: "".to_string(),
						text: LyricLine::as_plaintext(&all_lines)
					};
					let _ = id3::TagLike::add_frame(&mut tag, uslt);
					let sylt = id3::frame::Frame::with_content(
						"SYLT",
						id3::frame::Content::SynchronisedLyrics(id3::frame::SynchronisedLyrics {
							lang: lang_code,
							timestamp_format: id3::frame::TimestampFormat::Ms,
							content_type: id3::frame::SynchronisedLyricsType::Other,
							description: "".to_string(),
							content: LyricLine::as_sylt_data(&all_lines)
						})
					);
					let _ = id3::TagLike::add_frame(&mut tag, sylt);
				}
				if tag
					.write_to_path(destination, id3::Version::Id3v24)
					.is_err()
				{
					panic!("Couldn't write mp3 metadata for {}", self);
				}
			}
			Codec::Flac => {
				let mut tag = metaflac::Tag::read_from_path(destination).unwrap_or_else(|_| {
					panic!(
						"Want to write flac metadata for {}, but can't read the file",
						self
					)
				});
				tag.set_vorbis("TITLE", vec![&self.title]);
				tag.set_vorbis("ARTIST", vec![&self.artist]);
				match self.parent_album {
					Some((album_index, song_index)) => {
						let parent_album = &all_albums[album_index];
						tag.set_vorbis("ALBUM", vec![&parent_album.title]);
						tag.set_vorbis("ALBUMARTIST", vec![&parent_album.artist]);
						tag.set_vorbis("TRACKNUMBER", vec![(song_index + 1).to_string()]);
						tag.set_vorbis(
							"TRACKTOTAL",
							vec![parent_album.non_bonus_song_count().to_string()]
						);
					}
					None => {
						tag.set_vorbis("ALBUM", vec![&self.title]);
						tag.set_vorbis("ALBUMARTIST", vec![&self.artist]);
					}
				}
				tag.set_vorbis("LENGTH", vec![self.length.to_string()]);
				let release_date = self.release_date(all_albums);
				tag.set_vorbis(
					"DATE",
					vec![format!(
						"{:04}-{:02}-{:02}",
						release_date.year, release_date.month, release_date.day
					)]
				);
				tag.set_vorbis("YEAR", vec![release_date.year.to_string()]);
				if let Some(isrc) = &self.isrc {
					tag.set_vorbis("ISRC", vec![isrc]);
				}
				tag.set_vorbis("WOAR", vec!["https://astronomy487.com"]);
				tag.add_picture(
					"image/jpeg",
					metaflac::block::PictureType::Other,
					self.grab_artwork(all_albums)
				);
				if let Some(woaf_string) = woaf_string {
					tag.set_vorbis(
						"WOAF",
						vec!["https://music.astronomy487.com/".to_owned() + &woaf_string]
					);
				}
				if let Some(woas_string) = woas_string {
					tag.set_vorbis(
						"WOAS",
						vec!["https://music.astronomy487.com/".to_owned() + &woas_string]
					);
				}
				tag.set_vorbis("ENCODER", vec![globals::ENCODER]);
				tag.set_vorbis("FILETYPE", vec!["flac"]);
				if let Some((album_index, _)) = self.parent_album {
					tag.set_vorbis(
						"COPYRIGHT",
						vec![all_albums[album_index].copyright_message()]
					);
				}
				if let Some(lyrics) = &self.lyrics {
					tag.set_vorbis("LYRICS", vec![&LyricLine::as_plaintext(lyrics)]);
					tag.set_vorbis("LYRICS_SYNCED", vec![&LyricLine::as_lrc(lyrics)]);
				}
				if tag.save().is_err() {
					panic!("Couldn't write flac metadata for {}", self);
				}
			}
		}
	}
	fn release_date<'a>(&'a self, all_albums: &'a [Album]) -> &'a Date {
		match &self.released {
			Some(date) => date,
			None => match self.parent_album {
				Some((album_index, _)) => &all_albums[album_index].released,
				None => {
					panic!("Song {} without parent album must have release date", self);
				}
			}
		}
	}
	fn grab_artwork(&self, all_albums: &[Album]) -> Vec<u8> {
		imagedeal::grab_image(match &self.single_artwork {
			Some(artwork) => artwork.to_string(),
			None => match self.parent_album {
				Some((album_index, _)) => all_albums[album_index].slug(),
				None => "fallback".to_string()
			}
		})
	}
	pub fn try_encode(&self, all_albums: &[Album]) {
		self.do_encode(&Codec::Mp3, all_albums);
		self.do_encode(&Codec::Flac, all_albums);
	}
}

pub fn get_music_data(json_path: &std::path::Path) -> (Vec<Album>, Vec<Song>) {
	let file =
		std::fs::File::open(json_path).unwrap_or_else(|_| panic!("Couldn't find discog.json"));
	let reader = std::io::BufReader::new(file);
	let json_value: serde_json::Value = serde_json::from_reader(reader)
		.unwrap_or_else(|error| panic!("discog.json is invalid JSON: {}", error));
	let remixes: Vec<Song> = json_value
		.get("remixes")
		.expect("discog.json has no attribute \"remixes\"")
		.as_array()
		.expect("discog.json \"remixes\" attribute is not an array")
		.iter()
		.map(Song::from_json)
		.collect();
	let mut albums: Vec<Album> = json_value
		.get("albums")
		.expect("discog.json has no attribute \"albums\"")
		.as_array()
		.expect("discog.json \"albums\" attribute is not an array")
		.iter()
		.map(Album::from_json)
		.collect();

	// assign parent_album refs
	for (album_index, album) in albums.iter_mut().enumerate() {
		for (song_index, song) in album.songs.iter_mut().enumerate() {
			song.parent_album = Some((album_index, song_index));
		}
	}

	// validation
	for remix in &remixes {
		if remix.released.is_none() {
			panic!("Remix {} must have a release date", remix);
		}
		if remix.event && remix.lyrics.is_some() {
			panic!("Remix {} (marked as an event) must not have lyrics", remix);
		}
		if remix.bonus {
			panic!("Remix {} must not be marked as a bonus track", remix)
		}
	}
	let mut seen_slugs = std::collections::HashSet::new();
	let mut check_slug_collision = |s: String| {
		if !seen_slugs.insert(s.clone()) {
			panic!("Two items cannot both have the slug {}", s);
		}
	};
	for album in &albums {
		if album.single {
			if album.title != album.songs[0].title {
				panic!(
					"Single cannot have two different titles: {}, {}",
					album.title, album.songs[0].title
				);
			}
			if album.artist != album.songs[0].artist {
				panic!(
					"Single cannot have two different artists: {}, {}",
					album.title, album.songs[0].title
				);
			}
			check_slug_collision(album.slug());
			for i in 2..album.songs.len() {
				if !album.songs[i].bonus {
					panic!(
						"Additional track in single {} must be marked as bonus",
						album.songs[i]
					);
				}
				check_slug_collision(album.songs[i].slug());
			}
		} else {
			let album_slug = album.slug();
			check_slug_collision(album_slug);
			for song in &album.songs {
				check_slug_collision(song.slug());
			}
		}
	}
	for album in &albums {
		for song in &album.songs {
			if song.event {
				panic!("Album track {} must not be marked as an event", song);
			}
			if song.bonus && song.url.is_some() {
				panic!("Bonus track {} must not have a URL set", song);
			}
			if !song.bonus && song.url.is_none() {
				panic!("Non-bonus track {} must have a URL set", song);
			}
		}
		assert!(!album.songs[0].bonus);
	}

	(albums, remixes)
}
