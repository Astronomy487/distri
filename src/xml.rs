pub struct XmlNode {
	tag: &'static str,
	attributes: Vec<(&'static str, String)>,
	content: XmlNodeContent
}
pub enum XmlNodeContent {
	Children(Vec<XmlNode>),
	Text(String),
	None
}
impl XmlNode {
	pub fn new(tag: &'static str) -> XmlNode {
		XmlNode {
			tag,
			attributes: Vec::new(),
			content: XmlNodeContent::None
		}
	}
	pub fn add_attribute(&mut self, att: &'static str, val: impl Into<String>) {
		self.attributes.push((att, val.into()));
	}
	pub fn add_child(&mut self, child: XmlNode) {
		match &mut self.content {
			XmlNodeContent::Children(children) => {
				children.push(child);
			}
			XmlNodeContent::Text(_) => {
				panic!("Cannot put child on XML node that contains text");
			}
			XmlNodeContent::None => {
				self.content = XmlNodeContent::Children(vec![child]);
			}
		}
	}
	pub fn add_text_unescaped(&mut self, text: impl Into<String>) {
		match &self.content {
			XmlNodeContent::Children(_) => {
				panic!("Cannot put text on XML node that already has children");
			}
			XmlNodeContent::None => {
				self.content = XmlNodeContent::Text(text.into());
			}
			XmlNodeContent::Text(_) => {
				panic!("Cannot put text on XML node that already has text");
			}
		}
	}
	pub fn add_text(&mut self, text: impl Into<String>) {
		self.add_text_unescaped(escape(text.into()))
	}
	pub fn with_attribute(mut self, att: &'static str, val: impl Into<String>) -> Self {
		self.add_attribute(att, val);
		self
	}
	pub fn with_child(mut self, child: XmlNode) -> Self {
		self.add_child(child);
		self
	}
	pub fn with_text_unescaped(mut self, text: impl Into<String>) -> Self {
		self.add_text_unescaped(text);
		self
	}
	pub fn with_text(mut self, text: impl Into<String>) -> Self {
		self.add_text(text);
		self
	}
	pub fn maybe_with_child(mut self, maybe_child: Option<XmlNode>) -> Self {
		if let Some(child) = maybe_child {
			self.add_child(child)
		}
		self
	}
	pub fn maybe_with_attribute(
		mut self, att: &'static str, maybe_val: Option<impl Into<String>>
	) -> Self {
		if let Some(val) = maybe_val {
			self.add_attribute(att, val);
		}
		self
	}
}
impl std::fmt::Display for XmlNode {
	fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(fmt, "<{}", self.tag)?;
		for (att, val) in &self.attributes {
			if val.is_empty() {
				write!(fmt, " {}", att)?;
			} else {
				write!(fmt, " {}=\"{}\"", att, val)?;
			}
		}
		match &self.content {
			XmlNodeContent::None => {
				write!(fmt, "/>")
			}
			XmlNodeContent::Text(text) => {
				write!(fmt, ">{}</{}>", text, self.tag)
			}
			XmlNodeContent::Children(children) => {
				write!(fmt, ">")?;
				for child in children {
					write!(fmt, "{}", child)?;
				}
				write!(fmt, "</{}>", self.tag)
			}
		}
	}
}

fn escape(original: String) -> String {
	let mut string = original;
	for (unescaped, escaped) in [
		("&", "&amp;"),
		("<", "&lt;"),
		(">", "&gt;"),
		("'", "&apos;"),
		("\"", "&quot;"),
		("\0&lt;", "<"),
		("\0&gt;", ">")
	] {
		string = string.replace(unescaped, escaped);
	}
	string
}
