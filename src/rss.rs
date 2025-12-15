use crate::date;
use crate::globals;
use crate::musicdata;

struct XmlNode {
	tag: &'static str,
	attributes: Vec<(&'static str, String)>,
	content: XmlNodeContent
}
enum XmlNodeContent {
	Children(Vec<XmlNode>),
	Text(String),
	None
}
impl XmlNode {
	fn new(t: &'static str) -> XmlNode {
		XmlNode {
			tag: t,
			attributes: Vec::new(),
			content: XmlNodeContent::None
		}
	}
	fn add_attribute(&mut self, att: &'static str, val: impl Into<String>) {
		self.attributes.push((att, val.into()));
	}
	fn add_child(&mut self, child: XmlNode) {
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
	fn add_text(&mut self, text: impl Into<String>) {
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
	fn with_attribute(mut self, att: &'static str, val: impl Into<String>) -> Self {
		self.add_attribute(att, val);
		self
	}
	fn with_child(mut self, child: XmlNode) -> Self {
		self.add_child(child);
		self
	}
	fn with_text(mut self, text: impl Into<String>) -> Self {
		self.add_text(text);
		self
	}
}
impl std::fmt::Display for XmlNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{}", self.tag)?;
		for (att, val) in &self.attributes {
			write!(f, " {}=\"{}\"", att, val)?;
		}
		match &self.content {
			XmlNodeContent::None => {
				write!(f, "/>")
			}
			XmlNodeContent::Text(text) => {
				write!(f, ">{}</{}>", text, self.tag)
			}
			XmlNodeContent::Children(children) => {
				write!(f, ">")?;
				for child in children {
					write!(f, "{}", child)?;
				}
				write!(f, "</{}>", self.tag)
			}
		}
	}
}

pub fn make_rss(albums: &[musicdata::Album]) {
	let mut channel = XmlNode::new("channel")
		.with_child(XmlNode::new("title").with_text("Astro's discography"))
		.with_child(XmlNode::new("link").with_text("https://music.astronomy487.com"))
		.with_child(
			XmlNode::new("description").with_text("All albums released by Astro (astronomy487)")
		)
		.with_child(XmlNode::new("lastBuildDate").with_text(date::Date::now_rfc822()))
		.with_child(XmlNode::new("category").with_text("music"))
		.with_child(XmlNode::new("language").with_text("en-US"))
		.with_child(XmlNode::new("docs").with_text("https://www.rssboard.org/rss-specification"))
		.with_child(
			XmlNode::new("image")
				.with_child(
					XmlNode::new("url").with_text("https://music.astronomy487.com/squarelogo.png")
				)
				.with_child(XmlNode::new("title").with_text("Astro's logo"))
				.with_child(XmlNode::new("link").with_text("https://music.astronomy487.com"))
		)
		.with_child(
			XmlNode::new("atom:link")
				.with_attribute("href", "https://music.astronomy487.com/rss.xml")
				.with_attribute("rel", "self")
				.with_attribute("type", "application/rss+xml")
		);
	for album in albums.iter().rev() {
		if !album.temporary {
			let mut item = XmlNode::new("item")
				.with_child(XmlNode::new("title").with_text(format!("{}", album)))
				.with_child(
					XmlNode::new("link")
						.with_text(format!("https://music.astronomy487.com/{}", album.slug()))
				)
				.with_child(
					XmlNode::new("guid")
						.with_text(format!("https://music.astronomy487.com/{}", album.slug()))
				)
				.with_child(XmlNode::new("category").with_text("music"))
				.with_child(XmlNode::new("category").with_text("album"))
				.with_child(
					XmlNode::new("source")
						.with_attribute("url", "https://music.astronomy487.com/rss.xml")
						.with_text("Astro's discography")
				)
				.with_child(
					XmlNode::new("enclosure")
						.with_attribute(
							"url",
							format!("https://music.astronomy487.com/{}.jpg", album.slug())
						)
						.with_attribute("length", {
							let path = std::path::Path::new(globals::filezone())
								.join("music.astronomy487.com")
								.join(album.slug())
								.with_extension("jpg");
							format!("{}", crate::fileops::filesize(&path))
						})
						.with_attribute("type", "image/jpeg")
				)
				.with_child(XmlNode::new("pubDate").with_text(album.released.to_rfc822()));
			if let Some(description) = &album.about {
				item.add_child(
					XmlNode::new("description").with_text(description.replace("\n\n", " "))
				);
			}
			channel.add_child(item);
		}
	}
	let rss = XmlNode::new("rss")
		.with_attribute("version", "2.0")
		.with_child(channel);

	let mut f = std::fs::File::create(
		std::path::Path::new(globals::filezone())
			.join("music.astronomy487.com")
			.join("rss")
			.with_extension("xml")
	)
	.expect("Couldn't write to rss.xml");
	let _ = std::io::Write::write_all(&mut f, format!("{}", rss).as_bytes());
}
