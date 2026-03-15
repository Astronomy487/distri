use crate::build::smartquotes;
use crate::build::xml::XmlNode;
use crate::types::language::Language;

const MAX_LINE_LENGTH: usize = 100;
const MAX_LINE_LENGTH_FOR_SRT: usize = 65;
const LYRIC_FILL_FORWARDS_MARGIN_SECONDS: f64 = 0.0; // i never know what's best

#[derive(Debug)]
pub enum TextCodec {
	Txt,
	Srt,
	Lrc,
	Vtt,
	Tsv
}
pub const ALL_TEXT_CODECS: [TextCodec; 5] = [
	TextCodec::Txt,
	TextCodec::Srt,
	TextCodec::Lrc,
	TextCodec::Vtt,
	TextCodec::Tsv
];
impl TextCodec {
	pub fn ext(&self) -> &'static str {
		match self {
			TextCodec::Txt => "txt",
			TextCodec::Srt => "srt",
			TextCodec::Lrc => "lrc",
			TextCodec::Vtt => "vtt",
			TextCodec::Tsv => "tsv"
		}
	}
	pub fn description(&self) -> &'static str {
		match self {
			TextCodec::Txt => "Plain text",
			TextCodec::Srt => "SRT",
			TextCodec::Lrc => "LRC",
			TextCodec::Vtt => "VTT",
			TextCodec::Tsv => "TSV"
		}
	}
}

