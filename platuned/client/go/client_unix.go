//go:build !windows
// +build !windows

package platune

import (
	grpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func GetIpcClient() (*grpc.ClientConn, error) {

	return grpc.NewClient(
		"unix:///tmp/platune/platuned.sock",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
}
