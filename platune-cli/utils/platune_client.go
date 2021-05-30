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
	client platune.PlayerClient
}

func NewPlatuneClient() PlatuneClient {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())
	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	client := platune.NewPlayerClient(conn)
	return PlatuneClient{client: client}
}

func (p *PlatuneClient) AddToQueue(song string) {
	p.runCommand("Added", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.AddToQueue(ctx, &platune.AddToQueueRequest{Song: song})
	})
}

func (p *PlatuneClient) Pause() {
	p.runCommand("Paused", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.Pause(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Stop() {
	p.runCommand("Stopped", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.Stop(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Next() {
	p.runCommand("Next", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.Next(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Previous() {
	p.runCommand("Previous", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.Previous(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Resume() {
	p.runCommand("Resumed", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.Resume(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) SetQueue(queue []string) {
	p.runCommand("Queue Set", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.SetQueue(ctx, &platune.QueueRequest{Queue: queue})
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
	p.runCommand("Seeked to "+time, func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
		return p.client.Seek(ctx, &platune.SeekRequest{Millis: totalMillis})
	})
}

func (p *PlatuneClient) runCommand(successMsg string, cmdFunc func(platune.PlayerClient, context.Context) (*emptypb.Empty, error)) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	_, err := cmdFunc(p.client, ctx)
	cancel()
	if err != nil {
		fmt.Println(err)
		return
	}

	fmt.Println(successMsg)
}

var Client = NewPlatuneClient()
