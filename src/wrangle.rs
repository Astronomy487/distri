use crate::globals;

pub fn music_astronomy487_com() {
	globals::log_2("Publishing", "music.astronomy487.com", globals::ANSI_CYAN);

	let output = std::process::Command::new("wrangler.cmd")
		.arg("pages")
		.arg("deploy")
		.arg(
			std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.to_string_lossy()
				.to_string()
		)
		.arg("--project-name")
		.arg(globals::CF_PAGES_NAME)
		.arg("--commit-message")
		.arg("Publish from distri")
		.output()
		.expect("Could not execute wrangler command");

	if !output.status.success() {
		let mut msg = String::new();

		msg.push_str("Wrangler deployment failed:\n\n");

		msg.push_str("stdout:\n");
		msg.push_str(&String::from_utf8_lossy(&output.stdout));
		msg.push_str("\n\nstderr:\n");
		msg.push_str(&String::from_utf8_lossy(&output.stderr));

		panic!("{}", msg);
	}
}
