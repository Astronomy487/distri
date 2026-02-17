use crate::date;
use crate::fileops;
use crate::globals;
use crate::musicdata;
use crate::xml;

pub fn make_rss(
	all_albums: &[musicdata::Album], all_remixes: &[musicdata::Song],
	all_assists: &[musicdata::Assist]
) {
	let mut channel = xml::XmlNode::new("channel")
		.with_child(xml::XmlNode::new("title").with_text("Astro's discography"))
		.with_child(xml::XmlNode::new("link").with_text("https://music.astronomy487.com"))
		.with_child(
			xml::XmlNode::new("description")
				.with_text("All music released by Astro (astronomy487)")
		)
		.with_child(xml::XmlNode::new("lastBuildDate").with_text(date::Date::now_rfc822()))
		.with_child(
			xml::XmlNode::new("category")
				.with_attribute("domain", "topic")
				.with_text("music")
		)
		.with_child(xml::XmlNode::new("language").with_text("en-US"))
		.with_child(xml::XmlNode::new("ttl").with_text("1440"))
		.with_child(
			xml::XmlNode::new("copyright")
				.with_text(format!("© {} Astro", date::Date::today().year))
		)
		.with_child(xml::XmlNode::new("generator").with_text("distri"))
		.with_child(
			xml::XmlNode::new("docs").with_text("https://www.rssboard.org/rss-specification")
		)
		.with_child(
			xml::XmlNode::new("image")
				.with_child(
					xml::XmlNode::new("url")
						.with_text("https://music.astronomy487.com/squarelogo.png")
				)
				.with_child(xml::XmlNode::new("title").with_text("Astro's logo"))
				.with_child(xml::XmlNode::new("link").with_text("https://music.astronomy487.com"))
		)
		.with_child(
			xml::XmlNode::new("atom:link")
				.with_attribute("href", "https://music.astronomy487.com/rss.xml")
				.with_attribute("rel", "self")
				.with_attribute("type", "application/rss+xml")
		);
	let mut rss_entries_to_make: Vec<(xml::XmlNode, &date::Date)> = Vec::new();
	for album in all_albums {
		rss_entries_to_make.push((
			rss_item_for_titlable(musicdata::Titlable::Album(album)),
			&album.released
		));
		for song in &album.songs {
			if song.released_as_single && song.released <= album.released {
				rss_entries_to_make.push((
					rss_item_for_titlable(musicdata::Titlable::Song(song)),
					&song.released
				));
			}
		}
	}
	for remix in all_remixes {
		rss_entries_to_make.push((
			rss_item_for_titlable(musicdata::Titlable::Song(remix)),
			&remix.released
		));
	}
	for assist in all_assists {
		rss_entries_to_make.push((rss_item_for_assist(assist), &assist.released));
	}
	rss_entries_to_make.reverse(); // ensure any date-ties are actually newest-to-oldest
	rss_entries_to_make.sort_by(|a, b| b.1.cmp(a.1)); // sort by release date
	let mut to_show = 35;
	for (rss_item, _date) in rss_entries_to_make {
		channel.add_child(rss_item);
		to_show -= 1;
		if to_show == 0 {
			break;
		}
	}

	let rss = xml::XmlNode::new("rss")
		.with_attribute("version", "2.0")
		.with_attribute("xmlns:atom", "http://www.w3.org/2005/Atom")
		.with_attribute("xmlns:media", "http://search.yahoo.com/mrss/")
		.with_child(channel);

	let mut file = std::fs::File::create(
		std::path::Path::new(globals::filezone())
			.join("music.astronomy487.com")
			.join("rss")
			.with_extension("xml")
	)
	.expect("Couldn't write to rss.xml");
	let _ = std::io::Write::write_all(&mut file, format!("{}", rss).as_bytes());
}

