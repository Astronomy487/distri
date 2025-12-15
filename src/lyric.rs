use crate::language;

pub struct Lyrics {
	stanzas: Vec<Vec<LyricLine>>
}
impl Lyrics {
	pub fn from(text: &str) -> Lyrics {
		let mut last_lang: Option<language::Language> = None;
		let mut last_vocalist: Option<String> = None;
		let vvll: Vec<Vec<LyricLine>> = text
			.split("\n\n")
			.map(|stanza| {
				stanza
					.lines()
					.filter_map(|line| {
						if line.is_empty() {
							return None;
						}
						let parts: Vec<&str> = line.split('\t').collect();
						if parts.len() < 3 {
							panic!("Invalid lyric line \"{}\"", line);
						}
						let start = parts[0].parse::<f32>().ok()?;
						let end = parts[1].parse::<f32>().ok()?;
						let text = parts[2].to_string();

						let mut lang_override: Option<language::Language> = None;
						let mut vocalist_override: Option<String> = None;

						for kv in &parts[3..] {
							if let Some((key, value)) = kv.split_once(":") {
								match key {
									"language" => {
										lang_override = Some(language::Language::from(value))
									}
									"vocalist" => vocalist_override = Some(value.to_string()),
									_ => panic!("Invalid lyric tag \"{}\"", key)
								}
							}
						}
						let language = if let Some(lang) = lang_override {
							last_lang = Some(lang);
							lang
						} else if let Some(lang) = last_lang {
							lang
						} else {
							panic!("First lyric line (\"{}\") has no language tag", line);
						};
						let vocalist = if let Some(v) = vocalist_override {
							last_vocalist = Some(v.clone());
							v
						} else if let Some(v) = &last_vocalist {
							v.clone()
						} else {
							panic!("First lyric line (\"{}\") has no vocalist tag", line);
						};
						if text.is_empty() {
							panic!("Empty lyric line ({} to {})", start, end);
						}
						Some(LyricLine {
							start,
							end,
							text,
							language,
							vocalist
						})
					})
					.collect()
			})
			.collect();
		for vll in &vvll {
			if vll.is_empty() {
				panic!("Stanza has no lines. Lyric text:\n```\n{}\n```", text);
			}
		}
		if vvll.is_empty() {
			panic!("Stanza has no lines. Lyric text:\n```\n{}\n```", text);
		}

		// validate timing attributes
		let mut prev = None;
		for stanza in &vvll {
			for line in stanza {
				if line.start >= line.end {
					panic!(
						"Invalid lyric timing on line \"{}\" ({} !< {})",
						line.text, line.start, line.end
					);
				}
				if let Some((prev_time, prev_text)) = prev
					&& prev_time > line.start
				{
					panic!(
						"Invalid lyric timing between lines \"{}\" and line \"{}\" ({} !<= {})",
						prev_text, line.text, prev_time, line.start
					);
				}
				prev = Some((line.end, &line.text));
			}
		}

		Lyrics { stanzas: vvll }
	}
	pub fn as_plaintext(&self) -> String {
		self.stanzas
			.iter()
			.map(|stanza_lines| {
				stanza_lines
					.iter()
					.map(|line| line.to_unsynced())
					.collect::<Vec<_>>()
					.join("\n")
			})
			.collect::<Vec<_>>()
			.join("\n\n")
	}
	pub fn as_lrc(&self) -> String {
		self.stanzas
			.iter()
			.map(|stanza_lines| {
				let stanza_lines_text: Vec<String> = stanza_lines
					.iter()
					.map(|line| line.to_synced_text())
					.collect();
				stanza_lines_text.join("\n")
			})
			.collect::<Vec<_>>()
			.join("\n\n")
	}
	pub fn as_sylt_data(&self) -> Vec<(u32, String)> {
		let mut synced_content: Vec<(u32, String)> = Vec::new();
		for stanza_lines in &self.stanzas {
			for line in stanza_lines {
				synced_content.push(line.to_synced_pair());
			}
			if let Some(last_line) = stanza_lines.last() {
				synced_content.push((last_line.end_ms(), String::new()));
			}
		}
		synced_content
	}
	pub fn most_common_language(&self) -> language::Language {
		let mut counts: std::collections::HashMap<language::Language, usize> =
			std::collections::HashMap::new();
		for group in &self.stanzas {
			for line in group {
				let weight = line.text.len();
				*counts.entry(line.language).or_insert(0) += weight;
			}
		}
		counts
			.into_iter()
			.max_by_key(|(_, weight)| *weight)
			.map(|(lang, _)| lang)
			.expect("Could not find most common language for lyrics without lyrics")
	}
}

enum StanzaType {
	Intro,
	Outro,
	Chorus,
	PreChorus,
	PostChorus,
	Verse,
	Bridge
}

struct LyricLine {
	start: f32,
	end: f32,
	text: String,
	language: language::Language,
	vocalist: String
}
impl LyricLine {
	fn start_ms(&self) -> u32 {
		(self.start * 1000.0).round() as u32
	}
	fn end_ms(&self) -> u32 {
		(self.end * 1000.0).round() as u32
	}
	fn to_unsynced(&self) -> String {
		self.text.clone()
	}
	fn to_synced_pair(&self) -> (u32, String) {
		(self.start_ms(), self.text.clone())
	}
	fn to_synced_text(&self) -> String {
		let total_ms = self.start_ms();
		let minutes = total_ms / 60000;
		let seconds = (total_ms % 60000) / 1000;
		let hundredths = (total_ms % 1000) / 10; // two‑digit fraction
		format!(
			"[{:02}:{:02}.{:02}] {}",
			minutes, seconds, hundredths, self.text
		)
	}
}
