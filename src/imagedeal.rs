#[cached::proc_macro::cached(size = 100)]
pub fn grab_image(name: String) -> Vec<u8> {
	let img_path = std::path::Path::new("C:/Users/astro/Code/distri/filezone/source/image")
		.join(&name)
		.with_extension("png");
	let dest_path =
		std::path::Path::new("C:/Users/astro/Code/distri/filezone/music.astronomy487.com")
			.join(&name)
			.with_extension("jpg");
	if !dest_path.exists() {
		let img = image::ImageReader::open(&img_path)
			.expect(&format!("The png image {} didn't exist", name))
			.decode()
			.expect("We couldn't decode");
		let resized_img = img.resize(1000, 1000, image::imageops::FilterType::Gaussian);
		resized_img
			.save(&dest_path)
			.expect("Couldn't write the jpg");
	}
	std::fs::read(&dest_path).expect("Couldn't read cached jpg file")
}
