use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("{:?}", out_dir);

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("player_rpc_descriptor.bin"))
        .compile(&["src/player_rpc.proto"], &["src/"])
        .unwrap();
}
