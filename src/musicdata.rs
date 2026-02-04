use crate::color;
use crate::date;
use crate::fileops;
use crate::genre;
use crate::globals;
use crate::imagedeal;
use crate::lyric;
use crate::url;

pub fn format_duration(total_seconds: u32) -> String {
	let minutes = (total_seconds / 60) % 60;
	let hours = (total_seconds / 3600) % 60;
	let seconds = total_seconds % 60;
	if hours > 0 {
		format!("{}h{:02}m{:02}s", hours, minutes, seconds)
	} else {
		format!("{}m{:02}s", minutes, seconds)
	}
}

#[derive(Debug)]
pub enum AudioCodec {
	Mp3,
	Flac
}
impl AudioCodec {
	pub fn ext(&self) -> &'static str {
		match self {
			AudioCodec::Mp3 => "mp3",
			AudioCodec::Flac => "flac"
		}
	}
	fn ffmpeg_args(&self, input: &str, output: &str) -> Vec<String> {
		match self {
			AudioCodec::Mp3 => vec![
				"-y".into(),
				"-i".into(),
				input.into(),
				"-codec:a".into(),
				"libmp3lame".into(),
				"-b:a".into(),
				"320k".into(),
				"-map_metadata".into(),
				"-1".into(),
				output.into(),
			],
			AudioCodec::Flac => vec![
				"-y".into(),
				"-i".into(),
				input.into(),
				"-codec:a".into(),
				"flac".into(),
				"-compression_level".into(),
				"8".into(),
				"-map_metadata".into(),
				"-1".into(),
				output.into(),
			]
		}
	}
}

#[derive(Debug)]
pub struct Album {
	pub slug: String,
	pub songs: Vec<Song>,
	pub title: String,
	pub artist: String,
	pub released: date::Date,
	pub length: u32,
	upc: Option<String>,
	bcid: Option<String>,
	pub about: Option<String>,
	pub palette: color::Palette,
	pub single: bool,
	pub compilation: bool,
	pub url: url::UrlSet,
	genre: genre::Genre,
	pub unreleased: bool
}

#[derive(Debug)]
pub struct Song {
	pub slug: String,
	pub parent_album_indices: Option<(usize, usize)>, // album-index, position in tracklist
	pub title: String,
	pub artist: String,
	pub released: date::Date, // may inherit from parent
	pub released_as_single: bool,
	pub bonus: bool,
	event: bool,
	pub artwork: Option<String>, // songs on albums inherit from parents; remixes have None
	pub length: u32,
	isrc: Option<String>,
	pub lyrics: Option<lyric::Lyrics>,
	palette: color::Palette, // may inherit from parent
	genre: genre::Genre,     // MUST inherit from parent if on an album
	pub unreleased: bool,    // may inherit from parent
	pub url: url::UrlSet,
	samples: Option<Vec<String>>, // report as "Mix tracklist" if event
	about: Option<String>
}

