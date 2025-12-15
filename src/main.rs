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
#![deny(clippy::unwrap_used)]
#![deny(unsafe_code)]

mod color;
mod date;
mod fileops;
mod globals;
mod imagedeal;
mod language;
mod lyric;
mod musicdata;
mod rclone;
mod rss;
mod url;
mod wrangle;

fn main() {
	set_panic_hook();
	check_if_can_run();
	
	let mut args = std::env::args().skip(1);
	match args.next().as_deref() {
		Some("validate") => distri_encode(false, false),
		Some("encode") => distri_encode(true, false),
		Some("build") => distri_encode(false, true),
		Some("publish") => distri_publish(),
		Some("clean") => distri_clean(),
		Some("logo") => globals::print_logo(),
		_ => distri_help()
	}

	std::process::exit(0);
}
fn distri_encode(build_r2_bucket: bool, build_static_website: bool) {
	let (all_albums, all_remixes) = musicdata::get_music_data(
		&std::path::Path::new(globals::filezone())
			.join("source")
			.join("discog")
			.with_extension("json")
	);

	if build_r2_bucket {
		globals::log_3("Building", "", "audio.astronomy487.com", globals::ANSI_CYAN);
		for album in &all_albums {
			album.try_encode(&all_albums);
		}
		for remix in &all_remixes {
			remix.try_encode(&all_albums);
		}
	}

	if build_static_website {
		globals::log_3("Building", "", "music.astronomy487.com", globals::ANSI_CYAN);
		fileops::clear_directory(
			&std::path::Path::new(globals::filezone()).join("music.astronomy487.com")
		);
		for album in &all_albums {
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

		fileops::copy_recursive(
			&std::path::Path::new(globals::filezone())
				.join("source")
				.join("webassets"),
			&std::path::Path::new(globals::filezone()).join("music.astronomy487.com")
		);
		fileops::copy_recursive(
			&std::path::Path::new(globals::filezone())
				.join("private")
				.join("jpg"),
			&std::path::Path::new(globals::filezone()).join("music.astronomy487.com")
		);
		std::fs::write(
			std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.join("discog")
				.with_extension("js"),
			format!(
				"let discog = {};",
				std::fs::read_to_string(
					std::path::Path::new(globals::filezone())
						.join("source")
						.join("discog")
						.with_extension("json")
				)
				.expect("Couldn't read discog.json")
			)
		)
		.expect("Couldn't write discog.js");
		rss::make_rss(&all_albums);
	}
}

fn distri_publish() {
	let connected_to_internet = std::net::TcpStream::connect_timeout(
		&"1.1.1.1:80"
			.parse()
			.expect("Couldn't setup ping to 1.1.1.1"),
		std::time::Duration::from_secs(3)
	)
	.is_ok();
	if !connected_to_internet {
		panic!("Cannot connect to the internet right now");
	} else {
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
		"private/about",
		"private/flac",
		"private/mp3",
		"private/jpg",
		"music.astronomy487.com",
		"audio.astronomy487.com/mp3",
		"audio.astronomy487.com/flac"
	];
	let mut total_bytes: u64 = 0;
	for dir in dirs {
		let path = filezone.join(dir);
		total_bytes += fileops::dir_size_recursive(&path);
	}
	let total_bytes = fileops::format_file_size(total_bytes);
	println!(
		"This will delete {}{}{} of cached files across several directories.",
		globals::ANSI_CYAN,
		total_bytes,
		globals::ANSI_RESET
	);
	if globals::ask_to_continue() {
		globals::log_3(
			"Deleting",
			"",
			format!("{} of cached files", total_bytes),
			globals::ANSI_CYAN
		);
		for dir in dirs {
			fileops::clear_directory(&filezone.join(dir));
		}
	}
}

fn distri_help() {
	println!("distri usage:");
	for (command_name, description) in [
		(
			"validate",
			"Validate discog.json without encoding anything."
		),
		("encode", "Encode audio for audio.astronomy487.com."),
		("build", "Build static website for music.astronomy487.com."),
		(
			"clean",
			"Clean out non-source files from the directory. Re-encoding everything will take a while, so be careful!"
		),
		(
			"publish",
			"Publish content to Cloudflare R2 bucket and pages workers."
		)
	] {
		let description = wrap(description, 35);
		for (i, line) in description.iter().enumerate() {
			globals::log_2(
				if i == 0 {
					"distri ".to_string() + command_name
				} else {
					"".to_string()
				},
				line,
				globals::ANSI_YELLOW
			);
		}
	}
	println!(
		"{}distri v{}{}",
		globals::ANSI_GRAY,
		std::env!("CARGO_PKG_VERSION"),
		globals::ANSI_RESET
	);
}

fn check_if_can_run() {
	let mut can_run = true;
	
	let mut missing_paths = Vec::new();
	for d in [
		"audio.astronomy487.com/mp3",
		"audio.astronomy487.com/flac",
		"music.astronomy487.com",
		"private/about",
		"private/flac",
		"private/jpg",
		"private/mp3",
		"source/discog.json",
		"source/webassets",
		"source/audio",
		"source/image"
	] {
		let p = std::path::Path::new(globals::filezone()).join(d);
		if !p.exists() {
			missing_paths.push(d);
			can_run = false;
		}
	}
	if !missing_paths.is_empty() {
		globals::log_2("Using", globals::filezone(), globals::ANSI_MAGENTA);
		for missing_path in &missing_paths {
			globals::log_2(
				"Missing",
				format!("Path \"{}\"", missing_path),
				globals::ANSI_YELLOW
			);
			can_run = false;
		}
	}
	std::mem::drop(missing_paths);

	for (cmd, version_check) in [
		("ffmpeg.exe", "-version"),
		("wrangler.cmd", "--version"),
		("rclone.exe", "--version")
	] {
		match std::process::Command::new(cmd)
			.arg(version_check)
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status()
		{
			Ok(s) if s.success() => {}
			_ => {
				globals::log_2(
					"Missing",
					format!("Executable \"{}\"", cmd),
					globals::ANSI_YELLOW
				);
				can_run = false;
			}
		}
	}
	
	// check for credentials
	if !std::process::Command::new("wrangler.cmd")
		.arg("whoami")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.map(|s| s.success())
		.unwrap_or(false)
	{
		globals::log_2(
			"Missing",
			"Credentials for wrangler; run `wrangler login`",
			globals::ANSI_YELLOW
		);
	}
	if !std::process::Command::new("rclone")
		.arg("lsd")
		.arg("audio-astronomy487-com:")
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.map(|s| s.success())
		.unwrap_or(false)
	{
		globals::log_2(
			"Missing",
			"Credentials for rclone; run `rclone config`",
			globals::ANSI_YELLOW
		);	
	}

	if !can_run {
		panic!("Cannot continue with missing prerequisites");
	}
}

fn set_panic_hook() {
	std::panic::set_hook(Box::new(|info| {
		if let Some(location) = info.location() {
			let short_file_name = std::path::Path::new(location.file())
				.file_stem()
				.expect("Couldn't find short file name for panic hook")
				.to_string_lossy();
			eprintln!(
				"\n{}{}:{}:{}{}",
				globals::ANSI_GRAY,
				short_file_name,
				location.line(),
				location.column(),
				globals::ANSI_RESET
			);
		}

		let payload = info.payload();
		let msg = if let Some(s) = payload.downcast_ref::<&str>() {
			Some((*s).to_string())
		} else {
			payload.downcast_ref::<String>().cloned()
		};
		if let Some(msg) = msg {
			eprintln!("{}", msg);
		}

		std::process::exit(487);
	}));
}

fn wrap(text: &'static str, max_width: usize) -> Vec<String> {
	let mut lines = Vec::new();
	let mut current = String::new();

	for word in text.split_whitespace() {
		if current.len() + word.len() + 1 > max_width {
			lines.push(current.trim_end().to_string());
			current = String::new();
		}
		current.push_str(word);
		current.push(' ');
	}

	if !current.is_empty() {
		lines.push(current.trim_end().to_string());
	}

	lines
}
