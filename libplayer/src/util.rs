use gstreamer::ClockTime;

#[cfg(not(target_os = "windows"))]
pub static SEPARATOR: &str = "/";
#[cfg(target_os = "windows")]
pub static SEPARATOR: &str = "\\";

pub fn get_filename_from_path(path: &str) -> String {
    path.split(SEPARATOR).last().unwrap().to_owned()
}

pub fn clocktime_to_seconds(clocktime: ClockTime) -> f64 {
    clocktime.nseconds().unwrap_or_default() as f64 / 1e9
}
