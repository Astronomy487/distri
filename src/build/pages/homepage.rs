use crate::build::{icons, minify, smartquotes, xml::XmlNode};
use crate::globals;
use crate::media::{album::Album, assist::Assist, song::Song};
use crate::types::color::Color;
use crate::types::date::Date;

const OG_DESCRIPTION: &str = "Astro (f.k.a. astronomy487) is an independent electronic dance music artist with a love for synthesizers and pop music.";

// (bool, &str) : (only show on desktop, paragraph)
const HEADER_PARAGRAPHS: [(bool, &str); 3] = [
	(
		false,
		"I am an independent electronic dance music artist with a love for synthesizers and pop music. I create whichever sounds I find the most interesting."
	),
	(
		true,
		"Below is an exhaustive catalog of every released song I’ve ever made or worked on."
	),
	(
		false,
		"All my music is freely available for download in its original quality."
	)
];

#[derive(PartialEq, Clone, Copy)]
enum EntryType {
	Album,
	Song,
	Assist
}

fn song_xml(song: &Song, for_album: Option<(&Album, &str)>, have_link: bool) -> XmlNode {
	XmlNode::new("a-s")
		/* .maybe_with_attribute("style", {
			let accent_color = &song.palette.accent;
			if accent_color.contrast(&Color::BLACK) > 3.0 {
				Some(format!("--acc:{}", accent_color))
			} else {
				None
			}
		}) */
		.with_child(
			XmlNode::new("s-c")
				.with_child(
					XmlNode::new("the-date")
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
					XmlNode::new("s-t")
						.with_child({
							// boolean tells if it is an external link
							let maybe_link: Option<(String, bool)> = if have_link {
								Some((format!("{}/", song.slug), false))
							} else {
								song.url.try_to_get_at_least_one_link().map(|s| (s.to_owned(), true))
							};
							let is_hover = song.title == "Hover" && song.artist == "Astro";
							let mut span = XmlNode::new("span");
							if let Some((link, is_external)) = maybe_link {
								span.add_child(
									XmlNode::new("a")
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
									XmlNode::new("span")
										.with_attribute("style", "color: var(--gray);")
										.with_text(" — Not everyone was there !")
								);
								span.add_child({
									assert!(icons::valid_icon("greenheart"));
									XmlNode::new("img")
										.with_attribute("src", "icons/greenheart.svg")
										.with_attribute("style", "margin-left: 0.5rem; height: 1.25rem; margin-top: -0.125rem; user-select: none; vertical-align: middle;")
										.with_attribute("id", "hoverheart")
										.with_attribute("aria-hidden", "true")
										.with_attribute("alt", "")
								});
							}
							span
						})
						.maybe_with_child(for_album.map(|(album, what_kind_of_single)| {
							XmlNode::new("as-single-for").with_text(format!(
								"{} for \0<cite\0>{}\0</cite\0>",
								what_kind_of_single,
								album.format_title_short()
							))
						}))
				)
		)
}

