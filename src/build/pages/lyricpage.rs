use crate::build::{icons, smartquotes, xml::XmlNode};
use crate::globals;
use crate::media::song::Song;

use crate::media::audiocodec::AudioCodec;
use crate::media::lyric;
use lyric::Lyrics;

pub fn make_lyric_page(song: &Song, lyrics: &Lyrics) {
	let destination_folder = globals::filezone()
		.join("music.astronomy487.com")
		.join(&song.slug)
		.join("lyrics");
	std::fs::create_dir(&destination_folder).unwrap_or_else(|_| {
		panic!(
			"Couldn't create directory {} in music.astronomy487.com",
			destination_folder.display()
		)
	});

	for codec in lyric::ALL_TEXT_CODECS {
		let lyrics_location = destination_folder
			.join("lyrics")
			.with_extension(codec.ext());
		let mut file = std::fs::File::create(&lyrics_location)
			.unwrap_or_else(|_| panic!("Couldn't create file {}", lyrics_location.display()));
		let _ = std::io::Write::write(&mut file, lyrics.as_filetype(codec).as_bytes())
			.unwrap_or_else(|_| panic!("Couldn't write to file {}", lyrics_location.display()));
	}

	let webpage_title = format!("{} (Lyrics)", song.format_title());
	let webpage_description = format!("Lyrics for {}", song.format_title());
	let webpage_url = format!("https://music.astronomy487.com/{}/lyrics/", song.slug);

	let open_graph_artwork = match &song.artwork {
		Some(artwork) => &format!("../../artwork/{}.jpg", artwork.name_without_slash),
		None => "../../squarelogo.png"
	};

	let body = XmlNode::new("body")
		.with_child(
			XmlNode::new("header")
				/* .with_child(
					XmlNode::new("h3")
						.with_child(
							XmlNode::new("a")
								.with_attribute("href", "../")
								.with_text("Go back")
						)
				) */
				.with_child(XmlNode::new("h3").with_text("Lyrics"))
				.with_child(
					XmlNode::new("h2")
						.with_text(smartquotes::smart_quotes(&song.format_title_short()))
				)
				.with_child({
					let mut the_downloads = XmlNode::new("the-downloads");
					for codec in lyric::ALL_TEXT_CODECS {
						the_downloads.add_child(
							XmlNode::new("a")
								.with_attribute("href", format!("lyrics.{}", codec.ext()))
								.with_attribute(
									"download",
									format!("{} (Lyrics).{}", song.public_filename(), codec.ext())
								)
								.with_child(icons::inline_download_icon_svg())
								.with_child(XmlNode::new("span").with_text(
									// codec.ext().to_uppercase()
									//format!(".{}", codec.ext())
									codec.description()
								))
						)
					}
					the_downloads
				})
				.with_child({
					let all_vocalist_sets = lyrics.all_vocalist_sets();
					if all_vocalist_sets.len() > 1 {
						// let vocalists = lyrics.all_vocalists();
						XmlNode::new("form")
							.with_child(
								XmlNode::new("input")
									.with_attribute("type", "checkbox")
									.with_attribute("id", "show-vocalists")
									.with_attribute("name", "show-vocalists")
							)
							.with_child(
								XmlNode::new("label")
									.with_attribute("for", "show-vocalists")
									.with_text(
										// format!("Show {} vocalists", vocalists.len())
										"Show vocalists"
									)
							)
					} else {
						XmlNode::new("form")
							.with_text(format!("Vocals by {}", all_vocalist_sets[0]))
					}
				})
		)
		.with_child(lyrics.lyric_page_xml())
		.with_child(XmlNode::new("script").with_text_unescaped(format!(
			"const source = \"{}\";",
			song.audio_download_url(&AudioCodec::Mp3)
		)))
		.with_child(
			XmlNode::new("script")
				.with_attribute("src", "../../lyricpage-script.js")
				.with_text("")
		);
	let head = XmlNode::new("head")
		.with_child(XmlNode::new("meta").with_attribute("charset", "UTF-8"))
		.with_child(XmlNode::new("title").with_text(&webpage_title))
		.with_child(XmlNode::new("style").with_text(song.palette.style_tag()))
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "icon")
				.with_attribute("href", "../../favicon.ico")
				.with_attribute("type", "image/png")
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "canonical")
				.with_attribute("href", &webpage_url)
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "stylesheet")
				.with_attribute("href", "../../lyricpage-style.css")
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "theme-color")
				.with_attribute("content", song.palette.html_theme_color())
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "description")
				.with_attribute("content", &webpage_description)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "keywords")
				.with_attribute("content", format!("lyrics, {}", globals::OG_KEYWORDS))
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "author")
				.with_attribute("content", globals::OG_AUTHOR)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "robots")
				.with_attribute("content", globals::OG_ROBOTS)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "og:site_name")
				.with_attribute("content", globals::OG_SITE_NAME)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "og:title")
				.with_attribute("content", &webpage_title)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "og:description")
				.with_attribute("content", &webpage_description)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "og:image")
				.with_attribute("content", open_graph_artwork)
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "apple-touch-icon")
				.with_attribute("href", open_graph_artwork)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "og:url")
				.with_attribute("content", &webpage_url)
		);

	let html = XmlNode::new("html")
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
