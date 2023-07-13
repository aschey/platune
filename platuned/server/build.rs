use std::{env, error::Error, path::PathBuf};

use daemon_slayer::build_info::vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("rpc_descriptor.bin"))
        .compile(
            &["../proto/player_rpc.proto", "../proto/management_rpc.proto"],
            &["../proto/"],
        )?;

    EmitBuilder::builder()
        .build_timestamp()
        .cargo_debug()
        .cargo_target_triple()
        .git_branch()
        .git_commit_count()
        .git_commit_timestamp()
        .git_describe(true, true, None)
        .git_sha(false)
        .rustc_semver()
        .sysinfo_os_version()
        .emit()?;
    Ok(())
}
