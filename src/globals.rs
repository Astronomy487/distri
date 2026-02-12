static FILEZONE: std::sync::OnceLock<&'static str> = std::sync::OnceLock::new();

pub fn filezone() -> &'static str {
	FILEZONE.get_or_init(|| {
		let zone = std::env::current_dir()
			.expect("Could not get the current working directory")
			.to_string_lossy()
			.into_owned();
		Box::leak(zone.into_boxed_str()) // make FILEZONE's &str live forever
	})
}

pub fn is_in_path(exe: &str) -> bool {
	match std::env::var_os("PATH") {
		Some(path) => path
			.to_string_lossy()
			.split(';')
			.filter_map(|p| {
				let candidate = std::path::Path::new(p).join(exe);
				if candidate.exists() {
					Some(candidate)
				} else {
					None
				}
			})
			.next()
			.is_some(),
		None => false
	}
}

pub const ANSI_RED: &str = "\x1b[91m"; // deletion or missing something or bad stuff
pub const ANSI_YELLOW: &str = "\x1b[93m"; // zipping
pub const ANSI_GREEN: &str = "\x1b[92m"; // validation
pub const ANSI_CYAN: &str = "\x1b[96m"; // encoding
pub const ANSI_BLUE: &str = "\x1b[94m"; // static site
pub const ANSI_PURPLE: &str = "\x1b[95m"; // internet and publishing
pub const ANSI_RESET: &str = "\x1b[0m";

pub const FALLBACK_ARTWORK_NAME: &str = "fallback";

pub static PANIC_ON_MISSING_LYRICS: bool = false;

// Particular to the Cloudflare pages project I use to host the static website
pub const CF_PAGES_NAME: &str = "astronomy487-music";

pub fn log_3<A, B, C>(col1: A, col2: B, message: C, color: &'static str)
where
	A: std::fmt::Display,
	B: std::fmt::Display,
	C: std::fmt::Display
{
	println!("{}{:<11}{:<7}{}{}", color, col1, col2, ANSI_RESET, message);
}

pub fn log_2<A, B>(col: A, message: B, color: &'static str)
where
	A: std::fmt::Display,
	B: std::fmt::Display
{
	println!("{}{:<18}{}{}", color, col, ANSI_RESET, message);
}

pub fn ask_to_continue() -> bool {
	print!("Are you sure you want to continue? [yes/no] ");
	let _ = std::io::Write::flush(&mut std::io::stdout());
	let mut input = String::new();
	let _ = std::io::stdin()
		.read_line(&mut input)
		.unwrap_or_else(|_| panic!("Failed to read input"));
	let user_choice = input.trim().to_lowercase();
	// print!("{}", ANSI_RESET);
	user_choice == "yes"
}

pub fn map_with_only_these_keys<'a>(
	val: &'a serde_json::Value, label: &'static str, allowed: &'static [&'static str]
) -> &'a serde_json::Map<String, serde_json::Value> {
	let obj = val
		.as_object()
		.unwrap_or_else(|| panic!("{} from JSON is not an object: {}", label, val));
	let bad: Vec<_> = obj
		.keys()
		.filter(|k| !allowed.contains(&k.as_str()))
		.cloned()
		.collect();
	if !bad.is_empty() {
		let formatted = bad
			.iter()
			.map(|s| format!("\"{}\"", s))
			.collect::<Vec<_>>()
			.join(", ");
		panic!("{} from JSON has unexpected keys: {}", label, formatted);
	}
	obj
}

pub fn compute_slug(artist: &str, title: &str) -> String {
	let mut slug = if artist == "Astro" {
		title.to_owned()
	} else {
		format!("{} {}", artist, title)
	};
	slug = slug.to_lowercase();
	slug = unicode_normalization::UnicodeNormalization::nfd(slug.chars())
		.filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
		.collect();
	slug = slug.replace("a$tro", "astro"); // i love kesha
	let re_punct = regex::Regex::new(r#"[()\[\],.?!'"*\$]"#).expect("re_punct is invalid regex");
	let re_sep = regex::Regex::new(r#"[_/&+:;\s]+"#).expect("re_sep is invalid regex");
	let re_dash = regex::Regex::new(r#"-+"#).expect("re_dash is invalid regex");
	slug = re_punct.replace_all(&slug, "").into_owned();
	slug = re_sep.replace_all(&slug, "-").into_owned();
	slug = re_dash.replace_all(&slug, "-").into_owned();
	slug = slug.chars().filter(char::is_ascii).collect();
	while slug.starts_with('-') {
		slug = slug[1..].to_string();
	}
	while slug.ends_with('-') {
		slug = slug[..slug.len() - 1].to_string();
	}
	slug = slug.replace("--", "-");
	assert!(
		slug.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
		"Could not generate a valid slug for artist {} and title {}; we arrived at {}",
		artist,
		title,
		slug
	);
	slug
}

pub fn check_custom_slug(slug: &str) {
	assert!(
		slug.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
		"Custom slug \"{}\" is not valid",
		slug
	);
}
