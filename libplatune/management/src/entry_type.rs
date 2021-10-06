use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum EntryType {
    Song,
    Artist,
    #[strum(serialize = "album_artist")]
    AlbumArtist,
    Album,
}
