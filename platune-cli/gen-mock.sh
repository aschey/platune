mockgen -package=test -source=../platuned/client/go/player_rpc_grpc.pb.go > test/player_mock.go
mockgen -package=test -source=../platuned/client/go/management_rpc_grpc.pb.go > test/management_mock.go