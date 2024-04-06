use itertools::Itertools;
use lofty::{Accessor, AudioFile, ItemKey, TaggedFile, TaggedFileExt};

#[derive(Debug, Hash, Default)]
pub(crate) struct Tag {
    pub(crate) title: String,
    pub(crate) album_artists: String,
    pub(crate) album: String,
    pub(crate) artists: String,
    pub(crate) track_number: u32,
    pub(crate) disc_number: u32,
    pub(crate) year: u32,
    pub(crate) duration: i64,
    pub(crate) sample_rate: u32,
    pub(crate) bitrate: u32,
}

impl From<TaggedFile> for Tag {
    fn from(tagged_file: TaggedFile) -> Self {
        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => Some(primary_tag),
            // If the "primary" tag doesn't exist, we just grab the
            // first tag we can find. Realistically, a tag reader would likely
            // iterate through the tags to find a suitable one.
            None => tagged_file.first_tag(),
        };
        let props = tagged_file.properties();
        match tag {
            Some(tag) => {
                let artists = tag.get_strings(&ItemKey::TrackArtist).join("/");
                let mut album_artists = tag.get_strings(&ItemKey::AlbumArtist).join("/");
                if album_artists.is_empty() {
                    album_artists.clone_from(&artists);
                }
                Tag {
                    title: tag.title().unwrap_or_default().into_owned(),
                    artists,
                    album: tag.album().unwrap_or_default().into_owned(),
                    track_number: tag.track().unwrap_or(1),
                    disc_number: tag.disk().unwrap_or(1),
                    year: tag.year().unwrap_or(0),
                    duration: props.duration().as_millis() as i64,
                    sample_rate: props.sample_rate().unwrap_or(0),
                    bitrate: props.audio_bitrate().unwrap_or(0),
                    album_artists,
                }
            }
            None => Default::default(),
        }
    }
}
