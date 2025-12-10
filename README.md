# distri

distri is a music management suite I am making to handle all my music. Because I have a lot of music.

**The mission**: Take a whole bunch of source files (wav/flac audio, png images, json data) and convert them into downloadable mp3/flac audio with rich metadata (available at audio.astronomy487.com) and static website content (available at music.astronomy487.com).

## feature wishlist

- [X] Read source/discog.json into data structures
- [X] Encode songs
- [X] Give the mp3s and flacs nice metadata
- [X] Zip into nice album things
- [X] Generate the website for me
- [X] Put everything in audio.astronomy487.com directory organized so I can easily sync with R2 bucket
- [X] rss.xml
- [ ] Rework the `do_encode` logic to only find `parent_album: Option<Album>` as it needs to, in conjunction with a match on `self.parent_album: (usize, usize)`. I cannot keep using asserts like this This is not idiomatic of me