fn rss_item_for_titlable(titlable: musicdata::Titlable) -> xml::XmlNode {
	let image_name: String = match &titlable {
		musicdata::Titlable::Album(album) => album.slug.clone(),
		musicdata::Titlable::Song(song) => song
			.artwork
			.clone()
			.unwrap_or(globals::FALLBACK_ARTWORK_NAME.to_owned())
	};
	let mut item = xml::XmlNode::new("item")
		.with_child(xml::XmlNode::new("title").with_text(titlable.format_title()))
		.with_child(xml::XmlNode::new("link").with_text(format!(
			"https://music.astronomy487.com/{}/",
			titlable.slug()
		)))
		.with_child(
			xml::XmlNode::new("guid")
				.with_attribute("isPermaLink", "true")
				.with_text(format!(
					"https://music.astronomy487.com/{}/",
					titlable.slug()
				))
		)
		.with_child(
			xml::XmlNode::new("category")
				.with_attribute("domain", "topic")
				.with_text("music")
		)
		.with_child(
			xml::XmlNode::new("category")
				.with_attribute("domain", "format")
				.with_text(match titlable {
					musicdata::Titlable::Album(_) => "album",
					musicdata::Titlable::Song(_) => "song"
				})
		)
		.with_child(
			xml::XmlNode::new("category")
				.with_attribute("domain", "genre")
				.with_text(titlable.genre().to_string().to_lowercase())
		)
		.with_child(
			xml::XmlNode::new("source")
				.with_attribute("url", "https://music.astronomy487.com/rss.xml")
				.with_text("Astro's discography")
		)
		.with_child(
			xml::XmlNode::new("enclosure")
				.with_attribute(
					"url",
					format!("https://music.astronomy487.com/artwork/{}.jpg", image_name)
				)
				.with_attribute("length", {
					let path = std::path::Path::new(globals::filezone())
						.join("music.astronomy487.com")
						.join("artwork")
						.join(&image_name)
						.with_extension("jpg");
					format!(
						"{}",
						crate::fileops::filesize(&path).unwrap_or_else(|| panic!(
							"RSS could not find the promised artwork \"{}.jpg\" in music.astronomy487.com directory",
							image_name
						))
					)
				})
				.with_attribute("type", "image/jpeg")
		)
		.with_child(xml::XmlNode::new("pubDate").with_text(titlable.released().to_rfc822()))
		.with_child(xml::XmlNode::new("media:title").with_text(titlable.format_title()))
		.with_child(
			xml::XmlNode::new("media:thumbnail")
				.with_text(format!("https://music.astronomy487.com/{}.jpg", image_name))
		)
		.with_child(xml::XmlNode::new("media:credit").with_text("Astro"))
		.with_child(xml::XmlNode::new("media:keywords").with_text("electronic music"));
	let download_filename = titlable.slug().to_owned()
		+ match &titlable {
			musicdata::Titlable::Album(_) => "zip",
			musicdata::Titlable::Song(_) => "mp3"
		};
	if let Some(download_file_size) = fileops::filesize(
		&std::path::Path::new(globals::filezone())
			.join("audio.astronomy487.com")
			.join("mp3")
			.join(&download_filename)
	) {
		item.add_child(
			xml::XmlNode::new("media:content")
				.with_attribute(
					"url",
					format!("https://audio.astronomy487.com/mp3/{}", download_filename)
				)
				.with_attribute("fileSize", download_file_size.to_string())
				.with_attribute(
					"type",
					match &titlable {
						musicdata::Titlable::Album(_) => "application/zip",
						musicdata::Titlable::Song(_) => "audio/mpeg"
					}
				)
				.with_attribute("bitrate", "320")
				//.with_attribute("samplingrate", "44100") // found out some of my flacs are 48khz ....
				.with_attribute("channels", "2")
				.with_attribute(
					"duration",
					match &titlable {
						musicdata::Titlable::Album(album) => album.length,
						musicdata::Titlable::Song(song) => song.length
					}
					.to_string()
				)
		)
	}
	if let musicdata::Titlable::Album(album) = titlable
		&& let Some(description) = &album.about
	{
		item.add_child(xml::XmlNode::new("description").with_text(description.join(" ")));
		item.add_child(xml::XmlNode::new("media:description").with_text(description.join(" ")));
	}
	item
}
fn rss_item_for_assist(assist: &musicdata::Assist) -> xml::XmlNode {
	xml::XmlNode::new("item")
		.with_child(
			xml::XmlNode::new("title").with_text(format!("{} ({})", assist.titlable, assist.role))
		)
		.with_child(xml::XmlNode::new("link").with_text(&assist.url))
		/* .with_child(
			xml::XmlNode::new("guid")
				.with_attribute("isPermaLink", "true")
				.with_text(&assist.url)
		) */
		.with_child(
			xml::XmlNode::new("category")
				.with_attribute("domain", "topic")
				.with_text("music")
		)
		/* .with_child(
			xml::XmlNode::new("category")
				.with_attribute("domain", "format")
				.with_text("album")
		) */
		.with_child(
			xml::XmlNode::new("source")
				.with_attribute("url", "https://music.astronomy487.com/rss.xml")
				.with_text("Astro's discography")
		)
		.with_child(
			xml::XmlNode::new("enclosure")
				.with_attribute("url", &assist.artwork)
				.maybe_with_attribute("type", {
					if assist.artwork.ends_with(".png") {
						Some("image/png")
					} else if assist.artwork.ends_with(".jpg") {
						Some("image/jpeg")
					} else {
						globals::log_2(
							"Warning",
							format!(
								"Cannot determine mime type of assist artwork {}",
								assist.artwork
							),
							globals::ANSI_RED
						);
						None
					}
				})
		)
		.with_child(xml::XmlNode::new("pubDate").with_text(assist.released.to_rfc822()))
		.with_child(xml::XmlNode::new("media:title").with_text(&assist.titlable))
		.with_child(xml::XmlNode::new("media:thumbnail").with_attribute("url", &assist.artwork))
		/* .with_child(
			xml::XmlNode::new("media:credit")
				.with_text("Astro")
		) */
		.with_child(xml::XmlNode::new("media:keywords").with_text("electronic music"))
}
