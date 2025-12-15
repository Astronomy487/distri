use crate::globals;

pub fn audio_astronomy487_com() {
	globals::log_2("Publishing", "audio.astronomy487.com", globals::ANSI_CYAN);

	let local_path = std::path::Path::new(globals::filezone())
		.join("audio.astronomy487.com")
		.to_string_lossy()
		.to_string();

	let mut child = std::process::Command::new("rclone.exe")
		.arg("sync")
		.arg(local_path)
		.arg("audio-astronomy487-com:audio-astronomy487-com")
		.arg("--verbose")
		.arg("--progress")
		.arg("--stats-one-line")
		.stdout(std::process::Stdio::piped())
		.stderr(std::process::Stdio::piped())
		.spawn()
		.expect("Could not spawn rclone process");

	let stdout = child.stdout.take().expect("Failed to capture stdout");
	let reader = std::io::BufReader::new(stdout);

	#[allow(clippy::manual_flatten)]
	for line in std::io::BufRead::lines(reader) {
		if let Ok(text) = line {
			let t = text.trim();
			if t.contains("Copied (new)") {
				globals::log_2("Putting", t, globals::ANSI_YELLOW);
			} else if t.contains("Deleted") {
				globals::log_2("Removing", t, globals::ANSI_YELLOW);
			} else if t.contains("Updated") {
				globals::log_2("Putting", t, globals::ANSI_YELLOW);
			}
		}
	}

	let status = child.wait().expect("Publish with rclone failed.");

	if !status.success() {
		panic!("Publish with rclone failed.");
	}
}
