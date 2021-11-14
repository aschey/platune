#!/usr/bin/env sh
set -e

cargo clean -p platuned-client -p platuned
cargo build
echo "regenerated rust client/server"
(cd "$(dirname $0)/../platuned/client/go" && ./gen.sh)
(cd "$(dirname $0)/../platune-cli" && ./gen-mock.sh)
