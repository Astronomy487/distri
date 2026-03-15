#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct ISRC([u8; 12]); // stored as sequence of ascii

impl ISRC {
	pub fn from(isrc: &str) -> Option<Self> {
		if isrc.len() != 12
			|| !isrc[..2].bytes().all(|bytes| bytes.is_ascii_uppercase())
			|| !isrc[2..5]
				.bytes()
				.all(|bytes| bytes.is_ascii_alphanumeric())
			|| !isrc[5..7].bytes().all(|bytes| bytes.is_ascii_digit())
			|| !isrc[7..].bytes().all(|bytes| bytes.is_ascii_digit())
		{
			None
		} else {
			Some(Self(isrc.as_bytes().try_into().expect("Can't make isrc")))
		}
	}
	pub fn as_dense(&self) -> String {
		String::from_utf8(self.0.to_vec()).expect("cmon")
	}
}

impl std::fmt::Display for ISRC {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let bytes = &self.0;
		let s0 = std::str::from_utf8(&bytes[0..2]).expect("Could not display ISRC");
		let s1 = std::str::from_utf8(&bytes[2..5]).expect("Could not display ISRC");
		let s2 = std::str::from_utf8(&bytes[5..7]).expect("Could not display ISRC");
		let s3 = std::str::from_utf8(&bytes[7..12]).expect("Could not display ISRC");
		fmt.write_str(s0)?;
		fmt.write_str("-")?;
		fmt.write_str(s1)?;
		fmt.write_str("-")?;
		fmt.write_str(s2)?;
		fmt.write_str("-")?;
		fmt.write_str(s3)
	}
}
