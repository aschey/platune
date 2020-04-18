use super::super::schema::folder;

#[derive(Queryable)]
pub struct Folder {
    pub folder_id: i32,
    pub full_path_unix: String,
    pub full_path_windows: String
}

#[derive(Insertable)]
#[table_name = "folder"]
pub struct NewFolder {
    pub full_path_unix: String,
    pub full_path_windows: String
}