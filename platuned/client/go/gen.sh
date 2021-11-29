#!/usr/bin/env sh
CURRENT_DIR=$(dirname $0)
protoc --go_out="${CURRENT_DIR}" --go_opt=paths=source_relative --go-grpc_out="${CURRENT_DIR}" --go-grpc_opt=paths=source_relative --proto_path "${CURRENT_DIR}/../../proto" --go_opt=Mplayer_rpc.proto=github.com/aschey/platune/protos/player_rpc "${CURRENT_DIR}/../../proto/player_rpc.proto" --go_opt=Mplayer_rpc.proto=github.com/aschey/platune/protos/management_rpc "${CURRENT_DIR}/../../proto/management_rpc.proto"
echo "regenerated go client"