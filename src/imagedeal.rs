use crate::globals;

// #[cached::proc_macro::cached(size = 100)]
pub fn grab_image(name: String) -> Vec<u8> {
	let img_path = std::path::Path::new(globals::filezone())
		.join("source")
		.join("image")
		.join(&name)
		.with_extension("png");
	let dest_path = std::path::Path::new(globals::filezone())
		.join("private")
		.join("jpg")
		.join(&name)
		.with_extension("jpg");
	if !dest_path.exists() {
		globals::log_3("Making", "jpg", &(name.clone() + ".jpg"), globals::ANSI_YELLOW);
		let img = image::ImageReader::open(&img_path)
			.unwrap_or_else(|_| panic!("Couldn't find {}.png", name))
			.decode()
			.unwrap_or_else(|_| panic!("Couldn't decode {}.png", name));
		if img.width() != 3000 || img.height() != 3000 {
			panic!("Image {}.png must be 3000x3000", name);
		}
		let resized_img = img.resize(1000, 1000, image::imageops::FilterType::Gaussian);
		resized_img
			.save(&dest_path)
			.unwrap_or_else(|_| panic!("Couldn't write {}.jpg", name));
	}
	std::fs::read(&dest_path).unwrap_or_else(|_| panic!("Couldn't read cached JPG file {}", name))
}
