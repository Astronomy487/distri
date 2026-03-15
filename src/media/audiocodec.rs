#[derive(Debug)]
pub enum AudioCodec {
	Mp3,
	Flac
}
impl AudioCodec {
	pub fn ext(&self) -> &'static str {
		match self {
			AudioCodec::Mp3 => "mp3",
			AudioCodec::Flac => "flac"
		}
	}
	pub fn ffmpeg_args(&self, input: &str, output: &str) -> Vec<String> {
		match self {
			AudioCodec::Mp3 => vec![
				"-y".into(),
				"-i".into(),
				input.into(),
				"-codec:a".into(),
				"libmp3lame".into(),
				"-b:a".into(),
				"320k".into(),
				"-map_metadata".into(),
				"-1".into(),
				output.into(),
			],
			AudioCodec::Flac => vec![
				"-y".into(),
				"-i".into(),
				input.into(),
				"-codec:a".into(),
				"flac".into(),
				"-compression_level".into(),
				"8".into(),
				"-map_metadata".into(),
				"-1".into(),
				output.into(),
			]
		}
	}
}
