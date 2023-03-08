go build -o out/platune-cli
cp out/platune-cli ~/.bin/platune-cli
./out/platune-cli "$@"