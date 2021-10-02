go build -o out/platune-cli.exe
Copy-Item out/platune-cli.exe ~/Programs/platune-cli.exe
./out/platune-cli.exe "$args"