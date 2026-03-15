# distri

distri is a music management suite I am making to handle all my music. It routes information from a folder of source audio/images/data into two directories: one to be served as a static website, and the other to be an object storage bucket.

It expects the current working directory to contain the following four folders:

- `audio.astronomy487.com`, stores the contents to be synced to the R2 bucket served at audio.astronomy487.com
- `music.astronomy487.com`, stores the contents to be served at the static site music.astronomy487.com (actually this one is optional; distri generates it anew every run, so it can create this folder if it's missing)
- `private`, stores private intermediate files
- `source`, stores the original copies of all files

In the source folder, it expects

- `source/audio`, flacs of every song (albums in folders)
- `source/artwork`, 3000x3000 PNGs of album/single artwork and their captions
- `source/88x31`, 88x31 gifs to be associated with albums
- `source/lyrics`, lyric tsvs
- `source/discog.json`, a JSON of all music data, as described in [[discog format.md]]. a version of this is shown at [[discog-example.json]] (no guarantee of updating, this just exists as an example)

It can perform the following functions:

- `distri validate` Validate discog.json without encoding anything.
- `distri encode` Encode audio for audio.astronomy487.com.
- `distri build` Build static website for music.astronomy487.com.
- `distri clean` Clean out non-source files from the directory. Re-encoding everything will take a while, so be careful!
- `distri publish` Publish content to Cloudflare R2 bucket and pages workers. (Will run encode and build beforehand.)

It depends on tools rclone.exe, ffmpeg.exe, and wrangler.cmd to be installed and available on your path. rclone and wrangler should already be configured with your credentials.

I write "you" as if anybody other than me is expected to execute this program

## wishlist before it's ready for actual use

- anything marked TODO or WISHLIST
- make sure synced lyric data actually makes it to USLT in mp3s; the audio players i own have failed me and i'm suspicious
- get rid of "Unknown" vocalists once and for all
- standardize songs with multiple artists to always use commas instead of ampersand. i don't like ampersand
- go back into your lyrics - whenever many people sing at once, you should just pick one as the primary and mark that for lyrics

### changelog

- v0.7.0
	- song/album durations are now read from the .flac sources at validation time, instead of included in discog.json
	- durations are now managed by the `Duration` type in duration.rs
	- source/artwork (formlerly source/image) and source/lyrics now have folders for each album, just like source/audio expects
	- `globals::filezone()` now gives a `PathBuf` instead of a `&'static str`. this is nicer i think
	- link pages css has been cleaned up; three sections are now tables identified with classes: .metadata, .streamlinks, .bottom
	- link pages' download links now have an icon! it even bounces boioioioioing
	- add lyric webpages. sync play along!
	- rework lyric vocalists to store Rc<Vec<String>> instead of just a string. saves on a lil memory probably makes cloning faster
		- several vocalists are now indicated by several `vocalist:` entries in source lyrics tsvs
	- remixes can have artwork now. idk if i'll ever go through and do that. but partial participation is totally supported so it's chill
	- colored scrollbars because firefox supports that now
	- artwork is no longer passed as strings (sometimes-but-not-always with a parent folder for the album). `Artwork` struct exists (yay!)
		- alt="" and aria-hidden="true" strictly followed for decorative items
		- artwork (which is non-decorative!) will always come with alt text; `source/artwork` now requires txt caption files
	- make modules hierarchical for better organization (imo)
	- upcs and isrcs now have their own types, `UPC` and `ISRC`
	- albums can now have cute little 88x31 gifs
- v0.6.0
	- minor fixes to logos/icons
	- bouncier icons (squash && stretch)
	- random homepage tweaks, minor css animations
	- remove unused dependencies itertools, rayon, and sha2. i forget why they were there. goodbye!
	- `compute_slug` now turns em/en dashes into hyphens
		- global.rs now has a test function. i should use test functions more often
- v0.5.0
	- added icon for external links on home page (assets/icon/external.svg)
	- `Album` attribute `about` is now an `Option<Vec<String>>` instead of a `Option<String>`, so paragraphs are already separated. whitespace is validated. the original discog.json still has \n\n
	- removed `<meta name="theme-color" />` from the home page
	- albums, remixes, and assists are now required to be non-decreasing (instead of just monotonic)
	- missing lyrics will panic only if we tell it to panic in globals.rs. this lets me build the static website with missing lyrics if i want - this will eventually be phased out once all the lyrics are done. encoding is not allowed if missing lyrics are allowed
	- albums and songs can now specify custom slugs; useful for titles that don't use roman alphabet
	- more lyric formats for download (full list: txt lrc srt vtt tsv)
	- album titles in webpages now use `<cite>` instead of `<i>` because it's more semantic
	- include RSS items for assists
	- static website only includes public-facing artwork. no bonus track artwork
- v0.4.0
	- moved link page generation to its own page, using my own xml.rs instead of maud
	- moved home page generation to its own page, using my own xml.rs instead of maud
	- moved lyric tsvs from within the json to its own source directory
	- web assets are now generated by distri itself (no source/webassets folder)
	- xml can do escape sequences now
	- runtime css compression, manual js compression (it's fine)
	- april 1 css nuke easter egg
	- remixes must provide a genre (no longer implicit Electronic)
- v0.3.0
	- stronger lints, better printing
	- albums now have genres
	- public filenames no longer use slugs - they are the actual title! wow
	- made `released`, `palette`, `artwork` are no longer optional on Songs. they inherit these properties from parent albums when absent in the json
	- made it so the slug for a song/album is only ever computed once. this took so much refactoring
- v0.2.0
	- move authorization checks for wrangler and rclone to `distri_publish()`
	- hasten check that ffmpeg, wrangler, and rclone and installed by just checking path environment variable
	- made album palettes mandatory
	- encoding now includes genre as "Electronic", for both mp3 and flac
	- further validate URLs, UPCs, ISRCs, and lyrics. normalize urls for astronomy487.com to end with slashes
	- decouple `parent_album: Option<&Album>` from `Titlable::Song`; now we check `song.parent_album_indices` to see if a parent album exists, and `all_albums: &[Album]` gets passed around everywhere
	- also re-encode the PNGs i distribute (because metadata concerns; the `image` crate wipes metadata on writes)
	- Codec enum becomes AudioCodec, introduce ImageCodec and TextCodec (do text file formats really count as codecs? whatever)
- v0.1.0
	- first fully working version, start of changelog
	- i'm just relieved i got wrangler and rclone to do my bidding