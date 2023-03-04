//go:build !windows
// +build !windows

package platune

import (
	"os"
	"path"

	grpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func GetIpcClient() (*grpc.ClientConn, error) {
	socketBaseDir, isSet := os.LookupEnv("XDG_RUNTIME_DIR")
	if !isSet {
		socketBaseDir = "/tmp"
	}
	return grpc.Dial(
		"unix://"+path.Join(socketBaseDir, "platuned/platuned.sock"),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
}
