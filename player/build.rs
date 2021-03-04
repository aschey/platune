use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("player_rpc_descriptor.bin"))
        .compile(
            &["../player_client/proto/player_rpc.proto"],
            &["../player_client/proto/"],
        )
        .unwrap();
}
