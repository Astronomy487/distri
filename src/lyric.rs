use crate::globals;
use crate::language;

#[derive(Debug)]
pub enum TextCodec {
	Txt,
	Srt,
	Lrc
}
impl TextCodec {
	pub fn ext(&self) -> &'static str {
		match self {
			TextCodec::Txt => "txt",
			TextCodec::Srt => "srt",
			TextCodec::Lrc => "lrc"
		}
	}
}

fn parse_time(raw: &str, line: &str, text: &str) -> f32 {
	let mut split = raw.split('.');
	let maybe_whole = split.next();
	let maybe_frac = split.next();
	let valid = matches!((maybe_whole, maybe_frac, split.next()),
		(Some(whole), Some(frac), None)
			if !whole.is_empty()
			&& whole.chars().all(|c| c.is_ascii_digit())
			&& frac.len() == 6
			&& frac.chars().all(|c| c.is_ascii_digit())
	);
	assert!(
		valid,
		"Invalid timestamp format \"{}\" in lyric line \"{}\"\n\n{}",
		raw, line, text
	);
	let number = raw.parse::<f32>().unwrap_or_else(|_| {
		panic!(
			"Couldn't read timestamp \"{}\" in lyric line \"{}\"\n\n{}",
			raw, line, text
		)
	});
	assert!(!number.is_nan());
	assert!(number >= 0.0);
	number
}

