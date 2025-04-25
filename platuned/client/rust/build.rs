fn main() {
    println!("cargo::rerun-if-changed=../../proto/*");

    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &[
                "../../proto/player/v1/player.proto",
                "../../proto/management/v1/management.proto",
            ],
            &["../../proto/"],
        )
        .unwrap();
}
