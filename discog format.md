# discog format

https://music.astronomy487.com/discog.js is a big UTF-8 json record of every song i've ever released in any capacity. the big object (called `discog`) has two attributes :

- `discog.albums`, an array of Albums i've released
- `discog.remixes`, an array of Songs, each being a remix i have released (i.e. based on someone else's music) sorted in reverse-chronological release order

singles are stored as a type of album. for each of the types below, all fields are optional unless marked as required.

## Album format

an Album contains the following fields :

- `title` (required), a string title for the album
- `compilation`, a boolean indicating if this is a compilation album
- `artist`, a string for the artist of the album. if not present, artist is assumed to be "Astro"
- `single`, a boolean indicating if this is a single. if so, this album has exactly 1 non-bonus song. the song's title and artist fields will match the album's. urls will probably only be supplied by the parent (but like, your code should check for song.url and use album.url as a fallback anyways. so)
- `released` (required), a string in YYYY-MM-DD format for the release date of the album
- `length` (required), the total length of the album (excluding bonus tracks) in seconds. may differ a little from the sum of song lengths because of rounding
- `bcid`, a string identifier used for bandcamp album embeds
- `url`, a Url object (more details below) that links to this album on various platforms
- `genre` (required), a string representing the genre. see src/genre.rs for currently accepted genres
- `color` (required), a Color object (more details below) for a three-color palette that complements the artwork
- `songs`, a list of Songs present on the album, including bandcamp-exclusive bonus tracks
- `about`, a string message that describes the album
- `upc`, the album UPC
- `temporary`, marked as true if this is just a single for an upcoming album

## Song format

songs can either be remixes (found in `discog.remixes`) or non-remixes (found in `discog.albums[i].songs`)

- `title` (required), a string title for the song. a fully formatted song title should include artist ("[Artist] - [Title]").
- `artist`, a string for the artist of the song. if not present, artist is assumed to be "Astro". remixers and featured artists are kept in the title. if a remix has no primary artist listed, then no artist should be presented in a fully formatted song title.
- `length` (required), the length of the song in seconds
- `released`, a string in YYYY-MM-DD format for the release date of the song. required for remixes; non-remixes may inherit release date from the parent album
- `url`, a Url object (more details below) that links to the song on various platforms
- `artwork`, artwork to represent a single song. either `true` if the location of single artwork is named after the song, or a string if it has some other name
- `bonus`, a boolean indicating if this is a bandcamp-exclusive bonus track. bonus tracks usually don't have public urls, except when they do
- `lyrics`, a boolean if lyrics are provided
- `color`, a Color object (more details below) for a three-color palette that complements the artwork. non-remixes may inherit color palette from the parent album
- `samples`, a list of strings, where each string is the full name of a song sampled on this song
- `event` a boolean indicating if this is a dj set for an event. if so, then a fully formatted song title should appear as "Astro @ [Title]" (the artist field won't be supplied!)
- `isrc`, the track ISRC
- `about`, a string message that describes the track, with paragraphs separated by \n\n
- `genre`, a string representing the genre. cannot be provided if song is on an album. must be provided if song is not on an album.

## Color format

Color objects have just three properties, each of which are a string hex code (e.g. "#FF0000"). they form a little color palette for use with a particular song or album.

- `background`, the background
- `foreground`, the foreground
- `accent`, the accent color

colors have an optional fourth property, `mode`, one of "black" or "white". if present, colorful ui elements (namely logos of other services) should appear as black or white instead of in color. i only use this when the background is not sufficiently black (greater than like 25% brightness)

the foreground and accent colors should appear on top of the background. the accent color should only be used for larger/bolder text. the foreground color and accent color should not appear on top of one another!

the foreground and background colors have a contrast ratio of at least 4.5. the accent and background colors have a contrast ratio of at least 3. these meet the WCAG AA accessibility guidelines. distri checks for this

## Url format

a Url object holds urls to an item across streaming platforms. the possible keys are `Bandcamp`, `YouTube`, `YouTube Full Mix`, `Apple Music`, `Spotify`, `Soundcloud`, `Amazon Music`, `iHeartRadio`, and `Tencent Music`

## Lyrics format

if a song has lyrics, distri will check source/lyrics for a tsv. the first three columns are fixed:

1. a start time for a line (six decimal points of precision)
2. an end time for a line (six decimal points of precision)
3. the text for the line

subsequent columns should be key:value pairs (e.g. language:en and vocalist:Astro). future rows will inherit values above if they are not specified. every row must have a defined language and vocalist. language use ISO 639 langauge codes (2 or 3 letters acceptable)

empty lines (not even \t allowed) can be used to separate stanzas

## Notes on slugs

every release and song has an ascii identifier composed of lowercase alphanumeric characters and hyphens. it is derived from the item's title and artist. this is how i generate them:

```js
// where item has attributes .title: string and .artist: string (optional)
function linkName(item) {
	let title = item.title;
	if ("artist" in item && item.artist != "Astro" && item) title = item.artist + " " + item.title;
	title = title.toLowerCase();
	title = title.normalize("NFD").replace(/[\u0300-\u036f]/g, '');
	title = title.replace("a$tro", "astro");
	title = title.replace(/[()[\],.?!'"*$]/g, "");
	title = title.replace(/[_/&+:;\s]+/g, '-').replace(/-+/g, '-');
	title = title.split('').filter(c => c.charCodeAt(0) >= 0x00 && c.charCodeAt(0) <= 0x7f).join("");
	title = title.replaceAll("--", "-");
	while (title.startsWith("-")) title = title.substring(1);
	while (title.endsWith("-")) title = title.substring(0, title.length-1);
	return title;
}
```

```rust
pub fn compute_slug(artist: &str, title: &str) -> String {
	let mut slug = if artist == "Astro" {
		title.to_owned()
	} else {
		format!("{} {}", artist, title)
	};
	slug = slug.to_lowercase();
	slug = unicode_normalization::UnicodeNormalization::nfd(slug.chars())
		.filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
		.collect();
	slug = slug.replace("a$tro", "astro");
	let re_punct = regex::Regex::new(r#"[()\[\],.?!'"*\$]"#).expect("re_punct is invalid regex");
	let re_sep = regex::Regex::new(r#"[_/&+:;\s]+"#).expect("re_sep is invalid regex");
	let re_dash = regex::Regex::new(r#"-+"#).expect("re_dash is invalid regex");
	slug = re_punct.replace_all(&slug, "").into_owned();
	slug = re_sep.replace_all(&slug, "-").into_owned();
	slug = re_dash.replace_all(&slug, "-").into_owned();
	slug = slug.chars().filter(char::is_ascii).collect();
	while slug.starts_with('-') {
		slug = slug[1..].to_string();
	}
	while slug.ends_with('-') {
		slug = slug[..slug.len() - 1].to_string();
	}
	slug = slug.replace("--", "-");
	assert!(
		slug.chars()
			.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
		"Could not generate a valid slug for artist {} and title {}; we arrived at {}",
		artist,
		title,
		slug
	);
	slug
}
```

these are only non-unique for singles, in which case the 'song' and the 'single' from which it came have the same identifier (i think that's ok)