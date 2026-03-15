#![deny(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![warn(unused_mut, unused_results, unused_variables, unused_imports)]
#![deny(unsafe_code, unused_unsafe)]
#![deny(unreachable_code, unreachable_patterns)]
#![allow(dead_code)] // i re-enable this every once in a while just to see what's up
#![deny(private_interfaces)]
#![deny(absolute_paths_not_starting_with_crate)]
#![deny(clippy::unwrap_used)]
#![warn(clippy::many_single_char_names)]
#![deny(clippy::shadow_reuse, clippy::shadow_same, clippy::shadow_unrelated)]
#![deny(clippy::cast_lossless)]
#![warn(clippy::manual_assert)]

// #![deny(missing_docs, clippy::missing_docs_in_private_items)]

mod fileops;
mod globals;

mod build;
mod media;
mod types;

fn main() {
	let runtime = std::time::SystemTime::now();

	set_panic_hook();
	check_if_can_run();

	let args: Vec<String> = std::env::args().skip(1).collect();
	if args.is_empty() {
		distri_help();
		return;
	}
	let allowed = ["publish", "validate", "encode", "build", "clean"];
	if args.iter().any(|a| !allowed.contains(&a.as_str())) {
		return distri_help();
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
			return distri_help();
		}
	}

	if let Ok(elapsed) = runtime.elapsed() {
		let duration = crate::types::duration::Duration::from_milliseconds(
			(elapsed.as_secs_f32() * 1_000.0) as u32
		);
		println!(
			"{}Ran in {}{}",
			globals::ANSI_GRAY,
			duration.display(),
			globals::ANSI_RESET
		);
	}
}
fn distri_encode(build_r2_bucket: bool, build_static_website: bool) {
	let just_validating = !build_r2_bucket && !build_static_website;

	let json_location = globals::filezone()
		.join("source")
		.join("discog")
		.with_extension("json");

	let (all_albums, all_remixes, all_assists) = crate::media::get_music_data(&json_location);

	if build_r2_bucket {
		for album in &all_albums {
			album.try_encode(&all_albums);
		}
		for remix in &all_remixes {
			remix.try_encode(&all_albums);
		}
	}

	if build_static_website {
		let music_astronomy487_com = globals::filezone().join("music.astronomy487.com");
		if music_astronomy487_com.exists() {
			fileops::clear_directory(&music_astronomy487_com);
		} else {
			std::fs::create_dir(&music_astronomy487_com)
				.unwrap_or_else(|_| panic!("Couldn't create directory music.astronomy487.com"));
		}

		globals::log_2("Building", "Home page", globals::ANSI_BLUE);
		crate::build::pages::homepage::make_home_page(&all_albums, &all_remixes, &all_assists);

		globals::log_2("Building", "Link pages", globals::ANSI_BLUE);
		for album in &all_albums {
			crate::build::pages::linkpage::make_link_page(
				&crate::media::titlable::Titlable::Album(album),
				&all_albums,
				build_r2_bucket
			);
			if !album.single {
				for song in &album.songs {
					if !song.bonus {
						crate::build::pages::linkpage::make_link_page(
							&crate::media::titlable::Titlable::Song(song),
							&all_albums,
							build_r2_bucket
						);
					}
				}
			}
		}
		for remix in &all_remixes {
			crate::build::pages::linkpage::make_link_page(
				&crate::media::titlable::Titlable::Song(remix),
				&all_albums,
				build_r2_bucket
			);
		}

		globals::log_3("Building", "", "Other web assets", globals::ANSI_BLUE);
		// album art jpgs
		{
			use crate::media::artwork::Artwork;
			let mut artwork_that_needs_copying: std::collections::HashSet<Artwork> =
				std::collections::HashSet::new();
			let _ = artwork_that_needs_copying.insert(Artwork::fallback());
			for album in &all_albums {
				let _ = artwork_that_needs_copying.insert(album.artwork.clone());
				for song in &album.songs {
					if !song.bonus
						&& let Some(art) = &song.artwork
					{
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
			let dest_dir = globals::filezone()
				.join("music.astronomy487.com")
				.join("artwork");
			std::fs::create_dir(&dest_dir).unwrap_or_else(|_| {
				panic!("Couldn't create directory music.astronomy487.com/artwork")
			});
			for artwork in artwork_that_needs_copying {
				artwork.make_jpg_exist();
				let dest = dest_dir
					.join(&artwork.name_without_slash)
					.with_extension("jpg");
				let _ = std::fs::copy(artwork.jpg_path, &dest).unwrap_or_else(|_| {
					panic!(
						"Couldn't copy artwork {}.jpg for site",
						artwork.name_with_slash
					)
				});
			}
		}
		// 8831
		{
			let dest_dir = globals::filezone()
				.join("music.astronomy487.com")
				.join("8831");
			std::fs::create_dir(&dest_dir).unwrap_or_else(|_| {
				panic!("Couldn't create directory music.astronomy487.com/8831")
			});
			for album in &all_albums {
				if album.has_8831 {
					let dest = dest_dir.join(&album.slug).with_extension("gif");
					let _ = std::fs::copy(
						globals::filezone()
							.join("source")
							.join("8831")
							.join(&album.slug)
							.with_extension("gif"),
						&dest
					)
					.unwrap_or_else(|_| panic!("Couldn't copy 8831 {}.gif for site", album.slug));
				}
			}
		}
		// linkpage styles
		fileops::write_file(
			&globals::filezone()
				.join("music.astronomy487.com")
				.join("linkpage-style")
				.with_extension("css"),
			crate::build::minify::compress_css(
				include_str!("assets/linkpage-style.css").to_owned()
					+ &crate::types::urlset::UrlSet::linkpage_css_for_platforms()
			)
		);
		// lyricpage styles
		fileops::write_file(
			&globals::filezone()
				.join("music.astronomy487.com")
				.join("lyricpage-style")
				.with_extension("css"),
			crate::build::minify::compress_css(
				include_str!("assets/lyricpage-style.css").to_owned()
			)
		);
		// lyricpage js
		fileops::write_file(
			&globals::filezone()
				.join("music.astronomy487.com")
				.join("lyricpage-script")
				.with_extension("js"),
			crate::build::minify::compress_js(
				include_str!("assets/lyricpage-script.js").to_owned()
			)
		);
		// fonts
		std::fs::create_dir(
			globals::filezone()
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
				&globals::filezone()
					.join("music.astronomy487.com")
					.join("font")
					.join(font)
					.with_extension("woff2"),
				data
			);
		}
		// icons
		crate::build::icons::put_icons();
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
				&globals::filezone()
					.join("music.astronomy487.com")
					.join(name),
				data
			);
		}
		/* fileops::write_file(
			&globals::filezone()
				.join("music.astronomy487.com")
				.join("discog")
				.with_extension("json"),
			std::fs::read_to_string(&json_location).expect("Couldn't read discog.json")
		); */

		globals::log_3("Building", "", "RSS feed", globals::ANSI_BLUE);
		crate::build::pages::rss::make_rss(&all_albums, &all_remixes, &all_assists);
		globals::log_3("Building", "", "Sitemap", globals::ANSI_BLUE);
		crate::build::pages::sitemap::make_sitemap(&all_albums, &all_remixes, &all_assists);
	}

	if just_validating {
		println!("Validation was successful");
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
		crate::build::publish::wrangle::music_astronomy487_com();
		// wrangler can be talkative - delete its extra directories
		for dir_name in [".wrangler", "node_modules"] {
			let dot_wrangler_folder = std::env::current_dir()
				.expect("Could not find current working directory")
				.join(dir_name);
			if dot_wrangler_folder.is_dir() {
				let _ = std::fs::remove_dir_all(dot_wrangler_folder);
			}
		}
		crate::build::publish::rclone::audio_astronomy487_com();
	}
}

fn distri_clean() {
	let filezone = globals::filezone();
	let dirs = [
		"private/flac",
		"private/mp3",
		"private/jpg",
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

	crate::types::urlset::UrlSet::check_valid_icons();

	let mut missing_paths = Vec::new();
	for directory in [
		"audio.astronomy487.com/mp3",
		"audio.astronomy487.com/flac",
		"private/flac",
		"private/jpg",
		"private/mp3",
		"source/discog.json",
		"source/audio",
		"source/artwork",
		"source/8831",
		"source/lyrics"
	] {
		let path = globals::filezone().join(directory);
		if !path.exists() {
			missing_paths.push(directory);
			can_run = false;
		}
	}
	if !missing_paths.is_empty() {
		globals::log_2("Using", globals::filezone().display(), globals::ANSI_GREEN);
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
