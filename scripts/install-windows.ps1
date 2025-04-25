$repoDir = Resolve-Path $PSScriptRoot/..
Set-Location $repoDir\platuned\server

cargo build --release
platunectl stop
taskkill -f -im 'platune-tray'
Set-Location $repoDir/platune-tray
cargo packager --release

Copy-Item "${repo_dir}\target\release\platuned" "${LOCAL_BIN}\" 
Copy-Item "${repo_dir}\target\release\platunectl" "${LOCAL_BIN}\"
Copy-Item "${repo_dir}\platune-cli"

go build .
Copy-Item .\cli "${LOCAL_BIN}\platune-cli"

platunectl start
&"${repo_dir}\target\release\platune-tray_0.1.0_x64-setup.exe"

platunectl tray enable
