use crate::build::xml::XmlNode;
use crate::fileops;
use crate::globals;
use crate::types::color::Color;

macro_rules! define_icons {
	( $( $name:literal ),* $(,)? ) => {
		pub fn valid_icon(name: &str) -> bool {
			match name {
				$( $name => true, )*
				_ => false,
			}
		}
		pub fn put_icons() {
			let base = globals::filezone()
				.join("music.astronomy487.com")
				.join("icons");

			std::fs::create_dir(&base).unwrap_or_else(|_| {
				panic!("Couldn't create directory music.astronomy487.com/icons");
			});

			$(
				{
					// Each include_bytes! expands to a literal path at compile time
					let data = include_bytes!(concat!("../assets/icons/", $name, ".svg"));
					fileops::write_file(
						&base.join($name).with_extension("svg"),
						data,
					);
				}
			)*
		}
	};
}

define_icons!(
	"amazonmusic",
	"applemusic",
	"bandcamp",
	"deezer",
	"discord",
	"external",
	"github",
	"greenheart",
	"iheartradio",
	"pandora",
	"rss",
	"soundcloud",
	"spotify",
	"tencentmusic",
	"tidal",
	"twitter",
	"youtube",
	"youtubefullmix"
);

pub fn inline_download_icon_svg() -> XmlNode {
	XmlNode::new("svg")
		.with_attribute("aria-label", "Download")
		.with_attribute("viewBox", "0, 0, 1000, 1000")
		.with_child(XmlNode::new("polygon").with_attribute(
			"points",
			"0,1000 1000,1000 1000,700 900,700, 900,900 100,900 100,700 0,700"
		))
		.with_child(XmlNode::new("polygon").with_attribute("points", "450,0 550,0 550,700 450,700"))
		.with_child(
			XmlNode::new("polygon")
				.with_attribute("points", "400,0 1000,0 1000,600 900,600 900,100 400,100")
				.with_attribute("transform", "translate(1200,100) rotate(135)")
		)
}

// based on https://astronomy487.com/logo/assets/cmy-big.svg
pub fn inline_logo_svg(cyan: &Color, magenta: &Color, yellow: &Color) -> XmlNode {
	XmlNode::new("svg")
		.with_attribute("viewBox", "-1000, -1000, 2000, 2000")
		.with_child(
			XmlNode::new("polygon")
				.with_attribute("points", "63,0 531.5,811.5 1000,0 531.5,-811.5")
				.with_attribute("fill", cyan.to_string())
				.with_attribute("transform", "rotate(0)")
		)
		.with_child(
			XmlNode::new("polygon")
				.with_attribute("points", "63,0 531.5,811.5 1000,0 531.5,-811.5")
				.with_attribute("fill", magenta.to_string())
				.with_attribute("transform", "rotate(120)")
		)
		.with_child(
			XmlNode::new("polygon")
				.with_attribute("points", "63,0 531.5,811.5 1000,0 531.5,-811.5")
				.with_attribute("fill", yellow.to_string())
				.with_attribute("transform", "rotate(240)")
		)
}
