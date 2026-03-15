use crate::build::smartquotes;
use crate::types::date::Date;

#[derive(Debug)]
pub struct Assist {
	pub titlable: String,
	pub released: Date,
	pub artwork: String,
	pub url: String,
	pub role: String
}
impl Assist {
	pub fn from_json(val: &serde_json::Value) -> Assist {
		let assist = Assist {
			titlable: val
				.get("titlable")
				.unwrap_or_else(|| panic!("Assists JSON has no \"titlable\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| {
					panic!(
						"Assists JSON has non-string \"titlable\" attribute: {}",
						val
					)
				})
				.to_string(),
			artwork: val
				.get("artwork")
				.unwrap_or_else(|| panic!("Assists JSON has no \"artwork\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| {
					panic!("Assists JSON has non-string \"artwork\" attribute: {}", val)
				})
				.to_string(),
			url: val
				.get("url")
				.unwrap_or_else(|| panic!("Assists JSON has no \"url\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| panic!("Assists JSON has non-string \"url\" attribute: {}", val))
				.to_string(),
			role: val
				.get("role")
				.unwrap_or_else(|| panic!("Assists JSON has no \"role\" attribute: {}", val))
				.as_str()
				.unwrap_or_else(|| {
					panic!("Assists JSON has non-string \"role\" attribute: {}", val)
				})
				.to_string(),
			released: Date::from(
				val.get("released")
					.unwrap_or_else(|| {
						panic!("Assists JSON has no \"released\" attribute: {}", val)
					})
					.as_str()
					.unwrap_or_else(|| {
						panic!(
							"Assists JSON has non-string \"released\" attribute: {}",
							val
						)
					})
			)
		};
		// whitespace
		assert!(
			assist.titlable.trim() == assist.titlable,
			"assist.titlable has leading/trailing whitespace: '{}'",
			assist.titlable
		);
		assert!(
			assist.artwork.trim() == assist.artwork,
			"assist.artwork has leading/trailing whitespace: '{}'",
			assist.artwork
		);
		assert!(
			assist.url.trim() == assist.url,
			"assist.url has leading/trailing whitespace: '{}'",
			assist.url
		);
		assert!(
			assist.role.trim() == assist.role,
			"assist.role has leading/trailing whitespace: '{}'",
			assist.role
		);

		// artwork validation
		let artwork = &assist.artwork;
		let valid_prefix = artwork.starts_with("https://");
		let valid_suffix = artwork.ends_with(".jpg") || artwork.ends_with(".png");
		assert!(
			valid_prefix && valid_suffix,
			"assist.artwork must start with http(s):// and end with .jpg or .png: '{}'",
			artwork
		);

		// "role" text validation
		if let Some(first_char) = assist.role.chars().next() {
			assert!(
				first_char.to_uppercase().to_string() == first_char.to_string(),
				"assist.role must start with an uppercase character: '{}'",
				assist.role
			);
		} else {
			panic!("assist.role is empty");
		}

		assert!(!smartquotes::contains_smart_quotes(&assist.titlable));

		assist
	}
}
