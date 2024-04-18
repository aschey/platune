go build -o out/platune-cli.exe
Copy-Item out/platune-cli.exe ~/.bin/platune-cli.exe
./out/platune-cli.exe "$args"