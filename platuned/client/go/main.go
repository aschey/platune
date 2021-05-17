package main

import (
	"context"
	"time"

	gen "github.com/aschey/platune/client/v2/gen"
	grpc "google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

func main() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:50051", opts...)
	defer conn.Close()
	if err == nil {
		client := gen.NewPlayerClient(conn)
		ctx, _ := context.WithTimeout(context.Background(), 10*time.Second)
		client.Pause(ctx, &emptypb.Empty{})
	} else {
		println(err.Error())
	}

}
