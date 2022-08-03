fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile(
            &[
                "../../proto/player_rpc.proto",
                "../../proto/management_rpc.proto",
            ],
            &["../../proto/"],
        )
        .unwrap();
}
