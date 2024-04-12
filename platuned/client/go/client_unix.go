//go:build !windows
// +build !windows

package platune

import (
	"os"
	"path"
	"path/filepath"
	"runtime"

	grpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func GetIpcClient() (*grpc.ClientConn, error) {
	socketBaseDir := ""
	if runtime.GOOS == "darwin" {
		homeDir, err := os.UserHomeDir()
		if err == nil {
			tempPath := filepath.Join(homeDir, "Library/Caches/TemporaryItems")
			if _, err := os.Stat(tempPath); err == nil {
				socketBaseDir = tempPath
			} else {
				socketBaseDir = os.TempDir()
			}
		} else {
			socketBaseDir = os.TempDir()
		}
	} else {
		xdgDir, isSet := os.LookupEnv("XDG_RUNTIME_DIR")
		if isSet {
			socketBaseDir = xdgDir
		} else {
			socketBaseDir = os.TempDir()
		}
	}

	return grpc.NewClient(
		"unix://"+path.Join(socketBaseDir, "platune/platuned.sock"),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
}