pub fn make_home_page(all_albums: &[Album], all_remixes: &[Song], all_assists: &[Assist]) {
	assert!(icons::valid_icon("external")); // used in homepage-styles.css

	let head = XmlNode::new("head")
		.with_child(XmlNode::new("meta").with_attribute("charset", "utf-8"))
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "icon")
				.with_attribute("href", "favicon.ico")
				.with_attribute("type", "image/ico")
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "canonical")
				.with_attribute("href", "https://music.astronomy487.com/")
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "description")
				.with_attribute("content", OG_DESCRIPTION)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "og:description")
				.with_attribute("content", OG_DESCRIPTION)
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "keywords")
				.with_attribute("content", format!("discography, {}", globals::OG_KEYWORDS))
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
				.with_attribute("content", "Astro")
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "og:image")
				.with_attribute("content", "squarelogo.png")
		)
		.with_child(
			XmlNode::new("link")
				.with_attribute("rel", "apple-touch-icon")
				.with_attribute("href", "squarelogo.png")
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("property", "og:url")
				.with_attribute("content", "https://music.astronomy487.com/")
		)
		.with_child(
			XmlNode::new("meta")
				.with_attribute("name", "viewport")
				.with_attribute("content", "width=device-width, initial-scale=1")
		)
		.with_child(XmlNode::new("title").with_text("Astro"))
		.with_child(
			XmlNode::new("style").with_text_unescaped(minify::compress_css(include_str!(
				"../../assets/homepage-style.css"
			)))
		);
	let body = XmlNode::new("body")
		.with_child(
			XmlNode::new("header")
				.with_child(
					XmlNode::new("header-content")
						.with_child(
							XmlNode::new("header-column")
								.with_child(
									XmlNode::new("h1")
										.with_text("Astro")
								)
								.with_child(
									XmlNode::new("fka-label")
										.with_text("f.k.a. astronomy487")
								)
								.with_child(
									XmlNode::new("pronoun-label")
										.with_text("he/him")
								)
								.with_child({
									let mut table = XmlNode::new("table")
										.with_attribute("style", "margin-top: 1rem; width: 90%; table-layout: fixed; --acc: cyan;");
									for chunk in [
										("Apple Music", "applemusic", "https://music.apple.com/us/artist/astro/1468743818"),
										("Spotify", "spotify", "https://open.spotify.com/artist/2ajNTg6axZGx5gFZF0Upb5"),
										("YouTube", "youtube", "https://youtube.com/astronomy487"),
										("Bandcamp", "bandcamp", "https://astronomy487.bandcamp.com/"),
										("Twitter", "twitter", "https://twitter.com/astronomy487/"),
										("Discord", "discord", "https://discord.gg/ZnMsetP"),
										("GitHub", "github", "https://github.com/Astronomy487"),
										("RSS Feed", "rss", "rss.xml"),
									].chunks(2) {
										let mut tr = XmlNode::new("tr");
										for (link_name, icon_name, url) in chunk {
											assert!(
												icons::valid_icon(icon_name),
												"Home page icon {} is not found in icon set; requires recompilation",
												icon_name
											);
											tr.add_child(
												XmlNode::new("td")
													.with_child(
														XmlNode::new("a")
															.with_attribute("href", *url)
															.with_child(
																XmlNode::new("img")
																	.with_attribute("src", format!("icons/{}.svg", icon_name))
																	.with_attribute("aria-hidden", "true")
																	.with_attribute("alt", "")
															)
															.with_child(
																XmlNode::new("span")
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
					.with_child({
						let mut column = XmlNode::new("header-column")
							.with_attribute("style", "--acc: magenta;");
						for (_, paragraph) in HEADER_PARAGRAPHS {
							column.add_child(
								XmlNode::new("p")
									.with_text(paragraph)
							);
						}
						column.with_child(
							icons::inline_logo_svg(
								&Color::WHITE,
								&Color::WHITE,
								&Color::WHITE
							)
								.with_attribute("style", "position: absolute; bottom: 2rem; right: 2rem; width: 2rem; height: 2rem; user-select: none")
						)
					})
					/* .with_child(
						XmlNode::new("header-arrow")
							.with_child(
								XmlNode::new("svg")
									.with_attribute("viewBox", "-100, -100, 1200, 1200")
									.with_child(
										XmlNode::new("polygon")
											.with_attribute("points", "300,-40 700,-40 700,800 300,800")
											.with_attribute("fill", "#00000066")
									)
									.with_child(
										XmlNode::new("polygon")
											.with_attribute("points", "200,-100 1100,-100 1100,800 700,800 700,300 200,300")
											.with_attribute("transform", "translate(1200,300) rotate(135)")
											.with_attribute("fill", "#00000066")
									)
									.with_child(
										XmlNode::new("polygon")
											.with_attribute("points", "400,60 600,60 600,900 400,900")
											.with_attribute("fill", "white")
									)
									.with_child(
										XmlNode::new("polygon")
											.with_attribute("points", "300,0 1000,0 1000,700 800,700 800,200 300,200")
											.with_attribute("transform", "translate(1200,300) rotate(135)")
											.with_attribute("fill", "white")
									)
									.with_attribute("style", "width:1.5rem;height:1.5rem;margin:0.25rem")
							)
							.with_attribute("style", "position:absolute;bottom:-1rem;border-radius:1rem;left:50%;transform:translateX(-50%);width:2rem;height:2rem;display:block;background:conic-gradient(cyan,magenta,yellow,cyan)")
					) */
					// end <header-content>

				)				.with_child(
					XmlNode::new("rhombus-container").with_text("")
				)
		) // end <header>
		.with_child({
			let mut mobile_header = XmlNode::new("mobile-header")
				.with_child(
					XmlNode::new("h1").with_text("Astro")
				);
			for (desktop_only, paragraph) in HEADER_PARAGRAPHS {
				if !desktop_only {
					mobile_header.add_child(
						XmlNode::new("p").with_text(paragraph)
					)
				}
			}
			mobile_header
		})
		.with_child({
			let mut entries: Vec<(EntryType, &Date, XmlNode)> = Vec::new();
			for album in all_albums {
				let main_songs = album.songs.iter().filter(|s| !s.bonus).collect::<Vec<_>>();
				let bonus_songs = album.songs.iter().filter(|s| s.bonus).collect::<Vec<_>>();
				entries.push((
					EntryType::Album,
					&album.released,
					XmlNode::new("a-a")
						.with_attribute("style", album.palette.style_text())
						.maybe_with_attribute("class", if album.palette.home_page_album_needs_borders() {
							Some("extra-border")
						} else {
							None
						})
						.with_child(
							XmlNode::new("album-c")
								.with_child({
									let mut column = XmlNode::new("a-c")
										.with_child(
											XmlNode::new("img")
												.with_attribute("src", format!("artwork/{}.jpg", album.artwork.name_without_slash))
												.with_attribute("class", "album-art")
												.with_attribute("alt", smartquotes::smart_quotes(&album.artwork.caption))
										)
										.with_child(
											XmlNode::new("the-date")
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
													XmlNode::new("the-runtime")
														.with_text(album.duration.display())
												)
											}
										);
									if album.has_8831 {
										column.add_child(
											XmlNode::new("img")
												.with_attribute("src", format!("8831/{}.gif", album.slug))
												.with_attribute("aria-hidden", "true")
												.with_attribute("alt", "")
												.with_attribute("class", "x8831")
										)
									}
									/* if let Some(upc) = &album.upc {
										column.add_child(
											upc.svg()
												.with_attribute("class", "upc")
										);
									} */
									column
								})
								.with_child({
									let mut column = XmlNode::new("a-c")
										.maybe_with_child(
											if album.artist == "Astro" {
												None
											} else {
												Some(XmlNode::new("h3")
													.with_text(
														smartquotes::smart_quotes(&album.artist)
													)
												)
											}
										)
										.with_child(
											XmlNode::new("h2")
												.with_child(
													XmlNode::new("a")
														.with_attribute("href", format!("{}/", album.slug))
														.with_text(
															smartquotes::smart_quotes(&album.title)
														)
												)
										);
									if let Some(text) = &album.about {
										for paragraph_slice in text {
											let mut paragraph = smartquotes::smart_quotes(paragraph_slice);
											for album_to_italicize in all_albums.iter().filter(|a| !a.single) {
												let title_text_to_replace = smartquotes::smart_quotes(&album_to_italicize.title);
												/* paragraph = paragraph.replace(&title_text_to_replace, &format!(
													"\0<a href=#{}\0>{}\0</a\0>",
													album.slug,
													title_text_to_replace
												)) */
												paragraph = paragraph.replace(&title_text_to_replace, &format!(
													"\0<cite\0>{}\0</cite\0>",
													title_text_to_replace
												))
											}
											column.add_child(
												XmlNode::new("p").with_text(paragraph)
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
										{
											#![allow(clippy::needless_range_loop)]
											// clippy's proposed 'idiomatic' solution is stupid
											for i in 0..(main_songs.len() % column_count) {
												column_heights[i] += 1;
											}
										}
										assert!(
											column_heights.iter().sum::<usize>() == main_songs.len(),
											"Column heights for {} do not sum correctly",
											album.title
										);
										let mut the_tracklist = XmlNode::new("tl-a");
										let mut song_index = 0;
										for column_height in column_heights {
											let mut tracklist_column = XmlNode::new("tl-c");
											for _ in 0..column_height {
												if let Some(discs) = &album.discs {
													let mut cumulative = 0;
													for (count, name) in discs {
														if song_index == cumulative {
															tracklist_column.add_child(
																XmlNode::new("tl-dh").with_text(smartquotes::smart_quotes(name))
															);
														}
														if cumulative >= song_index {
															break;
														}
														cumulative += count;
													}
												}
												let song = &album.songs[song_index];
												tracklist_column.add_child(
													XmlNode::new("tl-i")
														.with_child(
															XmlNode::new("tl-in")
																.with_text((song_index+1).to_string())
														)
														.with_child(
															XmlNode::new("tl-it")
																.with_child(
																	XmlNode::new("a")
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
											XmlNode::new("bonus-tracks")
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
					let mut the_singles: Vec<&Song> = album.songs.iter().filter(|s| s.released_as_single).collect();
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
					XmlNode::new("a-h")
						.with_child(
							XmlNode::new("assist-c")
								.with_child(
									XmlNode::new("the-date")
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
									XmlNode::new("img")
										.with_attribute("src", &assist.artwork)
										.with_attribute("loading", "lazy")
										.with_attribute("aria-hidden", "true")
										.with_attribute("alt", "")
								)
								.with_child(
									XmlNode::new("div")
										.with_child(
											XmlNode::new("a")
												.with_attribute("href", &assist.url)
												.with_attribute("class", "external")
												.with_text(smartquotes::smart_quotes(&assist.titlable))
										)
										.with_child(
											XmlNode::new("p")
												.with_text(smartquotes::smart_quotes(&assist.role))
										)
								)
						)
				))
			}
			entries.reverse(); // ensure any date-ties are actually newest-to-oldest
			entries.sort_by(|a, b| b.1.cmp(a.1));
			let mut main = XmlNode::new("main");
			let mut previous_entry_type = EntryType::Album;
			for (entry_type, _, entry_xml) in entries {
				// entry types are EntryType::Album, EntryType::Assist, or EntryType::Song
				if let Some(spacer_tag) = match (previous_entry_type, entry_type) {
					(EntryType::Album, EntryType::Album) => None,
					(EntryType::Album, _) => Some("sp-4"),
					(_, EntryType::Album) => Some("sp-4"),
					(_, EntryType::Assist) => Some("sp-1"),
					(pt, ct) if pt == ct => None,
					(_, _) => Some("sp-1")
				} {
					main.add_child(
						XmlNode::new(spacer_tag).with_text("")
					);
				}
				main.add_child(entry_xml);
				previous_entry_type = entry_type;
			}
			main
		})
		.with_child(
			XmlNode::new("footer")
				.with_text("Compiled by ")
				.with_child(
					XmlNode::new("a")
						.with_text("distri")
						.with_attribute("href", "https://github.com/Astronomy487/distri/")
				)
				.with_child(
					XmlNode::new("div")
						.with_text(format!("© {} Astro “astronomy487”", Date::today().year))
				)
				.with_child(
					icons::inline_logo_svg(
						&Color::CYAN,
						&Color::MAGENTA,
						&Color::YELLOW
					)
						.with_attribute("style", "width: 2rem; height: 2rem; user-select: none; margin-top: 1rem;")
						// .with_attribute("style", "margin: 3rem auto; display: block; width: 2rem; height: 2rem; user-select: none;")
				)
		)
		.with_child(
			XmlNode::new("script")
				.with_text_unescaped(
					minify::compress_js(include_str!("../../assets/homepage-rhombus.js"))
					+ &minify::compress_js(include_str!("../../assets/homepage-css-nuke.js"))
				)
		)
	;
	let html = XmlNode::new("html")
		.with_attribute("lang", "en")
		.with_child(head)
		.with_child(body);
	{
		let index_html_location = globals::filezone()
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
