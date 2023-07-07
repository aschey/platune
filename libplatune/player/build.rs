#[cfg(feature = "ffi")]
fn main() {
    let target = std::env::var("TARGET").expect("ERR: Could not check the target for the build.");

    if target.contains("android") {
        add_lib("c++_shared", false);
    }

    uniffi::generate_scaffolding("src/player.udl").unwrap();
}

#[cfg(not(feature = "ffi"))]
fn main() {}

#[cfg(feature = "ffi")]
fn add_lib(name: impl AsRef<str>, _static: bool) {
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        name.as_ref()
    );
}
