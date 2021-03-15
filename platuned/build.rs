use std::{env, ops::Deref, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("player_rpc_descriptor.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &["../platuned_client/proto/player_rpc.proto"],
            &["../platuned_client/proto/"],
        )
        .unwrap();
}
