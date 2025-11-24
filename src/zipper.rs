pub struct Zipper {
	writer: zip::ZipWriter<std::fs::File>
}

impl Zipper {
	pub fn new(dest: &std::path::Path) -> Self {
		let file = std::fs::File::create(dest).expect(&format!(
			"Failed to create destination zip {}",
			dest.display()
		));
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
		let mut buffer = Vec::new();
		let _ = std::io::Read::read_to_end(&mut src_file, &mut buffer)
			.expect("Failed to read source file");

		std::io::Write::write_all(&mut self.writer, &buffer).expect("Failed to write file to ZIP");
	}

	pub fn add_text_file(&mut self, content: &str, dest_in_zip: &std::path::Path) {
		let options = zip::write::FileOptions::default()
			.compression_method(zip::CompressionMethod::Deflated)
			.unix_permissions(0o644);

		self.writer
			.start_file(dest_in_zip.to_string_lossy(), options)
			.expect("Failed to start text file in ZIP");

		std::io::Write::write_all(&mut self.writer, content.as_bytes())
			.expect("Failed to write text file to ZIP");
	}

	pub fn finish(mut self) {
		let _ = self.writer.finish().expect("Failed to finish ZIP archive");
	}
}
