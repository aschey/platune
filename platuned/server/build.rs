use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_files = ["../proto/player_rpc.proto", "../proto/management_rpc.proto"];
    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={proto_file}");
    }

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("rpc_descriptor.bin"))
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::pbjson_types")
        .compile(&proto_files, &["../proto/"])
        .unwrap();

    let descriptor_path = out_dir.join("rpc_descriptor.bin");
    let descriptor_set = std::fs::read(descriptor_path).unwrap();

    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)
        .unwrap()
        .build(&["."])
        .unwrap();
}
