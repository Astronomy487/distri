use crate::globals;
use crate::imagedeal;
use crate::zipper;

pub struct Date {
	year: u32,
	month: u32, // 1 = january
	day: u32
}
impl Date {
	fn from(s: &str) -> Date {
		assert!(s.len() == 10, "Date must be in YYYY-MM-DD format");
		let year = s[0..4].parse().unwrap();
		let month = s[5..7].parse().unwrap();
		let day = s[8..10].parse().unwrap();
		Date { year, month, day }
	}
}
fn format_duration(n: u32) -> String {
	let seconds = n % 60;
	let minutes = (n / 60) % 60;
	let hours = (n / 3600) % 60;
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
    let b = bytes as f64;
    fn fmt_sig3(value: f64) -> String {
        let sci = format!("{:.2e}", value); // e.g. "1.234e+2"
        let num: f64 = sci.parse().unwrap();
        format!("{}", num)
    }
    if b >= GB {
        format!("{} GB", fmt_sig3(b / GB))
    } else if b >= MB {
        format!("{} MB", fmt_sig3(b / MB))
    } else if b >= KB {
        format!("{} kB", fmt_sig3(b / KB))
    } else {
        format!("{:.0} B", b)
	}
}
impl std::fmt::Display for Date {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
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
				_ => {
					panic!("......what kind of month is {}", self.month)
				}
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
	fn from(v: &serde_json::Value, is_album: bool) -> UrlSet {
		let obj = v.as_object().expect("discog `url` is not object");
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
				panic!("Unknown music service in discog `url`: {}", key);
			}
		}

		UrlSet {
			spotify: obj.get("Spotify").map(|v| {
				v.as_str()
					.expect("discog `Spotify` url is not string")
					.to_string()
			}),
			apple_music: obj.get("Apple Music").map(|v| {
				v.as_str()
					.expect("discog `Apple Music` url is not string")
					.to_string()
			}),
			youtube_video: if !is_album {
				obj.get("YouTube").map(|v| {
					v.as_str()
						.expect("discog `YouTube` url is not string")
						.to_string()
				})
			} else {
				None
			},
			youtube_playlist: if is_album {
				obj.get("YouTube").map(|v| {
					v.as_str()
						.expect("discog `YouTube Playlist` url is not string")
						.to_string()
				})
			} else {
				None
			},
			youtube_full_mix: obj.get("YouTube Full Mix").map(|v| {
				v.as_str()
					.expect("discog `YouTube Full Mix` url is not string")
					.to_string()
			}),
			bandcamp: obj.get("Bandcamp").map(|v| {
				v.as_str()
					.expect("discog `Bandcamp` url is not string")
					.to_string()
			}),
			amazon_music: obj.get("Amazon Music").map(|v| {
				v.as_str()
					.expect("discog `Amazon Music` url is not string")
					.to_string()
			}),
			iheartradio: obj.get("iHeartRadio").map(|v| {
				v.as_str()
					.expect("discog `iHeartRadio` url is not string")
					.to_string()
			}),
			soundcloud: obj.get("Soundcloud").map(|v| {
				v.as_str()
					.expect("discog `Soundcloud` url is not string")
					.to_string()
			}),
			tencent_music: obj.get("Tencent Music").map(|v| {
				v.as_str()
					.expect("discog `Tencent Music` url is not string")
					.to_string()
			})
		}
	}
	fn iter(&self) -> Vec<(&'static str, &String)> {
		let mut v = Vec::new();
		
		if let Some(x) = &self.bandcamp {
			v.push(("Bandcamp", x));
		}
		if let Some(x) = &self.youtube_video {
			v.push(("YouTube", x));
		}
		if let Some(x) = &self.youtube_playlist {
			v.push(("YouTube", x));
		}
		if let Some(x) = &self.youtube_full_mix {
			v.push(("YouTube Full Mix", x));
		}
		if let Some(x) = &self.apple_music {
			v.push(("Apple Music", x));
		}
		if let Some(x) = &self.spotify {
			v.push(("Spotify", x));
		}
		if let Some(x) = &self.soundcloud {
			v.push(("Soundcloud", x));
		}
		if let Some(x) = &self.amazon_music {
			v.push(("Amazon Music", x));
		}
		if let Some(x) = &self.iheartradio {
			v.push(("iHeartRadio", x));
		}
		if let Some(x) = &self.tencent_music {
			v.push(("Tencent Music", x));
		}
		v
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
			panic!("Color string must start with '#', got: {s}");
		}
		if s.len() != 7 {
			panic!("Color string must be 7 characters long (e.g. #ff00aa), got: {s}");
		}
		let r = u8::from_str_radix(&s[1..3], 16)
			.unwrap_or_else(|_| panic!("Invalid red component in color string: {s}"));
		let g = u8::from_str_radix(&s[3..5], 16)
			.unwrap_or_else(|_| panic!("Invalid green component in color string: {s}"));
		let b = u8::from_str_radix(&s[5..7], 16)
			.unwrap_or_else(|_| panic!("Invalid blue component in color string: {s}"));
		Color(r, g, b)
	}
}
impl std::fmt::Display for Color {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
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
		Palette {
			foreground: Color::from(
				v.as_object()
					.expect("discog `color` is not object")
					.get("fg")
					.expect("discog `color` has no fg")
					.as_str()
					.expect("discog palette has non-string foreground")
			),
			background: Color::from(
				v.as_object()
					.expect("discog `color` is not object")
					.get("bg")
					.expect("discog `color` has no bg")
					.as_str()
					.expect("discog palette has non-string foreground")
			),
			accent: Color::from(
				v.as_object()
					.expect("discog `color` is not object")
					.get("acc")
					.expect("discog `color` has no acc")
					.as_str()
					.expect("discog palette has non-string foreground")
			),
			palette_mode: match v
				.get("mode")
				.map(|v| v.as_str().expect("discog palette `mode` is not string"))
			{
				None => PaletteMode::Normal,
				Some("white") => PaletteMode::White,
				Some("black") => PaletteMode::Black,
				Some(&_) => panic!("discog palette `mode` is poorly-formed")
			}
		}
	}
}