#[derive(Debug)]
pub enum Titlable<'a> {
	Song(&'a Song),
	Album(&'a Album)
}
impl Titlable<'_> {
	fn artist(&self) -> &str {
		match self {
			Titlable::Song(song) => &song.artist,
			Titlable::Album(album) => &album.artist
		}
	}
	pub fn palette(&self) -> &color::Palette {
		match self {
			Titlable::Song(song) => &song.palette,
			Titlable::Album(album) => &album.palette
		}
	}
	fn title(&self) -> &str {
		match self {
			Titlable::Song(song) => &song.title,
			Titlable::Album(album) => &album.title
		}
	}
	pub fn slug(&self) -> &str {
		match self {
			Titlable::Song(song) => &song.slug,
			Titlable::Album(album) => &album.slug
		}
	}
	pub fn released(&self) -> &date::Date {
		match self {
			Titlable::Song(song) => &song.released,
			Titlable::Album(album) => &album.released
		}
	}
	pub fn length(&self) -> u32 {
		match self {
			Titlable::Song(song) => song.length,
			Titlable::Album(album) => album.length
		}
	}
	pub fn genre(&self) -> &genre::Genre {
		match self {
			Titlable::Song(song) => &song.genre,
			Titlable::Album(album) => &album.genre
		}
	}
	pub fn unreleased(&self) -> bool {
		match self {
			Titlable::Song(song) => song.unreleased,
			Titlable::Album(album) => album.unreleased
		}
	}
	pub fn artwork(&self) -> Option<&str> {
		match self {
			Titlable::Song(song) => song.artwork.as_deref(),
			Titlable::Album(album) => Some(&album.slug)
		}
	}
	fn dash(&self) -> &'static str {
		match self {
			Titlable::Song(song) => {
				if song.event {
					"@"
				} else {
					"–" // en dash btw
				}
			}
			Titlable::Album(_) => "–" // en dash btw
		}
	}
	pub fn audio_download_url(&self, codec: &AudioCodec) -> String {
		format!(
			"https://audio.astronomy487.com/{}/{}.{}",
			codec.ext(),
			self.public_filename(),
			match self {
				Titlable::Song(_) => codec.ext(),
				Titlable::Album(_) => "zip"
			}
		)
	}
	fn audio_download_local_location(&self, codec: &AudioCodec) -> std::path::PathBuf {
		match self {
			Titlable::Song(song) => std::path::Path::new(globals::filezone())
				.join(if song.bonus {
					"private"
				} else {
					"audio.astronomy487.com"
				})
				.join(codec.ext())
				.join(song.public_filename())
				.with_extension(codec.ext()),
			Titlable::Album(album) => std::path::Path::new(globals::filezone())
				.join("audio.astronomy487.com")
				.join(codec.ext())
				.join(album.public_filename())
				.with_extension("zip")
		}
	}
	pub fn audio_download_size(&self, codec: &AudioCodec) -> Option<u64> {
		let path = self.audio_download_local_location(codec);
		std::fs::metadata(&path).ok().map(|m| m.len())
	}
	pub fn format_title(&self) -> String {
		match self {
			Titlable::Song(song)
				if song.parent_album_indices.is_none() && song.artist == "Astro" && !song.event =>
			{
				self.title().to_string()
			}
			_ => format!("{} {} {}", self.artist(), self.dash(), self.title())
		}
	}
	fn public_filename(&self) -> String {
		let title = self.format_title();
		let re_forbidden =
			regex::Regex::new(r#"[<>.:"/\\|?*\x00-\x1F]"#).expect("re_forbidden is invalid regex");
		let mut cleaned = re_forbidden.replace_all(&title, "").into_owned();
		let re_spaces = regex::Regex::new(r#"\s+"#).expect("re_spaces is invalid regex");
		cleaned = re_spaces.replace_all(&cleaned, " ").into_owned();
		cleaned = cleaned.trim().trim_end_matches('.').to_string();
		const RESERVED: &[&str] = &[
			"CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
			"COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
			"CONIN$", "CONOUT$"
		];
		if RESERVED.contains(&cleaned.to_ascii_uppercase().as_str()) {
			cleaned.push('_');
		}
		cleaned
	}
	pub fn format_title_short(&self) -> String {
		if self.artist() == "Astro"
			&& match self {
				Titlable::Album(_) => true,
				Titlable::Song(song) => !song.event
			} {
			self.title().to_string()
		} else {
			format!("{} {} {}", self.artist(), self.dash(), self.title())
		}
	}
	pub fn description(&self, all_albums: &[Album]) -> String {
		match self {
			Titlable::Album(album) => {
				let released = album.released.to_display();
				let length = format_duration(album.length);
				let track_count = album.songs.iter().filter(|s| !s.bonus).count();
				format!(
					"Album released {}, {} tracks, {}",
					released, track_count, length
				)
			}
			Titlable::Song(song) => {
				let released = song.released.to_display();
				let length = format_duration(song.length);
				match song.parent_album_indices {
					None => {
						if song.event {
							format!("DJ set for {} on {}, {}", song.title, released, length)
						} else if !song.artist.is_empty() {
							format!("Remix released {}, {}", released, length)
						} else {
							format!("Mix released {}, {}", released, length)
						}
					}
					Some((parent_album_index, track_number)) => {
						if all_albums[parent_album_index].single {
							format!("Song released {}, {}", released, length)
						} else {
							format!(
								"Track {} on {}, released {}, {}",
								track_number + 1,
								all_albums[parent_album_index].title,
								released,
								length
							)
						}
					}
				}
			}
		}
	}
}

impl Album {
	fn from_json(val: &serde_json::Value) -> Album {
		let obj = globals::map_with_only_these_keys(
			val,
			"Album",
			&[
				"about",
				"bcid",
				"color",
				"genre",
				"length",
				"released",
				"songs",
				"title",
				"upc",
				"url",
				"compilation",
				"artist",
				"single",
				"unreleased"
			]
		);
		let url_set = {
			let url_val = obj
				.get("url")
				.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"url\"", val));

			url::UrlSet::from(url_val)
		};

		let artist = match obj.get("artist") {
			None => "Astro".to_string(),
			Some(artist_val) => {
				let artist = artist_val.as_str().unwrap_or_else(|| {
					panic!(
						"Album JSON has non-string \"artist\" attribute: {}",
						artist_val
					)
				});
				assert!(
					!artist.starts_with(char::is_whitespace)
						&& !artist.ends_with(char::is_whitespace),
					"Album JSON has poorly formed \"artist\" string: {}",
					artist
				);
				artist.to_string()
			}
		};
		let title = {
			let title_object = obj
				.get("title")
				.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"title\"", val));
			let title = title_object.as_str().unwrap_or_else(|| {
				panic!(
					"Album JSON attribute \"title\" is not a string: {}",
					title_object
				)
			});
			assert!(
				!title.starts_with(char::is_whitespace) && !title.ends_with(char::is_whitespace),
				"Album JSON has poorly formed \"title\" string: {}",
				title
			);
			title.to_string()
		};
		let slug = globals::compute_slug(&artist, &title);

		let mut album = Album {
			songs: Vec::new(),
			slug,
			artist,
			title,
			released: {
				let rel_val = obj
					.get("released")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"released\"", val));
				let rel_str = rel_val.as_str().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"released\" is not a string: {}",
						rel_val
					)
				});
				date::Date::from(rel_str)
			},
			genre: {
				let genre_val = obj
					.get("genre")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"genre\"", val));
				let genre_str = genre_val.as_str().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"genre\" is not a string: {}",
						genre_val
					)
				});
				genre::Genre::from(genre_str)
			},
			length: {
				let len_val = obj
					.get("length")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"length\"", val));
				len_val.as_i64().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"length\" is not an integer: {}",
						len_val
					)
				}) as u32
			},
			unreleased: match obj.get("unreleased") {
				None => false,
				Some(val_for_unreleased) => val_for_unreleased.as_bool().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"unreleased\" is not a boolean: {}",
						val_for_unreleased
					)
				})
			},
			single: match obj.get("single") {
				None => false,
				Some(val_for_single) => val_for_single.as_bool().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"single\" is not a boolean: {}",
						val_for_single
					)
				})
			},
			compilation: match obj.get("compilation") {
				None => false,
				Some(val_for_compilation) => val_for_compilation.as_bool().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"compilation\" is not a boolean: {}",
						val_for_compilation
					)
				})
			},
			upc: obj.get("upc").map(|v| {
				let upc = v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"upc\" is not a string: {}", v)
					})
					.to_owned();
				assert!(
					upc.len() == 12 && upc.chars().all(|c| c.is_ascii_digit()),
					"Album JSON attribute \"upc\" is not a valid UPC: \"{}\"",
					upc
				);
				upc
			}),
			bcid: obj.get("bcid").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"bcid\" is not a string: {}", v)
					})
					.to_owned()
			}),
			about: obj.get("about").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned()
				// TODO check trimmed, make sure paragraphs are separated by \n\n
			}),
			palette: color::Palette::from(
				obj.get("color")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"color\"", val)),
				&url_set
			),
			url: url_set
		};
		{
			let songs_val = obj
				.get("songs")
				.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"songs\"", val));
			let songs_arr = songs_val.as_array().unwrap_or_else(|| {
				panic!(
					"Album JSON attribute \"songs\" is not an array: {}",
					songs_val
				)
			});
			for song_json in songs_arr.iter() {
				album.songs.push(Song::from_json(song_json, Some(&album)));
			}
		}
		album.slug = globals::compute_slug(&album.artist, &album.title);
		album
	}
	fn public_filename(&self) -> String {
		Titlable::Album(self).public_filename()
	}
	pub fn try_encode(&self, all_albums: &[Album]) {
		for song in &self.songs {
			song.try_encode(all_albums);
		}
		if !self.unreleased {
			self.zip(&AudioCodec::Mp3);
			self.zip(&AudioCodec::Flac);
		}
	}
	fn zip(&self, codec: &AudioCodec) {
		let destination = std::path::Path::new(globals::filezone())
			.join("audio.astronomy487.com")
			.join(codec.ext())
			.join(self.public_filename())
			.with_extension("zip");
		if destination.exists() {
			return;
		}
		globals::log_3(
			"Zipping",
			codec.ext(),
			self.format_title(),
			globals::ANSI_YELLOW
		);
		let mut zipper = fileops::Zipper::new(&destination);
		fn pad_digits(max_tracks: usize, track_number: usize) -> String {
			let width = max_tracks.to_string().len();
			format!("{:0width$}", track_number, width = width)
		}
		for (song_index, song) in self.songs.iter().enumerate() {
			globals::log_3(
				"",
				format!("+ {}", song_index + 1),
				song.format_title(),
				globals::ANSI_YELLOW
			);
			zipper.add_file(
				&song.destination_location(codec),
				std::path::Path::new(&format!(
					"{}{} {}.{}",
					if song.bonus { "bonus/" } else { "" },
					pad_digits(self.songs.len(), song_index + 1),
					song.public_filename(),
					codec.ext()
				))
			);
			if let Some(lyrics) = &song.lyrics {
				// only include .txt - don't bother with lrc or srt in zips
				zipper.add_text_file(
					&lyrics.as_filetype(lyric::TextCodec::Txt),
					std::path::Path::new(&format!(
						"lyrics/{} {}.txt",
						pad_digits(self.songs.len(), song_index + 1),
						song.public_filename()
					))
				);
			}
		}
		zipper.add_text_file(&self.description(), std::path::Path::new("README.txt"));
		zipper.add_file(
			&std::path::Path::new(globals::filezone())
				.join("private")
				.join("png")
				.join(&self.slug)
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
		// utf-8, but use ascii ' "
		let mut text = Vec::new();

		text.push(self.format_title());
		text.push(self.released.to_display());

		if let Some(about) = &self.about {
			text.push(String::new());
			text.push(about.to_string());
		}

		text.push(String::new());
		for (song_index, song) in self.songs.iter().enumerate() {
			if !song.bonus {
				text.push(format!("{}. {}", song_index + 1, song.title));
			}
		}

		let bonus_song_count = self.songs.len() - self.non_bonus_song_count();
		if bonus_song_count > 0 {
			text.push(String::new());
			text.push(format!(
				"Bonus track{} included with digital download:",
				if bonus_song_count == 1 { "" } else { "s" }
			));
			text.push(String::new());
			for (song_index, song) in self.songs.iter().enumerate() {
				if song.bonus {
					text.push(format!("{}. {}", song_index + 1, song.title));
				}
			}
		}

		text.push(String::new());
		text.push(self.copyright_message());
		if self.artist == "Astro" {
			text.push("Shared under CC BY-NC-SA 4.0 license".to_string());
		}
		text.push("Thank you for downloading!".to_string());
		text.push(format!("https://music.astronomy487.com/{}/", self.slug));

		text.join("\n")
	}
	pub fn format_title(&self) -> String {
		Titlable::Album(self).format_title()
	}
	pub fn format_title_short(&self) -> String {
		Titlable::Album(self).format_title_short()
	}
}

