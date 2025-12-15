use crate::color;
use crate::date;
use crate::fileops;
use crate::globals;
use crate::imagedeal;
use crate::lyric;
use crate::url;

pub fn format_duration(seconds: u32) -> String {
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
	if seconds > 0 || parts.is_empty() {
		parts.push(format!("{}s", seconds));
	}
	parts.join(" ")
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
				output.into(),
			]
		}
	}
}

pub struct Album {
	pub songs: Vec<Song>,
	title: String,
	artist: String,
	pub released: date::Date,
	length: u32,
	pub temporary: bool,
	upc: Option<String>,
	bcid: Option<String>,
	pub about: Option<String>,
	palette: Option<color::Palette>,
	single: bool,
	compilation: bool,
	url: url::UrlSet
}

pub struct Song {
	parent_album_indices: Option<(usize, usize)>, // album-index, position in tracklist
	title: String,
	artist: String,
	released: Option<date::Date>,
	pub bonus: bool,
	event: bool,
	single_artwork: Option<String>,
	length: u32,
	isrc: Option<String>,
	lyrics: Option<lyric::Lyrics>,
	palette: Option<color::Palette>,
	url: Option<url::UrlSet>,
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
	fn palette(&self) -> &Option<color::Palette> {
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
			Titlable::Song(song, _) => std::path::Path::new(globals::filezone())
				.join("audio.astronomy487.com")
				.join(codec.ext())
				.join(song.slug())
				.with_extension(codec.ext()),
			Titlable::Album(album) => std::path::Path::new(globals::filezone())
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
		let destination_folder = std::path::Path::new(globals::filezone())
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
			let _ = std::io::Write::write(&mut file, lyric::Lyrics::as_plaintext(lxs).as_bytes())
				.unwrap_or_else(|_| {
					panic!("Couldn't write to file {}", lyrics_txt_location.display())
				});
			let lyrics_lrc_location = destination_folder.join("lyrics.lrc");
			let mut file = std::fs::File::create(&lyrics_lrc_location).unwrap_or_else(|_| {
				panic!("Couldn't create file {}", lyrics_lrc_location.display())
			});
			let _ = std::io::Write::write(&mut file, lyric::Lyrics::as_lrc(lxs).as_bytes())
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
						"Song {} has no release date::Date and no parent album",
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
			Titlable::Song(s, parent) => {
				&url::UrlSet::combine(s.url.as_ref(), parent.map(|a| &a.url))
			}
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
						meta name="theme-color" content=(p.html_theme_color());
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
							( palette.palette_mode_as_css_class_name() )
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
										(fileops::format_file_size(mp3_size))
									}
								}
								td {
									a href=(thing_to_offer_as_download.audio_download_url(&Codec::Flac)) download {
										(Codec::Flac.ext())
										@if let Titlable::Album(_) = thing_to_offer_as_download {
											" zip"
										}
										", "
										(fileops::format_file_size(flac_size))
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
									@for format in ["txt", "lrc"] {
										td {
											a href=("lyrics.".to_owned() + format) download=(&format!("{}-lyrics.{}", song.slug(), format)) {
												(format)
												", "
												(fileops::format_file_size(fileops::filesize(
													&std::path::Path::new(globals::filezone()).join("music.astronomy487.com").join(song.slug()).join("lyrics").with_extension(format)
												)))
											}
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
						panic!(
							"Song {} has no release date::Date or parent album",
							song.slug()
						)
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
								.parent_album_indices
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
		let obj = globals::map_with_only_these_keys(
			val,
			"Album",
			&[
				"about",
				"bcid",
				"color",
				"length",
				"released",
				"songs",
				"title",
				"upc",
				"url",
				"compilation",
				"artist",
				"single",
				"temporary"
			]
		);
		Album {
			songs: {
				let songs_val = obj
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
			artist: match obj.get("artist") {
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
				let title_val = obj
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
			temporary: match obj.get("temporary") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Album JSON attribute \"temporary\" is not a boolean: {}", v)
				})
			},
			single: match obj.get("single") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Album JSON attribute \"single\" is not a boolean: {}", v)
				})
			},
			compilation: match obj.get("compilation") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"compilation\" is not a boolean: {}",
						v
					)
				})
			},
			upc: obj.get("upc").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"upc\" is not a string: {}", v)
					})
					.to_owned()
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
			}),
			palette: obj.get("color").map(color::Palette::from),
			url: {
				let url_val = obj
					.get("url")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"url\"", val));

				url::UrlSet::from(url_val, true)
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
		let destination = std::path::Path::new(globals::filezone())
			.join("audio.astronomy487.com")
			.join(codec.ext())
			.join(self.slug())
			.with_extension("zip");
		if destination.exists() {
			return;
		}
		globals::log_3(
			"Zipping",
			codec.ext(),
			self,
			globals::ANSI_MAGENTA
		);
		let mut zipper = fileops::Zipper::new(&destination);
		for (song_index, song) in self.songs.iter().enumerate() {
			globals::log_3(
				"",
				format!("+ {}", song_index + 1),
				song,
				globals::ANSI_MAGENTA
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
					&lyric::Lyrics::as_plaintext(lyrics),
					std::path::Path::new(&format!(
						"lyrics/txt/{:03}-{}.txt",
						song_index,
						song.slug()
					))
				);
				zipper.add_text_file(
					&lyric::Lyrics::as_lrc(lyrics),
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
			&std::path::Path::new(globals::filezone())
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
		let obj = globals::map_with_only_these_keys(
			val,
			"Song",
			&[
				"artist", "title", "released", "bonus", "event", "length", "isrc", "lyrics",
				"color", "url", "samples", "about", "artwork"
			]
		);
		let mut song = Song {
			parent_album_indices: None,
			artist: match obj.get("artist") {
				None => "Astro".to_string(),
				Some(v) => v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"artist\" is not a string: {}", v)
					})
					.to_string()
			},
			title: {
				let t = obj
					.get("title")
					.unwrap_or_else(|| panic!("Song JSON {} has no attribute \"title\"", val));
				t.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"title\" is not a string: {}", t)
					})
					.to_string()
			},
			released: obj.get("released").map(|v| {
				let s = v.as_str().unwrap_or_else(|| {
					panic!("Song JSON attribute \"released\" is not a string: {}", v)
				});
				date::Date::from(s)
			}),
			bonus: match obj.get("bonus") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Song JSON attribute \"bonus\" is not a boolean: {}", v)
				})
			},
			event: match obj.get("event") {
				None => false,
				Some(v) => v.as_bool().unwrap_or_else(|| {
					panic!("Song JSON attribute \"event\" is not a boolean: {}", v)
				})
			},
			single_artwork: None, // single_artwork determined right after construction
			length: {
				let l = obj
					.get("length")
					.unwrap_or_else(|| panic!("Song JSON {} has no attribute \"length\"", val));

				l.as_i64().unwrap_or_else(|| {
					panic!("Song JSON attribute \"length\" is not an integer: {}", l)
				}) as u32
			},
			isrc: obj.get("isrc").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"isrc\" is not a string: {}", v)
					})
					.to_owned()
			}),
			lyrics: obj.get("lyrics").map(|v| {
				let s = v.as_str().unwrap_or_else(|| {
					panic!("Song JSON attribute \"lyrics\" is not a string: {}", v)
				});
				lyric::Lyrics::from(s)
			}),
			palette: obj.get("color").map(color::Palette::from),
			url: obj.get("url").map(|v| url::UrlSet::from(v, false)),
			samples: obj.get("samples").map(|v| {
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
			about: obj.get("about").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned()
			})
		};

		// artwork handling
		match obj.get("artwork") {
			None => {}
			Some(v) => match v {
				serde_json::Value::String(s) => {
					if !s.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-')) {
						panic!(
							"Invalid artwork name \"{}\": must contain only lowercase alphanumeric and hyphens",
							s
						);
					}
					song.single_artwork = Some(s.to_string());

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
		std::path::Path::new(globals::filezone())
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
		let input_file_name = match self.parent_album_indices {
			// input_file_name includes album directory where we expect it
			Some((album_index, _)) => all_albums[album_index].slug() + "/" + &self.slug(),
			None => self.slug()
		} + ".flac";
		let input_file = std::path::Path::new(globals::filezone())
			.join("source")
			.join("audio")
			.join(&input_file_name);
		if !input_file.exists() {
			panic!("Could not find audio source {}", input_file_name)
		}
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
			self,
			globals::ANSI_YELLOW
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
		let params = &track.codec_params;
		let sr = params.sample_rate.expect("Symphonia couldn't identify sample rate");
		assert!(
			sr >= 44_100,
			"Expected 44.1 kHz (or higher), but file has {} Hz",
			sr
		);
		let bit_depth = params.bits_per_sample.expect("Symphonia couldn't identify bit depth");
		assert!(
			bit_depth >= 16,
			"Expected 16-bit audio (or higher), but file is {}-bit",
			bit_depth
		);
		
		// woaf is for a public-facing song page; woas is for parent album if it exists, else the song
		let (woaf_string, woas_string) = match (self.bonus, self.parent_album_indices) {
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
						description: "".to_string(),
						text: lyric::Lyrics::as_plaintext(lyrics)
					};
					let _ = id3::TagLike::add_frame(&mut tag, uslt);
					let sylt = id3::frame::Frame::with_content(
						"SYLT",
						id3::frame::Content::SynchronisedLyrics(id3::frame::SynchronisedLyrics {
							lang: lang_code,
							timestamp_format: id3::frame::TimestampFormat::Ms,
							content_type: id3::frame::SynchronisedLyricsType::Other,
							description: "".to_string(),
							content: lyric::Lyrics::as_sylt_data(lyrics)
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
				let release_date = self.release_date(all_albums);
				tag.set_vorbis(
					"date::Date",
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
				if let Some((album_index, _)) = self.parent_album_indices {
					tag.set_vorbis(
						"COPYRIGHT",
						vec![all_albums[album_index].copyright_message()]
					);
				}
				if let Some(lyrics) = &self.lyrics {
					tag.set_vorbis("LYRICS", vec![&lyric::Lyrics::as_plaintext(lyrics)]);
					tag.set_vorbis("LYRICS_SYNCED", vec![&lyric::Lyrics::as_lrc(lyrics)]);
				}
				if tag.save().is_err() {
					panic!("Couldn't write flac metadata for {}", self);
				}
			}
		}
	}
	fn release_date<'a>(&'a self, all_albums: &'a [Album]) -> &'a date::Date {
		match &self.released {
			Some(date) => date,
			None => match self.parent_album_indices {
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
			None => match self.parent_album_indices {
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
	globals::log_3("Parsing", "", "Discography JSON", globals::ANSI_CYAN);
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
			song.parent_album_indices = Some((album_index, song_index));
		}
	}

	// validation
	for remix in &remixes {
		if remix.released.is_none() {
			panic!("Remix {} must have a release date::Date", remix);
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
			if !song.bonus && song.url.is_none() {
				panic!("Non-bonus track {} must have a URL set", song);
			}
		}
		assert!(!album.songs[0].bonus);
		for window in album.songs.windows(2) {
			if window[0].bonus && !window[1].bonus {
				panic!(
					"Bonus track {} is followed by non-bonus track {}",
					window[0], window[1]
				);
			}
		}
	}

	// Check that all album artwork is where it needs to be
	for album in &albums {
		let _ = imagedeal::grab_image(album.slug());
		for song in &album.songs {
			let _ = song.grab_artwork(&albums);
		}
	}
	for remix in &remixes {
		let _ = remix.grab_artwork(&albums);
	}

	(albums, remixes)
}
