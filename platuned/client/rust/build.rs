fn main() {
    println!("cargo::rerun-if-changed=../../proto/*");

    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "../../proto/player_rpc.proto",
                "../../proto/management_rpc.proto",
            ],
            &["../../proto/"],
        )
        .unwrap();
}
