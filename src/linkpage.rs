// Responsible for creating link pages for Titlables at music.astronomy487.com/slug

use crate::fileops;
use crate::globals;
use crate::lyric;
use crate::musicdata;
use crate::smartquotes;
use crate::url;

use crate::xml;

pub fn make_link_page(
	titlable: &musicdata::Titlable, all_albums: &[musicdata::Album],
	everything_should_be_encoded: bool
) {
	// only executes once for every slug
	// (singles like rosalie only execute on Album, not on their songs)
	// (bonus tracks are not given link pages)

	// 0. Gather information
	let slug = titlable.slug();
	let canonical_url = format!("https://music.astronomy487.com/{}/", slug);
	let palette = titlable.palette();
	let released = titlable.released();
	let length = titlable.length();
	let description = titlable.description(all_albums);
	let format_title = titlable.format_title();
	let format_title_short = titlable.format_title_short();
	let open_graph_artwork = match titlable.artwork() {
		Some(artwork_name) => &format!("../artwork/{}.jpg", artwork_name),
		None => "../squarelogo.png"
	};
	let display_artwork = titlable
		.artwork()
		.map(|artwork_name| format!("../artwork/{}.jpg", artwork_name));
	let url_set = match titlable {
		musicdata::Titlable::Album(album) => &album.url,
		musicdata::Titlable::Song(song) => &url::UrlSet::combine(
			Some(&song.url),
			song.parent_album_indices
				.map(|(parent_album_index, _)| &all_albums[parent_album_index].url)
		)
	};

	// 1. Make the directory
	let destination_folder = std::path::Path::new(globals::filezone())
		.join("music.astronomy487.com")
		.join(slug);
	assert!(
		!destination_folder.exists(),
		"Directory {} already exists in music.astronomy487.com",
		slug
	);
	std::fs::create_dir(&destination_folder).unwrap_or_else(|_| {
		panic!(
			"Couldn't create directory {} in music.astronomy487.com",
			destination_folder.display()
		)
	});

	// 2. Make lyric files
	if !titlable.unreleased()
		&& let musicdata::Titlable::Song(song) = titlable
		&& let Some(lxs) = &song.lyrics
	{
		for codec in [
			lyric::TextCodec::Txt,
			lyric::TextCodec::Lrc,
			lyric::TextCodec::Srt
		] {
			let lyrics_location = destination_folder
				.join("lyrics")
				.with_extension(codec.ext());
			let mut file = std::fs::File::create(&lyrics_location)
				.unwrap_or_else(|_| panic!("Couldn't create file {}", lyrics_location.display()));
			let _ = std::io::Write::write(&mut file, lxs.as_filetype(codec).as_bytes())
				.unwrap_or_else(|_| panic!("Couldn't write to file {}", lyrics_location.display()));
		}
	}

	// 3. Make the url redirects
	for (_, redirect_url, platform_slug, _) in url_set.entries() {
		let mini_directory = destination_folder.join(platform_slug);
		std::fs::create_dir(&mini_directory)
			.unwrap_or_else(|_| panic!("Couldn't create directory {}", mini_directory.display()));
		let redirect_html_location = mini_directory.join("index").with_extension("html");
		let mut redirect_file =
			std::fs::File::create(&redirect_html_location).unwrap_or_else(|_| {
				panic!("Couldn't create file {}", redirect_html_location.display())
			});
		let content = format!(
			"<!DOCTYPE html><meta http-equiv=refresh content=\"0;url={}\">",
			redirect_url
		);
		let _ =
			std::io::Write::write(&mut redirect_file, content.as_bytes()).unwrap_or_else(|_| {
				panic!(
					"Couldn't write to file {}",
					redirect_html_location.display()
				)
			});
	}

	// 4. Make the big file
	let mut head = xml::XmlNode::new("head")
		.with_child(xml::XmlNode::new("meta").with_attribute("charset", "utf-8"))
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "icon")
				.with_attribute("href", "../favicon.ico")
				.with_attribute("type", "image/ico")
		)
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "canonical")
				.with_attribute("href", &canonical_url)
		)
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "stylesheet")
				.with_attribute("href", "../style.css")
		)
		.with_child(xml::XmlNode::new("title").with_text(&format_title))
		.with_child(xml::XmlNode::new("style").with_text(palette.style_tag()))
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "theme-color")
				.with_attribute("content", palette.html_theme_color())
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "description")
				.with_attribute("content", &description)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "keywords")
				.with_attribute(
					"content",
					"electronic, dance, music, astro, artist, indie, edm"
				)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "author")
				.with_attribute("content", "Astro, astronomy487")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "robots")
				.with_attribute("content", "index, follow")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:site_name")
				.with_attribute("content", "astronomy487.com")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:title")
				.with_attribute("content", &format_title)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:description")
				.with_attribute("content", description)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:image")
				.with_attribute("content", open_graph_artwork)
		)
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "apple-touch-icon")
				.with_attribute("href", open_graph_artwork)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:url")
				.with_attribute("content", canonical_url)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "music:musician")
				.with_attribute("content", "https://www.astronomy487.com/")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "music:release_date")
				.with_attribute("content", released.to_iso8601())
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "music:duration")
				.with_attribute("content", length.to_string())
		);
	match &titlable {
		musicdata::Titlable::Album(album) => {
			head.add_child(
				xml::XmlNode::new("meta")
					.with_attribute("property", "og:type")
					.with_attribute("content", "music.album")
			);
			for (i, song) in album.songs.iter().enumerate() {
				if !song.bonus {
					head.add_child(
						xml::XmlNode::new("meta")
							.with_attribute("property", "music:song")
							.with_attribute(
								"content",
								format!("https://music.astronomy487.com/{}/", song.slug)
							)
					);
					head.add_child(
						xml::XmlNode::new("meta")
							.with_attribute("property", "music:song:track")
							.with_attribute("content", (i + 1).to_string())
					);
				}
			}
		}
		musicdata::Titlable::Song(song) => {
			head.add_child(
				xml::XmlNode::new("meta")
					.with_attribute("property", "og:type")
					.with_attribute("content", "music.song")
			);
			if let Some((parent_album_index, track_number)) = song.parent_album_indices {
				head.add_child(
					xml::XmlNode::new("meta")
						.with_attribute("property", "music:album")
						.with_attribute(
							"content",
							format!(
								"https://music.astronomy487.com/{}/",
								all_albums[parent_album_index].slug
							)
						)
				);
				head.add_child(
					xml::XmlNode::new("meta")
						.with_attribute("property", "music:album:track")
						.with_attribute("content", (track_number + 1).to_string())
				);
			}
		}
	}
	let mut body = xml::XmlNode::new("body")
		.maybe_with_attribute("class", palette.palette_mode_as_css_class_name())
		.maybe_with_attribute(
			"style",
			if display_artwork.is_none() {
				Some("margin-top:16rem")
			} else {
				None
			}
		)
		.maybe_with_child(display_artwork.map(|the_display_artwork| {
			xml::XmlNode::new("img")
				.with_attribute("src", the_display_artwork)
				.with_attribute("alt", &format_title_short)
		}))
		.with_child(
			xml::XmlNode::new("h1").with_text(smartquotes::smart_quotes(format_title_short))
		)
		.with_child(
			xml::XmlNode::new("table").with_child(
				xml::XmlNode::new("tr")
					.with_child(xml::XmlNode::new("td").with_text(format!(
						"{}{}",
						if titlable.unreleased() { "Coming " } else { "" },
						released.to_display()
					)))
					.maybe_with_child(match titlable {
						musicdata::Titlable::Album(album) => {
							Some(xml::XmlNode::new("td").with_text(format!(
								"{} tracks",
								album.songs.iter().filter(|s| !s.bonus).count()
							)))
						}
						musicdata::Titlable::Song(_) => None
					})
					.maybe_with_child(if titlable.unreleased() {
						None
					} else {
						Some(xml::XmlNode::new("td").with_text(musicdata::format_duration(length)))
					})
			)
		)
		.with_child({
			let mut table = xml::XmlNode::new("table");
			for chunk in url_set.entries().chunks(2) {
				let mut tr = xml::XmlNode::new("tr");
				for (key, value, short, with_color) in chunk {
					tr.add_child(
						xml::XmlNode::new("td").with_child(
							xml::XmlNode::new("a")
								.with_attribute("class", format!("{} streamlink {}", short, with_color))
								.with_attribute("href", value.to_string())
								.with_child(
									xml::XmlNode::new("img")
										.with_attribute("src", format!("../icons/{}.svg", short))
										.with_attribute("alt", "")
										.with_attribute("aria-hidden", "true")
								)
								.with_child(xml::XmlNode::new("span").with_text(key.to_string()))
						)
					)
				}
				if chunk.len() == 1 {
					tr.add_child(xml::XmlNode::new("td"))
				}
				table.add_child(tr);
			}
			table
		});
	if !titlable.unreleased() {
		// When finding filesizes, WARN if they aren't present. user better read output
		let mp3_size = titlable
			.audio_download_size(&musicdata::AudioCodec::Mp3)
			.unwrap_or_else(|| {
				assert!(
					!everything_should_be_encoded,
					"{} has no mp3 size",
					format_title
				);
				0
			});
		let flac_size = titlable
			.audio_download_size(&musicdata::AudioCodec::Flac)
			.unwrap_or_else(|| {
				assert!(
					!everything_should_be_encoded,
					"{} has no flac size",
					format_title
				);
				0
			});
		body.add_child(
			xml::XmlNode::new("table")
				.with_attribute("class", "bottomlinks")
				.with_child({
					let mut tr = xml::XmlNode::new("tr")
						.with_child(xml::XmlNode::new("td").with_text("Download"));
					for (codec, size) in [
						(musicdata::AudioCodec::Mp3, mp3_size),
						(musicdata::AudioCodec::Flac, flac_size)
					] {
						tr.add_child(
							xml::XmlNode::new("td").with_child(
								xml::XmlNode::new("a")
									.with_attribute("href", titlable.audio_download_url(&codec))
									.with_attribute("download", "")
									.with_text(
										codec.ext().to_string()
											+ match titlable {
												musicdata::Titlable::Album(_) => " zip, ",
												musicdata::Titlable::Song(_) => ", "
											} + &fileops::format_file_size(size)
									)
							)
						);
					}
					tr
				})
		)
	}
	if !titlable.unreleased()
		&& let musicdata::Titlable::Song(song) = titlable
		&& let Some(_) = &song.lyrics
	{
		body.add_child(
			xml::XmlNode::new("table")
				.with_attribute("class", "bottomlinks")
				.with_child({
					let mut tr = xml::XmlNode::new("tr")
						.with_child(xml::XmlNode::new("td").with_text("Lyrics"));
					for (codec, codec_description) in [
						(lyric::TextCodec::Txt, "Text"),
						(lyric::TextCodec::Lrc, "Synced") // (lyric::TextCodec::Srt, "Captions")
					] {
						tr.add_child(
							xml::XmlNode::new("td").with_child(
								xml::XmlNode::new("a")
									.with_attribute("href", format!("lyrics.{}", codec.ext()))
									.with_attribute(
										"download",
										format!("{} [Lyrics].{}", format_title, codec.ext())
									)
									.with_text(codec_description)
							)
						);
					}
					tr
				})
		)
	}
	let html = xml::XmlNode::new("html")
		.with_attribute("lang", "en")
		.with_child(head)
		.with_child(body);
	{
		let index_html_location = destination_folder.join("index").with_extension("html");
		let mut file = std::fs::File::create(&index_html_location)
			.unwrap_or_else(|_| panic!("Couldn't create file {}", index_html_location.display()));
		let _ = std::io::Write::write(&mut file, format!("<!DOCTYPE html>{}", html).as_bytes())
			.unwrap_or_else(|_| panic!("Couldn't write to file {}", index_html_location.display()));
	}
}
