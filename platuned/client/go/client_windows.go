//go:build windows
// +build windows

package platune

import (
	context "context"
	"net"

	"github.com/Microsoft/go-winio"
	grpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func GetIpcClient() (*grpc.ClientConn, error) {
	return grpc.Dial("dummy", grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithContextDialer(func(ctx context.Context, s string) (net.Conn, error) {
			conn, err := winio.DialPipe(`\\.\pipe\platuned`, nil)
			return conn, err
		}))
}
