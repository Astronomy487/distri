use crate::css;
use crate::date;
use crate::globals;
use crate::icons;
use crate::musicdata;
use crate::smartquotes;
use crate::xml;

const OG_DESCRIPTION: &str = "Astro (f.k.a. astronomy487) is an independent electronic dance music artist with a love for synthesizers and pop music.";
const HEADER_PARAGRAPH: &str = "I am Astro, an independent electronic dance music artist with a love for synthesizers and pop music. I create whichever sounds I find the most interesting.";
const HEADER_PARAGRAPH_EXTRA_DESKTOP: &str =
	"Below is an exhaustive catalog of every released song I’ve ever made or worked on.";

#[derive(PartialEq, Clone, Copy)]
enum EntryType {
	Album,
	Song,
	Assist
}

fn song_xml(
	song: &musicdata::Song, for_album: Option<(&musicdata::Album, &str)>, have_link: bool
) -> xml::XmlNode {
	xml::XmlNode::new("a-song")
		.with_child(
			xml::XmlNode::new("song-c")
				.with_child(
					xml::XmlNode::new("the-date")
						.with_text(song.released.to_display())
						.maybe_with_attribute("class",
							if song.released.birthday() {
								Some("bday")
							} else {
								None
							}
						)
				)
				.with_child(
					xml::XmlNode::new("song-title")
						.with_child({
							// boolean tells if it is an external link
							let maybe_link: Option<(String, bool)> = if have_link {
								Some((format!("{}/", song.slug), false))
							} else {
								song.url.try_to_get_at_least_one_link().map(|s| (s.to_owned(), true))
							};
							let is_hover = song.title == "Hover" && song.artist == "Astro";
							let mut span = xml::XmlNode::new("span");
							if let Some((link, is_external)) = maybe_link {
								span.add_child(
									xml::XmlNode::new("a")
										.with_attribute("href", link)
										.maybe_with_attribute("class", if is_external {Some("external")} else {None})
										.with_text(smartquotes::smart_quotes(&song.format_title_short()))
								);
							} else {
								span.add_text(smartquotes::smart_quotes(&song.format_title_short()));
							}
							if is_hover {
								// technically panics in the case Hover has no url, since the <span> will have direct text children by now. lol I LOVE AVOIDING TYPE-SAFE PATTERNS!!
								span.add_child(
									xml::XmlNode::new("span")
										.with_attribute("style", "color: var(--gray);")
										.with_text(" — Not everyone was there !")
								);
								span.add_child({
									assert!(icons::valid_icon("greenheart"));
									xml::XmlNode::new("img")
										.with_attribute("src", "icons/greenheart.svg")
										.with_attribute("style", "margin-left: 0.5rem; height: 1.25rem; margin-top: -0.125rem; user-select: none; vertical-align: middle;")
								});
							}
							span
						})
						.maybe_with_child(for_album.map(|(album, what_kind_of_single)| {
							xml::XmlNode::new("as-single-for").with_text(format!(
								"{} for \0<cite\0>{}\0</cite\0>",
								what_kind_of_single,
								album.format_title_short()
							))
						}))
				)
		)
}

