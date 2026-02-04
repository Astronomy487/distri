use crate::fileops;
use crate::globals;

macro_rules! define_icons {
	( $( $name:literal ),* $(,)? ) => {
		pub fn valid_icon(name: &str) -> bool {
			match name {
				$( $name => true, )*
				_ => false,
			}
		}
		pub fn put_icons() {
			let base = std::path::Path::new(globals::filezone())
				.join("music.astronomy487.com")
				.join("icons");

			std::fs::create_dir(&base).unwrap_or_else(|_| {
				panic!("Couldn't create directory music.astronomy487.com/icons");
			});

			$(
				{
					// Each include_bytes! expands to a literal path at compile time
					let data = include_bytes!(concat!("assets/icons/", $name, ".svg"));
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
	"github",
	"iheartradio",
	"pandora",
	"rss",
	"soundcloud",
	"spotify",
	"tencentmusic",
	"tidal",
	"twitter",
	"youtube",
	"youtubefullmix",
	"greenheart"
);
