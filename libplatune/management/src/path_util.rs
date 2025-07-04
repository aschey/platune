use std::io;
use std::path::Path;

use normpath::PathExt;

use crate::database::LookupEntry;

pub(crate) trait PathMut {
    fn get_path(&self) -> String;
    fn update_path(&mut self, path: String);
}

impl PathMut for LookupEntry {
    fn get_path(&self) -> String {
        self.path.to_owned()
    }
    fn update_path(&mut self, path: String) {
        self.path = path;
    }
}

impl PathMut for String {
    fn get_path(&self) -> String {
        self.to_owned()
    }

    fn update_path(&mut self, path: String) {
        *self = path
    }
}

pub(crate) fn clean_file_path<P>(file_path: &P, mount: &Option<String>) -> io::Result<String>
where
    P: AsRef<Path>,
{
    let file_path = file_path.as_ref();

    if file_path.starts_with("http://") || file_path.starts_with("https://") {
        // No need to normalize http urls
        return Ok(file_path.to_string_lossy().to_string());
    }

    let file_path = if file_path.exists() {
        file_path
            .normalize()
            .map_err(|e| {
                io::Error::new(e.kind(), format!("Error normalizing path {file_path:?}"))
            })?
            .as_path()
            .to_path_buf()
    } else {
        // Normalize call fails if the path doesn't exist so just leave it alone
        file_path.to_path_buf()
    };

    let mut file_path_str = file_path.into_os_string().to_string_lossy().to_string();
    if cfg!(windows) {
        file_path_str = file_path_str.replace('\\', "/");
    }

    if let Some(mount) = mount
        && file_path_str.starts_with(&mount[..]) {
            file_path_str = file_path_str.replace(&mount[..], "");
        }

    Ok(file_path_str)
}

pub(crate) fn update_path<T, P>(entry: &mut T, mount: &P)
where
    T: PathMut,
    P: AsRef<Path>,
{
    let mount = mount.as_ref();
    entry.update_path(mount.join(entry.get_path()).to_string_lossy().to_string())
}
