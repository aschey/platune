fn main() {
    if cfg!(windows) {
        println!("cargo:rustc-link-search=C:\\Program Files (x86)\\taglib\\lib");
    }
}
