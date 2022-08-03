use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("rpc_descriptor.bin"))
        .compile(
            &["../proto/player_rpc.proto", "../proto/management_rpc.proto"],
            &["../proto/"],
        )
        .unwrap();
}
