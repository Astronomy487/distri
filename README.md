# distri

distri is a music management suite I am making to handle all my music. It routes information from a folder of source audio/images/data into two directories: one to be served as a static website, and the other to be an object storage bucket.

It expects the current working directory to contain the following four folders:

- `audio.astronomy487.com`, stores the contents to be synced to the R2 bucket served at audio.astronomy487.com
- `music.astronomy487.com`, stores the contents to be served at the static site music.astronomy487.com
- `private`, stores private intermediate files
- `source`, stores the original copies of all files

In the source folder, it expects

- `source/webassets`, files to be included with the static website build
- `source/audio`, flacs of every song
- `source/image`, 3000x3000 PNGs of album artwork
- `source/discog.json`, a JSON of all music data, as described in [[discog format.md]]

It can perform the following functions:

- `distri validate` Validate discog.json without encoding anything.
- `distri encode` Encode audio for audio.astronomy487.com.
- `distri build` Build static website for music.astronomy487.com.
- `distri clean` Clean out non-source files from the directory. Re-encoding everything will take a while, so be careful!
- `distri publish` Publish content to Cloudflare R2 bucket and pages workers. (Will run encode and build beforehand.)

It depends on tools rclone.exe, ffmpeg.exe, and wrangler.cmd to be installed and available on your path. rclone and wrangler should already be configured with your credentials.

I write "you" as if anybody other than me is expected to execute this program

### changelog

- v0.1.0 - first fully working version, start of changelog