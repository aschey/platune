package platune

import (
	"crypto/tls"
	"os"
	"time"

	grpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

func GetHttpClient(address string) (*grpc.ClientConn, error) {
	var transportCreds credentials.TransportCredentials
	if os.Getenv("PLATUNE_CLIENT_TLS") == "1" {
		cert, err := tls.LoadX509KeyPair(os.Getenv("PLATUNE_MTLS_CLIENT_CERT_PATH"), os.Getenv("PLATUNE_MTLS_CLIENT_KEY_PATH"))
		if err != nil {
			return nil, err
		}
		transportCreds = credentials.NewTLS(&tls.Config{
			Certificates: []tls.Certificate{cert},
			ClientAuth:   tls.RequireAndVerifyClientCert,
		})
	} else {
		transportCreds = insecure.NewCredentials()
	}

	// Eagerly test connection on initial connect
	_, err := grpc.NewClient(address, grpc.WithConnectParams(grpc.ConnectParams{MinConnectTimeout: 1 * time.Second}), grpc.WithTransportCredentials(transportCreds), grpc.WithBlock())
	if err != nil {
		return nil, err
	}
	return grpc.NewClient(address, grpc.WithTransportCredentials(transportCreds))
}
