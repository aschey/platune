#!/usr/bin/env bash
cargo test --features=dummy
(cd "$(dirname $0)/../platune-cli" && go test ./...)

