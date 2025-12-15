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

pub fn clear_directory(path: &std::path::Path) {
	for entry in std::fs::read_dir(path)
		.unwrap_or_else(|e| panic!("Failed to read directory {}: {}", path.display(), e))
	{
		let entry =
			entry.unwrap_or_else(|e| panic!("Failed to read entry in {}: {}", path.display(), e));

		let p = entry.path();

		if p.is_dir() {
			std::fs::remove_dir_all(&p)
				.unwrap_or_else(|e| panic!("Failed to remove directory {}: {}", p.display(), e));
		} else {
			std::fs::remove_file(&p)
				.unwrap_or_else(|e| panic!("Failed to remove file {}: {}", p.display(), e));
		}
	}
}

pub fn filesize(location: &std::path::Path) -> u64 {
	std::fs::metadata(location)
		.unwrap_or_else(|_| panic!("Could not find the size of file {}", location.display()))
		.len()
}

pub fn dir_size_recursive(path: &std::path::Path) -> u64 {
	let mut size = 0;

	if let Ok(entries) = std::fs::read_dir(path) {
		for entry in entries {
			let entry = entry.unwrap_or_else(|_| {
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
	const KB: f64 = 1024.0;
	const MB: f64 = 1024.0 * 1024.0;
	const GB: f64 = 1024.0 * 1024.0 * 1024.0;
	let bytes = bytes as f64;
	fn sig3(amount: f64) -> String {
		if amount >= 100.0 {
			format!("{:.0}", amount)
		} else if amount >= 10.0 {
			format!("{:.1}", amount)
		} else {
			format!("{:.2}", amount)
		}
	}
	if bytes >= GB {
		format!("{} GB", sig3(bytes / GB))
	} else if bytes >= MB {
		format!("{} MB", sig3(bytes / MB))
	} else if bytes >= KB {
		format!("{} kB", sig3(bytes / KB))
	} else {
		format!("{:.0} B", bytes)
	}
}

pub fn walk_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
	let mut out = Vec::new();
	let mut stack = vec![root.to_path_buf()];

	while let Some(path) = stack.pop() {
		let meta = std::fs::metadata(&path)
			.unwrap_or_else(|e| panic!("metadata failed for {:?}: {}", path, e));

		if meta.is_dir() {
			for entry in std::fs::read_dir(&path)
				.unwrap_or_else(|e| panic!("read_dir failed for {:?}: {}", path, e))
			{
				let entry = entry.unwrap_or_else(|e| panic!("bad dir entry: {}", e));
				stack.push(entry.path());
			}
		} else if meta.is_file() {
			out.push(path);
		}
	}

	out
}
