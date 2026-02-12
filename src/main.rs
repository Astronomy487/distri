#![deny(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![warn(unused_mut, unused_results, unused_variables, unused_imports)]
#![deny(unsafe_code, unused_unsafe)]
#![deny(unreachable_code, unreachable_patterns)]
#![allow(dead_code)]
#![deny(private_interfaces)]
#![deny(absolute_paths_not_starting_with_crate)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::many_single_char_names)]
#![deny(clippy::shadow_reuse, clippy::shadow_same, clippy::shadow_unrelated)]
#![deny(clippy::cast_lossless)]
#![warn(clippy::manual_assert)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_range_loop)]

mod color;
mod css;
mod date;
mod fileops;
mod genre;
mod globals;
mod homepage;
mod icons;
mod imagedeal;
mod language;
mod linkpage;
mod lyric;
mod musicdata;
mod rclone;
mod rss;
mod smartquotes;
mod url;
mod wrangle;
mod xml;

fn main() {
	set_panic_hook();
	check_if_can_run();

	let args: Vec<String> = std::env::args().skip(1).collect();
	if args.is_empty() {
		distri_help();
		return;
	}
	let allowed = ["publish", "validate", "encode", "build", "clean"];
	if args.iter().any(|a| !allowed.contains(&a.as_str())) {
		distri_help();
		return;
	}
	let only = |s: &str| args.len() == 1 && args[0] == s;
	if only("clean") {
		distri_clean();
	} else if only("publish") {
		distri_publish();
	} else {
		let encode = args.contains(&"encode".into());
		let build = args.contains(&"build".into());
		let valid = args.contains(&"validate".into());
		if encode || build || valid {
			distri_encode(encode, build);
		} else {
			distri_help();
		}
	}
}
fn distri_encode(build_r2_bucket: bool, build_static_website: bool) {
	let just_validating = !build_r2_bucket && !build_static_website;

	let json_location = std::path::Path::new(globals::filezone())
		.join("source")
		.join("discog")
		.with_extension("json");

	let (all_albums, all_remixes, all_assists) = musicdata::get_music_data(&json_location);

	// Check that all album artwork is where it needs to be
	globals::log_2("Locating", "Artwork", globals::ANSI_GREEN);
	if !just_validating {
		for album in &all_albums {
			let _ = imagedeal::grab_artwork_data(album.slug.clone(), imagedeal::ImageCodec::Jpg);
			let _ = imagedeal::grab_artwork_data(album.slug.clone(), imagedeal::ImageCodec::Png);
			for song in &album.songs {
				let _ = song.grab_artwork_data(imagedeal::ImageCodec::Jpg);
				// let _ = song.grab_artwork_data(imagedeal::ImageCodec::Png);
			}
		}
		for remix in &all_remixes {
			let _ = remix.grab_artwork_data(imagedeal::ImageCodec::Jpg);
			// let _ = remix.grab_artwork_data(imagedeal::ImageCodec::Png);
		}
	} else {
		for album in &all_albums {
			imagedeal::check_artwork(album.slug.clone());
			for song in &album.songs {
				song.check_artwork();
			}
		}
		for remix in &all_remixes {
			remix.check_artwork();
		}
	}

	if build_r2_bucket {
		assert!(
			globals::PANIC_ON_MISSING_LYRICS,
			"Cannot encode right now because missing lyrics will not panic; change globals::PANIC_ON_MISSING_LYRICS and recompile"
		);
		for album in &all_albums {
			album.try_encode(&all_albums);
		}
		for remix in &all_remixes {
			remix.try_encode(&all_albums);
		}
	}

	if build_static_website {
		let music_astronomy487_com =
			std::path::Path::new(globals::filezone()).join("music.astronomy487.com");
		if music_astronomy487_com.exists() {
			fileops::clear_directory(&music_astronomy487_com);
		} else {
			std::fs::create_dir(&music_astronomy487_com)
				.unwrap_or_else(|_| panic!("Couldn't create directory music.astronomy487.com"));
		}

		globals::log_3("Building", "", "Home page", globals::ANSI_BLUE);
		homepage::make_home_page(&all_albums, &all_remixes, &all_assists);

		globals::log_3("Building", "", "Link pages", globals::ANSI_BLUE);
		for album in &all_albums {
			linkpage::make_link_page(
				&musicdata::Titlable::Album(album),
				&all_albums,
				build_r2_bucket
			);
			if !album.single {
				for song in &album.songs {
					if !song.bonus {
						linkpage::make_link_page(
							&musicdata::Titlable::Song(song),
							&all_albums,
							build_r2_bucket
						);
					}
				}
			}
		}
		for remix in &all_remixes {
			linkpage::make_link_page(
				&musicdata::Titlable::Song(remix),
				&all_albums,
				build_r2_bucket
			);
		}

		globals::log_3("Building", "", "Other web assets", globals::ANSI_BLUE);
		// album art jpgs
		{
			let mut artwork_that_needs_copying = std::collections::HashSet::new();
			for album in &all_albums {
				let _ = artwork_that_needs_copying.insert(album.slug.clone());
				for song in &album.songs {
					if !song.bonus && let Some(art) = &song.artwork {
						let _ = artwork_that_needs_copying.insert(art.clone());
					}
				}
			}
			for remix in &all_remixes {
				if let Some(art) = &remix.artwork {
					// remixes aren't supposed to have artwork. whatever that's not my job
					let _ = artwork_that_needs_copying.insert(art.clone());
				}
			}
			let src_dir = std::path::Path::new(globals::filezone())
				.join("private")
				.join("jpg");
			let dest_dir = std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.join("artwork");
			for name in artwork_that_needs_copying {
				let src = src_dir.join(&name).with_extension("jpg");
				let dest = dest_dir.join(&name).with_extension("jpg");
				let _ = std::fs::copy(&src, &dest).expect("Couldn't copy JPG artwork");
			}
		}
		// linkpage styles
		fileops::write_file(
			&std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.join("style")
				.with_extension("css"),
			css::compress_css(
				include_str!("assets/linkpage-style.css").to_owned()
					+ &url::UrlSet::linkpage_css_for_platforms()
			)
		);
		// fonts
		std::fs::create_dir(
			std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.join("font")
		)
		.unwrap_or_else(|_| panic!("Couldn't create directory music.astronomy487.com/font",));
		for (font, data) in [
			(
				"ClashDisplay-Variable.woff2",
				include_bytes!("assets/font/ClashDisplay-Variable.woff2").to_vec()
			),
			(
				"PublicSans-Variable.woff2",
				include_bytes!("assets/font/PublicSans-Variable.woff2").to_vec()
			),
			(
				"PublicSans-VariableItalic.woff2",
				include_bytes!("assets/font/PublicSans-VariableItalic.woff2").to_vec()
			)
		] {
			fileops::write_file(
				&std::path::Path::new(globals::filezone())
					.join("music.astronomy487.com")
					.join("font")
					.join(font)
					.with_extension("woff2"),
				data
			);
		}
		// icons
		icons::put_icons();
		// others
		for (name, data) in [
			(
				"squarelogo.png",
				include_bytes!("assets/squarelogo.png").to_vec()
			),
			("404.html", include_bytes!("assets/404.html").to_vec()),
			("favicon.ico", include_bytes!("assets/favicon.ico").to_vec())
		] {
			fileops::write_file(
				&std::path::Path::new(globals::filezone())
					.join("music.astronomy487.com")
					.join(name),
				data
			);
		}
		/* fileops::write_file(
			&std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.join("discog")
				.with_extension("json"),
			std::fs::read_to_string(&json_location).expect("Couldn't read discog.json")
		); */

		globals::log_3("Building", "", "RSS feed", globals::ANSI_BLUE);
		rss::make_rss(&all_albums, &all_remixes, &all_assists);
	}
}

