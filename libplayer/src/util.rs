#[cfg(not(target_os = "windows"))]
static SEPARATOR: &str = "/";
#[cfg(target_os = "windows")]
static SEPARATOR: &str = "\\";

pub fn get_filename_from_path(path: &str) -> String {
    path.split(SEPARATOR).last().unwrap().to_owned()
}