pub fn make_home_page(
	all_albums: &[musicdata::Album], all_remixes: &[musicdata::Song],
	all_assists: &[musicdata::Assist]
) {
	assert!(icons::valid_icon("external")); // used in homepage-styles.css

	let head = xml::XmlNode::new("head")
		.with_child(xml::XmlNode::new("meta").with_attribute("charset", "utf-8"))
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "icon")
				.with_attribute("href", "favicon.ico")
				.with_attribute("type", "image/ico")
		)
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "canonical")
				.with_attribute("href", "https://music.astronomy487.com/")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "description")
				.with_attribute("content", OG_DESCRIPTION)
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "og:description")
				.with_attribute("content", OG_DESCRIPTION)
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
				.with_attribute("content", "Astro")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:image")
				.with_attribute("content", "squarelogo.png")
		)
		.with_child(
			xml::XmlNode::new("link")
				.with_attribute("rel", "apple-touch-icon")
				.with_attribute("href", "squarelogo.png")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("property", "og:url")
				.with_attribute("content", "https://music.astronomy487.com/")
		)
		.with_child(
			xml::XmlNode::new("meta")
				.with_attribute("name", "viewport")
				.with_attribute("content", "width=device-width, initial-scale=1")
		)
		.with_child(xml::XmlNode::new("title").with_text("Astro"))
		.with_child(
			xml::XmlNode::new("style").with_text_unescaped(css::compress_css(include_str!(
				"assets/homepage-styles.css"
			)))
		);
	let body = xml::XmlNode::new("body")
		.with_child(
			xml::XmlNode::new("header")
				.with_child(
					xml::XmlNode::new("header-content")
						.with_child(
							xml::XmlNode::new("header-column")
								.with_child(
									xml::XmlNode::new("h1")
										.with_text("Astro")
								)
								.with_child(
									xml::XmlNode::new("fka-label")
										.with_text("f.k.a. astronomy487")
								)
								.with_child(
									xml::XmlNode::new("pronoun-label")
										.with_text("he/him")
								)
								.with_child({
									let mut table = xml::XmlNode::new("table")
										.with_attribute("style", "margin-top: 1rem; width: 90%; table-layout: fixed; --acc: cyan;");
									for chunk in [
										("Apple Music", "applemusic", "https://music.apple.com/us/artist/astro/1468743818"),
										("Spotify", "spotify", "https://open.spotify.com/artist/2ajNTg6axZGx5gFZF0Upb5"),
										("YouTube", "youtube", "https://youtube.com/astronomy487"),
										("Bandcamp", "bandcamp", "https://astronomy487.bandcamp.com/"),
										("Twitter", "twitter", "https://twitter.com/astronomy487/"),
										("Discord", "discord", "https://discord.gg/ZnMsetP"),
										("GitHub", "github", "https://github.com/Astronomy487"),
										("RSS Feed", "rss", "https://music.astronomy487.com/rss.xml"),
									].chunks(2) {
										let mut tr = xml::XmlNode::new("tr");
										for (link_name, icon_name, url) in chunk {
											assert!(
												icons::valid_icon(icon_name),
												"Home page icon {} is not found in icon set; requires recompilation",
												icon_name
											);
											tr.add_child(
												xml::XmlNode::new("td")
													.with_child(
														xml::XmlNode::new("a")
															.with_attribute("href", *url)
															.with_child(
																xml::XmlNode::new("img")
																	.with_attribute("src", format!("icons/{}.svg", icon_name))
																	.with_attribute("aria-hidden", "true")
															)
															.with_child(
																xml::XmlNode::new("span")
																	.with_text(*link_name)
															)
													)
											)
										}
										table.add_child(tr);
									}
									table
								})
						)
					.with_child(
						xml::XmlNode::new("header-column")
							.with_attribute("style", "--acc: magenta;")
							.with_child(
								xml::XmlNode::new("p")
									.with_text(HEADER_PARAGRAPH)
							)
							.with_child(
								xml::XmlNode::new("p")
									.with_text(HEADER_PARAGRAPH_EXTRA_DESKTOP)
							)
							/* .with_child({
								let mut p = xml::XmlNode::new("p");
								for color_str in ["cyan", "magenta", "yellow"] {
									p.add_child(
										xml::XmlNode::new("div")
											.with_attribute("style", format!("width: 0.5rem; height: 0.5rem; display: inline-block; margin-right: 0.5rem; background-color: {}; border-radius: 100%;", color_str))
											.with_text("")
									)
								}
								p
							}) */
							.with_child(
								xml::XmlNode::new("img")
									.with_attribute("style", "position: absolute; bottom: 2rem; right: 2rem; width: 2rem; height: 2rem; user-select: none")
									.with_attribute("src", "https://astronomy487.com/logo/assets/white-big.svg")
							)
					) // end <header-content>

				)				.with_child(
					xml::XmlNode::new("rhombus-container").with_text("")
				)
		) // end <header>
		.with_child(
			xml::XmlNode::new("mobile-header")
				.with_child(
					xml::XmlNode::new("h1").with_text("Astro")
				)
				.with_child(
					xml::XmlNode::new("p").with_text(HEADER_PARAGRAPH)
				)
		)
		.with_child({
			let mut entries: Vec<(EntryType, &date::Date, xml::XmlNode)> = Vec::new();
			for album in all_albums {
				let main_songs = album.songs.iter().filter(|s| !s.bonus).collect::<Vec<_>>();
				let bonus_songs = album.songs.iter().filter(|s| s.bonus).collect::<Vec<_>>();
				entries.push((
					EntryType::Album,
					&album.released,
					xml::XmlNode::new("an-album")
						.with_attribute("style", album.palette.style_text())
						.maybe_with_attribute("class", if album.palette.home_page_album_needs_borders() {
							Some("extra-border")
						} else {
							None
						})
						.with_child(
							xml::XmlNode::new("album-c")
								.with_child(
									xml::XmlNode::new("a-column")
										.with_child(
											xml::XmlNode::new("img")
												.with_attribute("src", format!("artwork/{}.jpg", album.slug))
												.with_attribute("class", "album-art")
										)
										.with_child(
											xml::XmlNode::new("span")
												.maybe_with_attribute(
													"style",
													if album.unreleased {
														Some("width: 100%")
													} else {
														None
													}
												)
												.with_text(format!(
													"{}{}",
													if album.unreleased {"Coming "} else {""},
													album.released.to_display()
												))
												.maybe_with_attribute(
													"class",
													if album.released.birthday() {
														Some("bday")
													} else {
														None
													}
												)
										)
										.maybe_with_child(
											if album.unreleased {
												None
											} else {
												Some(
													xml::XmlNode::new("span")
														.with_text(musicdata::format_duration(album.length))
												)
											}
										)
								)
								.with_child({
									let mut column = xml::XmlNode::new("a-column")
										.with_child(
											xml::XmlNode::new("h2")
												.with_child(
													xml::XmlNode::new("a")
														.with_attribute("href", format!("{}/", album.slug))
														.with_text(album.format_title_short())
												)
										);
									if let Some(text) = &album.about {
										for paragraph_slice in text {
											let mut paragraph = smartquotes::smart_quotes(paragraph_slice);
											for album_title_to_italicize in all_albums.iter().filter(|a| !a.single).map(|a| a.title.clone()) {
												paragraph = paragraph.replace(&album_title_to_italicize, &format!("\0<cite\0>{}\0</cite\0>", album_title_to_italicize))
											}
											column.add_child(
												xml::XmlNode::new("p").with_text(paragraph)
											)
										}
									}
									if !album.single {
										let useful_title_length = {
											let max_title_length: usize = main_songs.iter().map(|s| s.title.len()).max().expect("No songs");
											let average_title_length: usize = main_songs.iter().map(|s| s.title.len()).sum::<usize>() / main_songs.len();
											(max_title_length + average_title_length * 3) / 4
										};
										let column_count: usize = if useful_title_length > 9 {2} else {11 - useful_title_length};
										let mut column_heights: Vec<usize> = vec![main_songs.len() / column_count; column_count];
										for i in 0..(main_songs.len() % column_count) {
											column_heights[i] += 1;
										}
										assert!(
											column_heights.iter().sum::<usize>() == main_songs.len(),
											"Column heights for {} do not sum correctly",
											album.title
										);
										let mut the_tracklist = xml::XmlNode::new("the-tracklist");
										let mut song_index = 0;
										for column_height in column_heights {
											let mut tracklist_column = xml::XmlNode::new("tracklist-column");
											for _ in 0..column_height {
												let song = &album.songs[song_index];
												tracklist_column.add_child(
													xml::XmlNode::new("tracklist-item")
														.with_child(
															xml::XmlNode::new("tracklist-item-number")
																.with_text((song_index + 1).to_string())
														)
														.with_child(
															xml::XmlNode::new("tracklist-item-title")
																.with_child(
																	xml::XmlNode::new("a")
																		.with_attribute("href", format!("{}/", song.slug))
																		.with_text(smartquotes::smart_quotes(&song.format_title_short()))
																)
														)
												);
												song_index += 1;
											}
											the_tracklist.add_child(tracklist_column);
										}
										column.add_child(the_tracklist);
									}
									if !bonus_songs.is_empty() {
										column.add_child(
											xml::XmlNode::new("bonus-tracks")
												.with_text({
													format!(
														"Digital download includes bonus {} {}",
														if bonus_songs.len() == 1 {"track"} else {"tracks"},
														{
															let bonus_songs_texts = bonus_songs
																.iter()
																.map(|bonus_song|
																	format!(
																		"“{}”",
																		smartquotes::smart_quotes(&bonus_song.format_title_short())
																	)
																)
																.collect::<Vec<_>>();
															match &bonus_songs_texts[..] {
																[] => unreachable!(),
																[one] => one.to_owned(),
																[first, second] => format!("{} and {}", first, second),
																[first, middle @ .., last] => {
																	let middles = middle.join(", ");
																	format!("{}, {}, and {}", first, middles, last)
																}
															}
														}
													)
												})
										);
									}
									column
								})
						)
				));
				// now should we also add entries for songs?
				// if less than half of album was released as singles and non-compilation, and song.released_as_single, ya! mention me!
				// else if compilation, yes report them, but don't say "from [album name]"
				let proportion_of_album_released_as_single = (main_songs.iter().filter(|s| s.released_as_single).collect::<Vec<_>>().len() as f32) / (main_songs.len() as f32);
				if !album.compilation && proportion_of_album_released_as_single < 0.5 {
					let mut the_singles: Vec<&musicdata::Song> = album.songs.iter().filter(|s| s.released_as_single).collect();
					the_singles.sort_by(|a, b| a.released.cmp(&b.released));
					match the_singles.len() {
						1 => {
							for single in the_singles {
								entries.push((
									EntryType::Song,
									&single.released,
									song_xml(single, Some((album, "Single")), true)
								));
							}
						},
						_ => {
							for (single_number, single) in the_singles.iter().enumerate() {
								entries.push((
									EntryType::Song,
									&single.released,
									song_xml(single, Some((album, describe_single_number(single_number+1))), true)
								));
							}
						}
					}
				} else if album.compilation {
					for song in &album.songs {
						if song.released < album.released {
							entries.push((
								EntryType::Song,
								&song.released,
								song_xml(song, None, !song.bonus)
							));
						}
					}
				}
			}
			for remix in all_remixes {
				entries.push((
					EntryType::Song,
					&remix.released,
					song_xml(remix, None, true)
				))
			}
			for assist in all_assists {
				entries.push((
					EntryType::Assist,
					&assist.released,
					xml::XmlNode::new("an-assist")
						.with_child(
							xml::XmlNode::new("assist-c")
								.with_child(
									xml::XmlNode::new("the-date")
										.with_text(assist.released.to_display())
										.maybe_with_attribute(
											"class",
											if assist.released.birthday() {
												Some("bday")
											} else {
												None
											}
										)
								)
								.with_child(
									xml::XmlNode::new("img")
										.with_attribute("src", &assist.artwork)
										.with_attribute("loading", "lazy")
								)
								.with_child(
									xml::XmlNode::new("div")
										.with_child(
											xml::XmlNode::new("a")
												.with_attribute("href", &assist.url)
												.with_attribute("class", "external")
												.with_text(smartquotes::smart_quotes(&assist.titlable))
										)
										.with_child(
											xml::XmlNode::new("p")
												.with_text(smartquotes::smart_quotes(&assist.role))
										)
								)
						)
				))
			}
			entries.reverse(); // ensure any date-ties are actually newest-to-oldest
			entries.sort_by(|a, b| b.1.cmp(a.1));
			let mut main = xml::XmlNode::new("main");
			let mut previous_entry_type = EntryType::Album;
			for (entry_type, _, entry_xml) in entries {
				// entry types are EntryType::Album, EntryType::Assist, or EntryType::Song
				let spacer = match (previous_entry_type, entry_type) {
					(EntryType::Album, EntryType::Album) => 0,
					(EntryType::Album, _) => 4,
					(_, EntryType::Album) => 4,
					(_, EntryType::Assist) => 1,
					(pt, ct) if pt == ct => 0,
					(_, _) => 1
				};
				if spacer > 0 {
					main.add_child(
						xml::XmlNode::new("div")
							.with_text("")
							.with_attribute("class", "spacer")
							.with_attribute("style", format!("height: {}rem;", spacer))
					);
				}
				main.add_child(entry_xml);
				previous_entry_type = entry_type;
			}
			main
		})
		.with_child(
			xml::XmlNode::new("footer")
				.with_child(
					xml::XmlNode::new("span")
						.with_text("Powered by ")
				)
				.with_child(
					xml::XmlNode::new("a")
						.with_text("distri")
						.with_attribute("href", "https://github.com/Astronomy487/distri/")
				)
				/* .with_child(
					xml::XmlNode::new("span")
						.with_text(format!(" on {}", date::Date::today().to_display()))
				) */
				.with_child(
					xml::XmlNode::new("div")
						.with_text(format!("© {} Astro “astronomy487”", date::Date::today().year))
				)
		)
		.with_child(
			xml::XmlNode::new("script")
				.with_text_unescaped(
					concat!(
						include_str!("assets/homepage-rhombus-min.js"),
						include_str!("assets/homepage-css-nuke.js")
					)
				)
		)
	;
	let html = xml::XmlNode::new("html")
		.with_attribute("lang", "en")
		.with_child(head)
		.with_child(body);
	{
		let index_html_location = std::path::Path::new(globals::filezone())
			.join("music.astronomy487.com")
			.join("index")
			.with_extension("html");
		let mut file = std::fs::File::create(&index_html_location)
			.unwrap_or_else(|_| panic!("Couldn't create file {}", index_html_location.display()));
		let _ = std::io::Write::write(&mut file, format!("<!DOCTYPE html>{}", html).as_bytes())
			.unwrap_or_else(|_| panic!("Couldn't write to file {}", index_html_location.display()));
	}
}

fn describe_single_number(single_number: usize) -> &'static str {
	match single_number {
		0 => unreachable!(),
		1 => "Lead single",
		2 => "Second single",
		3 => "Third single",
		4 => "Fourth single",
		5 => "Fifth single",
		6 => "Sixth single",
		7 => "Seventh single",
		8 => "Eighth single",
		9 => "Ninth single",
		10 => "Tenth single",
		11 => "Eleventh single",
		12 => "Twelfth single",
		13 => "Thirteenth single",
		14 => "Fourteenth single",
		15 => "Fifteenth single",
		16 => "Sixteenth single",
		17 => "Seventeenth single",
		18 => "Eighteenth single",
		19 => "Nineteenth single",
		20 => "Twentieth single",
		_ => {
			panic!("Too many singles")
		}
	}
}
