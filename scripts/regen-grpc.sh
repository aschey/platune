#! /usr/bin/env bash
cargo clean -p platuned-client -p platuned
cargo build
"$(dirname $0)/../platuned/client/go/gen.sh"
