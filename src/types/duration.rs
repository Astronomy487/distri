use crate::globals;

#[derive(Clone, Copy, Debug)]
pub struct Duration {
	milliseconds: u32
}

/* impl std::ops::Add for Duration {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Self {
			milliseconds: self.milliseconds + other.milliseconds
		}
	}
} */

impl Duration {
	pub fn display(&self) -> String {
		let total_seconds = self.seconds();
		let minutes = (total_seconds / 60) % 60;
		let hours = (total_seconds / 3600) % 60;
		let seconds = total_seconds % 60;
		if hours > 0 {
			format!("{}h{:02}m{:02}s", hours, minutes, seconds)
		} else {
			format!("{}m{:02}s", minutes, seconds)
		}
	}
	pub fn from_audio_file_and_validate(
		maybe_parent_album_slug: Option<&str>, song_slug: &str
	) -> Duration {
		// gets this audio file, validates its flac metadata
		// (at least 44.1kHz and 16-bit)
		// then return its duration

		// this code is repeated in musicdata.rs
		let input_file_name = match maybe_parent_album_slug {
			Some(parent_album_slug) => format!("{}/{}", parent_album_slug, song_slug),
			None => song_slug.to_string()
		} + ".flac";
		let input_file = globals::filezone()
			.join("source")
			.join("audio")
			.join(&input_file_name);

		// various audio validation
		let file = std::fs::File::open(&input_file)
			.unwrap_or_else(|_| panic!("Could not find audio source {}", input_file_name));
		let mss = symphonia::core::io::MediaSourceStream::new(
			Box::new(file),
			symphonia::core::io::MediaSourceStreamOptions::default()
		);
		let hint = symphonia::core::probe::Hint::new();
		let probed = symphonia::default::get_probe()
			.format(
				&hint,
				mss,
				&symphonia::core::formats::FormatOptions::default(),
				&symphonia::core::meta::MetadataOptions::default()
			)
			.expect("Symphonia cannot handle flac audio");
		let track = probed
			.format
			.tracks()
			.iter()
			.find(|t| t.codec_params.sample_rate.is_some())
			.expect("Symphonia couldn't process audio");
		let sample_rate = track
			.codec_params
			.sample_rate
			.expect("Symphonia couldn't identify audio sample rate");
		let bit_depth = track
			.codec_params
			.bits_per_sample
			.expect("Symphonia couldn't identify audio bit depth");
		let frames = track
			.codec_params
			.n_frames
			.expect("Symphonia couldn't identify audio length");
		let dur_milliseconds = ((frames * 1000) as f64 / f64::from(sample_rate)).floor() as u32;
		assert!(
			sample_rate >= 44_100,
			"Expected 44.1 kHz (or higher), but file has {} Hz",
			sample_rate
		);
		assert!(
			bit_depth >= 16,
			"Expected 16-bit audio (or higher), but file is {}-bit",
			bit_depth
		);
		Self {
			milliseconds: dur_milliseconds
		}
	}
	pub fn seconds(&self) -> u32 {
		// rounds up or down to nearest integer
		(self.milliseconds + 500) / 1000
	}
	pub fn milliseconds(&self) -> u32 {
		self.milliseconds
	}
	pub fn zero() -> Duration {
		Self { milliseconds: 0 }
	}
	pub fn from_milliseconds(milliseconds: u32) -> Self {
		Self { milliseconds }
	}
	pub fn accumulate(durations: impl Iterator<Item = Duration>) -> Duration {
		let mut total_milliseconds = 0;
		for duration in durations {
			total_milliseconds += duration.milliseconds;
		}
		Self {
			milliseconds: total_milliseconds
		}
	}
}
