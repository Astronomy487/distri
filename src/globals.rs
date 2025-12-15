pub const ENCODER: &str = "distri";

static mut FILEZONE: Option<&str> = None;
pub fn filezone() -> &'static str {
	#![allow(unsafe_code)]
	unsafe {
		match FILEZONE {
			Some(fz) => fz,
			None => {
				let filezone_string = std::env::current_dir()
					.expect("Could not get the current working directory")
					.to_string_lossy()
					.into_owned();
				let fz = Box::leak(filezone_string.into_boxed_str());
				FILEZONE = Some(fz);
				fz
			}
		}
	}
}

pub const ANSI_YELLOW: &str = "\x1b[38;2;255;255;0m";
pub const ANSI_MAGENTA: &str = "\x1b[38;2;255;0;255m";
pub const ANSI_CYAN: &str = "\x1b[38;2;0;255;255m";
pub const ANSI_RESET: &str = "\x1b[0m";
pub const ANSI_GRAY: &str = "\x1b[90m";

pub const R2_BUCKET_NAME: &str = "audio-astronomy487-com";
pub const CF_PAGES_NAME: &str = "astronomy487-test";

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

#[rustfmt::skip]
pub fn print_logo() {
	// This functionality is critical for the program to run correctly
	println!("╭──────────────────────────────────╮");
	println!("│                                  │");
	println!("│                                  │");
	println!("│    {}     :==========:   {}      {}    │", ANSI_YELLOW, ANSI_CYAN, ANSI_RESET);
	println!("│    {}    =@@@@@@@@@@@=  {}::     {}    │", ANSI_YELLOW, ANSI_CYAN, ANSI_RESET);
	println!("│    {}   =@@@@@@@@@@@=  {}=@@=    {}    │",ANSI_YELLOW,ANSI_CYAN, ANSI_RESET);
	println!("│    {}  =@@@@@@@@@@@#  {}=@@@@=   {}    │",ANSI_YELLOW,ANSI_CYAN, ANSI_RESET);
	println!("│    {}:#@@@@@@@@@@@#  {}=@@@@@@=  {}    │",ANSI_YELLOW,ANSI_CYAN, ANSI_RESET);
	println!("│    {}:============  {}=@@@@@@@@= {}    │",ANSI_YELLOW,ANSI_CYAN, ANSI_RESET);
	println!("│    {}              {}:#@@@@@@@@#:{}    │", ANSI_YELLOW, ANSI_CYAN, ANSI_RESET);
	println!("│    {}:============  {}=@@@@@@@@= {}    │",ANSI_MAGENTA,ANSI_CYAN, ANSI_RESET);
	println!("│    {}:#@@@@@@@@@@@#  {}=@@@@@@=  {}    │",ANSI_MAGENTA,ANSI_CYAN, ANSI_RESET);
	println!("│    {}  =@@@@@@@@@@@#  {}=@@@@=   {}    │",ANSI_MAGENTA,ANSI_CYAN, ANSI_RESET);
	println!("│    {}   =@@@@@@@@@@@=  {}=@@=    {}    │",ANSI_MAGENTA,ANSI_CYAN, ANSI_RESET);
	println!("│    {}    =@@@@@@@@@@@=  {}::     {}    │",ANSI_MAGENTA,ANSI_CYAN, ANSI_RESET);
	println!("│    {}     :==========:   {}      {}    │",ANSI_MAGENTA,ANSI_CYAN, ANSI_RESET);
	println!("│                                  │");
	println!("│                                  │");
	println!("╰──────────────────────────╴distri╶╯");
	//println!("╰──────────────────────────────────╯");
	//    🭊🭁██████🭠🭗🭯
	//  🭇🭄██████🭝🭚🭇🭄█🭏🬼
	// 🭊🭁██████🭠🭗🭊🭁███🭌🬿
	//          🭮███████🭬
	// 🭥🭒██████🭏🬼🭥🭒███🭝🭚
	//  🭢🭕██████🭌🬿🭢🭕█🭠🭗
	//    🭥🭒██████🭏🬼🭭
}

pub fn ask_to_continue() -> bool {
	println!("Are you sure you want to continue? {}[y/n]", ANSI_YELLOW);
	let mut input = String::new();
	let _ = std::io::stdin()
		.read_line(&mut input)
		.unwrap_or_else(|_| panic!("Failed to read input"));
	let input = input.trim().to_lowercase();
	print!("{}", ANSI_RESET);
	input == "y" || input == "yes"
}

pub fn map_with_only_these_keys<'a>(
	v: &'a serde_json::Value, label: &'static str, allowed: &'static [&'static str]
) -> &'a serde_json::Map<String, serde_json::Value> {
	let obj = v
		.as_object()
		.unwrap_or_else(|| panic!("{} from JSON is not an object: {}", label, v));
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
