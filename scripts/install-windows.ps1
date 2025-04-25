$repoDir = Resolve-Path $PSScriptRoot/..
Set-Location $repoDir\platuned\server

cargo build --release
platunectl stop
taskkill -f -im 'platune-tray.exe'
Set-Location $repoDir/platune-tray
cargo packager --release

Copy-Item "${repoDir}\target\release\platuned.exe" "$Env:LOCAL_BIN\" 
Copy-Item "${repoDir}\target\release\platunectl.exe" "$Env:LOCAL_BIN\"
Set-Location "${repoDir}\platune-cli"

go build .
Copy-Item .\cli.exe "$Env:LOCAL_BIN\platune-cli.exe"

platunectl start
&"${repoDir}\target\release\platune-tray_0.1.0_x64-setup.exe"

platunectl tray enable