fn parse_time(raw_text: &str, line: &str, text: &str) -> f64 {
	let mut split = raw_text.split('.');
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
		raw_text, line, text
	);
	let number = raw_text.parse::<f64>().unwrap_or_else(|_| {
		panic!(
			"Couldn't read timestamp \"{}\" in lyric line \"{}\"\n\n{}",
			raw_text, line, text
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
	pub fn all_vocalist_sets(&self) -> Vec<&VocalistSet> {
		let mut vocalist_sets: Vec<&VocalistSet> = Vec::new();
		for stanza in &self.stanzas {
			for line in stanza {
				if !vocalist_sets.contains(&&line.vocalist_set) {
					// && lol
					vocalist_sets.push(&line.vocalist_set);
				}
			}
		}
		vocalist_sets
	}
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
		let mut last_lang: Option<Language> = None;
		let mut last_vocalist_set: Option<VocalistSet> = None;
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
							the_text.chars().count() <= MAX_LINE_LENGTH,
							"Lyric line \"{}\" is too long ({} chars > {})\n\n{}",
							the_text,
							the_text.len(),
							MAX_LINE_LENGTH,
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

						let mut language_override: Option<Language> = None;
						let mut vocalist_set_override: Option<VocalistSet> = None;

						let mut vocalists_collected_this_line: Vec<String> = Vec::new();
						for kv in &parts[3..] {
							if let Some((key, value)) = kv.split_once(':') {
								match key {
									"language" => {
										let new_language = Language::from(value);
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
										/* if new_vocalist.to_lowercase() == "unknown" {
											crate::globals::log_2(
												"Warning",
												format!("Unknown vocalist for line \"{}\"", the_text),
												crate::globals::ANSI_RED
											);
										} */
										vocalists_collected_this_line.push(new_vocalist);
									}
									_ => panic!("Invalid lyric tag \"{}\"\n\n{}", key, text)
								}
							} else {
								panic!(
									"Lyric line has extra columns that can't be read\nLine: \"{}\"",
									the_text
								);
							}
						}
						if !vocalists_collected_this_line.is_empty() {
							vocalist_set_override =
								Some(VocalistSet::from(vocalists_collected_this_line));
						}

						// use *_override to figure out language and vocalist
						let language = if let Some(lang) = language_override {
							last_lang = Some(lang);
							lang
						} else if let Some(lang) = last_lang {
							lang
						} else {
							panic!("First lyric line (\"{}\") has no language tag", line);
						};
						let vocalist_set = if let Some(v) = vocalist_set_override {
							last_vocalist_set = Some(v.clone());
							v
						} else if let Some(v) = &last_vocalist_set {
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
						// extra validation for style because i care about this
						for (banned_sequence, ok_supersequences, suggestion) in [
							("cause", vec!["'cause", "because"], "'cause"),
							("in'", vec!["ain't"], "ing"),
							("'em", vec![], "them"),
							("'bout", vec![], "about"),
							("'round", vec![], "round"),
							("aingt", vec![], "ain't"),
							("'til", vec![], "till"),
							("c'mon", vec![], "come on")
						] {
							if the_text.to_lowercase().contains(banned_sequence) {
								assert!(
									ok_supersequences.iter().any(|ok_supersequence| the_text
										.to_lowercase()
										.contains(ok_supersequence)),
									"Lyric line contains the banned sequence \"{}\"; prefer \"{}\"\n{}\n\n{}",
									banned_sequence,
									suggestion,
									the_text,
									text
								);
							}
						}
						Some(LyricLine {
							start,
							end,
							text: the_text,
							language,
							vocalist_set
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
				fn format_time(time: f64) -> String {
					let ms = (time * 1000.0).round() as u64;
					format!(
						"{:02}:{:02}:{:02},{:03}",
						ms / 3_600_000,
						(ms % 3_600_000) / 60_000,
						(ms % 60_000) / 1000,
						ms % 1000
					)
				}
				let lines: Vec<&LyricLine> = self.stanzas.iter().flatten().collect();

				fn split_line_if_needed(original: &str) -> String {
					if original.len() <= MAX_LINE_LENGTH_FOR_SRT {
						return original.to_string();
					}
					let len = original.len();
					let mid = len / 2;
					let mut best_split: Option<usize> = None;
					let mut best_dist = usize::MAX;
					for (byte_idx, ch) in original.char_indices() {
						if ch.is_whitespace() {
							let dist = byte_idx.abs_diff(mid);
							if dist < best_dist {
								best_dist = dist;
								best_split = Some(byte_idx);
							}
						}
					}
					let split_idx = if let Some(idx) = best_split {
						idx
					} else {
						let mut idx = mid;
						while idx > 0 && !original.is_char_boundary(idx) {
							idx -= 1;
						}
						idx
					};

					let (left, right) = original.split_at(split_idx);
					format!("{}\n{}", left.trim_end(), right.trim_start())
				}

				let mut out = Vec::new();
				let mut counter = 1;
				for index in 0..lines.len() {
					let line = lines[index];
					let mut end = line.end;
					if let Some(next) = lines.get(index + 1)
						&& next.start > line.start
						&& next.start - end < LYRIC_FILL_FORWARDS_MARGIN_SECONDS
					// if this one ends within LYRIC_FILL_FORWARDS_MARGIN_SECONDS s of the next starting, just join them together
					{
						end = next.start;
					}
					out.push(format!(
						"{}\n{} --> {}\n{}\n",
						counter,
						format_time(line.start),
						format_time(end),
						split_line_if_needed(&line.text)
					));
					counter += 1;
				}
				out.join("\n")
			}
			TextCodec::Vtt => {
				fn format_time(time: f64) -> String {
					let ms = (time * 1000.0).round() as u64;
					format!(
						"{:02}:{:02}:{:02}.{:03}",
						ms / 3_600_000,
						(ms % 3_600_000) / 60_000,
						(ms % 60_000) / 1000,
						ms % 1000
					)
				}
				let lines: Vec<&LyricLine> = self.stanzas.iter().flatten().collect();

				let mut out = vec!["WEBVTT".to_string()];
				// let mut counter = 1;
				for index in 0..lines.len() {
					let line = lines[index];
					let mut end = line.end;
					if let Some(next) = lines.get(index + 1)
						&& next.start > line.start
						&& next.start - end < LYRIC_FILL_FORWARDS_MARGIN_SECONDS
					// if this one ends within LYRIC_FILL_FORWARDS_MARGIN_SECONDS s of the next starting, just join them together
					{
						end = next.start;
					}
					out.push(format!(
						"{} --> {}\n<v {}>{}",
						// counter,
						format_time(line.start),
						format_time(end),
						line.vocalist_set,
						line.text
					));
					// counter += 1;
				}
				out.join("\n\n")
			}
			TextCodec::Tsv => {
				let mut out = vec![
					"Start (seconds)\tEnd (seconds)\tLanguage (ISO 639-1)\tVocalist\tText"
						.to_string(),
				];
				for stanza in &self.stanzas {
					for line in stanza {
						out.push(format!(
							"{:.6}\t{:.6}\t{}\t{}\t{}",
							line.start,
							line.end,
							line.language.iso_639_1(),
							line.vocalist_set,
							line.text
						));
					}
					out.push(String::new());
				}
				out.join("\n").trim().to_string()
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
	pub fn most_common_language(&self) -> Language {
		let mut counts: std::collections::HashMap<Language, usize> =
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
	pub fn lyric_page_xml(&self) -> XmlNode {
		let primary_language = self.most_common_language();
		let mut l_a = XmlNode::new("l-a").with_attribute("lang", primary_language.iso_639_1());
		// let mut last_stanza_end = None;
		let mut last_vocalist_set = &VocalistSet::empty();
		for (stanza_index, stanza) in self.stanzas.iter().enumerate() {
			if stanza_index > 0 {
				l_a.add_child(XmlNode::new("br"));
			}
			let mut l_s = XmlNode::new("l-s");
			for line in stanza {
				if line.vocalist_set != *last_vocalist_set {
					last_vocalist_set = &line.vocalist_set;
					for (index, one_vocalist) in last_vocalist_set.list.iter().enumerate() {
						l_s.add_child(
							XmlNode::new("l-v")
								.with_text(smartquotes::smart_quotes(&one_vocalist.to_string()))
								.maybe_with_attribute(
									"class",
									if index < last_vocalist_set.list.len() - 1 {
										Some("notlast")
									} else {
										None
									}
								)
						);
					}
				}
				l_s.add_child(
					XmlNode::new("l-l")
						.with_text(smartquotes::smart_quotes(&line.text))
						.maybe_with_attribute(
							"lang",
							if line.language == primary_language {
								None
							} else {
								Some(line.language.iso_639_1())
							}
						)
						.with_attribute("data-start", format!("{:.6}", line.start))
						.with_attribute("data-end", format!("{:.6}", line.end))
				)
			}
			l_a.add_child(l_s);
			// last_stanza_end = Some(stanza.last().expect("Empty stanza? Yeah right").end);
		}
		l_a
	}
}

#[derive(Debug)]
struct LyricLine {
	start: f64,
	end: f64,
	text: String,
	language: Language,
	vocalist_set: VocalistSet
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VocalistSet {
	list: std::rc::Rc<Vec<String>>
}
impl VocalistSet {
	fn from(mut vocalists: Vec<String>) -> Self {
		vocalists.sort();
		Self {
			list: vocalists.into()
		}
	}
	fn empty() -> Self {
		Self {
			list: Vec::new().into()
		}
	}
}
impl std::fmt::Display for VocalistSet {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "{}", self.list.join(", "))
	}
}
/* impl PartialEq for VocalistSet {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.list, &other.list)
	}
} */
