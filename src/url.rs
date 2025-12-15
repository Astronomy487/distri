macro_rules! define_urlset {
	(
		before_youtube: [ $( ($label_before:expr, $field_before:ident) ),* $(,)? ],
		after_youtube:  [ $( ($label_after:expr,  $field_after:ident ) ),* $(,)? ]
	) => {
		pub struct UrlSet {
			$( $field_before: Option<String>, )*
			youtube_video: Option<String>,
			youtube_playlist: Option<String>,
			$( $field_after: Option<String>, )*
		}
		impl UrlSet {
			pub fn from(val: &serde_json::Value, is_album: bool) -> UrlSet {
				let obj = crate::globals::map_with_only_these_keys(
					val,
					"UrlSet",
					&[
						$( $label_before, )*
						"YouTube",
						"YouTube Full Mix",
						$( $label_after, )*
					]
				);
				fn get_str(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<String> {
					obj.get(key).map(|v| {
						v.as_str()
							.unwrap_or_else(|| panic!("\"url\" attribute \"{}\" from JSON is not a string", key))
							.to_string()
					})
				}
				let yt = get_str(&obj, "YouTube");
				let (youtube_video, youtube_playlist) = if is_album {
					(None, yt)
				} else {
					(yt, None)
				};
				UrlSet {
					$( $field_before: get_str(&obj, $label_before), )*
					youtube_video,
					youtube_playlist,
					$( $field_after: get_str(&obj, $label_after), )*
				}
			}

			pub fn iter(&self) -> Vec<(&'static str, &String)> {
				let mut out = Vec::new();
				$(
					if let Some(v) = &self.$field_before {
						out.push(($label_before, v));
					}
				)*
				if let Some(v) = &self.youtube_video {
					out.push(("YouTube", v));
				}
				if let Some(v) = &self.youtube_playlist {
					out.push(("YouTube", v));
				}
				$(
					if let Some(v) = &self.$field_after {
						out.push(($label_after, v));
					}
				)*
				out
			}
			pub fn combine(a: Option<&UrlSet>, b: Option<&UrlSet>) -> UrlSet {
				fn pick(a: Option<&String>, b: Option<&String>) -> Option<String> {
					a.cloned().or_else(|| b.cloned())
				}
				let a_video = a.and_then(|x| x.youtube_video.as_ref());
				let a_playlist = a.and_then(|x| x.youtube_playlist.as_ref());
				let youtube_video = a.and_then(|x| x.youtube_video.clone()).or_else(|| {
					if a_playlist.is_some() { None }
					else { b.and_then(|x| x.youtube_video.clone()) }
				});
				let youtube_playlist = a.and_then(|x| x.youtube_playlist.clone()).or_else(|| {
					if a_video.is_some() { None }
					else { b.and_then(|x| x.youtube_playlist.clone()) }
				});
				UrlSet {
					$(
						$field_before: pick(
							a.and_then(|x| x.$field_before.as_ref()),
							b.and_then(|x| x.$field_before.as_ref())
						),
					)*
					youtube_video,
					youtube_playlist,
					$(
						$field_after: pick(
							a.and_then(|x| x.$field_after.as_ref()),
							b.and_then(|x| x.$field_after.as_ref())
						),
					)*
				}
			}
		}
	};
}

define_urlset! {
	before_youtube: [
		("Bandcamp", bandcamp)
	],
	after_youtube: [
		("YouTube Full Mix", youtube_full_mix),
		("Apple Music", apple_music),
		("Spotify", spotify),
		("Soundcloud", soundcloud),
		("Amazon Music", amazon_music),
		("iHeartRadio", iheartradio),
		("Tencent Music", tencent_music)
	]
}
