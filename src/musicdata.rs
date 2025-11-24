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
	fn as_string(&self) -> String {
		format!(
			"{} {}, {}",
			match self.month {
				1 => "January",
				2 => "February",
				3 => "March",
				4 => "April",
				5 => "May",
				6 => "June",
				7 => "July",
				8 => "August",
				9 => "September",
				10 => "October",
				11 => "November",
				12 => "December",
				_ => {
					panic!("......what kind of month is {}", self.month)
				}
			},
			self.day,
			self.year
		)
	}
}
impl std::fmt::Display for Date {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// pad month and day with two digits
		write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
	}
}

struct LyricLine {
	// TODO - flags ??
	start: f32,
	end: f32,
	text: String,
	flag_language: Option<String>, // kept as ISO 693-1, as all things should be
	flag_vocalist: Option<String>
}
impl LyricLine {
	fn flag_language_to_iso_639_2(flag_language: &Option<String>) -> &'static str {
		match flag_language.as_deref() {
			None => "eng",
			Some("en") => "eng",
			Some("ja") => "jpn",
			Some("fr") => "fra",
			Some(other) => {
				panic!("Unknown language ISO 693-1: {}", other);
			}
		}
	}
	fn from(text: &str) -> Vec<Vec<LyricLine>> {
		text.split("\n\n")
			.map(|stanza| {
				stanza
					.lines()
					.filter_map(|line| {
						let parts: Vec<&str> = line.split('\t').collect();
						if parts.len() >= 3 {
							let start = parts[0].parse::<f32>().ok()?;
							let end = parts[1].parse::<f32>().ok()?;
							let text = parts[2].to_string();
							let mut flag_language = None;
							let mut flag_vocalist = None;
							for kv in &parts[3..] {
								if let Some((key, value)) = kv.split_once(":") {
									match key {
										"language" => flag_language = Some(value.to_string()),
										"vocalist" => flag_vocalist = Some(value.to_string()),
										_ => {
											panic!("Unknown lyric tag: {}", key)
										}
									}
								}
							}
							Some(LyricLine {
								start,
								end,
								text,
								flag_language,
								flag_vocalist
							})
						} else {
							None
						}
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
				"-q:a".into(),
				"0".into(), // good quality but also variable bit rate
				"-map_metadata".into(),
				"-1".into(), // wipe metadata
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
				"-1".into(), // wipe all metadata
				"-loglevel".into(),
				"panic".into(),
				output.into(),
			]
		}
	}
}

pub struct Album {
	songs: Vec<Song>,
	title: String,
	artist: String,
	released: Date,
	length: u32,
	pub temporary: bool,
	upc: Option<String>,
	about: Option<String>
}

pub struct Song {
	parent_album: Option<(usize, usize)>, // album-index, position in tracklist
	title: String,
	artist: String,
	released: Option<Date>,
	bonus: bool,
	event: bool,
	single_artwork: Option<String>,
	length: u32,
	isrc: Option<String>,
	lyrics: Option<Vec<Vec<LyricLine>>>
}

pub trait Titlable {
	fn artist(&self) -> &str;
	fn title(&self) -> &str;
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
}

impl std::fmt::Display for Album {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} - {}", self.artist(), self.title())
	}
}
impl std::fmt::Display for Song {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} - {}", self.artist(), self.title())
	}
}
impl std::fmt::Debug for Album {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} - {}", self.artist(), self.title())
	}
}
impl std::fmt::Debug for Song {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} - {}", self.artist(), self.title())
	}
}

impl Album {
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
			upc: val.get("upc").map(|v| {
				v.as_str()
					.expect("discog album has non-string upc")
					.to_owned()
			}),
			about: val.get("about").map(|v| {
				v.as_str()
					.expect("discog album has non-string about")
					.to_owned()
			})
		}
	}
	pub fn try_encode(&self) -> bool {
		let mut all_successful = true;
		for song in &self.songs {
			let this_successful = song.try_encode(Some(&self));
			all_successful = all_successful && this_successful;
		}
		if !all_successful {
			println!("\x1b[91m{} could not be zipped together\x1b[0m", self);
		} else {
			self.zip(&Codec::Mp3);
			self.zip(&Codec::Flac);
		}
		all_successful
	}
	fn zip(&self, codec: &Codec) {
		// panic if can't
		let destination =
			std::path::Path::new("C:/Users/astro/Code/distri/filezone/audio.astronomy487.com")
				.join(codec.ext())
				.join(self.slug())
				.with_extension("zip");
		if destination.exists() {
			return;
		}
		println!("Zipping {} {}", self, codec.ext());
		let mut zipper = zipper::Zipper::new(&destination);
		for (song_index, song) in self.songs.iter().enumerate() {
			println!("- Adding {}", song);
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
			&std::path::Path::new("C:/Users/astro/Code/distri/filezone/source/image")
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
		text.push(self.released.as_string());

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
		text.push(self.copyright_message(self.released.year));
		text.push("Thank you for downloading!".to_string());
		text.push(format!("https://music.astronomy487.com/{}", self.slug()));

		text.join("\n")
	}
}

