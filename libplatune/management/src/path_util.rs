use std::path::Path;

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

pub(crate) fn clean_file_path<P>(file_path: &P, mount: &Option<String>) -> String
where
    P: AsRef<Path>,
{
    let file_path = file_path.as_ref();
    let mut file_path_str = file_path.to_string_lossy().to_string();
    if cfg!(windows) {
        file_path_str = file_path_str.replace(r"\", r"/");
    }

    if let Some(ref mount) = mount {
        if file_path_str.starts_with(&mount[..]) {
            file_path_str = file_path_str.replace(&mount[..], "");
        }
    }

    file_path_str
}

pub(crate) fn update_path<T, P>(entry: &mut T, mount: &P)
where
    T: PathMut,
    P: AsRef<Path>,
{
    let mount = mount.as_ref();
    entry.update_path(mount.join(entry.get_path()).to_string_lossy().to_string())
}