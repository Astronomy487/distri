#![deny(non_snake_case)]
#![deny(non_camel_case_types)]
#![deny(non_upper_case_globals)]
#![deny(unused_mut)]
#![deny(unreachable_code)]
#![deny(unreachable_patterns)]
#![deny(unused_results)]
#![deny(unused_unsafe)]
#![allow(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(private_interfaces)]
#![deny(absolute_paths_not_starting_with_crate)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::many_single_char_names)]
// #![warn(clippy::expect_used)]

mod globals;
mod imagedeal;
mod musicdata;
mod rss;
mod zipper;

fn main() {
	if !std::process::Command::new("ffmpeg")
		.arg("-version")
		.output()
		.map(|output| output.status.success())
		.unwrap_or(false)
	{
		panic!("FFmpeg is not installed or is not on your path")
	}

	println!(
		"Using directory {}",
		std::path::Path::new(globals::FILEZONE).display()
	);

	let (all_albums, all_remixes) = musicdata::get_music_data(
		&std::path::Path::new(globals::FILEZONE)
			.join("discog")
			.with_extension("json")
	);

	println!(
		"{}Building audio.astronomy487.com{}",
		globals::ANSI_CYAN,
		globals::ANSI_RESET
	);
	for album in &all_albums {
		album.try_encode(&all_albums);
	}
	for remix in &all_remixes {
		remix.try_encode(&all_albums);
	}

	println!(
		"{}Building music.astronomy487.com{}",
		globals::ANSI_CYAN,
		globals::ANSI_RESET
	);
	for album in &all_albums {
		if album.temporary {
			break;
		}
		crate::musicdata::Titlable::make_link_page(&musicdata::Titlable::Album(album));
		for song in &album.songs {
			if !song.bonus {
				crate::musicdata::Titlable::make_link_page(&musicdata::Titlable::Song(
					song,
					Some(album)
				));
			}
		}
	}
	for remix in &all_remixes {
		crate::musicdata::Titlable::make_link_page(&musicdata::Titlable::Song(remix, None));
	}

	zipper::copy_recursive(
		&std::path::Path::new(globals::FILEZONE)
			.join("source")
			.join("root"),
		&std::path::Path::new(globals::FILEZONE).join("music.astronomy487.com")
	);
	std::fs::write(
		std::path::Path::new(globals::FILEZONE)
			.join("music.astronomy487.com")
			.join("discog")
			.with_extension("json"),
		format!(
			"let discog = {};",
			std::fs::read_to_string(
				std::path::Path::new(globals::FILEZONE)
					.join("discog")
					.with_extension("json")
			)
			.expect("Couldn't read discog.json")
		)
	)
	.expect("Couldn't write discog.js");
	rss::make_rss(&all_albums);

	println!("distri ran successfully.");
	println!("Press enter to exit.");
	let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0u8])
		.expect("Couldn't get user to press enter to exit");
}
