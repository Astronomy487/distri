pub fn compress_css<T: Into<String>>(original: T) -> String {
	let mut text = original.into();
	text = text.replace(['\n', '\t', '\r'], " ");
	while text.contains("  ") {
		text = text.replace("  ", " ");
	}
	let re_comments = regex::Regex::new(r#"\/\*[^*]*\*+([^/*][^*]*\*+)*\/"#).expect("re_comments is invalid regex");
	// ^^ found at https://www.w3.org/TR/CSS2/grammar.html#scanner
	text = re_comments.replace_all(&text, "").into_owned();
	text = text.replace(" {", "{");
	text = text.replace("{ ", "{");
	text = text.replace("} ", "}");
	text = text.replace(", ", ",");
	text = text.replace(": ", ":");
	text = text.replace("; ", ";");
	/* text = text.replace("> ", ">"); // ok so these are invalid transformations actually. whatever
	text = text.replace(" >", ">");
	text = text.replace(" <", "<");
	text = text.replace("< ", "<");
	text = text.replace(" +", "+");
	text = text.replace("+ ", "+");
	text = text.replace(" -", "-");
	text = text.replace("- ", "-"); */
	text = text.replace(";}", "}");
	text.trim().to_string()
}

// TODO remove css comments