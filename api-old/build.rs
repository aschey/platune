fn main() {
    if cfg!(windows) {
        println!("cargo:rustc-link-search=C:\\Program Files (x86)\\taglib\\lib");
        println!("cargo:rustc-link-search=C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\MSVC\\14.25.28610\\lib\\x64")
    }
}
