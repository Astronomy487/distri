#![deny(non_snake_case)]
#![deny(non_camel_case_types)]
#![deny(non_upper_case_globals)]
#![deny(unused_mut)]
#![deny(unreachable_code)]
#![deny(unreachable_patterns)]
#![deny(unused_results)]
#![deny(unused_unsafe)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod imagedeal;
mod musicdata;
mod zipper;

fn main() {
	// println!("\x1b[96m1\x1b[0m Reading discog.json");
	let (albums, remixes) = musicdata::get_music_data(std::path::Path::new(
		"C:/Users/astro/Code/distri/filezone/discog.json"
	));
	// println!("\x1b[96m2\x1b[0m Performing encoding");
	for album in &albums {
		if album.temporary {
			break;
		}
		if !album.try_encode() {}
	}
	for remix in &remixes {
		if !remix.try_encode(None) {}
	}
}