fn distri_publish() {
	globals::log_2("Checking", "Internet connection", globals::ANSI_GREEN);
	let connected_to_internet = std::net::TcpStream::connect_timeout(
		&"1.1.1.1:80"
			.parse()
			.expect("Couldn't setup ping to 1.1.1.1"),
		std::time::Duration::from_secs(3)
	)
	.is_ok();
	assert!(
		connected_to_internet,
		"Cannot connect to the internet right now"
	);

	// check for credentials
	globals::log_2(
		"Validating",
		"Credentials for wrangler",
		globals::ANSI_GREEN
	);
	if !std::process::Command::new("wrangler.cmd")
		.arg("whoami")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.map(|s| s.success())
		.unwrap_or(false)
	{
		panic!("Missing credentials for wrangler; run `wrangler login`");
	}
	globals::log_2("Validating", "Credentials for rclone", globals::ANSI_GREEN);
	if !std::process::Command::new("rclone")
		.arg("lsd")
		.arg("audio-astronomy487-com:")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.map(|s| s.success())
		.unwrap_or(false)
	{
		panic!("Missing credentials for rclone; run `rclone config");
	}

	println!("This will publish content to the internet.");
	if globals::ask_to_continue() {
		distri_encode(true, true);
		wrangle::music_astronomy487_com();
		// wrangler can be talkative - delete its extra directories
		for dir_name in [".wrangler", "node_modules"] {
			let dot_wrangler_folder = std::env::current_dir()
				.expect("Could not find current working directory")
				.join(dir_name);
			if dot_wrangler_folder.is_dir() {
				let _ = std::fs::remove_dir_all(dot_wrangler_folder);
			}
		}
		rclone::audio_astronomy487_com();
	}
}

