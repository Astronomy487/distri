use crate::build::smartquotes;
use crate::globals;
use crate::media::{
	album::Album, artwork::Artwork, audiocodec::AudioCodec, lyric, lyric::Lyrics,
	titlable::Titlable
};
use crate::types::{
	color::Palette, date::Date, duration::Duration, genre::Genre, isrc::ISRC, urlset::UrlSet
};

#[derive(Debug)]
pub struct Song {
	pub slug: String,
	pub parent_album_indices: Option<(usize, usize)>, // album-index, position in tracklist
	pub title: String,
	pub artist: String,
	pub released: Date, // may inherit from parent
	pub released_as_single: bool,
	pub bonus: bool,
	pub event: bool,
	pub artwork: Option<Artwork>, // songs on albums inherit from parents; remixes have None
	pub duration: Duration,
	pub isrc: Option<ISRC>,
	pub lyrics: Option<Lyrics>,
	pub palette: Palette, // may inherit from parent
	pub genre: Genre,     // MUST inherit from parent if on an album
	pub unreleased: bool, // may inherit from parent
	pub url: UrlSet,
	pub samples: Option<Vec<String>>, // report as "Mix tracklist" if event
	pub about: Option<Vec<String>>
}

impl Song {
	pub fn from_json(val: &serde_json::Value, parent_album: Option<&Album>) -> Song {
		let obj = globals::map_with_only_these_keys(
			val,
			"Song",
			&[
				"artist",
				"title",
				"released",
				"bonus",
				"event",
				"isrc",
				"lyrics",
				"color",
				"url",
				"samples",
				"about",
				"artwork",
				"unreleased",
				"genre",
				"slug"
			]
		);
		let url_set = match obj.get("url") {
			None => UrlSet::empty(),
			Some(val_for_url) => UrlSet::from(val_for_url)
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

		let slug = match obj.get("slug") {
			Some(serde_json::Value::String(string)) => {
				globals::check_custom_slug(string);
				string.to_string()
			}
			Some(other) => panic!("Custom slug \"{}\" is not a string", other),
			None => globals::compute_slug(&artist, &title)
		};

		let song = Song {
			parent_album_indices: None,
			artwork: match obj.get("artwork") {
				None => parent_album.map(|album| album.artwork.clone()),
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
						assert!(
							*string != slug,
							"Custom artwork string \"{}\" cannot be the same as a slug; use true boolean instead",
							string
						);
						Some(Artwork::from(
							parent_album.map(|album| album.slug.as_str()),
							string
						))
					}
					serde_json::Value::Bool(true) => Some(Artwork::from(
						parent_album.map(|album| album.slug.as_str()),
						&slug
					)),
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
					Date::from(string)
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
			duration: Duration::from_audio_file_and_validate(
				parent_album.map(|a| a.slug.as_str()),
				&slug
			),
			isrc: obj.get("isrc").map(|v| {
				let isrc = v.as_str().unwrap_or_else(|| {
					panic!("Song JSON attribute \"isrc\" is not a string: {}", v)
				});
				ISRC::from(isrc).unwrap_or_else(|| {
					panic!(
						"Song JSON attribute \"isrc\" is not a valid ISRC: \"{}\"",
						isrc
					)
				})
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
				let lyrics_location = {
					let mut location = globals::filezone().join("source").join("lyrics");
					if let Some(album) = parent_album {
						location = location.join(&album.slug);
					}
					location.join(&slug).with_extension("tsv")
				};
				match std::fs::read_to_string(lyrics_location) {
					Ok(text) => Some(Lyrics::from(&text)),
					Err(_) => {
						panic!(
							"Couldn't read lyrics text {}{}.tsv",
							match parent_album {
								Some(album) => format!("{}/", album.slug),
								None => String::new()
							},
							slug
						);
					}
				}
			} else {
				None
			},
			
			palette: obj
				.get("color")
				.map(|color_obj| Palette::from(color_obj, &url_set))
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
				let string = v
					.as_str()
					.unwrap_or_else(|| {
						panic!("Song JSON attribute \"about\" is not a string: {}", v)
					})
					.to_owned();
				string
					.split("\n\n")
					.map(|paragraph| {
						assert!(
							!paragraph.starts_with(char::is_whitespace)
								&& !paragraph.ends_with(char::is_whitespace),
							"Song JSON attribute \"about\" has non-trimmed paragraph: {}",
							paragraph
						);
						assert!(
							!paragraph.contains("\n"),
							"Song JSON attribute \"about\" has lonely newline: {}",
							paragraph
						);
						paragraph.to_string()
					})
					.collect()
			}),
			genre: match (
				parent_album,
				obj.get("genre").map(|s| {
					s.as_str().unwrap_or_else(|| {
						panic!("Song JSON attribute \"genre\" is not a string: {}", s)
					})
				})
			) {
				(None, None) => panic!("Song {} must provide a genre for itself", title),
				(None, Some(genre_string)) => Genre::from(genre_string),
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

		assert!(!smartquotes::contains_smart_quotes(&song.title));
		assert!(!smartquotes::contains_smart_quotes(&song.artist));

		song
	}
	pub fn public_filename(&self) -> String {
		Titlable::Song(self).public_filename()
	}
	pub fn destination_location(&self, codec: &AudioCodec) -> std::path::PathBuf {
		globals::filezone()
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
		// this code is repeated in duration.rs
		let input_file_name = match self.parent_album_indices {
			// input_file_name includes album directory where we expect it
			Some((album_index, _)) => format!("{}/{}", all_albums[album_index].slug, &self.slug),
			None => self.slug.clone()
		} + ".flac";
		let input_file = globals::filezone()
			.join("source")
			.join("audio")
			.join(&input_file_name);
		assert!(
			input_file.exists(),
			"Could not find audio source {}",
			input_file_name
		);
		let final_destination = self.destination_location(codec);
		let temporary_destination = globals::filezone().join("temp").with_extension(codec.ext());
		if final_destination.exists() {
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
			temporary_destination.to_str().unwrap_or_else(|| {
				panic!(
					"FFmpeg output file {} could not be made into a string",
					temporary_destination.display()
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
				id3::TagLike::set_duration(&mut tag, self.duration.seconds());
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::frame::Picture {
						mime_type: "image/jpeg".to_string(),
						picture_type: id3::frame::PictureType::Other,
						description: String::new(),
						data: match &self.artwork {
							Some(artwork) => artwork.jpg_data(),
							None => Artwork::fallback().jpg_data()
						}
					}
				);
				let _ = id3::TagLike::add_frame(
					&mut tag,
					id3::Frame::text("TCON", self.genre.to_string())
				);
				id3::TagLike::set_year(&mut tag, i32::from(self.released.year));
				id3::TagLike::set_date_released(
					&mut tag,
					id3::Timestamp {
						year: i32::from(self.released.year),
						month: Some(self.released.month),
						day: Some(self.released.day),
						hour: None,
						minute: None,
						second: None
					}
				);
				if let Some(isrc) = &self.isrc {
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text("TSRC", isrc.as_dense())
					);
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
							all_albums[album_index].copyright_message_no_symbol()
						)
					);
				}
				if let Some(lyrics) = &self.lyrics {
					let lang_code = Lyrics::most_common_language(lyrics).iso_639_2().to_string();
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
							content_type: id3::frame::SynchronisedLyricsType::Lyrics,
							description: String::new(),
							content: Lyrics::as_sylt_data(lyrics)
						})
					);
					let _ = id3::TagLike::add_frame(&mut tag, sylt);
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text("TLAN", lyrics.most_common_language().iso_639_2())
					);
					let _ = id3::TagLike::add_frame(
						&mut tag,
						id3::frame::Frame::text("TLEN", self.duration.milliseconds().to_string())
					);
				}
				if tag
					.write_to_path(&temporary_destination, id3::Version::Id3v24)
					.is_err()
				{
					panic!("Couldn't write mp3 metadata for {}", self.format_title());
				}
			}
			AudioCodec::Flac => {
				let mut tag =
					metaflac::Tag::read_from_path(&temporary_destination).unwrap_or_else(|_| {
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
				tag.set_vorbis("LENGTH", vec![self.duration.seconds().to_string()]);
				tag.set_vorbis(
					"Date",
					vec![format!(
						"{:04}-{:02}-{:02}",
						self.released.year, self.released.month, self.released.day
					)]
				);
				tag.set_vorbis("YEAR", vec![self.released.year.to_string()]);
				if let Some(isrc) = &self.isrc {
					tag.set_vorbis("ISRC", vec![isrc.as_dense()]);
				}
				tag.set_vorbis("WOAR", vec!["https://astronomy487.com"]);
				tag.add_picture(
					"image/jpeg",
					metaflac::block::PictureType::Other,
					match &self.artwork {
						Some(artwork) => artwork.jpg_data(),
						None => Artwork::fallback().jpg_data()
					}
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
						vec![
							all_albums[album_index].copyright_message_c_line(),
							all_albums[album_index].copyright_message_p_line(),
						]
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

		std::fs::rename(&temporary_destination, &final_destination)
			.expect("Failed to move song to final destination");
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
	pub fn audio_download_url(&self, codec: &AudioCodec) -> String {
		Titlable::Song(self).audio_download_url(codec)
	}
}
