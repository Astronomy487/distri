use crate::globals;

pub fn audio_astronomy487_com() {
	globals::log_2("Publishing", "audio.astronomy487.com", globals::ANSI_PURPLE);

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
			let trimmed = text.trim();
			if trimmed.contains("Copied (new)") {
				globals::log_2("Putting", trimmed, globals::ANSI_PURPLE);
			} else if trimmed.contains("Deleted") {
				globals::log_2("Removing", trimmed, globals::ANSI_PURPLE);
			} else if trimmed.contains("Updated") {
				globals::log_2("Putting", trimmed, globals::ANSI_PURPLE);
			}
		}
	}

	let status = child.wait().expect("Publish with rclone failed.");

	assert!(status.success(), "Publish with rclone failed.");
}