impl Song {
	fn from_json(val: &serde_json::Value, parent_album: Option<&Album>) -> Song {
		let obj = globals::map_with_only_these_keys(
			val,
			"Song",
			&[
				"artist",
				"title",
				"released",
				"bonus",
				"event",
				"length",
				"isrc",
				"lyrics",
				"color",
				"url",
				"samples",
				"about",
				"artwork",
				"unreleased",
				"genre"
			]
		);
		let url_set = match obj.get("url") {
			None => url::UrlSet::empty(),
			Some(val_for_url) => url::UrlSet::from(val_for_url)
		};

		let artist = match obj.get("artist") {
			None => "Astro".to_string(),
			Some(val_for_artist) => {
				let artist = val_for_artist.as_str().unwrap_or_else(|| {
					panic!(
						"Song JSON attribute \"artist\" is not a string: {}",
						val_for_artist
					)
				});
				assert!(
					!artist.starts_with(char::is_whitespace)
						&& !artist.ends_with(char::is_whitespace),
					"Song JSON has poorly formed \"artist\" string: {}",
					artist
				);
				artist.to_string()
			}
		};

		let title = {
			let title_object = obj
				.get("title")
				.unwrap_or_else(|| panic!("Song JSON {} has no attribute \"title\"", val));
			let title = title_object.as_str().unwrap_or_else(|| {
				panic!(
					"Song JSON attribute \"title\" is not a string: {}",
					title_object
				)
			});
			assert!(
				!title.starts_with(char::is_whitespace) && !title.ends_with(char::is_whitespace),
				"Song JSON has poorly formed \"title\" string: {}",
				title
			);
			title.to_string()
		};

		let slug = globals::compute_slug(&artist, &title);

		let mut song = Song {
			parent_album_indices: None,
			artwork: match obj.get("artwork") {
				None => parent_album.map(|album| album.slug.clone()),
				Some(val_for_artwork) => match val_for_artwork {
					serde_json::Value::String(string) => {
						if !string
							.chars()
							.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-'))
						{
							panic!(
								"Invalid artwork name \"{}\": must contain only lowercase alphanumeric and hyphens",
								string
							);
						}
						let owned_string = string.to_string();
						assert!(
							owned_string != slug,
							"Custom artwork string \"{}\" cannot be the same as a slug; use true boolean instead",
							owned_string
						);
						Some(owned_string)
					}
					serde_json::Value::Bool(true) => Some(slug.clone()),
					_ => panic!(
						"Song JSON attribute \"artwork\" must be a string or true boolean: {}",
						val
					)
				}
			},
			released: obj
				.get("released")
				.map(|v| {
					let string = v.as_str().unwrap_or_else(|| {
						panic!("Song JSON attribute \"released\" is not a string: {}", v)
					});
					date::Date::from(string)
				})
				.unwrap_or_else(|| {
					parent_album
						.unwrap_or_else(|| {
							panic!(
								"Song JSON has no \"released\" attribute or parent album: {}",
								val
							)
						})
						.released
						.clone()
				}),
			unreleased: obj
				.get("unreleased")
				.map(|v| {
					v.as_bool().unwrap_or_else(|| {
						panic!("Song JSON attribute \"unreleased\" is not a bool: {}", v);
					})
				})
				.unwrap_or_else(|| parent_album.map(|album| album.unreleased).unwrap_or(false)),
			released_as_single: obj.get("released").is_some(),
			bonus: match obj.get("bonus") {
				None => false,
				Some(val_for_bonus) => val_for_bonus.as_bool().unwrap_or_else(|| {
					panic!(
						"Song JSON attribute \"bonus\" is not a boolean: {}",
						val_for_bonus
					)
				})
			},
			event: match obj.get("event") {
				None => false,
				Some(val_for_event) => val_for_event.as_bool().unwrap_or_else(|| {
					panic!(
						"Song JSON attribute \"event\" is not a boolean: {}",
						val_for_event
					)
				})
			},
			length: {
				let length = obj
					.get("length")
					.unwrap_or_else(|| panic!("Song JSON {} has no attribute \"length\"", val));

				length.as_i64().unwrap_or_else(|| {
					panic!(
						"Song JSON attribute \"length\" is not an integer: {}",
						length
					)
				}) as u32
			},
			isrc: obj.get("isrc").map(|v| {
				let isrc = v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"isrc\" is not a string: {}", v)
					})
					.to_owned();
				if isrc.len() != 12
					|| !isrc[..2].bytes().all(|b| b.is_ascii_uppercase())
					|| !isrc[2..5].bytes().all(|b| b.is_ascii_alphanumeric())
					|| !isrc[5..7].bytes().all(|b| b.is_ascii_digit())
					|| !isrc[7..].bytes().all(|b| b.is_ascii_digit())
				{
					panic!(
						"Song JSON attribute \"isrc\" is not a valid ISRC: \"{}\"",
						isrc
					);
				}
				isrc
			}),
			lyrics: if match obj.get("lyrics") {
				None => false,
				Some(val_for_lyrics) => val_for_lyrics.as_bool().unwrap_or_else(|| {
					panic!(
						"Song JSON attribute \"lyrics\" is not a boolean: {}",
						val_for_lyrics
					)
				})
			} {
				let lyrics_location = std::path::Path::new(globals::filezone())
					.join("source")
					.join("lyrics")
					.join(&slug)
					.with_extension("tsv");
				match std::fs::read_to_string(lyrics_location) {
					Ok(text) => Some(lyric::Lyrics::from(&text)),
					Err(_) => {
						/* globals::log_2(
							"Warning",
							format!("Couldn't read lyrics text {}.tsv", slug),
							globals::ANSI_RED
						);
						None */
						panic!("Couldn't read lyrics text {}.tsv", slug);
						// TODO only panic iff we are supposed to encode
						// building with missing lyrics is fine
						// encoding with missing lyrics is problematic because the mp3 and flac files with missing metadata will not be corrected later
						// but i don't really want to pass a boolean around all these constructors :P hmmmm
					}
				}
			} else {
				None
			},
			palette: obj
				.get("color")
				.map(|color_obj| color::Palette::from(color_obj, &url_set))
				.unwrap_or_else(|| {
					parent_album
						.unwrap_or_else(|| {
							panic!(
								"Song JSON has no \"color\" attribute or parent album: {}",
								val
							)
						})
						.palette
						.clone()
				}),
			url: url_set,
			samples: obj.get("samples").map(|v| {
				let arr = v.as_array().unwrap_or_else(|| {
					panic!("Song JSON attribute \"samples\" is not an array: {}", v)
				});
				arr.iter()
					.map(|s| {
						let sample = s
							.as_str()
							.unwrap_or_else(|| {
								panic!("Song JSON \"samples\" element is not a string: {}", s)
							})
							.to_owned();
						assert!(
							!sample.starts_with(char::is_whitespace)
								&& !sample.ends_with(char::is_whitespace),
							"Song JSON \"samples\" element is poorly formed string: \"{}\"",
							sample
						);
						sample
					})
					.collect()
			}),
			about: obj.get("about").map(|v| {
				let about = v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned();
				assert!(
					!about.starts_with(char::is_whitespace)
						&& !about.ends_with(char::is_whitespace),
					"Song JSON attribute \"about\" is poorly formed string: \"{}\"",
					about
				);
				about
			}),
			genre: match (
				parent_album,
				obj.get("genre").map(|s| {
					s.as_str().unwrap_or_else(|| {
						panic!("Song JSON attribute \"genre\" is not a string: {}", s)
					})
				})
			) {
				(None, None) => panic!(
					"Song {} must provide a genre for itself",
					title
				),
				(None, Some(genre_string)) => genre::Genre::from(genre_string),
				(Some(album), None) => album.genre.clone(),
				(Some(album), Some(genre_string)) => panic!(
					"Song on album {} must not specify its own genre {}",
					album.format_title(),
					genre_string
				)
			},
			slug,
			artist,
			title
		};

		song.slug = globals::compute_slug(&song.artist, &song.title);

		song
	}
	fn public_filename(&self) -> String {
		Titlable::Song(self).public_filename()
	}
	fn destination_location(&self, codec: &AudioCodec) -> std::path::PathBuf {
		std::path::Path::new(globals::filezone())
			.join(if self.bonus {
				"private"
			} else {
				"audio.astronomy487.com"
			})
			.join(codec.ext())
			.join(self.public_filename())
			.with_extension(codec.ext())
	}
	fn do_encode(&self, codec: &AudioCodec, all_albums: &[Album]) {
		let input_file_name = match self.parent_album_indices {
			// input_file_name includes album directory where we expect it
			Some((album_index, _)) => format!("{}/{}", all_albums[album_index].slug, &self.slug),
			None => self.slug.clone()
		} + ".flac";
		let input_file = std::path::Path::new(globals::filezone())
			.join("source")
			.join("audio")
			.join(&input_file_name);
		assert!(
			input_file.exists(),
			"Could not find audio source {}",
			input_file_name
		);
		let destination = self.destination_location(codec);
		if destination.exists() {
			// We don't need to encode it again :)
			// Or even rewrite any of its metadata!
			return;
		}

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
		globals::log_3(
			"Encoding",
			codec.ext(),
			self.format_title(),
			globals::ANSI_CYAN
		);
		let output = std::process::Command::new("ffmpeg.exe")
			.args(&args)
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.output()
			.expect("Failed to run ffmpeg");
		if !output.status.success() {
			let stderr = String::from_utf8_lossy(&output.stderr);
			panic!("FFmpeg encoding failed:\n{}", stderr);
		}

		// various audio validation
		let file = std::fs::File::open(&input_file).expect("Suddenly the input file doesn't exist");
		let mss = symphonia::core::io::MediaSourceStream::new(
			Box::new(file),
			symphonia::core::io::MediaSourceStreamOptions::default()
		);
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
			.expect("Symphonia couldn't process audio");
		let sr = track
			.codec_params
			.sample_rate
			.expect("Symphonia couldn't identify audio sample rate");
		let bit_depth = track
			.codec_params
			.bits_per_sample
			.expect("Symphonia couldn't identify audio bit depth");
		let frames = track
			.codec_params
			.n_frames
			.expect("Symphonia couldn't identify audio length");
		let dur_seconds = (frames as f64 / f64::from(sr)).floor() as u32;
		let dur_milliseconds = ((frames * 1000) as f64 / f64::from(sr)).floor() as u32;
		assert!(
			dur_seconds == self.length,
			"JSON reports that {} has length {}, but it has length {}",
			self.format_title(),
			self.length,
			dur_seconds
		);
		assert!(
			sr >= 44_100,
			"Expected 44.1 kHz (or higher), but file has {} Hz",
			sr
		);
		assert!(
			bit_depth >= 16,
			"Expected 16-bit audio (or higher), but file is {}-bit",
			bit_depth
		);

		// woaf is for a public-facing song page; woas is for parent album if it exists, else the song
		let (maybe_woaf_string_slug, maybe_woas_string_slug): (Option<&str>, Option<&str>) =
			match (self.bonus, self.parent_album_indices) {
				(true, None) => {
					panic!("Bonus track {} has no parent album", self.format_title())
				}
				(true, Some((album_index, _))) => (None, Some(&all_albums[album_index].slug)),
				(false, None) => (Some(&self.slug), Some(&self.slug)),
				(false, Some((album_index, _))) => {
					(Some(&self.slug), Some(&all_albums[album_index].slug))
				}
			};
		let maybe_woaf_string =
			maybe_woaf_string_slug.map(|s| format!("https://music.astronomy487.com/{}/", s));
		let maybe_woas_string =
			maybe_woas_string_slug.map(|s| format!("https://music.astronomy487.com/{}/", s));
		match codec {
			AudioCodec::Mp3 => {
				let mut tag = id3::Tag::new();
				id3::TagLike::set_title(&mut tag, &self.title);
				id3::TagLike::set_artist(&mut tag, &self.artist);
				match self.parent_album_indices {
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
						description: String::new(),
						data: self.grab_artwork_data(imagedeal::ImageCodec::Jpg)
					}
				);
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::Frame::text("TCON", self.genre.to_string())
				);
				id3::TagLike::set_year(&mut tag, self.released.year as i32);
				id3::TagLike::set_date_released(
					&mut tag,
					id3::Timestamp {
						year: self.released.year as i32,
						month: Some(self.released.month as u8),
						day: Some(self.released.day as u8),
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
				if let Some(woaf_string) = maybe_woaf_string {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::link("WOAF", woaf_string)
					);
				}
				if let Some(woas_string) = maybe_woas_string {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::link("WOAS", woas_string)
					);
				}
				let _ =
					id3::TagLike::add_frame(&mut tag, id3::frame::Frame::text("TENC", "distri"));
				let _ = id3::TagLike::add_frame(&mut tag, id3::frame::Frame::text("TFLT", "mp3"));
				if let Some((album_index, _)) = self.parent_album_indices {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text(
							"TCOP",
							all_albums[album_index].copyright_message()
						)
					);
				}
				if let Some(lyrics) = &self.lyrics {
					let lang_code = lyric::Lyrics::most_common_language(lyrics)
						.iso_639_2()
						.to_string();
					let uslt = id3::frame::Lyrics {
						lang: lang_code.clone(),
						description: String::new(),
						text: lyrics.as_filetype(lyric::TextCodec::Txt)
					};
					let _ = id3::TagLike::add_frame(&mut tag, uslt);
					let sylt = id3::frame::Frame::with_content(
						"SYLT",
						id3::frame::Content::SynchronisedLyrics(id3::frame::SynchronisedLyrics {
							lang: lang_code,
							timestamp_format: id3::frame::TimestampFormat::Ms,
							content_type: id3::frame::SynchronisedLyricsType::Other,
							description: String::new(),
							content: lyric::Lyrics::as_sylt_data(lyrics)
						})
					);
					let _ = id3::TagLike::add_frame(&mut tag, sylt);
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text("TLAN", lyrics.most_common_language().iso_639_2())
					);
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text("TLEN", dur_milliseconds.to_string())
					);
				}
				if tag
					.write_to_path(destination, id3::Version::Id3v24)
					.is_err()
				{
					panic!("Couldn't write mp3 metadata for {}", self.format_title());
				}
			}
			AudioCodec::Flac => {
				let mut tag = metaflac::Tag::read_from_path(destination).unwrap_or_else(|_| {
					panic!(
						"Want to write flac metadata for {}, but can't read the file",
						self.format_title()
					)
				});
				tag.set_vorbis("TITLE", vec![&self.title]);
				tag.set_vorbis("ARTIST", vec![&self.artist]);
				match self.parent_album_indices {
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
				tag.set_vorbis(
					"date::Date",
					vec![format!(
						"{:04}-{:02}-{:02}",
						self.released.year, self.released.month, self.released.day
					)]
				);
				tag.set_vorbis("YEAR", vec![self.released.year.to_string()]);
				if let Some(isrc) = &self.isrc {
					tag.set_vorbis("ISRC", vec![isrc]);
				}
				tag.set_vorbis("WOAR", vec!["https://astronomy487.com"]);
				tag.add_picture(
					"image/jpeg",
					metaflac::block::PictureType::Other,
					self.grab_artwork_data(imagedeal::ImageCodec::Jpg)
				);
				if let Some(woaf_string) = maybe_woaf_string {
					tag.set_vorbis("WOAF", vec![woaf_string]);
				}
				if let Some(woas_string) = maybe_woas_string {
					tag.set_vorbis("WOAS", vec![woas_string]);
				}
				tag.set_vorbis("GENRE", vec![self.genre.to_string()]);
				tag.set_vorbis("ENCODER", vec!["distri"]);
				tag.set_vorbis("FILETYPE", vec!["flac"]);
				if let Some((album_index, _)) = self.parent_album_indices {
					tag.set_vorbis(
						"COPYRIGHT",
						vec![all_albums[album_index].copyright_message()]
					);
				}
				if let Some(lyrics) = &self.lyrics {
					tag.set_vorbis("LYRICS", vec![&lyrics.as_filetype(lyric::TextCodec::Txt)]);
					tag.set_vorbis(
						"LYRICS_SYNCED",
						vec![&lyrics.as_filetype(lyric::TextCodec::Lrc)]
					);
				}
				assert!(
					tag.save().is_ok(),
					"Couldn't write flac metadata for {}",
					self.format_title()
				);
			}
		}
	}
	pub fn artwork_name(&self) -> String {
		match &self.artwork {
			Some(art) => art.clone(),
			None => globals::FALLBACK_ARTWORK_NAME.to_owned()
		}
	}
	pub fn grab_artwork_data(&self, codec: imagedeal::ImageCodec) -> Vec<u8> {
		imagedeal::grab_artwork_data(self.artwork_name(), codec)
	}
	pub fn check_artwork(&self) {
		imagedeal::check_artwork(self.artwork_name().to_string());
	}
	pub fn try_encode(&self, all_albums: &[Album]) {
		if !self.unreleased {
			self.do_encode(&AudioCodec::Mp3, all_albums);
			self.do_encode(&AudioCodec::Flac, all_albums);
		}
	}
	pub fn format_title(&self) -> String {
		Titlable::Song(self).format_title()
	}
	pub fn format_title_short(&self) -> String {
		Titlable::Song(self).format_title_short()
	}
}

