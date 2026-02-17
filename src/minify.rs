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

pub fn compress_js<T: Into<String>>(original: T) -> String {
	original.into()
	// sorry ! no js compression 4 u! (it's totally fine)
	
    /*
	// i tried out this library and it just always panics. what r we doing
	let input = original.into();
    let session = better_minify_js::Session::new();
    let mut out = Vec::new();

    better_minify_js::minify(
        &session,
        better_minify_js::TopLevelMode::Global,
        input.as_bytes(),
        &mut out,
    )
    .expect("JS minification failed");

    String::from_utf8(out).expect("Minifier produced invalid UTF-8") */
}