#[derive(Debug)]
pub struct Lyrics {
	stanzas: Vec<Vec<LyricLine>>
}
impl Lyrics {
	pub fn from(text: &str) -> Lyrics {
		assert!(
			!text.starts_with(char::is_whitespace),
			"Lyric text has untrimmed whitespace at start\n\n{}",
			text
		);
		assert!(
			!text.ends_with(char::is_whitespace),
			"Lyric text has untrimmed whitespace at end\n\n{}",
			text
		);
		let mut last_lang: Option<language::Language> = None;
		let mut last_vocalist: Option<String> = None;
		let vvll: Vec<Vec<LyricLine>> = text
			.replace("\r", "")
			.split("\n\n")
			.map(|stanza| {
				stanza
					.lines()
					.filter_map(|line| {
						if line.is_empty() {
							return None;
						}
						let parts: Vec<&str> = line.split('\t').collect();
						assert!(
							parts.len() >= 3,
							"Invalid lyric line \"{}\"\n\n{}",
							line,
							text
						);
						let start_and_end = parts[0..2]
							.iter()
							.map(|raw| parse_time(raw, line, text))
							.collect::<Vec<_>>();
						let (start, end) = (start_and_end[0], start_and_end[1]);
						let the_text = parts[2].to_string();
						assert!(
							!the_text.starts_with(char::is_whitespace),
							"Lyric line \"{}\" ({}-{}) must not have whitespace at start\n\n{}",
							the_text,
							start,
							end,
							text
						);
						assert!(
							!the_text.ends_with(char::is_whitespace),
							"Lyric line \"{}\" ({}-{}) must not have whitespace at end\n\n{}",
							the_text,
							start,
							end,
							text
						);
						let illegal_characters = "–“”‘’()（）\r\t\n";
						assert!(
							the_text.chars().all(|c| !illegal_characters.contains(c)),
							"Lyric line \"{}\" contains illegal character ({})\n\n{}",
							the_text,
							illegal_characters,
							text
						);
						assert!(
							the_text.chars().count() <= 100,
							"Lyric line \"{}\" is too long ({} chars)\n\n{}",
							the_text,
							the_text.len(),
							text
						);
						// vvv messy capitalization validation
						if let Some(c) = the_text
							.chars()
							.find(|c| c.is_alphabetic() || c.is_ascii_digit())
						{
							if c.is_ascii_digit() {
							} else if c.to_uppercase().ne(c.to_lowercase()) {
								assert!(
									c.is_uppercase(),
									"Lyric line \"{}\" must start with a capitalized letter or digit\n\n{}",
									the_text,
									text
								);
							}
						}

						let mut language_override: Option<language::Language> = None;
						let mut vocalist_override: Option<String> = None;

						for kv in &parts[3..] {
							if let Some((key, value)) = kv.split_once(':') {
								match key {
									"language" => {
										let new_language = language::Language::from(value);
										if let Some(old_language) = language_override
											&& old_language == new_language
										{
											panic!(
												"Lyrics have redundant language:{} tag\n\n{}",
												value, text
											);
										}
										language_override = Some(new_language);
									}
									"vocalist" => {
										let new_vocalist = value.to_string();
										assert!(
											!new_vocalist.starts_with(char::is_whitespace),
											"Vocalist \"{}\" must not have whitespace at start\n\n{}",
											new_vocalist,
											text
										);
										assert!(
											!new_vocalist.ends_with(char::is_whitespace),
											"Vocalist \"{}\" must not have whitespace at end\n\n{}",
											new_vocalist,
											text
										);
										if new_vocalist.to_lowercase() == "unknown" {
											globals::log_2(
												"Warning",
												format!(
													"Vocalist should not be \"{}\"; vocalist unused for now",
													new_vocalist
												),
												globals::ANSI_RED
											);
											globals::log_2(
												"",
												format!("Line: \"{}\"", the_text),
												globals::ANSI_RED
											);
										}
										if let Some(old_vocalist) = vocalist_override
											&& old_vocalist == new_vocalist
										{
											panic!(
												"Lyrics have redundant vocalist:{} tag\n\n{}",
												new_vocalist, text
											);
										}
										vocalist_override = Some(new_vocalist);
									}
									_ => panic!("Invalid lyric tag \"{}\"\n\n{}", key, text)
								}
							}
						}
						let language = if let Some(lang) = language_override {
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
						assert!(
							!the_text.is_empty(),
							"Empty lyric line ({} to {})",
							start,
							end
						);
						Some(LyricLine {
							start,
							end,
							text: the_text,
							language,
							vocalist
						})
					})
					.collect()
			})
			.collect();
		for vll in &vvll {
			assert!(
				!vll.is_empty(),
				"Stanza has no lines. Lyric text:\n```\n{}\n```",
				text
			);
		}
		assert!(
			!vvll.is_empty(),
			"Stanza has no lines. Lyric text:\n```\n{}\n```",
			text
		);

		// validate timing attributes
		let mut prev = None;
		for stanza in &vvll {
			for line in stanza {
				assert!(
					line.start < line.end,
					"Invalid lyric timing on line \"{}\" ({} !< {})",
					line.text,
					line.start,
					line.end
				);
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
	pub fn as_filetype(&self, codec: TextCodec) -> String {
		match codec {
			TextCodec::Txt => self
				.stanzas
				.iter()
				.map(|stanza_lines| {
					stanza_lines
						.iter()
						.map(LyricLine::to_unsynced)
						.collect::<Vec<_>>()
						.join("\n")
				})
				.collect::<Vec<_>>()
				.join("\n\n"),
			TextCodec::Lrc => self
				.stanzas
				.iter()
				.map(|stanza_lines| {
					let stanza_lines_text: Vec<String> =
						stanza_lines.iter().map(LyricLine::to_synced_text).collect();
					stanza_lines_text.join("\n")
				})
				.collect::<Vec<_>>()
				.join("\n\n"),
			TextCodec::Srt => {
				fn format_time(time: f32) -> String {
					let ms = (time * 1000.0).round() as u64;
					format!(
						"{:02}:{:02}:{:02},{:03}",
						ms / 3_600_000,
						(ms % 3_600_000) / 60_000,
						(ms % 60_000) / 1000,
						ms % 1000
					)
				}
				fn split_two_lines(line: &str) -> String {
					const MAX: usize = 50;
					if line.chars().count() <= MAX {
						return line.to_string();
					}
					let char_indices: Vec<(usize, char)> = line.char_indices().collect();
					let char_len = char_indices.len();
					let mid_char = char_len / 2;
					let mid_byte = char_indices[mid_char].0;
					let maybe_right = line[mid_byte..].find(' ').map(|o| mid_byte + o);
					let maybe_left = line[..mid_byte].rfind(' ');
					let index = match (maybe_left, maybe_right) {
						(_, Some(right)) if right.saturating_sub(mid_byte) <= 20 => right,
						(Some(left), _) => left,
						(_, Some(right)) => right,
						_ => mid_byte
					};
					let (left, right) = line.split_at(index);
					format!("{}\n{}", left.trim(), right.trim())
				}
				let lines: Vec<&LyricLine> = self.stanzas.iter().flatten().collect();

				let mut out = Vec::new();
				let mut counter = 1;
				for index in 0..lines.len() {
					let line = lines[index];
					let mut end = line.end;
					if let Some(next) = lines.get(index + 1)
						&& next.start > line.start
						&& next.start - end < 1.0
					// if this one ends within 1.0s of the next starting, just join them together
					{
						end = next.start;
					}
					out.push(format!(
						"{}\n{} --> {}\n{}\n",
						counter,
						format_time(line.start),
						format_time(end),
						split_two_lines(&line.text)
					));
					counter += 1;
				}
				out.join("\n")
			}
		}
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

#[derive(Debug)]
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
