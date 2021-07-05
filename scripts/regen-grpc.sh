#!/usr/bin/env bash
cargo clean -p platuned-client -p platuned
cargo build
(cd "$(dirname $0)/../platuned/client/go" && ./gen.sh)
(cd "$(dirname $0)/../platune-cli" && ./gen-mock.sh)
