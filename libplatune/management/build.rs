#[cfg(feature = "ffi")]
fn main() {
    uniffi::generate_scaffolding("src/management.udl").unwrap();
}

#[cfg(not(feature = "ffi"))]
fn main() {}
