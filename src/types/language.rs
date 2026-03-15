macro_rules! define_languages {
	(
		$(
			$iso1:expr, $iso2:expr, $name:ident ;
		)*
	) => {
		#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
		pub enum Language {
			$($name),*
		}
		impl Language {
			pub fn iso_639_1(&self) -> &'static str {
				match self {
					$(Language::$name => $iso1),*
				}
			}
			pub fn iso_639_2(&self) -> &'static str {
				match self {
					$(Language::$name => $iso2),*
				}
			}
			pub fn from(s: &str) -> Language {
				#[allow(unreachable_patterns)]
				match s {
					$(
						$iso1 => Language::$name,
						$iso2 => Language::$name,
					)*
					_ => panic!("Unrecognized ISO 639 language code \"{}\"", s),
				}
			}
		}
	};
}

define_languages! {
	"aa", "aar", Afar;
	"ab", "abk", Abkhazian;
	"ae", "ave", Avestan;
	"af", "afr", Afrikaans;
	"ak", "aka", Akan;
	"am", "amh", Amharic;
	"an", "arg", Aragonese;
	"ar", "ara", Arabic;
	"as", "asm", Assamese;
	"av", "ava", Avaric;
	"ay", "aym", Aymara;
	"az", "aze", Azerbaijani;
	"ba", "bak", Bashkir;
	"be", "bel", Belarusian;
	"bg", "bul", Bulgarian;
	"bi", "bis", Bislama;
	"bm", "bam", Bambara;
	"bn", "ben", Bengali;
	"bo", "bod", Tibetan;
	"br", "bre", Breton;
	"bs", "bos", Bosnian;
	"ca", "cat", Catalan;
	"ce", "che", Chechen;
	"ch", "cha", Chamorro;
	"co", "cos", Corsican;
	"cr", "cre", Cree;
	"cs", "ces", Czech;
	"cu", "chu", ChurchSlavonic;
	"cv", "chv", Chuvash;
	"cy", "cym", Welsh;
	"da", "dan", Danish;
	"de", "deu", German;
	"dv", "div", Dhivehi;
	"dz", "dzo", Dzongkha;
	"ee", "ewe", Ewe;
	"el", "ell", Greek;
	"en", "eng", English;
	"eo", "epo", Esperanto;
	"es", "spa", Spanish;
	"et", "est", Estonian;
	"eu", "eus", Basque;
	"fa", "fas", Persian;
	"ff", "ful", Fulah;
	"fi", "fin", Finnish;
	"fj", "fij", Fijian;
	"fo", "fao", Faroese;
	"fr", "fra", French;
	"fy", "fry", Frisian;
	"ga", "gle", Irish;
	"gd", "gla", ScottishGaelic;
	"gl", "glg", Galician;
	"gn", "grn", Guarani;
	"gu", "guj", Gujarati;
	"gv", "glv", Manx;
	"ha", "hau", Hausa;
	"he", "heb", Hebrew;
	"hi", "hin", Hindi;
	"ho", "hmo", HiriMotu;
	"hr", "hrv", Croatian;
	"ht", "hat", HaitianCreole;
	"hu", "hun", Hungarian;
	"hy", "hye", Armenian;
	"hz", "her", Herero;
	"ia", "ina", Interlingua;
	"id", "ind", Indonesian;
	"ie", "ile", Interlingue;
	"ig", "ibo", Igbo;
	"ii", "iii", Yi;
	"ik", "ipk", Inupiaq;
	"io", "ido", Ido;
	"is", "isl", Icelandic;
	"it", "ita", Italian;
	"iu", "iku", Inuktitut;
	"ja", "jpn", Japanese;
	"jbo", "jbo", Lojban; // technically invalid ISO 693-1 code
	"jv", "jav", Javanese;
	"ka", "kat", Georgian;
	"kg", "kon", Kongo;
	"ki", "kik", Kikuyu;
	"kj", "kua", Kwanyama;
	"kk", "kaz", Kazakh;
	"kl", "kal", Kalaallisut;
	"km", "khm", Khmer;
	"kn", "kan", Kannada;
	"ko", "kor", Korean;
	"kr", "kau", Kanuri;
	"ks", "kas", Kashmiri;
	"ku", "kur", Kurdish;
	"kv", "kom", Komi;
	"kw", "cor", Cornish;
	"ky", "kir", Kyrgyz;
	"la", "lat", Latin;
	"lb", "ltz", Luxembourgish;
	"lg", "lug", Ganda;
	"li", "lim", Limburgish;
	"ln", "lin", Lingala;
	"lo", "lao", Lao;
	"lt", "lit", Lithuanian;
	"lu", "lub", LubaKatanga;
	"lv", "lav", Latvian;
	"mg", "mlg", Malagasy;
	"mh", "mah", Marshallese;
	"mi", "mri", Maori;
	"mk", "mkd", Macedonian;
	"ml", "mal", Malayalam;
	"mn", "mon", Mongolian;
	"mr", "mar", Marathi;
	"ms", "msa", Malay;
	"mt", "mlt", Maltese;
	"my", "mya", Burmese;
	"na", "nau", Nauru;
	"nb", "nob", NorwegianBokmal;
	"nd", "nde", NorthNdebele;
	"ne", "nep", Nepali;
	"ng", "ndo", Ndonga;
	"nl", "nld", Dutch;
	"nn", "nno", NorwegianNynorsk;
	"no", "nor", Norwegian;
	"nr", "nbl", SouthNdebele;
	"nv", "nav", Navaho;
	"ny", "nya", Chichewa;
	"oc", "oci", Occitan;
	"oj", "oji", Ojibwa;
	"om", "orm", Oromo;
	"or", "ori", Oriya;
	"os", "oss", Ossetian;
	"pa", "pan", Punjabi;
	"pi", "pli", Pali;
	"pl", "pol", Polish;
	"ps", "pus", Pashto;
	"pt", "por", Portuguese;
	"qu", "que", Quechua;
	"rm", "roh", Romansh;
	"rn", "run", Rundi;
	"ro", "ron", Romanian;
	"ru", "rus", Russian;
	"rw", "kin", Kinyarwanda;
	"sa", "san", Sanskrit;
	"sc", "srd", Sardinian;
	"sd", "snd", Sindhi;
	"se", "sme", Sami;
	"sg", "sag", Sango;
	"si", "sin", Sinhala;
	"sk", "slk", Slovak;
	"sl", "slv", Slovenian;
	"sm", "smo", Samoan;
	"sn", "sna", Shona;
	"so", "som", Somali;
	"sq", "sqi", Albanian;
	"sr", "srp", Serbian;
	"ss", "ssw", Swati;
	"st", "sot", Sotho;
	"su", "sun", Sundanese;
	"sv", "swe", Swedish;
	"sw", "swa", Swahili;
	"ta", "tam", Tamil;
	"te", "tel", Telugu;
	"tg", "tgk", Tajik;
	"th", "tha", Thai;
	"ti", "tir", Tigrinya;
	"tk", "tuk", Turkmen;
	"tl", "tgl", Tagalog;
	"tn", "tsn", Tswana;
	"to", "ton", Tongan;
	"tok", "tok", TokiPona; // technically invalid ISO 693-1 code
	"tr", "tur", Turkish;
	"ts", "tso", Tsonga;
	"tt", "tat", Tatar;
	"tw", "twi", Twi;
	"ty", "tah", Tahitian;
	"ug", "uig", Uyghur;
	"uk", "ukr", Ukrainian;
	"ur", "urd", Urdu;
	"uz", "uzb", Uzbek;
	"ve", "ven", Venda;
	"vi", "vie", Vietnamese;
	"vo", "vol", Volapuk;
	"wa", "wln", Walloon;
	"wo", "wol", Wolof;
	"xh", "xho", Xhosa;
	"yi", "yid", Yiddish;
	"yo", "yor", Yoruba;
	"za", "zha", Zhuang;
	"zh", "zho", Chinese;
	"zu", "zul", Zulu;
}
