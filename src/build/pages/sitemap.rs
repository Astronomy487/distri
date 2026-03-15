use crate::globals;
use crate::media::{album::Album, assist::Assist, song::Song};

pub fn make_sitemap(all_albums: &[Album], all_remixes: &[Song], _all_assists: &[Assist]) {
	let mut list_of_urls: Vec<String> = Vec::new();
	list_of_urls.push("https://music.astronomy487.com/".to_string());

	for album in all_albums {
		list_of_urls.push(format!("https://music.astronomy487.com/{}/", album.slug));
		// list_of_urls.push(format!("https://audio.astronomy487.com/mp3/{}.zip", album.slug));
		// list_of_urls.push(format!("https://audio.astronomy487.com/flac/{}.zip", album.slug));
		for song in &album.songs {
			if !song.bonus {
				if song.slug != album.slug {
					list_of_urls.push(format!("https://music.astronomy487.com/{}/", song.slug));
				}
				if song.lyrics.is_some() {
					list_of_urls.push(format!(
						"https://music.astronomy487.com/{}/lyrics/",
						song.slug
					));
					/* for text_codec in crate::media::lyric::ALL_TEXT_CODECS {
						list_of_urls.push(format!(
							"https://music.astronomy487.com/{}/lyrics/lyrics.{}",
							song.slug,
							text_codec.ext()
						));
					} */
				}
				// list_of_urls.push(format!("https://audio.astronomy487.com/mp3/{}.mp3", song.slug));
				// list_of_urls.push(format!("https://audio.astronomy487.com/flac/{}.flac", song.slug));
			}
		}
	}
	for remix in all_remixes {
		list_of_urls.push(format!("https://music.astronomy487.com/{}/", remix.slug));
		if remix.lyrics.is_some() {
			list_of_urls.push(format!(
				"https://music.astronomy487.com/{}/lyrics/",
				remix.slug
			));
			/* for text_codec in crate::media::lyric::ALL_TEXT_CODECS {
				list_of_urls.push(format!(
					"https://music.astronomy487.com/{}/lyrics/lyrics.{}",
					remix.slug,
					text_codec.ext()
				));
			} */
		}
		// list_of_urls.push(format!("https://audio.astronomy487.com/mp3/{}.mp3", remix.slug));
		// list_of_urls.push(format!("https://audio.astronomy487.com/flac/{}.flac", remix.slug));
	}

	list_of_urls.sort();

	let mut file = std::fs::File::create(
		globals::filezone()
			.join("music.astronomy487.com")
			.join("sitemap")
			.with_extension("txt")
	)
	.expect("Couldn't write to sitemap.txt");
	let _ = std::io::Write::write_all(&mut file, list_of_urls.join("\n").as_bytes());
}
