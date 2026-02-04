use crate::globals;

static CHECK_IMAGE_CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
	std::sync::OnceLock::new();

fn check_image_cache() -> &'static std::sync::Mutex<std::collections::HashSet<String>> {
	CHECK_IMAGE_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

pub enum ImageCodec {
	Png,
	Jpg
}
impl ImageCodec {
	fn ext(&self) -> &'static str {
		match self {
			ImageCodec::Png => "png",
			ImageCodec::Jpg => "jpg"
		}
	}
	fn ideal_image_size(&self) -> u32 {
		match self {
			ImageCodec::Png => 3_000,
			ImageCodec::Jpg => 1_000
		}
	}
}

// good news this function is cached because i love you
pub fn check_artwork(name: String) {
	if let Some(cache) = check_image_cache().lock().ok()
		&& cache.contains(&name)
	{
		return;
	}
	let img_path = std::path::Path::new(globals::filezone())
		.join("source")
		.join("image")
		.join(&name)
		.with_extension("png");
	assert!(img_path.exists(), "Source image {}.png must exist", name);
}

pub fn grab_artwork_data(name: String, codec: ImageCodec) -> Vec<u8> {
	let img_path = std::path::Path::new(globals::filezone())
		.join("source")
		.join("image")
		.join(&name)
		.with_extension("png");
	let dest_path = std::path::Path::new(globals::filezone())
		.join("private")
		.join(codec.ext())
		.join(&name)
		.with_extension(codec.ext());
	if !dest_path.exists() {
		globals::log_3(
			"Making",
			codec.ext(),
			name.clone() + "." + codec.ext(),
			globals::ANSI_CYAN
		);
		let img = image::ImageReader::open(&img_path)
			.unwrap_or_else(|_| panic!("Couldn't find {}.png", name))
			.decode()
			.unwrap_or_else(|_| panic!("Couldn't decode {}.png", name));
		assert!(
			img.width() == 3_000 && img.height() == 3_000,
			"Image {}.png must be 3000x3000",
			name
		);
		let resized_img = img.resize(
			codec.ideal_image_size(),
			codec.ideal_image_size(),
			image::imageops::FilterType::Gaussian
		);
		match codec {
			ImageCodec::Png | ImageCodec::Jpg => resized_img.save(&dest_path).ok()
		}
		.unwrap_or_else(|| panic!("Couldn't write {}.{}", name, codec.ext()));
	}
	std::fs::read(&dest_path)
		.unwrap_or_else(|_| panic!("Couldn't read cached {}.{}", name, codec.ext()))
}