fn distri_clean() {
	let filezone = std::path::Path::new(globals::filezone());
	let dirs = [
		"private/flac",
		"private/mp3",
		"private/jpg",
		"private/png",
		"audio.astronomy487.com/mp3",
		"music.astronomy487.com",
		"audio.astronomy487.com/flac"
	];
	let mut total_bytes: u64 = 0;
	for dir in dirs {
		let path = filezone.join(dir);
		total_bytes += fileops::dir_size_recursive(&path);
	}
	let total_bytes_as_text = fileops::format_file_size(total_bytes);
	println!(
		"This will delete {}{}{} of cached files across several directories.",
		globals::ANSI_RED,
		total_bytes_as_text,
		globals::ANSI_RESET
	);
	if globals::ask_to_continue() {
		globals::log_3(
			"Deleting",
			"",
			format!("{} of cached files", total_bytes_as_text),
			globals::ANSI_RED
		);
		for dir in dirs {
			fileops::clear_directory(&filezone.join(dir));
		}
	}
}

fn distri_help() {
	println!(
		"distri v{}{} usage",
		env!("CARGO_PKG_VERSION"),
		if cfg!(debug_assertions) {
			" (debug build)"
		} else {
			""
		},
	);
	for (command, description, color) in [
		(
			"validate",
			"Validate discog.json without encoding or building anything.",
			globals::ANSI_GREEN
		),
		(
			"encode",
			"Encode audio for audio.astronomy487.com.",
			globals::ANSI_CYAN
		),
		(
			"build",
			"Build static website for music.astronomy487.com.",
			globals::ANSI_BLUE
		),
		(
			"clean",
			"Clean out non-source files from the directory.",
			globals::ANSI_RED
		),
		(
			"publish",
			"Encode, build, and publish content to Cloudflare R2 and Pages.",
			globals::ANSI_PURPLE
		)
	] {
		println!(
			"distri {}{:<11}{}{}",
			color,
			command,
			globals::ANSI_RESET,
			description
		);
	}
}

fn check_if_can_run() {
	let mut can_run = true;

	url::UrlSet::check_valid_icons();

	let mut missing_paths = Vec::new();
	for directory in [
		"audio.astronomy487.com/mp3",
		"audio.astronomy487.com/flac",
		"private/flac",
		"private/jpg",
		"private/mp3",
		"private/png",
		"source/discog.json",
		"source/audio",
		"source/image",
		"source/lyrics"
	] {
		let path = std::path::Path::new(globals::filezone()).join(directory);
		if !path.exists() {
			missing_paths.push(directory);
			can_run = false;
		}
	}
	if !missing_paths.is_empty() {
		globals::log_2(
			"Using",
			globals::filezone().replace("\\", "/"),
			globals::ANSI_GREEN
		);
		for missing_path in &missing_paths {
			globals::log_2(
				"Missing",
				format!("Path \"{}\"", missing_path),
				globals::ANSI_RED
			);
			can_run = false;
		}
	}
	std::mem::drop(missing_paths);

	for cmd in ["ffmpeg.exe", "wrangler.cmd", "rclone.exe"] {
		if !globals::is_in_path(cmd) {
			globals::log_2(
				"Missing",
				format!("Executable \"{}\"", cmd),
				globals::ANSI_RED
			);
			can_run = false;
		}
	}

	assert!(can_run, "Cannot continue with missing prerequisites");
}

fn set_panic_hook() {
	std::panic::set_hook(Box::new(|info| {
		eprintln!("\x07");

		if let Some(location) = info.location() {
			let short_file_name = std::path::Path::new(location.file())
				.file_name()
				.expect("Couldn't find short file name for panic hook")
				.to_string_lossy();
			if cfg!(debug_assertions) {
				eprintln!(
					"{}Error at {}:{}:{}{}",
					globals::ANSI_RED,
					short_file_name,
					location.line(),
					location.column(),
					globals::ANSI_RESET
				);
			} else {
				eprintln!("{}Error{}", globals::ANSI_RED, globals::ANSI_RESET);
			}
		}

		let payload = info.payload();
		let maybe_msg = if let Some(s) = payload.downcast_ref::<&str>() {
			Some((*s).to_string())
		} else {
			payload.downcast_ref::<String>().cloned()
		};
		if let Some(msg) = maybe_msg {
			eprintln!("{}", msg);
		}
		std::process::exit(1);
	}));
}
