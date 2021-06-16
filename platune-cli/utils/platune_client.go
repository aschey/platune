package utils

import (
	"context"
	"fmt"
	"math"
	"os"
	"strconv"
	"strings"
	"time"

	platune "github.com/aschey/platune/client"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

type PlatuneClient struct {
	playerClient     platune.PlayerClient
	managementClient platune.ManagementClient
}

func NewPlatuneClient() PlatuneClient {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())
	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	playerClient := platune.NewPlayerClient(conn)
	managementClient := platune.NewManagementClient(conn)
	return PlatuneClient{playerClient: playerClient, managementClient: managementClient}
}

func NewTestClient(playerClient platune.PlayerClient, managementClient platune.ManagementClient) PlatuneClient {
	return PlatuneClient{playerClient: playerClient, managementClient: managementClient}
}

func (p *PlatuneClient) AddToQueue(song string) {
	p.runPlayerCommand("Added", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.AddToQueue(ctx, &platune.AddToQueueRequest{Song: song})
	})
}

func (p *PlatuneClient) Pause() {
	p.runPlayerCommand("Paused", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Pause(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Stop() {
	p.runPlayerCommand("Stopped", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Stop(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Next() {
	p.runPlayerCommand("Next", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Next(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Previous() {
	p.runPlayerCommand("Previous", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Previous(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Resume() {
	p.runPlayerCommand("Resumed", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Resume(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) SetQueue(queue []string) {
	p.runPlayerCommand("Queue Set", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetQueue(ctx, &platune.QueueRequest{Queue: queue})
	})
}

func (p *PlatuneClient) Seek(time string) {
	timeParts := strings.Split(time, ":")
	totalMillis := uint64(0)
	for i := 0; i < len(timeParts); i++ {
		intVal, err := strconv.ParseUint(timeParts[i], 10, 64)
		if err != nil {
			fmt.Printf("Error: %s is not a valid integer\n", timeParts[i])
			return
		}
		pos := float64(len(timeParts) - 1 - i)
		totalMillis += uint64(math.Pow(60, pos)) * intVal * 1000
	}
	p.runPlayerCommand("Seeked to "+time, func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Seek(ctx, &platune.SeekRequest{Millis: totalMillis})
	})
}

func (p *PlatuneClient) Sync() (platune.Management_SyncClient, context.CancelFunc) {
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Second)
	sync, err := p.managementClient.Sync(ctx, &emptypb.Empty{})

	if err != nil {
		fmt.Println(err)
		return nil, cancel
	}
	return sync, cancel
}

func (p *PlatuneClient) runPlayerCommand(successMsg string, cmdFunc func(platune.PlayerClient, context.Context) (*emptypb.Empty, error)) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	_, err := cmdFunc(p.playerClient, ctx)
	cancel()
	if err != nil {
		fmt.Println(err)
		return
	}

	fmt.Println(successMsg)
}

var Client = NewPlatuneClient()
