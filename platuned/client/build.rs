fn main() {
    tonic_build::configure()
        .build_server(false)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["../proto/player_rpc.proto"], &["../proto/"])
        .unwrap();
}
