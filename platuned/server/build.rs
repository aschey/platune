use std::env;
use std::error::Error;
use std::path::PathBuf;

use vergen_gix::{Build, Cargo, Emitter, Gix, Rustc, Sysinfo};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=../proto/*");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("rpc_descriptor.bin"))
        .compile_protos(
            &[
                "../proto/player/v1/player.proto",
                "../proto/management/v1/management.proto",
            ],
            &["../proto/"],
        )?;

    Emitter::default()
        .add_instructions(&Build::builder().build_timestamp(true).build())?
        .add_instructions(&Cargo::builder().target_triple(true).build())?
        .add_instructions(
            &Gix::builder()
                .branch(true)
                .commit_count(true)
                .commit_timestamp(true)
                .sha(false)
                .describe(true, true, None)
                .build(),
        )?
        .add_instructions(&Rustc::builder().semver(true).build())?
        .add_instructions(&Sysinfo::builder().os_version(true).build())?
        .emit()?;
    Ok(())
}
