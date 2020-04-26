use super::super::schema::import_temp;

#[derive(Insertable, Clone)]
#[table_name = "import_temp"]
pub struct NewImport {
    pub import_song_path: String,
    pub import_modified_date: i32,
    pub import_artist: String,
    pub import_title: String,
    pub import_album: String
}