#[derive(PartialEq, Clone, Copy)]
enum Language {
	English,
	Japanese,
	French
}
impl Language {
	fn to_iso_639_2(&self) -> &'static str {
		match self {
			Language::English => "eng",
			Language::Japanese => "jpn",
			Language::French => "fra"
		}
	}
	fn to_iso_639_1(&self) -> &'static str {
		match self {
			Language::English => "en",
			Language::Japanese => "ja",
			Language::French => "fr"
		}
	}
	fn from(s: &str) -> Language {
		match s {
			"en" => Language::English,
			"ja" => Language::Japanese,
			"fr" => Language::French,
			_ => {
				panic!("Unrecognized language {}", s);
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
		text.split("\n\n")
			.map(|stanza| {
				stanza
					.lines()
					.filter_map(|line| {
						let parts: Vec<&str> = line.split('\t').collect();
						if parts.len() < 3 {
							return None;
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
									_ => panic!("Unknown lyric tag: {}", key)
								}
							}
						}
						let language = if let Some(lang) = lang_override {
							last_lang = Some(lang);
							lang
						} else if let Some(lang) = last_lang {
							lang
						} else {
							panic!("First lyric line in the file must specify a language");
						};
						let vocalist = if let Some(v) = vocalist_override {
							last_vocalist = Some(v.clone());
							v
						} else if let Some(v) = &last_vocalist {
							v.clone()
						} else {
							panic!("First lyric line in the file must specify a vocalist");
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
			.collect()
	}
	fn as_plaintext(lyrics: &Vec<Vec<LyricLine>>) -> String {
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
	fn as_lrc(lyrics: &Vec<Vec<LyricLine>>) -> String {
		lyrics
			.iter()
			.map(|stanza| {
				let stanza_lines: Vec<String> =
					stanza.iter().map(|line| line.to_synced_text()).collect();
				// if let Some(last_line) = stanza.last() {
				// stanza_lines.push(format!("{} ", last_line.end_ms()));
				// }
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
	fn ext(&self) -> &str {
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
	released: Date,
	length: u32,
	pub temporary: bool,
	upc: Option<String>,
	about: Option<String>,
	palette: Option<Palette>,
	single: bool,
	url: UrlSet
}

pub struct Song {
	parent_album: Option<(usize, usize)>, // album-index, position in tracklist
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
	url: Option<UrlSet>
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
				if let Some(_) = song.palette {
					&song.palette
				} else if let Some(album) = maybe_parent_album {
					&album.palette
				} else {
					&song.palette
				}
			}
			Titlable::Album(album) => &album.palette,
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
					"–"
				}
			} // en dash btw
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
		if self.artist() == "Astro" && self.dash() != "@" {
			format!("{}", self.title())
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
		let re_punct = regex::Regex::new(r#"[()\[\],.?!'"*\$]"#).unwrap();
		let re_sep = regex::Regex::new(r#"[_/&+:;\s]+"#).unwrap();
		let re_dash = regex::Regex::new(r#"-+"#).unwrap();
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
		slug
	}
	fn copyright_message(&self, year: u32) -> String {
		format!(
			"© {} {}",
			year,
			self.artist().replace("Astro", "Astro \"astronomy487\"")
		)
	}
	pub fn make_link_page(&self) {
		let destination_folder = std::path::Path::new(globals::FILEZONE)
			.join("music.astronomy487.com")
			.join(self.slug());

		if destination_folder.exists() {
			std::fs::remove_dir_all(&destination_folder).expect("Couldn't remove directory");
		}
		std::fs::create_dir(&destination_folder).expect("Couldn't create directory");

		// lyrics files come first. link pages need to report their size
		if let Titlable::Song(song, _) = self {
			if let Some(lxs) = &song.lyrics {
				let mut file = std::fs::File::create(destination_folder.join("lyrics.txt"))
					.expect("Can't save lyrics.txt file");
				let _ = std::io::Write::write(&mut file, LyricLine::as_plaintext(&lxs).as_bytes())
					.expect("Couldn't write lyrics.txt");
				let mut file = std::fs::File::create(destination_folder.join("lyrics.lrc"))
					.expect("Can't save lyrics.lrc file");
				let _ = std::io::Write::write(&mut file, LyricLine::as_lrc(&lxs).as_bytes())
					.expect("Couldn't write lyrics.lrc");
			}
		}

		let url = format!("https://music.astronomy487.com/{}", self.slug());
		let title = self.format_title();
		let title_short = self.format_title_short();
		let description = self.description();

		let (length, released, is_album, parent_album) = match self {
			Titlable::Album(a) => (a.length, a.released.to_string(), true, None),
			Titlable::Song(s, parent) => {
				let r = if let Some(d) = &s.released {
					d.to_string()
				} else if let Some(a) = parent {
					a.released.to_string()
				} else {
					panic!("Song has no release date and no parent album")
				};
				(s.length, r, false, *parent)
			}
		};

		let length_str = format_duration(length);

		let artwork = match self {
			Titlable::Album(a) => Some(format!("../{}.jpg", a.slug())),
			Titlable::Song(s, parent) => {
				if let Some(sa) = &s.single_artwork {
					Some(format!("../{}.jpg", sa))
				} else if let Some(a) = parent {
					Some(format!("../{}.jpg", a.slug()))
				} else {
					None
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
					@if let Some(a) = &artwork {
						meta property="og:image" content=(a);
						link rel="apple-touch-icon" href=(a);
					} @else {
						meta property="og:image" content="../squarelogo.png";
						link rel="apple-touch-icon" href="../squarelogo.png";
					}
					meta property="og:url" content=(url);
					meta property="music:musician" content="https://www.astronomy487.com";
					meta property="music:release_date" content=(released);
					meta property="music:duration" content=(length.to_string());
					@if is_album {
						meta property="og:type" content="music.album";
						@if let Titlable::Album(a) = self {
							@for (i, s) in a.songs.iter().enumerate() {
								@if !s.bonus {
									meta property="music:song" content=(format!("https://music.astronomy487.com/{}", s.slug()));
									meta property="music:song:track" content=((i+1).to_string());
								}
							}
						}
					} @else {
						meta property="og:type" content="music.song";
						@if let Some(a) = parent_album {
							meta property="music:album" content=(format!("https://music.astronomy487.com/{}", a.slug()));
							@let track_num = a.songs.iter().position(|s| {
								match self {
									Titlable::Song(me, _) => std::ptr::eq(*me, s),
									_ => false
								}
							}).map(|i| i+1).unwrap_or(1);
							meta property="music:album:track" content=(track_num.to_string());
						}
					}
					script src="../talktalk/talktalk.min.js" data-talktalk="../talktalk" {}
				}
				body class = { @match palette {
					None => "",
					Some(palette) => {
						@match palette.palette_mode {
							PaletteMode::Normal => "",
							PaletteMode::White => "mode-white",
							PaletteMode::Black => "mode-black"
						}
					}
				} } {
					@if let Some(a) = &artwork {
						img src=(a) {}
					} @else {
						style { "body { margin-top: 16rem; }" }
					}

					h1 { (title_short) }

					table {
						tr {
							td { (released) }
							@if is_album {
								@if let Titlable::Album(a) = self {
									td { (format!("{} tracks", a.songs.iter().filter(|s| !s.bonus).count())) }
								}
							}
							td { (length_str) }
						}
					}

					table {
						@for pairs in &itertools::Itertools::chunks(url_set.iter().iter(), 2) {
							tr {
								@for (key, value) in pairs {
									@let short = key.to_lowercase().replace(' ', "");
									td {
										a class=(format!("{} streamlink", short)) href=(value) {
											img src=(format!("../icons/{}.svg", short)) {}
											span { (key) }
										}
									}
								}
								/* @if pairs.len() == 1 {
									td { }
								} */
							}
						}
					}

					table class="bottomlinks" {
						@if let Some(mp3_size) = thing_to_offer_as_download.audio_download_size(&Codec::Mp3) {
							@let flac_size = thing_to_offer_as_download.audio_download_size(&Codec::Flac).expect("mp3 has size but not flac? get serious");
							tr {
								td {
									span data-talktalk="download" {}
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
										span data-talktalk="lyrics" {}
									}
									td {
										a href="lyrics.txt" download=(&format!("{}-lyrics.txt", song.slug())) {
											"txt, "
											(format_file_size( //TODO this is messy. in general
												std::fs::metadata(std::path::Path::new(globals::FILEZONE).join("music.astronomy487.com").join(song.slug()).join("lyrics").with_extension("txt")).expect(
													&format!("lyrics.txt doesn't exist though? for {}", song.slug())
												).len()
											))
										}
									}
									td {
										a href="lyrics.lrc" download=(&format!("{}-lyrics.lrc", song.slug())) {
											"lrc, "
											(format_file_size( //TODO this is messy. in general
												std::fs::metadata(std::path::Path::new(globals::FILEZONE).join("music.astronomy487.com").join(song.slug()).join("lyrics").with_extension("lrc")).expect(
													&format!("lyrics.lrc doesn't exist though? for {}", song.slug())
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

		let mut file = std::fs::File::create(destination_folder.join("index.html"))
			.expect("Can't save html file");
		let _ = std::io::Write::write(&mut file, html.into_string().as_bytes())
			.expect("Couldn't write HTML");
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
					.expect("Song has no release date and no parent album");

				let length = format_duration(song.length);

				if parent_opt.is_none() {
					if song.event {
						return format!("DJ set for {} on {}, {}", song.title, released, length);
					}

					if !song.artist.is_empty() {
						return format!("Remix released {}, {}", released, length);
					}

					return format!("Mix released {}, {}", released, length);
				}

				let parent = parent_opt.unwrap(); // TODO use match instead of unwrap and is_none and stuff

				if !parent.single {
					let track_number = song.parent_album.map(|(_, idx)| idx + 1).unwrap_or(0);

					return format!(
						"Track {} on {}, released {}, {}",
						track_number, parent.title, released, length
					);
				}

				format!("Song released {}, {}", released, length)
			}
		}
	}
}

impl std::fmt::Display for Album {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} – {}", self.artist, self.title)
	}
}
impl std::fmt::Display for Song {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} – {}", self.artist, self.title)
	}
}
impl std::fmt::Debug for Album {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} – {}", self.artist, self.title)
	}
}
impl std::fmt::Debug for Song {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} – {}", self.artist, self.title)
	}
}

impl Album {
	fn slug(&self) -> String {
		Titlable::Album(self).slug()
	}
	fn from_json(val: &serde_json::Value) -> Album {
		Album {
			songs: val
				.get("songs")
				.expect("album has no songs")
				.as_array()
				.expect("discog album is not songs")
				.into_iter()
				.map(|s_j| Song::from_json(s_j))
				.collect(),
			artist: val
				.get("artist")
				.and_then(|v| v.as_str())
				.unwrap_or("Astro")
				.to_string(),
			title: val
				.get("title")
				.expect("discog album has no title")
				.as_str()
				.expect("discog album has non-string title")
				.to_string(),
			released: Date::from(
				val.get("released")
					.expect("discog album has no release date")
					.as_str()
					.expect("discog album has non-string release date")
			),
			length: val
				.get("length")
				.expect("discog album has no length")
				.as_i64()
				.expect("discog album has bad length") as u32,
			temporary: match val.get("temporary") {
				None => false,
				Some(_) => true
			},
			single: match val.get("single") {
				None => false,
				Some(_) => true
			},
			upc: val.get("upc").map(|v| {
				v.as_str()
					.expect("discog album has non-string upc")
					.to_owned()
			}),
			about: val.get("about").map(|v| {
				v.as_str()
					.expect("discog album has non-string about")
					.to_owned()
			}),
			palette: val.get("color").map(|v| Palette::from(v)),
			url: UrlSet::from(val.get("url").expect("album has no url"), true)
		}
	}
	pub fn try_encode(&self, failures: &mut std::collections::HashSet<String>) -> bool {
		if self.temporary {
			return false;
		}
		let mut all_successful = true;
		for song in &self.songs {
			let this_successful = song.try_encode(Some(&self), failures);
			all_successful = all_successful && this_successful;
		}
		if all_successful {
			self.zip(&Codec::Mp3);
			self.zip(&Codec::Flac);
		}
		all_successful
	}
	fn zip(&self, codec: &Codec) {
		// panic if can't
		let destination = std::path::Path::new(globals::FILEZONE)
			.join("audio.astronomy487.com")
			.join(codec.ext())
			.join(self.slug())
			.with_extension("zip");
		if destination.exists() {
			return;
		}
		println!("  Zipping {} {}", self, codec.ext());
		let mut zipper = zipper::Zipper::new(&destination);
		for (song_index, song) in self.songs.iter().enumerate() {
			println!("  - Adding {}", song);
			zipper.add_file(
				&song.destination_location(&codec),
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
					&LyricLine::as_plaintext(&lyrics),
					&std::path::Path::new(&format!(
						"lyrics/txt/{:03}-{}.txt",
						song_index,
						song.slug()
					))
				);
				zipper.add_text_file(
					&LyricLine::as_lrc(&lyrics),
					&std::path::Path::new(&format!(
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
			&std::path::Path::new("artwork.png")
		);
		zipper.finish();
	}
	fn non_bonus_song_count(&self) -> usize {
		self.songs.iter().take_while(|song| !song.bonus).count()
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
		text.push(Titlable::Album(self).copyright_message(self.released.year));
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
			artist: val
				.get("artist")
				.and_then(|v| v.as_str())
				.unwrap_or("Astro")
				.to_string(),
			title: val
				.get("title")
				.expect("discog song has no title")
				.as_str()
				.expect("discog song has non-string title")
				.to_string(),
			released: val
				.get("released")
				.map(|v| Date::from(v.as_str().expect("discog song has non-string release date"))),
			bonus: match val.get("bonus") {
				None => false,
				Some(_) => true
			},
			event: match val.get("event") {
				None => false,
				Some(_) => true
			},
			single_artwork: None,
			length: val
				.get("length")
				.expect("discog song has no length")
				.as_i64()
				.expect("discog song has bad length") as u32,
			isrc: val.get("isrc").map(|v| {
				v.as_str()
					.expect("discog song has non-string isrc")
					.to_owned()
			}),
			lyrics: val
				.get("lyrics")
				.map(|v| LyricLine::from(v.as_str().expect("discog song has non-string lyrics"))),
			palette: val.get("color").map(|v| Palette::from(v)),
			url: val.get("url").map(|v| UrlSet::from(v, false))
		};
		match val.get("artwork") {
			None => {}
			Some(v) => match v {
				serde_json::Value::String(s) => {
					song.single_artwork = Some(s.to_string());
				}
				serde_json::Value::Bool(_) => {
					song.single_artwork = Some(song.slug());
				}
				_ => panic!("song has artwork that isn't boolean or string")
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
	fn do_encode(
		&self, codec: &Codec, parent_album: Option<&Album>,
		failures: &mut std::collections::HashSet<String>
	) -> bool {
		let input_file_base = std::path::Path::new(globals::FILEZONE)
			.join("source")
			.join("audio");
		let input_file_base = match parent_album {
			Some(album) => input_file_base.join(album.slug()).join(self.slug()),
			None => input_file_base.join(self.slug())
		};
		let mut input_file = input_file_base.with_extension("flac");
		if !input_file.exists() {
			input_file = input_file_base.with_extension("wav");
		}
		if !input_file.exists() {
			let _ = failures.insert(match parent_album {
				Some(album) => format!("{}/{}", album.slug(), self.slug()),
				None => format!("{}", self.slug())
			});
			return false;
		}
		let destination = self.destination_location(&codec);
		if !destination.exists() {
			let args =
				codec.ffmpeg_args(input_file.to_str().unwrap(), destination.to_str().unwrap());
			println!("  Encoding {}.{}", self.slug(), codec.ext());
			let ffmpeg_status = std::process::Command::new("ffmpeg").args(&args).status();
			if !matches!(ffmpeg_status, std::result::Result::Ok(s) if s.success()) {
				panic!("FFMPEG could not complete the action ???");
			}
		} // TODO if too much time spent on file writes, then stop rewriting metadata here. return true early

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
			.expect("Bad input audio format (I thought it was flac or wav always though?)");
		let track = probed
			.format
			.tracks()
			.iter()
			.find(|t| t.codec_params.sample_rate.is_some())
			.expect("Couldn't identify track metadata for song length");
		let sr = track
			.codec_params
			.sample_rate
			.expect("Couldn't identify track metadata for song length");
		let frames = track
			.codec_params
			.n_frames
			.expect("Couldn't identify track metadata for song length");
		let dur = (frames as f64 / sr as f64).floor() as u32;
		assert!(
			dur == self.length,
			"duration mismatch for {}. expected {} but found {}",
			self.slug(),
			self.length,
			dur
		);

		match codec {
			Codec::Mp3 => {
				let mut tag = id3::Tag::new();
				id3::TagLike::set_title(&mut tag, &self.title);
				id3::TagLike::set_artist(&mut tag, &self.artist);
				match parent_album {
					Some(parent_album) => {
						id3::TagLike::set_album(&mut tag, &parent_album.title);
						id3::TagLike::set_album_artist(&mut tag, &parent_album.artist);
						let (_, position_in_tracklist) = &self.parent_album.unwrap();
						id3::TagLike::set_track(&mut tag, (position_in_tracklist + 1) as u32);
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
				id3::TagLike::set_duration(&mut tag, self.length as u32);
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Picture {
						mime_type: "image/jpeg".to_string(),
						picture_type: id3::frame::PictureType::Other,
						description: "".to_string(),
						data: self.grab_artwork(parent_album)
					}
				);
				let release_date = self.release_date(parent_album);
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
				if self.bonus {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::link(
							"WOAS",
							"https://music.astronomy487.com/".to_owned()
								+ &parent_album.unwrap().slug()
						)
					);
				} else {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::link(
							"WOAF",
							"https://music.astronomy487.com/".to_owned() + &self.slug()
						)
					);
					if let Some(parent_album) = parent_album {
						let _ = id3::TagLike::add_frame(
							&mut tag,
							id3::frame::Frame::link(
								"WOAS",
								"https://music.astronomy487.com/".to_owned() + &parent_album.slug()
							)
						);
					} else {
						let _ = id3::TagLike::add_frame(
							&mut tag,
							id3::frame::Frame::link(
								"WOAS",
								"https://music.astronomy487.com/".to_owned() + &self.slug()
							)
						);
					}
				}
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Frame::text("TENC", "Astro \"astronomy487\"")
				);
				let _ = id3::TagLike::add_frame(&mut tag, id3::frame::Frame::text("TFLT", "mp3"));
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Frame::text(
						"TCOP",
						Titlable::Song(self, None).copyright_message(release_date.year)
					)
				);
				if let Some(lyrics) = &self.lyrics {
					let mut languages: Vec<Language> = Vec::new();
					for stanza in lyrics {
						for line in stanza {
							if !languages.contains(&line.language) {
								languages.push(line.language);
							}
						}
					}
					if languages.len() > 1 {
						panic!("Sorry I'm not yet built for songs that span multiple languages :(");
					}
					for lang in languages {
						let mut lines_for_lang: Vec<Vec<LyricLine>> = Vec::new();
						let mut current_stanza: Vec<LyricLine> = Vec::new();

						for stanza in lyrics {
							for line in stanza {
								if line.language == lang {
									current_stanza.push(line.clone());
								}
							}

							if !current_stanza.is_empty() {
								lines_for_lang.push(std::mem::take(&mut current_stanza));
							}
						}

						let lang_code = lang.to_iso_639_2().to_string();

						// --- USLT ---
						let uslt = id3::frame::Lyrics {
							lang: lang_code.clone(),
							description: "".to_string(),
							text: LyricLine::as_plaintext(&lines_for_lang)
						};
						let _ = id3::TagLike::add_frame(&mut tag, uslt);

						// --- SYLT ---
						let sylt = id3::frame::Frame::with_content(
							"SYLT",
							id3::frame::Content::SynchronisedLyrics(
								id3::frame::SynchronisedLyrics {
									lang: lang_code,
									timestamp_format: id3::frame::TimestampFormat::Ms,
									content_type: id3::frame::SynchronisedLyricsType::Other,
									description: "".to_string(),
									content: LyricLine::as_sylt_data(&lines_for_lang)
								}
							)
						);
						let _ = id3::TagLike::add_frame(&mut tag, sylt);
					}
				}
				if let std::result::Result::Err(_) =
					tag.write_to_path(destination, id3::Version::Id3v24)
				{
					panic!("Couldn't write mp3 metadata to {}", self);
				}
				true
			}
			Codec::Flac => {
				let mut tag = metaflac::Tag::read_from_path(destination)
					.expect("flac path doesn't exist now");
				tag.set_vorbis("TITLE", vec![&self.title]);
				tag.set_vorbis("ARTIST", vec![&self.artist]);
				match parent_album {
					Some(parent_album) => {
						tag.set_vorbis("ALBUM", vec![&parent_album.title]);
						tag.set_vorbis("ALBUMARTIST", vec![&parent_album.artist]);
						let (_, position_in_tracklist) = &self.parent_album.unwrap();
						tag.set_vorbis(
							"TRACKNUMBER",
							vec![(position_in_tracklist + 1).to_string()]
						);
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
				let release_date = self.release_date(parent_album);
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
				let _ = tag.add_picture(
					"image/jpeg",
					metaflac::block::PictureType::Other,
					self.grab_artwork(parent_album)
				);
				if self.bonus {
					tag.set_vorbis(
						"WOAS",
						vec![
							"https://music.astronomy487.com/".to_owned()
								+ &parent_album.unwrap().slug(),
						]
					);
				} else {
					tag.set_vorbis(
						"WOAF",
						vec!["https://music.astronomy487.com/".to_owned() + &self.slug()]
					);
					if let Some(parent_album) = parent_album {
						tag.set_vorbis(
							"WOAS",
							vec![
								"https://music.astronomy487.com/".to_owned() + &parent_album.slug(),
							]
						);
					} else {
						tag.set_vorbis(
							"WOAS",
							vec!["https://music.astronomy487.com/".to_owned() + &self.slug()]
						);
					}
				}
				tag.set_vorbis("ENCODER", vec!["Astro \"astronomy487\""]);
				tag.set_vorbis("FILETYPE", vec!["flac"]);
				tag.set_vorbis(
					"COPYRIGHT",
					vec![Titlable::Song(self, None).copyright_message(release_date.year)]
				);
				if let Some(lyrics) = &self.lyrics {
					tag.set_vorbis("LYRICS", vec![&LyricLine::as_plaintext(lyrics)]);
					tag.set_vorbis("LYRICS_SYNCED", vec![&LyricLine::as_lrc(lyrics)]);
				}
				if let std::result::Result::Err(_) = tag.save() {
					panic!("Couldn't write flac metadata to {}", self);
				}
				true
			}
		}
	}
	fn release_date<'a>(&'a self, parent_album: Option<&'a Album>) -> &'a Date {
		self.released.as_ref().unwrap_or_else(|| {
			&parent_album
				.expect("Song without release date has no parent album")
				.released
		})
	}
	fn grab_artwork(&self, parent_album: Option<&Album>) -> Vec<u8> {
		let artwork_name = self
			.single_artwork
			.to_owned()
			.or_else(|| parent_album.map(|album| album.slug()))
			.unwrap_or_else(|| "fallback".to_string());
		imagedeal::grab_image(artwork_name)
	}
	pub fn try_encode(
		&self, parent_album: Option<&Album>, failures: &mut std::collections::HashSet<String>
	) -> bool {
		let mp3_success = self.do_encode(&Codec::Mp3, parent_album, failures);
		let flac_success = self.do_encode(&Codec::Flac, parent_album, failures);
		mp3_success && flac_success
	}
}

pub fn get_music_data(json_path: &std::path::Path) -> (Vec<Album>, Vec<Song>) {
	let file =
		std::fs::File::open(json_path).expect(&format!("{} does not exist", json_path.display()));
	let reader = std::io::BufReader::new(file);
	let json_value: serde_json::Value = serde_json::from_reader(reader).expect(&format!(
		"{} is not correctly formatted json",
		json_path.display()
	));
	let remixes: Vec<Song> = json_value
		.get("remixes")
		.expect("discog has no remixes")
		.as_array()
		.expect("discog remixes are not a list")
		.iter()
		.map(|s_j| Song::from_json(s_j))
		.collect();
	let mut albums: Vec<Album> = json_value
		.get("albums")
		.expect("discog has no albums")
		.as_array()
		.expect("discog albums are not a list")
		.iter()
		.map(|a_j| Album::from_json(a_j))
		.collect();

	// assign parent_album refs
	for (album_index, album) in albums.iter_mut().enumerate() {
		for (song_index, song) in album.songs.iter_mut().enumerate() {
			song.parent_album = Some((album_index, song_index));
		}
	}

	// validation
	for remix in &remixes {
		if let None = remix.released {
			panic!("remix without release date")
		}
	}
	let mut seen_slugs = std::collections::HashSet::new();
	let mut assert_slug = |s: String| {
		if !seen_slugs.insert(s.clone()) {
			panic!("Duplicate slug: {}", s);
		}
	};
	for album in &albums {
		if album.single {
			let first_song_slug = album.songs[0].slug();
			let album_slug = album.slug();

			assert_eq!(first_song_slug, album_slug);
			assert_slug(album_slug);

			for i in 2..album.songs.len() {
				assert!(album.songs[i].bonus);
				assert_slug(album.songs[i].slug());
			}
		} else {
			let album_slug = album.slug();
			assert_slug(album_slug);

			for song in &album.songs {
				assert_slug(song.slug());
			}
		}
	}

	(albums, remixes)
}