impl Song {
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
				.map(|v| LyricLine::from(v.as_str().expect("discog song has non-string lyrics")))
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
	pub fn get_parent_album<'a>(&self, albums_list: &'a [Album]) -> Option<(&'a Album, usize)> {
		match self.parent_album {
			None => None,
			Some((parent_album_index, position_in_tracklist)) => {
				Some((&albums_list[parent_album_index], position_in_tracklist))
			}
		}
	}
	fn destination_location(&self, codec: &Codec) -> std::path::PathBuf {
		std::path::Path::new("C:/Users/astro/Code/distri/filezone")
			.join(if self.bonus {
				"private"
			} else {
				"audio.astronomy487.com"
			})
			.join(codec.ext())
			.join(self.slug())
			.with_extension(codec.ext())
	}
	fn do_encode(&self, codec: &Codec, parent_album: Option<&Album>) -> bool {
		let input_file_base =
			std::path::Path::new("C:/Users/astro/Code/distri/filezone/source/audio")
				.join(self.slug());
		let mut input_file = input_file_base.with_extension("flac");
		if !input_file.exists() {
			input_file = input_file_base.with_extension("wav");
		}
		if !input_file.exists() {
			// println!("\x1b[94mfile not found : {}.flac/wav\x1b[0m", self.slug());
			return false;
		}
		let destination = self.destination_location(&codec);
		if !destination.exists() {
			let args =
				codec.ffmpeg_args(input_file.to_str().unwrap(), destination.to_str().unwrap());
			println!("encoding {}.{}", self.slug(), codec.ext());
			let ffmpeg_status = std::process::Command::new("ffmpeg").args(&args).status();
			if !matches!(ffmpeg_status, std::result::Result::Ok(s) if s.success()) {
				panic!("FFMPEG could not complete the action ???");
			}
		}
		// TODO - in final version, don't re-write metadata if destination already exists
		match codec {
			Codec::Mp3 => {
				let mut tag = id3::Tag::new();
				id3::TagLike::set_title(&mut tag, &self.title);
				id3::TagLike::set_artist(&mut tag, &self.artist);
				if let Some(parent_album) = parent_album {
					// TODO - for standalone, how to handle lack of parent album? - check apps for what's best
					id3::TagLike::set_album(&mut tag, &parent_album.title);
					id3::TagLike::set_album_artist(&mut tag, &parent_album.artist);
					let (_, position_in_tracklist) = &self.parent_album.unwrap();
					id3::TagLike::set_track(&mut tag, (position_in_tracklist + 1) as u32);
					id3::TagLike::set_total_tracks(
						&mut tag,
						parent_album.non_bonus_song_count() as u32
					);
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
					id3::frame::Frame::text("TCOP", self.copyright_message(release_date.year))
				);
				if let Some(lyrics) = &self.lyrics {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Lyrics {
							// TODO - if song has several languages, how should metadata relay that?
							lang: LyricLine::flag_language_to_iso_639_2(
								&lyrics[0][0].flag_language
							)
							.to_string(),
							description: "".to_string(),
							text: LyricLine::as_plaintext(lyrics)
						}
					);
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::with_content(
							"SYLT",
							id3::frame::Content::SynchronisedLyrics(
								id3::frame::SynchronisedLyrics {
									lang: LyricLine::flag_language_to_iso_639_2(
										&lyrics[0][0].flag_language
									)
									.to_string(),
									timestamp_format: id3::frame::TimestampFormat::Ms,
									content_type: id3::frame::SynchronisedLyricsType::Other,
									description: "".to_string(),
									content: LyricLine::as_sylt_data(lyrics)
								}
							)
						)
					);
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
				if let Some(parent_album) = parent_album {
					tag.set_vorbis("ALBUM", vec![&parent_album.title]);
					tag.set_vorbis("ALBUMARTIST", vec![&parent_album.artist]);
					let (_, position_in_tracklist) = &self.parent_album.unwrap();
					tag.set_vorbis("TRACKNUMBER", vec![(position_in_tracklist + 1).to_string()]);
					tag.set_vorbis(
						"TRACKTOTAL",
						vec![parent_album.non_bonus_song_count().to_string()]
					);
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
				tag.set_vorbis("COPYRIGHT", vec![self.copyright_message(release_date.year)]);
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
	pub fn try_encode(&self, parent_album: Option<&Album>) -> bool {
		let mp3_success = self.do_encode(&Codec::Mp3, parent_album);
		let flac_success = self.do_encode(&Codec::Flac, parent_album);
		if !mp3_success || !flac_success {
			// println!("\x1b[91m{} could not be fully encoded\x1b[0m", self);
		}
		mp3_success && flac_success
	}
}

impl Titlable for Song {
	fn artist(&self) -> &str {
		return &self.artist;
	}
	fn title(&self) -> &str {
		return &self.title;
	}
}
impl Titlable for Album {
	fn artist(&self) -> &str {
		return &self.artist;
	}
	fn title(&self) -> &str {
		return &self.title;
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

	(albums, remixes)
}
