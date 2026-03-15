pub fn compress_css<T: Into<String>>(original: T) -> String {
	let mut text = original.into();
	text = text.replace(['\n', '\t', '\r'], " ");
	while text.contains("  ") {
		text = text.replace("  ", " ");
	}
	let re_comments = regex::Regex::new(r#"\/\*[^*]*\*+([^/*][^*]*\*+)*\/"#)
		.expect("re_comments is invalid regex");
	// ^^ found at https://www.w3.org/TR/CSS2/grammar.html#scanner
	text = re_comments.replace_all(&text, "").into_owned();
	text = text.replace(" {", "{");
	text = text.replace("{ ", "{");
	text = text.replace("} ", "}");
	text = text.replace(", ", ",");
	text = text.replace(": ", ":");
	text = text.replace("; ", ";");
	text = text.replace(";}", "}");
	text.trim().to_string()
}

pub fn compress_js<T: Into<String>>(original: T) -> String {
	// all the existing js minification crates are incorrect
	let mut text = original.into();

	let re_line = regex::Regex::new(r"//[^\n\r]*").expect("invalid line comment regex");
	text = re_line.replace_all(&text, "").into_owned();

	let re_block =
		regex::Regex::new(r"/\*[^*]*\*+([^/*][^*]*\*+)*/").expect("invalid block comment regex");
	text = re_block.replace_all(&text, "").into_owned();

	text = text.replace(['\n', '\t', '\r'], " ");

	while text.contains("  ") {
		text = text.replace("  ", " ");
	}

	/* for (from, to) in [
		(" {", "{"),
		("{ ", "{"),
		(" }", "}"),
		("} ", "}"),
		(" (", "("),
		("( ", "("),
		(" )", ")"),
		(") ", ")"),
		(" = ", "="),
		(" + ", "+"),
		(" - ", "-"),
		(" * ", "*"),
		(" / ", "/"),
		(" , ", ","),
		(" ;", ";"),
		("; ", ";"),
		(" : ", ":"),
		(";}", "}")
	] {
		text = text.replace(from, to);
	} */

	text.trim().to_string()
}
