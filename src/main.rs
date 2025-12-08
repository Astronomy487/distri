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

mod globals;
mod imagedeal;
mod musicdata;
mod zipper;

fn copy_recursive(from: &std::path::Path, to: &std::path::Path) {
	if from.is_dir() {
		std::fs::create_dir_all(to).expect("Can't perform copy");
		for entry in std::fs::read_dir(from).expect("Can't perform copy") {
			let entry = entry.expect("Can't perform copy");
			let src_path = entry.path();
			let dst_path = to.join(entry.file_name());
			if src_path.is_dir() {
				copy_recursive(&src_path, &dst_path);
			} else {
				let _ = std::fs::copy(&src_path, &dst_path).expect("Couldn't perform copy");
			}
		}
	} else {
		let _ = std::fs::copy(from, to).expect("Couldn't perform copy");
	}
}

fn main() {
	println!("\x1b[96m1\x1b[0m Reading discog.json");
	let (albums, remixes) = musicdata::get_music_data(
		&std::path::Path::new(globals::FILEZONE)
			.join("discog")
			.with_extension("json")
	);

	let mut failures = std::collections::HashSet::new();
	println!("\x1b[96m2\x1b[0m Performing encoding");
	for album in &albums {
		if album.temporary {
			break;
		}
		if !album.try_encode(&mut failures) {}
	}
	for remix in &remixes {
		if !remix.try_encode(None, &mut failures) {}
	}

	println!("\x1b[96m3\x1b[0m Building site");
	for album in &albums {
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
	for remix in &remixes {
		crate::musicdata::Titlable::make_link_page(&musicdata::Titlable::Song(remix, None));
	}

	copy_recursive(
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

	let mut list: Vec<&String> = failures.iter().collect();
	list.sort();
	let mut f = std::fs::File::create(
		std::path::Path::new(globals::FILEZONE)
			.join("not-found")
			.with_extension("txt")
	)
	.expect("Couldn't write to not-found.txt");
	for item in &list {
		let _ = std::io::Write::write_all(&mut f, item.as_bytes());
		let _ = std::io::Write::write_all(&mut f, b"\n");
	}
	println!(
		"\x1b[96m4\x1b[0m {} audio source{} could not be found (check not-found.txt)",
		list.len(),
		if list.len() == 1 { "" } else { "s" }
	);

	println!("\x1b[90mPress enter to exit.\x1b[0m");
	let _ = std::io::Read::read(&mut std::io::stdin(), &mut [0u8]).unwrap();
}
