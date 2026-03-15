pub mod album;
pub mod artwork;
pub mod assist;
pub mod audiocodec;
pub mod lyric;
pub mod song;
pub mod titlable;

use crate::globals;
use crate::media::{album::Album, assist::Assist, song::Song};

pub fn get_music_data(json_path: &std::path::Path) -> (Vec<Album>, Vec<Song>, Vec<Assist>) {
	globals::log_3("Parsing", "", "Discography JSON", globals::ANSI_GREEN);
	let file =
		std::fs::File::open(json_path).unwrap_or_else(|_| panic!("Couldn't find discog.json"));
	let reader = std::io::BufReader::new(file);
	let json_value: serde_json::Value = serde_json::from_reader(reader)
		.unwrap_or_else(|error| panic!("discog.json is invalid JSON: {}", error));
	let object = globals::map_with_only_these_keys(
		&json_value,
		"Discography",
		&["albums", "remixes", "assists"]
	);
	let all_remixes: Vec<Song> = {
		let mut remixes = Vec::new();
		for remix_json in object
			.get("remixes")
			.expect("discog.json has no attribute \"remixes\"")
			.as_array()
			.expect("discog.json \"remixes\" attribute is not an array")
			.iter()
		{
			remixes.push(Song::from_json(remix_json, None));
		}
		remixes
	};
	let mut all_albums: Vec<Album> = object
		.get("albums")
		.expect("discog.json has no attribute \"albums\"")
		.as_array()
		.expect("discog.json \"albums\" attribute is not an array")
		.iter()
		.map(Album::from_json)
		.collect();
	let all_assists: Vec<Assist> = object
		.get("assists")
		.expect("discog.json has no attribute \"assists\"")
		.as_array()
		.expect("discog.json \"assists\" attribute is not an array")
		.iter()
		.map(Assist::from_json)
		.collect();

	// assign parent_album refs
	for (album_index, album) in all_albums.iter_mut().enumerate() {
		for (song_index, song) in album.songs.iter_mut().enumerate() {
			song.parent_album_indices = Some((album_index, song_index));
		}
	}

	// validation
	for remix in &all_remixes {
		assert!(
			!remix.bonus,
			"Remix {} must not be marked as a bonus track",
			remix.format_title()
		);
		/* assert!(
			remix.artwork.is_none(),
			"Remix {} must not have artwork",
			remix.format_title()
		); */
	}
	let _ = crate::media::artwork::Artwork::fallback();
	let mut seen_slugs = std::collections::HashSet::new();
	let mut check_slug_collision = |s: &str| {
		assert!(
			seen_slugs.insert(s.to_owned()),
			"Two items cannot both have the slug {}",
			s
		);
	};
	check_slug_collision("");
	check_slug_collision("icons");
	check_slug_collision("artwork");
	check_slug_collision("8831");
	check_slug_collision("font");
	for album in &all_albums {
		if album.single {
			assert!(
				album.title == album.songs[0].title,
				"Single cannot have two different titles: {}, {}",
				album.title,
				album.songs[0].title
			);
			assert!(
				album.artist == album.songs[0].artist,
				"Single cannot have two different artists: {}, {}",
				album.title,
				album.songs[0].title
			);
			check_slug_collision(&album.slug);
			for i in 2..album.songs.len() {
				assert!(
					album.songs[i].bonus,
					"Additional track in single {} must be marked as bonus",
					album.songs[i].format_title()
				);
				check_slug_collision(&album.songs[i].slug);
			}
		} else {
			check_slug_collision(&album.slug);
			for song in &album.songs {
				check_slug_collision(&song.slug);
			}
		}
		if !album.unreleased {
			for song in &album.songs {
				assert!(
					!song.unreleased,
					"Album {} has unreleased song {}",
					album.format_title(),
					song.format_title()
				);
			}
		}
	}
	for album in &all_albums {
		for song in &album.songs {
			assert!(
				!song.event,
				"Album track {} must not be marked as an event",
				song.format_title()
			);
			assert!(
				song.artwork.is_some(),
				"Non-remix song {} on album {} must have its own artwork or inherit from a parent (How did this manage to happen?)",
				song.format_title(),
				album.format_title()
			);
		}
		assert!(
			!album.songs[0].bonus,
			"First track of {} must not be a bonus track",
			album.format_title()
		);
		for window in album.songs.windows(2) {
			assert!(
				!window[0].bonus || window[1].bonus,
				"Bonus track {} is followed by non-bonus track {}",
				window[0].format_title(),
				window[1].format_title()
			);
		}
	}
	// also validate that flac exists
	for album in &all_albums {
		for song in &album.songs {
			let flac_location = globals::filezone()
				.join("source")
				.join("audio")
				.join(&album.slug)
				.join(&song.slug)
				.with_extension("flac");
			assert!(
				flac_location.exists(),
				"Audio file {}/{}.flac missing",
				album.slug,
				song.slug
			);
		}
	}
	for remix in &all_remixes {
		let flac_location = globals::filezone()
			.join("source")
			.join("audio")
			.join(&remix.slug)
			.with_extension("flac");
		assert!(
			flac_location.exists(),
			"Audio file {}.flac missing",
			remix.slug
		);
	}

	// check for ascending release dates
	assert!(
		all_albums.is_sorted_by(|a, b| a.released <= b.released),
		"Albums are not sorted from oldest to newest"
	);
	assert!(
		all_remixes.is_sorted_by(|a, b| a.released <= b.released),
		"Remixes are not sorted from oldest to newest"
	);
	assert!(
		all_assists.is_sorted_by(|a, b| a.released <= b.released),
		"Assists are not sorted from oldest to newest"
	);

	(all_albums, all_remixes, all_assists)
}
