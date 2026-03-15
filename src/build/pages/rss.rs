use crate::build::xml::XmlNode;
use crate::fileops;
use crate::globals;
use crate::media::artwork::Artwork;
use crate::media::{album::Album, assist::Assist, song::Song, titlable::Titlable};
use crate::types::date::Date;

pub fn make_rss(all_albums: &[Album], all_remixes: &[Song], all_assists: &[Assist]) {
	let mut channel = XmlNode::new("channel")
		.with_child(XmlNode::new("title").with_text("Astro's discography"))
		.with_child(XmlNode::new("link").with_text("https://music.astronomy487.com"))
		.with_child(
			XmlNode::new("description").with_text("All music released by Astro (astronomy487)")
		)
		.with_child(XmlNode::new("lastBuildDate").with_text(Date::now_rfc822()))
		.with_child(
			XmlNode::new("category")
				.with_attribute("domain", "topic")
				.with_text("music")
		)
		.with_child(XmlNode::new("language").with_text("en-US"))
		.with_child(XmlNode::new("ttl").with_text("1440"))
		.with_child(
			XmlNode::new("copyright")
				.with_text(format!("© {} Astro \"astronomy487\"", Date::today().year))
		)
		.with_child(XmlNode::new("generator").with_text("distri"))
		.with_child(XmlNode::new("docs").with_text("https://www.rssboard.org/rss-specification"))
		.with_child(
			XmlNode::new("image")
				.with_child(
					XmlNode::new("url").with_text("https://music.astronomy487.com/squarelogo.png")
				)
				.with_child(XmlNode::new("title").with_text("Astro's logo"))
				.with_child(XmlNode::new("link").with_text("https://music.astronomy487.com"))
		)
		.with_child(
			XmlNode::new("atom:link")
				.with_attribute("href", "https://music.astronomy487.com/rss.xml")
				.with_attribute("rel", "self")
				.with_attribute("type", "application/rss+xml")
		);
	let mut rss_entries_to_make: Vec<(XmlNode, &Date)> = Vec::new();
	for album in all_albums {
		rss_entries_to_make.push((
			rss_item_for_titlable(Titlable::Album(album)),
			&album.released
		));
		for song in &album.songs {
			if song.released_as_single && song.released <= album.released {
				rss_entries_to_make
					.push((rss_item_for_titlable(Titlable::Song(song)), &song.released));
			}
		}
	}
	for remix in all_remixes {
		rss_entries_to_make.push((
			rss_item_for_titlable(Titlable::Song(remix)),
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

	let rss = XmlNode::new("rss")
		.with_attribute("version", "2.0")
		.with_attribute("xmlns:atom", "http://www.w3.org/2005/Atom")
		.with_attribute("xmlns:media", "http://search.yahoo.com/mrss/")
		.with_child(channel);

	let mut file = std::fs::File::create(
		globals::filezone()
			.join("music.astronomy487.com")
			.join("rss")
			.with_extension("xml")
	)
	.expect("Couldn't write to rss.xml");
	let _ = std::io::Write::write_all(&mut file, format!("{}", rss).as_bytes());
}

fn rss_item_for_titlable(titlable: Titlable) -> XmlNode {
	let image_name_without_slash: &str = match titlable.artwork() {
		Some(artwork) => &artwork.name_without_slash,
		None => &Artwork::fallback().name_without_slash
	};
	let mut item = XmlNode::new("item")
		.with_child(XmlNode::new("title").with_text(titlable.format_title()))
		.with_child(XmlNode::new("link").with_text(format!(
			"https://music.astronomy487.com/{}/",
			titlable.slug()
		)))
		.with_child(
			XmlNode::new("guid")
				.with_attribute("isPermaLink", "true")
				.with_text(format!(
					"https://music.astronomy487.com/{}/",
					titlable.slug()
				))
		)
		.with_child(
			XmlNode::new("category")
				.with_attribute("domain", "topic")
				.with_text("music")
		)
		.with_child(
			XmlNode::new("category")
				.with_attribute("domain", "format")
				.with_text(match titlable {
					Titlable::Album(_) => "album",
					Titlable::Song(_) => "song"
				})
		)
		.with_child(
			XmlNode::new("category")
				.with_attribute("domain", "genre")
				.with_text(titlable.genre().to_string().to_lowercase())
		)
		.with_child(
			XmlNode::new("source")
				.with_attribute("url", "https://music.astronomy487.com/rss.xml")
				.with_text("Astro's discography")
		)
		.with_child(
			XmlNode::new("enclosure")
				.with_attribute(
					"url",
					format!(
						"https://music.astronomy487.com/artwork/{}.jpg",
						image_name_without_slash
					)
				)
				.with_attribute("length", {
					let path = globals::filezone()
						.join("music.astronomy487.com")
						.join("artwork")
						.join(image_name_without_slash)
						.with_extension("jpg");
					format!(
						"{}",
						crate::fileops::filesize(&path).unwrap_or_else(|| panic!(
							"RSS could not find the promised artwork \"{}.jpg\" in music.astronomy487.com directory",
							image_name_without_slash
						))
					)
				})
				.with_attribute("type", "image/jpeg")
		)
		.with_child(XmlNode::new("pubDate").with_text(titlable.released().to_rfc822()))
		.with_child(XmlNode::new("media:title").with_text(titlable.format_title()))
		.with_child(XmlNode::new("media:thumbnail").with_text(format!(
			"https://music.astronomy487.com/{}.jpg",
			image_name_without_slash
		)))
		.with_child(XmlNode::new("media:credit").with_text("Astro"))
		.with_child(XmlNode::new("media:keywords").with_text("electronic music"));
	let download_filename = titlable.slug().to_owned()
		+ match &titlable {
			Titlable::Album(_) => "zip",
			Titlable::Song(_) => "mp3"
		};
	if let Some(download_file_size) = fileops::filesize(
		&globals::filezone()
			.join("audio.astronomy487.com")
			.join("mp3")
			.join(&download_filename)
	) {
		item.add_child(
			XmlNode::new("media:content")
				.with_attribute(
					"url",
					format!("https://audio.astronomy487.com/mp3/{}", download_filename)
				)
				.with_attribute("fileSize", download_file_size.to_string())
				.with_attribute(
					"type",
					match &titlable {
						Titlable::Album(_) => "application/zip",
						Titlable::Song(_) => "audio/mpeg"
					}
				)
				.with_attribute("bitrate", "320")
				//.with_attribute("samplingrate", "44100") // found out some of my flacs are 48khz ....
				.with_attribute("channels", "2")
				.with_attribute(
					"duration",
					match &titlable {
						Titlable::Album(album) => album.duration.seconds(),
						Titlable::Song(song) => song.duration.seconds()
					}
					.to_string()
				)
		)
	}
	if let Titlable::Album(album) = titlable
		&& let Some(description) = &album.about
	{
		item.add_child(XmlNode::new("description").with_text(description.join(" ")));
		item.add_child(XmlNode::new("media:description").with_text(description.join(" ")));
	}
	item
}
fn rss_item_for_assist(assist: &Assist) -> XmlNode {
	XmlNode::new("item")
		.with_child(
			XmlNode::new("title").with_text(format!("{} ({})", assist.titlable, assist.role))
		)
		.with_child(XmlNode::new("link").with_text(&assist.url))
		/* .with_child(
			XmlNode::new("guid")
				.with_attribute("isPermaLink", "true")
				.with_text(&assist.url)
		) */
		.with_child(
			XmlNode::new("category")
				.with_attribute("domain", "topic")
				.with_text("music")
		)
		/* .with_child(
			XmlNode::new("category")
				.with_attribute("domain", "format")
				.with_text("album")
		) */
		.with_child(
			XmlNode::new("source")
				.with_attribute("url", "https://music.astronomy487.com/rss.xml")
				.with_text("Astro's discography")
		)
		.with_child(
			XmlNode::new("enclosure")
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
		.with_child(XmlNode::new("pubDate").with_text(assist.released.to_rfc822()))
		.with_child(XmlNode::new("media:title").with_text(&assist.titlable))
		.with_child(XmlNode::new("media:thumbnail").with_attribute("url", &assist.artwork))
		/* .with_child(
			XmlNode::new("media:credit")
				.with_text("Astro")
		) */
		.with_child(XmlNode::new("media:keywords").with_text("electronic music"))
}
