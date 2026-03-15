use crate::globals;

#[derive(Debug, Clone)]
pub struct Artwork {
	pub name_with_slash: String,         // used to communicate
	pub name_without_slash: String,      // used to communicate
	pub source_path: std::path::PathBuf, // png stays here always
	pub jpg_path: std::path::PathBuf, // as kept in private/jpg. this is where encode and build looks for them
	pub caption: String
}

impl PartialEq for Artwork {
	fn eq(&self, other: &Artwork) -> bool {
		self.name_without_slash == other.name_without_slash
	}
}
impl Eq for Artwork {}
impl std::hash::Hash for Artwork {
	fn hash<H>(&self, state: &mut H)
	where
		H: std::hash::Hasher
	{
		self.name_without_slash.hash(state);
	}
}

impl Artwork {
	pub fn from(maybe_parent_slug: Option<&str>, artwork_name: &str) -> Self {
		let name_with_slash = if let Some(parent_slug) = maybe_parent_slug {
			format!("{}/{}", parent_slug, artwork_name)
		} else {
			String::from(artwork_name)
		};
		let source_path = globals::filezone()
			.join("source")
			.join("artwork")
			.join(&name_with_slash)
			.with_extension("png");
		let jpg_path = globals::filezone()
			.join("private")
			.join("jpg")
			.join(artwork_name)
			.with_extension("png");
		assert!(
			source_path.exists(),
			"Could not locate artwork for {}",
			name_with_slash
		);

		let caption_path = globals::filezone()
			.join("source")
			.join("artwork")
			.join(&name_with_slash)
			.with_extension("txt");
		let caption = std::fs::read_to_string(caption_path).unwrap_or_else(|e| {
			panic!("Couldn't find a caption for image {}: {}", name_with_slash, e)
		});
		assert!(
			caption.trim() == caption,
			"Image caption for {}.png is not trimmed",
			name_with_slash
		);
		assert!(
			!caption.contains(['\r', '\n']),
			"Image caption for {}.png contains newline",
			name_with_slash
		);
		assert!(
			caption.len() <= 200,
			"Image caption for {}.png is {} bytes long, which exceeds {} byte limit",
			name_with_slash,
			caption.len(),
			200
		);
		assert!(
			caption.len() >= 10,
			"Image caption for {}.png is {} bytes long, which is less than {} byte minimum",
			name_with_slash,
			caption.len(),
			10
		);

		Self {
			source_path,
			jpg_path,
			name_with_slash,
			name_without_slash: artwork_name.to_string(),
			caption
		}
	}
	pub fn make_jpg_exist(&self) {
		if self.jpg_path.exists() {
			return;
		}
		globals::log_3(
			"Making",
			"jpg",
			self.name_with_slash.clone() + ".jpg",
			globals::ANSI_CYAN
		);
		let img = image::ImageReader::open(&self.source_path)
			.unwrap_or_else(|_| panic!("Couldn't find {}.png", self.name_with_slash))
			.decode()
			.unwrap_or_else(|_| panic!("Couldn't decode {}.png", self.name_with_slash));
		assert!(
			img.width() == 3_000 && img.height() == 3_000,
			"Image {}.png must be 3000x3000",
			self.name_with_slash
		);
		let resized_img = img.resize(1_000, 1_000, image::imageops::FilterType::Gaussian);
		resized_img
			.save(&self.jpg_path)
			.ok()
			.unwrap_or_else(|| panic!("Couldn't write {}.jpg", self.name_with_slash));
	}
	pub fn jpg_data(&self) -> Vec<u8> {
		self.make_jpg_exist();
		std::fs::read(&self.jpg_path)
			.unwrap_or_else(|_| panic!("Couldn't read cached {}.jpg", self.name_with_slash))
	}
	pub fn fallback() -> Self {
		// init once, give &'static Self
		Self::from(None, "fallback")
	}
}
