use crate::build::smartquotes;
use crate::fileops;
use crate::globals;
use crate::media::{
	artwork::Artwork, audiocodec::AudioCodec, lyric::TextCodec, song::Song, titlable::Titlable
};
use crate::types::{
	color::Palette, date::Date, duration::Duration, genre::Genre, upc::UPC, urlset::UrlSet
};

#[derive(Debug)]
pub struct Album {
	pub slug: String,
	pub songs: Vec<Song>,
	pub title: String,
	pub artist: String,
	pub released: Date,
	pub duration: Duration,
	pub upc: Option<UPC>,
	pub bcid: Option<String>,
	pub about: Option<Vec<String>>,
	pub palette: Palette,
	pub single: bool,
	pub compilation: bool,
	pub url: UrlSet,
	pub genre: Genre,
	pub unreleased: bool,
	pub discs: Option<Vec<(usize, String)>>,
	pub artwork: Artwork,
	pub has_8831: bool
}

impl Album {
	pub fn from_json(val: &serde_json::Value) -> Album {
		let obj = globals::map_with_only_these_keys(
			val,
			"Album",
			&[
				"about",
				"bcid",
				"color",
				"discs",
				"genre",
				"released",
				"songs",
				"title",
				"upc",
				"url",
				"compilation",
				"artist",
				"single",
				"unreleased",
				"slug"
			]
		);
		let url_set = {
			let url_val = obj
				.get("url")
				.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"url\"", val));

			UrlSet::from(url_val)
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
		let slug = match obj.get("slug") {
			Some(serde_json::Value::String(string)) => {
				globals::check_custom_slug(string);
				string.to_string()
			}
			Some(other) => panic!("Custom slug \"{}\" is not a string", other),
			None => globals::compute_slug(&artist, &title)
		};

		let mut album = Album {
			songs: Vec::new(),
			artwork: Artwork::from(Some(&slug), &slug),
			has_8831: {
				let location = globals::filezone()
					.join("source")
					.join("8831")
					.join(&slug)
					.with_extension("gif");
				/* if location.exists() {
					let gif = image::ImageReader::open(&location)
						.unwrap_or_else(|e| panic!("Couldn't find 8831/{}.gif: {}", &slug, e))
						.decode()
						.unwrap_or_else(|e| panic!("Couldn't decode 8831/{}.gif: {}", &slug, e));
					assert!(
						gif.width() == 88 && gif.height() == 31,
						"Image 8831/{}.gif must be 88x31, but it is {}x{}",
						&slug,
						gif.width(),
						gif.height()
					);
					true
				} else {
					false
				} */
				location.exists()
			},
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
				Date::from(rel_str)
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
				Genre::from(genre_str)
			},
			duration: Duration::zero(), // later filled via songs
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
				let upc = v.as_str().unwrap_or_else(|| {
					panic!("Album JSON attribute \"upc\" is not a string: {}", v)
				});
				UPC::from(upc).unwrap_or_else(|| {
					panic!(
						"Album JSON attribute \"upc\" is not a valid UPC: \"{}\"",
						upc
					)
				})
			}),
			bcid: obj.get("bcid").map(|v| {
				v.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"bcid\" is not a string: {}", v)
					})
					.to_owned()
			}),
			about: obj.get("about").map(|v| {
				let string = v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned();
				string
					.split("\n\n")
					.map(|paragraph| {
						assert!(
							!paragraph.starts_with(char::is_whitespace)
								&& !paragraph.ends_with(char::is_whitespace),
							"Album JSON attribute \"about\" has non-trimmed paragraph: {}",
							paragraph
						);
						assert!(
							!paragraph.contains("\n"),
							"Album JSON attribute \"about\" has lonely newline: {}",
							paragraph
						);
						paragraph.to_string()
					})
					.collect()
			}),
			palette: Palette::from(
				obj.get("color")
					.unwrap_or_else(|| panic!("Album JSON {} has no attribute \"color\"", val)),
				&url_set
			),
			url: url_set,
			discs: obj.get("discs").map(|v| {
				v.as_array()
					.unwrap_or_else(|| {
						panic!("Album JSON attribute \"discs\" is not an array: {}", v)
					})
					.chunks(2)
					.map(|chunk| {
						(
							chunk[0].as_u64().unwrap_or_else(|| {
								panic!("Album JSON attribute \"discs\" has a bad integer: {}", v)
							}) as usize,
							chunk[1]
								.as_str()
								.unwrap_or_else(|| {
									panic!("Album JSON attribute \"discs\" has a bad string: {}", v)
								})
								.to_string()
						)
					})
					.collect::<Vec<_>>()
			})
		};
		assert!(!smartquotes::contains_smart_quotes(&album.title));
		assert!(!smartquotes::contains_smart_quotes(&album.artist));
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
			album.duration =
				Duration::accumulate(album.songs.iter().filter(|s| !s.bonus).map(|s| s.duration));
		}
		if let Some(discs) = &album.discs {
			assert!(
				album.songs.iter().filter(|s| !s.bonus).count()
					== discs.iter().map(|d| d.0).sum::<usize>(),
				"Album {} disc lengths do not add up correctly",
				album.title
			);
		}
		album
	}
	pub fn public_filename(&self) -> String {
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
		let destination = globals::filezone()
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
		let mut zipper = fileops::Zipper::new(&destination, &self.released);
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
					song.format_title(),
					codec.ext()
				))
			);
			if let Some(lyrics) = &song.lyrics {
				// only include .txt - don't bother with lrc or srt in zips
				zipper.add_text_file(
					&lyrics.as_filetype(TextCodec::Txt),
					std::path::Path::new(&format!(
						"lyrics/{} {}.txt",
						pad_digits(self.songs.len(), song_index + 1),
						song.format_title()
					))
				);
			}
		}
		zipper.add_text_file(&self.readme(), std::path::Path::new("README.txt"));
		zipper.add_file(
			&self.artwork.source_path,
			std::path::Path::new("artwork.png")
		);
		zipper.finish();
	}
	pub fn non_bonus_song_count(&self) -> usize {
		self.songs.iter().take_while(|song| !song.bonus).count()
	}
	pub fn copyright_message_no_symbol(&self) -> String {
		format!(
			"{} {}",
			self.released.year,
			self.artist.replace("Astro", "Astro \"astronomy487\"")
		)
	}
	pub fn copyright_message_c_line(&self) -> String {
		format!(
			"© {} {}",
			self.released.year,
			self.artist.replace("Astro", "Astro \"astronomy487\"")
		)
	}
	pub fn copyright_message_p_line(&self) -> String {
		format!(
			"℗ {} {}",
			self.released.year,
			self.artist.replace("Astro", "Astro \"astronomy487\"")
		)
	}
	fn readme(&self) -> String {
		// utf-8, but use ascii ' "
		let mut text = Vec::new();
		text.push(self.format_title());
		text.push(self.released.to_display());
		if let Some(about) = &self.about {
			for paragraph in about {
				text.push(String::new());
				text.push(paragraph.to_string());
			}
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
		text.push(self.copyright_message_c_line());
		text.push(self.copyright_message_p_line());
		if self.cc_by_nc_sa_40() {
			text.push("Shared under CC BY-NC-SA 4.0 license. For more information, please visit https://creativecommons.org/licenses/by-nc-sa/4.0/.".to_string());
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
	pub fn audio_download_url(&self, codec: &AudioCodec) -> String {
		Titlable::Album(self).audio_download_url(codec)
	}
	pub fn cc_by_nc_sa_40(&self) -> bool {
		self.artist == "Astro"
	}
}
