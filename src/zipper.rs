pub struct Zipper {
	writer: zip::ZipWriter<std::fs::File>
}

impl Zipper {
	pub fn new(dest: &std::path::Path) -> Self {
		let file = std::fs::File::create(dest)
			.unwrap_or_else(|_| panic!("Failed to create destination zip {}", dest.display()));
		let writer = zip::ZipWriter::new(file);
		Zipper { writer }
	}
	pub fn add_file(&mut self, src: &std::path::Path, dest_in_zip: &std::path::Path) {
		let options = zip::write::FileOptions::default()
			.compression_method(zip::CompressionMethod::Deflated)
			.unix_permissions(0o644);

		self.writer
			.start_file(dest_in_zip.to_string_lossy(), options)
			.expect("Failed to start file in ZIP");

		let mut src_file = std::fs::File::open(src).expect("Failed to open source file");
		let _ =
			std::io::copy(&mut src_file, &mut self.writer).expect("Failed to write file to ZIP");
	}
	pub fn add_text_file(&mut self, content: &str, dest_in_zip: &std::path::Path) {
		let options = zip::write::FileOptions::default()
			.compression_method(zip::CompressionMethod::Deflated)
			.unix_permissions(0o644);

		self.writer
			.start_file(
				dest_in_zip.to_str().unwrap_or_else(|| {
					panic!("Path {} in zip is not UTF-8", dest_in_zip.display())
				}),
				options
			)
			.expect("Failed to start text file in ZIP");

		std::io::Write::write_all(&mut self.writer, content.as_bytes())
			.expect("Failed to write text file to ZIP");
	}
	pub fn finish(mut self) {
		let _ = self.writer.finish().expect("Failed to finish ZIP archive");
	}
}

pub fn copy_recursive(from: &std::path::Path, to: &std::path::Path) {
	if from.is_dir() {
		std::fs::create_dir_all(to).expect("Can't perform copy");
		for entry in std::fs::read_dir(from).expect("Can't perform copy") {
			let entry = entry.expect("Can't perform copy");
			let src_path = entry.path();
			let dst_path = to.join(entry.file_name());
			if src_path.is_dir() {
				copy_recursive(&src_path, &dst_path);
			} else {
				let _ = std::fs::copy(&src_path, &dst_path).expect("Couldn't perform copy");
			}
		}
	} else {
		let _ = std::fs::copy(from, to).expect("Couldn't perform copy");
	}
}
