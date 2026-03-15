use crate::build::xml::XmlNode;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub struct UPC([u8; 12]); // stored as sequence of ascii digits '0' through '9'

impl UPC {
	pub fn from(upc: &str) -> Option<Self> {
		if upc.len() == 12 && upc.chars().all(|c| c.is_ascii_digit()) {
			Some(Self(
				//upc.as_bytes().iter().map(|x| x - b'0').collect::<Vec<_>>().try_into().expect("Can't make upc")
				upc.as_bytes().try_into().expect("Can't make upc")
			))
		} else {
			None
		}
	}
	pub fn digit_pattern(digit: u8) -> [bool; 7] {
		// on left side, false = white/bg, true = black/fg
		// on right side, false = black/fg, true = white/bg
		match digit {
			b'0' => [false, false, false, true, true, false, true],
			b'1' => [false, false, true, true, false, false, true],
			b'2' => [false, false, true, false, false, true, true],
			b'3' => [false, true, true, true, true, false, true],
			b'4' => [false, true, false, false, false, true, true],
			b'5' => [false, true, true, false, false, false, true],
			b'6' => [false, true, false, true, true, true, true],
			b'7' => [false, true, true, true, false, true, true],
			b'8' => [false, true, true, false, true, true, true],
			b'9' => [false, false, false, true, false, true, true],
			_ => unreachable!()
		}
	}
	#[allow(unused_assignments)]
	pub fn barcode(&self) -> [bool; 95] {
		// based on https://en.wikipedia.org/wiki/Universal_Product_Code#Encoding
		// false = white/bg, true = black/fg
		let mut pattern = [false; 95];
		let mut marker = 0;
		pattern[marker] = true;
		marker += 1;
		pattern[marker] = false;
		marker += 1;
		pattern[marker] = true;
		marker += 1;
		for digit in &self.0[0..6] {
			for pixel in Self::digit_pattern(*digit) {
				pattern[marker] = pixel;
				marker += 1;
			}
		}
		pattern[marker] = false;
		marker += 1;
		pattern[marker] = true;
		marker += 1;
		pattern[marker] = false;
		marker += 1;
		pattern[marker] = true;
		marker += 1;
		pattern[marker] = false;
		marker += 1;
		for digit in &self.0[6..12] {
			for pixel in Self::digit_pattern(*digit) {
				pattern[marker] = !pixel;
				marker += 1;
			}
		}
		pattern[marker] = true;
		marker += 1;
		pattern[marker] = false;
		marker += 1;
		pattern[marker] = true;
		marker += 1;
		pattern
	}
	pub fn svg(&self) -> XmlNode {
		let mut svg = XmlNode::new("svg")
			.with_attribute("viewBox", "0 0 95 1")
			.with_attribute("preserveAspectRatio", "none")
			.with_attribute("xmlns", "http://www.w3.org/2000/svg");
		let barcode = self.barcode();
		let mut marker = 0;
		while marker < 95 {
			if barcode[marker] {
				// grab a rectangle
				let mut right_edge = marker;
				while right_edge < 95 && barcode[right_edge] {
					right_edge += 1;
				}
				svg.add_child(
					XmlNode::new("rect")
						.with_attribute("x", marker.to_string())
						.with_attribute("y", "0")
						.with_attribute("width", (right_edge - marker).to_string())
						.with_attribute("height", "1")
				);
				marker = right_edge;
			} else {
				marker += 1;
			}
		}
		svg
	}
}

impl std::fmt::Display for UPC {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let _ = self.barcode();
		for digit in self.0 {
			write!(fmt, "{}", char::from(digit))?
		}
		Ok(())
	}
}
