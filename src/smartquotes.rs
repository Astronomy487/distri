fn open_after_char(previous: char) -> bool {
	previous.is_whitespace() || is_a_dash(previous) || is_opening_brackets(previous)
}

fn is_a_dash(character: char) -> bool {
	matches!(character, '\u{2012}'..='\u{2015}')
}

fn is_opening_brackets(character: char) -> bool {
	[
		'(', '[', '{', '⟨', '«', '‹', '〈', '《', '「', '『', '【', '〔', '〖', '〘', '〚'
	]
	.contains(&character)
}

pub fn smart_quotes(text: String) -> String {
	let mut previous_character = ' ';
	assert!(open_after_char(previous_character));
	let mut new_text = String::new();
	for character in text.chars() {
		let mut picked_up = false;
		for (ascii, opening, closing) in [('\'', '‘', '’'), ('"', '“', '”')] {
			if character == ascii {
				picked_up = true;
				new_text.push(if open_after_char(previous_character) {
					opening
				} else {
					closing
				});
				break;
			}
		}
		if !picked_up {
			new_text.push(character);
		}
		previous_character = character;
	}
	new_text
}
