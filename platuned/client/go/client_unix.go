//go:build !windows
// +build !windows

package platune

import (
	"fmt"

	grpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func GetIpcClient(name string) (*grpc.ClientConn, error) {

	return grpc.NewClient(
		fmt.Sprintf("unix:///tmp/platune/%s.sock", name),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
}
