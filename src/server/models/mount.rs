use super::super::schema::mount;

#[derive(Insertable)]
#[table_name = "mount"]
pub struct NewMount {
    pub unix_path: String,
    pub windows_path: String
}