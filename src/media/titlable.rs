use crate::globals;
use crate::media::{album::Album, artwork::Artwork, audiocodec::AudioCodec, song::Song};
use crate::types::{color::Palette, date::Date, duration::Duration, genre::Genre};

#[derive(Debug)]
pub enum Titlable<'a> {
	Song(&'a Song),
	Album(&'a Album)
}
impl Titlable<'_> {
	pub fn artist(&self) -> &str {
		match self {
			Titlable::Song(song) => &song.artist,
			Titlable::Album(album) => &album.artist
		}
	}
	pub fn palette(&self) -> &Palette {
		match self {
			Titlable::Song(song) => &song.palette,
			Titlable::Album(album) => &album.palette
		}
	}
	pub fn title(&self) -> &str {
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
	pub fn released(&self) -> &Date {
		match self {
			Titlable::Song(song) => &song.released,
			Titlable::Album(album) => &album.released
		}
	}
	pub fn duration(&self) -> Duration {
		match self {
			Titlable::Song(song) => song.duration,
			Titlable::Album(album) => album.duration
		}
	}
	pub fn genre(&self) -> &Genre {
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
	pub fn artwork(&self) -> Option<&Artwork> {
		match self {
			Titlable::Song(song) => song.artwork.as_ref(),
			Titlable::Album(album) => Some(&album.artwork)
		}
	}
	pub fn about(&self) -> Option<&Vec<String>> {
		match self {
			Titlable::Song(song) => song.about.as_ref(),
			Titlable::Album(album) => album.about.as_ref()
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
			Titlable::Song(song) => globals::filezone()
				.join(if song.bonus {
					"private"
				} else {
					"audio.astronomy487.com"
				})
				.join(codec.ext())
				.join(song.public_filename())
				.with_extension(codec.ext()),
			Titlable::Album(album) => globals::filezone()
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
	pub fn public_filename(&self) -> String {
		/* let title = self.format_title();
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
		cleaned */
		let mut cleaned = self.slug().to_string();
		const RESERVED: &[&str] = &[
			"CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
			"COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
			"CONIN$", "CONOUT$"
		];
		while RESERVED.contains(&cleaned.to_ascii_uppercase().as_str()) {
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
	pub fn brief_description(&self, all_albums: &[Album]) -> String {
		match self {
			Titlable::Album(album) => {
				let released = album.released.to_display();
				let track_count = album.songs.iter().filter(|s| !s.bonus).count();
				format!(
					"Album released {}, {} tracks, {}",
					released,
					track_count,
					album.duration.display()
				)
			}
			Titlable::Song(song) => {
				let released = song.released.to_display();
				match song.parent_album_indices {
					None => {
						if song.event {
							format!(
								"DJ set for {} on {}, {}",
								song.title,
								released,
								song.duration.display()
							)
						} else if !song.artist.is_empty() {
							format!("Remix released {}, {}", released, song.duration.display())
						} else {
							format!("Mix released {}, {}", released, song.duration.display())
						}
					}
					Some((parent_album_index, track_number)) => {
						if all_albums[parent_album_index].single {
							format!("Song released {}, {}", released, song.duration.display())
						} else {
							format!(
								"Track {} on {}, released {}, {}",
								track_number + 1,
								all_albums[parent_album_index].title,
								released,
								song.duration.display()
							)
						}
					}
				}
			}
		}
	}
}
