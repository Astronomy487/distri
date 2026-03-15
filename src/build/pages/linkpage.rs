// Responsible for creating link pages for Titlables at music.astronomy487.com/slug

use crate::build::{icons, pages::lyricpage, smartquotes, xml::XmlNode};
use crate::fileops;
use crate::globals;
use crate::media::{
	album::Album, audiocodec::AudioCodec, lyric::Lyrics, song::Song, titlable::Titlable
};
use crate::types::urlset::UrlSet;

pub fn make_link_page(
	titlable: &Titlable, all_albums: &[Album], everything_should_be_encoded: bool
) {
	// only executes once for every slug
	// (singles like rosalie only execute on Album, not on their songs)
	// (bonus tracks are not given link pages)

	// 0. Gather information
	let slug = titlable.slug();
	let canonical_url = format!("https://music.astronomy487.com/{}/", slug);
	let palette = titlable.palette();
	let released = titlable.released();
	let duration = titlable.duration();
	let description = titlable.brief_description(all_albums);
	let format_title = titlable.format_title();
	let format_title_short = titlable.format_title_short();
	let open_graph_artwork = match titlable.artwork() {
		Some(artwork) => &format!("../artwork/{}.jpg", artwork.name_without_slash),
		None => "../squarelogo.png"
	};
	let url_set = match titlable {
		Titlable::Album(album) => &album.url,
		Titlable::Song(song) => &UrlSet::combine(
			Some(&song.url),
			song.parent_album_indices
				.map(|(parent_album_index, _)| &all_albums[parent_album_index].url)
		)
	};

	// 1. Make the directory
	let destination_folder = globals::filezone()
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
	let lyrics_to_provide: Option<&Lyrics> = if titlable.unreleased() {
		None
	} else {
		match titlable {
			Titlable::Song(song) => song.lyrics.as_ref(),
			Titlable::Album(album) if album.single => album.songs[0].lyrics.as_ref(),
			Titlable::Album(_) => None
		}
	};

	if let Some(lxs) = lyrics_to_provide {
		let song: &Song = match titlable {
			Titlable::Album(album) if album.single => &album.songs[0],
			Titlable::Album(_) => panic!(
				"Trying to provide a link page to a non-single album; this shouldn't happen at runtime"
			),
			Titlable::Song(song) => song
		};
		lyricpage::make_lyric_page(song, lxs);
	}

	// 3. Make the url redirects
	/* for (_, redirect_url, platform_slug, _) in url_set.entries() {
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
	} */

	// 4. Make the big file
	let mut head = XmlNode::new("head")
		.with_child(XmlNode::new("meta").with_attribute("charset", "utf-8"))
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "icon")
				.with_attribute("href", "../favicon.ico")
				.with_attribute("type", "image/ico")
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "canonical")
				.with_attribute("href", &canonical_url)
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "stylesheet")
				.with_attribute("href", "../linkpage-style.css")
		)
		.with_child(XmlNode::new("title").with_text(&format_title))
		.with_child(XmlNode::new("style").with_text(palette.style_tag()))
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "theme-color")
				.with_attribute("content", palette.html_theme_color())
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "description")
				.with_attribute("content", &description)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "keywords")
				.with_attribute("content", globals::OG_KEYWORDS)
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
				.with_attribute("property", "og:site_name")
				.with_attribute("content", globals::OG_SITE_NAME)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "og:title")
				.with_attribute("content", &format_title)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "og:description")
				.with_attribute("content", description)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "og:image")
				.with_attribute("content", open_graph_artwork)
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "apple-touch-icon")
				.with_attribute("href", open_graph_artwork)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "og:url")
				.with_attribute("content", canonical_url)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "music:musician")
				.with_attribute("content", "https://www.astronomy487.com/")
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "music:release_date")
				.with_attribute("content", released.to_iso8601())
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "music:duration")
				.with_attribute("content", duration.seconds().to_string())
		);
	match &titlable {
		Titlable::Album(album) => {
			head.add_child(
				XmlNode::new("meta")
					.with_attribute("property", "og:type")
					.with_attribute("content", "music.album")
			);
			for (i, song) in album.songs.iter().enumerate() {
				if !song.bonus {
					head.add_child(
						XmlNode::new("meta")
							.with_attribute("property", "music:song")
							.with_attribute(
								"content",
								format!("https://music.astronomy487.com/{}/", song.slug)
							)
					);
					head.add_child(
						XmlNode::new("meta")
							.with_attribute("property", "music:song:track")
							.with_attribute("content", (i + 1).to_string())
					);
				}
			}
		}
		Titlable::Song(song) => {
			head.add_child(
				XmlNode::new("meta")
					.with_attribute("property", "og:type")
					.with_attribute("content", "music.song")
			);
			if let Some((parent_album_index, track_number)) = song.parent_album_indices {
				head.add_child(
					XmlNode::new("meta")
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
					XmlNode::new("meta")
						.with_attribute("property", "music:album:track")
						.with_attribute("content", (track_number + 1).to_string())
				);
			}
		}
	}
	let mut body = XmlNode::new("body")
		.maybe_with_attribute("class", palette.palette_mode_as_css_class_name())
		.maybe_with_attribute(
			"style",
			if titlable.artwork().is_none() {
				Some("margin-top:16rem")
			} else {
				None
			}
		)
		.maybe_with_child(titlable.artwork().map(|artwork| {
			XmlNode::new("img")
				.with_attribute(
					"src",
					format!("../artwork/{}.jpg", artwork.name_without_slash)
				)
				.with_attribute("alt", smartquotes::smart_quotes(&artwork.caption))
		}))
		.with_child(XmlNode::new("h1").with_text(smartquotes::smart_quotes(&format_title_short)))
		.with_child(
			XmlNode::new("table")
				.with_attribute("class", "metadata")
				.with_child(
					XmlNode::new("tr")
						.with_child(XmlNode::new("td").with_text(format!(
							"{}{}",
							if titlable.unreleased() { "Coming " } else { "" },
							released.to_display()
						)))
						.maybe_with_child(match titlable {
							Titlable::Album(album) if !album.single => {
								Some(XmlNode::new("td").with_text({
									let amount = album.songs.iter().filter(|s| !s.bonus).count();
									format!(
										"{} track{}",
										amount,
										if amount == 1 { "" } else { "s" }
									)
								}))
							}
							Titlable::Album(_) => None,
							Titlable::Song(_) => None
						})
						/* .maybe_with_child(match titlable {
							Titlable::Song(song) => {
								if let Some((parent_album_index, track_number)) = song.parent_album_indices {
									Some(
										XmlNode::new("td")
											.with_text(format!("Track {} of ", track_number + 1))
											.with_child(
												XmlNode::new("a")
													.with_attribute("href", format!("../{}/", &all_albums[parent_album_index].slug))
													.with_text(smartquotes::smart_quotes(&all_albums[parent_album_index].title))
											)
									)
								} else {
									None
								}
							},
							_ => None
						}) */
						.maybe_with_child(if titlable.unreleased() {
							None
						} else {
							Some(XmlNode::new("td").with_text(duration.display()))
						})
				)
		)
		.with_child({
			let mut table = XmlNode::new("table").with_attribute("class", "streamlinks");
			for chunk in url_set.entries().chunks(2).take(4) {
				let mut tr = XmlNode::new("tr");
				for (key, value, short, with_color) in chunk {
					tr.add_child(
						XmlNode::new("td").with_child(
							XmlNode::new("a")
								.with_attribute(
									"class",
									format!("{} streamlink with{}", short, with_color)
								)
								.with_attribute("href", value.to_string())
								.with_child(
									XmlNode::new("img")
										.with_attribute("src", format!("../icons/{}.svg", short))
										.with_attribute("alt", "")
										.with_attribute("aria-hidden", "true")
										.with_attribute("loading", "lazy")
								)
								.with_child(XmlNode::new("span").with_text(key.to_string()))
						)
					)
				}
				if chunk.len() == 1 {
					tr.add_child(XmlNode::new("td"))
				}
				table.add_child(tr);
			}
			table
		});

	let maybe_details = if let Some(about) = titlable.about() {
		let mut details = XmlNode::new("details").with_child(XmlNode::new("summary").with_text("See more"));
		for paragraph in about {
			details.add_child(XmlNode::new("p").with_text(smartquotes::smart_quotes(paragraph)));
		}
		Some(details)
	} else {
		None
	};

	let mut links_to_provide: Vec<(String, String, Option<String>)> = Vec::new(); // (text, url, download_tag). not used for anything external
	if !titlable.unreleased() {
		let offer_zip = match titlable {
			Titlable::Album(_) => true,
			Titlable::Song(_) => false
		};
		for codec in [AudioCodec::Mp3, AudioCodec::Flac] {
			let download_size = titlable.audio_download_size(&codec).unwrap_or_else(|| {
				assert!(
					!everything_should_be_encoded,
					"{} has no flac size",
					format_title
				);
				0
			});
			if true { // TODO change this if i want to show off website without actually making audio
				links_to_provide.push((
					format!(
						"{}{} {}",
						codec.ext(),
						if offer_zip { " zip" } else { "" },
						fileops::format_file_size(download_size)
					),
					titlable.audio_download_url(&codec),
					Some(format!(
						"{}.{}",
						titlable.format_title(),
						match titlable {
							Titlable::Album(_) => "zip",
							Titlable::Song(_) => codec.ext()
						}
					))
				));
			}
		}
		if lyrics_to_provide.is_some() {
			links_to_provide.push((String::from("Lyrics"), String::from("lyrics/"), None));
		}
	}
	let maybe_link_set = if !links_to_provide.is_empty() {
		let mut link_set = XmlNode::new("link-set");
		for (link_text, link_url, download_tag) in links_to_provide {
			link_set.add_child(
				XmlNode::new("a")
					.maybe_with_child(if download_tag.is_some() {
						// inline svg so that css can recolor it. :P so stupid
						Some(icons::inline_download_icon_svg())
					} else {
						None
					})
					.with_child(XmlNode::new("span").with_text(link_text))
					.with_attribute("href", link_url)
					.maybe_with_attribute("download", download_tag)
			);
		}
		Some(link_set)
	} else {
		None
	};
	
	// kinda messy but it's chill
	// below the streaming links is maybe 1. item description; forces us to have a detail, and 2. link sets for links internal to astronomy487.com
	match (maybe_details, maybe_link_set) {
		(None, None) => {},
		(Some(details), None) => {
			body.add_child(details);
		},
		(None, Some(link_set)) => {
			body.add_child(link_set);
		},
		(Some(mut details), Some(link_set)) => {
			details.add_child(link_set);
			body.add_child(details);
		},
	}

	/* let mut metadata_to_show: Vec<(&'static str, String)> = Vec::new();
	metadata_to_show.push(("Type", match titlable {
		Titlable::Album(album) if album.single => "Single",
		Titlable::Album(_) => "Album",
		Titlable::Song(song) if song.parent_album_indices.is_some() => "Song",
		Titlable::Song(_) => "Remix"
	}.to_string()));
	metadata_to_show.push(("Released", titlable.released().to_display()));
	metadata_to_show.push(("Artist", titlable.artist().to_string()));
	metadata_to_show.push(("Title", titlable.title().to_string()));
	if let Titlable::Song(song) = titlable && let Some((parent_album_index, _)) = song.parent_album_indices {
		metadata_to_show.push(("On album", all_albums[parent_album_index].title.clone()))
	}
	if let Titlable::Song(song) = titlable && let Some(isrc) = &song.isrc {
		metadata_to_show.push(("ISRC", format!("{}", isrc)));
	}
	if let Titlable::Album(album) = titlable && album.single && let Some(isrc) = &album.songs[0].isrc {
		metadata_to_show.push(("ISRC", format!("{}", isrc)));
	}
	if let Titlable::Album(album) = titlable && let Some(upc) = &album.upc {
		metadata_to_show.push(("UPC", format!("{}", upc)));
	}
	metadata_to_show.push(("Genre", titlable.genre().to_string().to_string()));
	metadata_to_show.push(("Duration", titlable.duration().display()));
	metadata_to_show.push(("Unique identifier", titlable.slug().to_string()));
	if !metadata_to_show.is_empty() {
		let mut table = XmlNode::new("table")
			.with_attribute("class", "bottom-metadata");
		for (field, content) in metadata_to_show {
			table.add_child(
				XmlNode::new("tr")
					.with_child(XmlNode::new("th").with_text(field))
					.with_child(XmlNode::new("td").with_text(smartquotes::smart_quotes(&content)))
			);
		}
		details.add_child(table);
	} */

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
