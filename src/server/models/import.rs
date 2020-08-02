use super::super::schema::import_temp;

#[derive(Insertable, Clone)]
#[table_name = "import_temp"]
pub struct NewImport {
    pub import_song_path_unix: String,
    pub import_song_path_windows: String,
    pub import_artist: String,
    pub import_album_artist: String,
    pub import_title: String,
    pub import_album: String,
    pub import_track_number: i32,
    pub import_disc_number: i32,
    pub import_year: i32,
    pub import_duration: i32,
    pub import_sample_rate: i32,
    pub import_bit_rate: i32,
    pub import_album_art: Option<Vec<u8>>,
}