#[derive(Debug)]
pub struct Assist {
	pub titlable: String,
	pub released: date::Date,
	pub artwork: String,
	pub url: String,
	pub role: String
}
impl Assist {
	fn from_json(val: &serde_json::Value) -> Assist {
		let assist = Assist {
			titlable: val
				.get("titlable")
				.unwrap_or_else(|| panic!("Assists JSON has no \"titlable\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| {
					panic!(
						"Assists JSON has non-string \"titlable\" attribute: {}",
						val
					)
				})
				.to_string(),
			artwork: val
				.get("artwork")
				.unwrap_or_else(|| panic!("Assists JSON has no \"artwork\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| {
					panic!("Assists JSON has non-string \"artwork\" attribute: {}", val)
				})
				.to_string(),
			url: val
				.get("url")
				.unwrap_or_else(|| panic!("Assists JSON has no \"url\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| panic!("Assists JSON has non-string \"url\" attribute: {}", val))
				.to_string(),
			role: val
				.get("role")
				.unwrap_or_else(|| panic!("Assists JSON has no \"role\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| {
					panic!("Assists JSON has non-string \"role\" attribute: {}", val)
				})
				.to_string(),
			released: date::Date::from(
				val.get("released")
					.unwrap_or_else(|| {
						panic!("Assists JSON has no \"released\" attribute: {}", val)
					})
					.as_str()
					.unwrap_or_else(|| {
						panic!(
							"Assists JSON has non-string \"released\" attribute: {}",
							val
						)
					})
			)
		};
		// whitespace
		assert!(
			assist.titlable.trim() == assist.titlable,
			"assist.titlable has leading/trailing whitespace: '{}'",
			assist.titlable
		);
		assert!(
			assist.artwork.trim() == assist.artwork,
			"assist.artwork has leading/trailing whitespace: '{}'",
			assist.artwork
		);
		assert!(
			assist.url.trim() == assist.url,
			"assist.url has leading/trailing whitespace: '{}'",
			assist.url
		);

		assert!(
			assist.role.trim() == assist.role,
			"assist.role has leading/trailing whitespace: '{}'",
			assist.role
		);

		// artwork validation
		let artwork = &assist.artwork;
		let valid_prefix = artwork.starts_with("https://");
		let valid_suffix = artwork.ends_with(".jpg") || artwork.ends_with(".png");
		assert!(
			valid_prefix && valid_suffix,
			"assist.artwork must start with http(s):// and end with .jpg or .png: '{}'",
			artwork
		);

		// "role" text validation
		if let Some(first_char) = assist.role.chars().next() {
			assert!(
				first_char.to_uppercase().to_string() == first_char.to_string(),
				"assist.role must start with an uppercase character: '{}'",
				assist.role
			);
		} else {
			panic!("assist.role is empty");
		}
		
		assist
	}
}

