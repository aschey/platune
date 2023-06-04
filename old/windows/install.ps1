Set-Location "$PSScriptRoot/../src/ui/platune"
yarn windows-pack
Set-Location ../../..
cargo build --release
$path = "$env:LOCALAPPDATA/Programs/platune-server"
If (!(test-path $path)) {
    New-Item -ItemType Directory -Force -Path $path
}
Copy-Item target/release/platune.exe "$path/platune-server.exe"
&"src/ui/platune/dist/Platune Setup 0.1.0.exe"