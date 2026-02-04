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
		for maybe_entry in std::fs::read_dir(from).expect("Can't perform copy") {
			let entry = maybe_entry.expect("Can't perform copy");
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

pub fn clear_directory(path: &std::path::Path) {
	for maybe_entry in std::fs::read_dir(path)
		.unwrap_or_else(|e| panic!("Failed to read directory {}: {}", path.display(), e))
	{
		let entry = maybe_entry
			.unwrap_or_else(|e| panic!("Failed to read entry in {}: {}", path.display(), e));

		let entry_path = entry.path();

		if entry_path.is_dir() {
			std::fs::remove_dir_all(&entry_path).unwrap_or_else(|e| {
				panic!("Failed to remove directory {}: {}", entry_path.display(), e)
			});
		} else {
			std::fs::remove_file(&entry_path).unwrap_or_else(|e| {
				panic!("Failed to remove file {}: {}", entry_path.display(), e)
			});
		}
	}
}

pub fn filesize(location: &std::path::Path) -> Option<u64> {
	Some(std::fs::metadata(location).ok()?.len())
}

pub fn dir_size_recursive(path: &std::path::Path) -> u64 {
	let mut size = 0;

	if let Ok(entries) = std::fs::read_dir(path) {
		for maybe_entry in entries {
			let entry = maybe_entry.unwrap_or_else(|_| {
				panic!(
					"Error while finding the size of directory {}",
					path.display()
				)
			});
			let metadata = entry.metadata().unwrap_or_else(|_| {
				panic!(
					"Error while finding the size of directory {}",
					path.display()
				)
			});

			if metadata.is_file() {
				size += metadata.len();
			} else if metadata.is_dir() {
				size += dir_size_recursive(&entry.path());
			}
		}
	}

	size
}

pub fn format_file_size(bytes: u64) -> String {
	fn sig(amount: f64) -> String {
		if amount >= 10.0 {
			format!("{:.0}", amount)
		} else {
			format!("{:.1}", amount)
		}
	}
	const KB: f64 = 1024.0;
	const MB: f64 = 1024.0 * 1024.0;
	const GB: f64 = 1024.0 * 1024.0 * 1024.0;
	let bytes_float = bytes as f64;
	if bytes_float >= GB {
		format!("{} GB", sig(bytes_float / GB))
	} else if bytes_float >= MB {
		format!("{} MB", sig(bytes_float / MB))
	} else if bytes_float >= KB {
		format!("{} KB", bytes / 1024)
	} else {
		format!("{} B", bytes)
	}
}

pub fn write_file<T: Into<Vec<u8>>>(destination: &std::path::Path, content: T) {
	let mut redirect_file = std::fs::File::create(destination)
		.unwrap_or_else(|_| panic!("Couldn't create file {}", destination.display()));
	let _ = std::io::Write::write(&mut redirect_file, &content.into())
		.unwrap_or_else(|_| panic!("Couldn't write to file {}", destination.display()));
}