pub fn get_music_data(json_path: &std::path::Path) -> (Vec<Album>, Vec<Song>, Vec<Assist>) {
	globals::log_3("Parsing", "", "Discography JSON", globals::ANSI_GREEN);
	let file =
		std::fs::File::open(json_path).unwrap_or_else(|_| panic!("Couldn't find discog.json"));
	let reader = std::io::BufReader::new(file);
	let json_value: serde_json::Value = serde_json::from_reader(reader)
		.unwrap_or_else(|error| panic!("discog.json is invalid JSON: {}", error));
	let object = globals::map_with_only_these_keys(
		&json_value,
		"Discography",
		&["albums", "remixes", "assists"]
	);
	let mut all_remixes: Vec<Song> = {
		let mut remixes = Vec::new();
		for remix_json in object
			.get("remixes")
			.expect("discog.json has no attribute \"remixes\"")
			.as_array()
			.expect("discog.json \"remixes\" attribute is not an array")
			.iter()
		{
			remixes.push(Song::from_json(remix_json, None));
		}
		remixes
	};
	let mut all_albums: Vec<Album> = object
		.get("albums")
		.expect("discog.json has no attribute \"albums\"")
		.as_array()
		.expect("discog.json \"albums\" attribute is not an array")
		.iter()
		.map(Album::from_json)
		.collect();
	let mut all_assists: Vec<Assist> = object
		.get("assists")
		.expect("discog.json has no attribute \"assists\"")
		.as_array()
		.expect("discog.json \"assists\" attribute is not an array")
		.iter()
		.map(Assist::from_json)
		.collect();

	// assign parent_album refs
	for (album_index, album) in all_albums.iter_mut().enumerate() {
		for (song_index, song) in album.songs.iter_mut().enumerate() {
			song.parent_album_indices = Some((album_index, song_index));
		}
	}

	// validation
	for remix in &all_remixes {
		assert!(
			!remix.bonus,
			"Remix {} must not be marked as a bonus track",
			remix.format_title()
		);
		assert!(
			remix.artwork.is_none(),
			"Remix {} must not have artwork",
			remix.format_title()
		);
	}
	let mut seen_slugs = std::collections::HashSet::new();
	let mut check_slug_collision = |s: &str| {
		assert!(
			seen_slugs.insert(s.to_owned()),
			"Two items cannot both have the slug {}",
			s
		);
	};
	check_slug_collision("");
	for album in &all_albums {
		if album.single {
			assert!(
				album.title == album.songs[0].title,
				"Single cannot have two different titles: {}, {}",
				album.title,
				album.songs[0].title
			);
			assert!(
				album.artist == album.songs[0].artist,
				"Single cannot have two different artists: {}, {}",
				album.title,
				album.songs[0].title
			);
			check_slug_collision(&album.slug);
			for i in 2..album.songs.len() {
				assert!(
					album.songs[i].bonus,
					"Additional track in single {} must be marked as bonus",
					album.songs[i].format_title()
				);
				check_slug_collision(&album.songs[i].slug);
			}
		} else {
			check_slug_collision(&album.slug);
			for song in &album.songs {
				check_slug_collision(&song.slug);
			}
		}
		if !album.unreleased {
			for song in &album.songs {
				assert!(
					!song.unreleased,
					"Album {} has unreleased song {}",
					album.format_title(),
					song.format_title()
				);
			}
		}
	}
	for album in &all_albums {
		for song in &album.songs {
			assert!(
				!song.event,
				"Album track {} must not be marked as an event",
				song.format_title()
			);
			assert!(
				song.artwork.is_some(),
				"Non-remix song {} on album {} must have its own artwork or inherit from a parent (How did this manage to happen?)",
				song.format_title(),
				album.format_title()
			);
		}
		assert!(
			!album.songs[0].bonus,
			"First track of {} must not be a bonus track",
			album.format_title()
		);
		for window in album.songs.windows(2) {
			assert!(
				!window[0].bonus || window[1].bonus,
				"Bonus track {} is followed by non-bonus track {}",
				window[0].format_title(),
				window[1].format_title()
			);
		}
	}
	// also validate that flac exists
	for album in &all_albums {
		for song in &album.songs {
			let flac_location = std::path::Path::new(globals::filezone())
				.join("source")
				.join("audio")
				.join(&album.slug)
				.join(&song.slug)
				.with_extension("flac");
			assert!(
				flac_location.exists(),
				"Audio file {}/{}.flac missing",
				album.slug,
				song.slug
			);
		}
	}
	for remix in &all_remixes {
		let flac_location = std::path::Path::new(globals::filezone())
			.join("source")
			.join("audio")
			.join(&remix.slug)
			.with_extension("flac");
		assert!(
			flac_location.exists(),
			"Audio file {}.flac missing",
			remix.slug
		);
	}

	// check for monotonic release dates and force ascending
	match date::all_ascending_all_descending(&all_albums.iter().map(|a| a.released.clone()).collect::<Vec<_>>()) {
		(false, false) => panic!("Albums must be ordered by release date"),
		(false, true) => all_albums.reverse(),
		(true, false) => {},
		(true, true) => {} // funny
	}
	match date::all_ascending_all_descending(&all_remixes.iter().map(|a| a.released.clone()).collect::<Vec<_>>()) {
		(false, false) => panic!("Remixes must be ordered by release date"),
		(false, true) => all_remixes.reverse(),
		(true, false) => {},
		(true, true) => {} // funny
	}
	match date::all_ascending_all_descending(&all_assists.iter().map(|a| a.released.clone()).collect::<Vec<_>>()) {
		(false, false) => panic!("Assists must be ordered by release date"),
		(false, true) => all_assists.reverse(),
		(true, false) => {},
		(true, true) => {} // funny
	}

	(all_albums, all_remixes, all_assists)
}
