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
