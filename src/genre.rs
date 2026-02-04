macro_rules! define_genres {
	(
		$(
			$text:expr, $name:ident ;
		)*
	) => {
		#[derive(Clone, Debug)]
		pub enum Genre {
			$($name),*
		}
		impl Genre {
			pub fn to_string(&self) -> &'static str {
				match self {
					$(Genre::$name => $text),*
				}
			}
			pub fn from(s: &str) -> Genre {
				match s {
					$(
						$text => Genre::$name,
					)*
					_ => panic!("Unrecognized genre \"{}\"", s),
				}
			}
		}
	};
}

define_genres!(
	"Electronic", Electronic;
	"Pop", Pop;
	"Downtempo", Downtempo;
	"Dance", Dance;
	"Ambient", Ambient;
	"Jazz", Jazz;
);
