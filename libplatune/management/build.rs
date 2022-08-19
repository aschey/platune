use sloughi::Sloughi;

fn main() {
    if let Err(e) = Sloughi::new()
        .ignore_env("CI")
        .ignore_env("GITHUB_ACTIONS")
        .install()
    {
        println!("cargo:warning=Error installing git hooks: {e:?}")
    }
}
