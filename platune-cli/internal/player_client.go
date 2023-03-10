package internal

import (
	"context"
	"fmt"
	"math"
	"strconv"
	"strings"
	"time"

	platune "github.com/aschey/platune/client"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/emptypb"
)

type PlayerClient struct {
	conn              *grpc.ClientConn
	playerClient      platune.PlayerClient
	playerEventClient *platune.Player_SubscribeEventsClient
	attemptReconnect  bool
}

func NewPlayerClient() (*PlayerClient, error) {
	conn, err := platune.GetIpcClient()
	if err != nil {
		return nil, err
	}

	playerClient := platune.NewPlayerClient(conn)
	client := &PlayerClient{
		conn:         conn,
		playerClient: playerClient,
	}

	return client, nil
}

func NewTestPlayerClient(
	playerClient platune.PlayerClient,
) PlayerClient {
	return PlayerClient{playerClient: playerClient}
}

func (p *PlayerClient) EnableReconnect() {
	p.attemptReconnect = true
}

func (p *PlayerClient) GetConnection() *grpc.ClientConn {
	return p.conn
}

func (p *PlayerClient) SubscribePlayerEvents(eventCh chan *platune.EventResponse) error {
	if err := p.initPlayerEventClient(); err != nil {
		return err
	}
	for {
		if *p.playerEventClient == nil {
			time.Sleep(10 * time.Millisecond)
			continue
		}

		msg, err := (*p.playerEventClient).Recv()
		if err == nil {
			eventCh <- msg
		}
	}
}

func (p *PlayerClient) GetCurrentStatus() (*platune.StatusResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return p.playerClient.GetCurrentStatus(ctx, &emptypb.Empty{})
}

func (p *PlayerClient) SetQueue(queue []string) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetQueue(ctx, &platune.QueueRequest{Queue: queue})
	})
}

func (p *PlayerClient) SetQueueFromSearchResults(entries []*platune.LookupEntry) error {
	paths := p.getPathsFromLookup(entries)
	return p.SetQueue(paths)
}

func (p *PlayerClient) AddSearchResultsToQueue(entries []*platune.LookupEntry) error {
	paths := p.getPathsFromLookup(entries)
	return p.AddToQueue(paths)
}

func (p *PlayerClient) AddToQueue(songs []string) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.AddToQueue(ctx, &platune.AddToQueueRequest{Songs: songs})
	})
}

func (p *PlayerClient) Pause() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Pause(ctx, &emptypb.Empty{})
	})
}

func (p *PlayerClient) Stop() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Stop(ctx, &emptypb.Empty{})
	})
}

func (p *PlayerClient) Next() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Next(ctx, &emptypb.Empty{})
	})
}

func (p *PlayerClient) Previous() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Previous(ctx, &emptypb.Empty{})
	})
}

func (p *PlayerClient) Resume() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Resume(ctx, &emptypb.Empty{})
	})
}

func (p *PlayerClient) SetVolume(volume float64) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetVolume(ctx, &platune.SetVolumeRequest{Volume: volume})
	})
}

func (p *PlayerClient) Seek(seekTime string) error {
	timeParts := strings.Split(seekTime, ":")
	totalMillis := uint64(0)
	for i := 0; i < len(timeParts); i++ {
		intVal, err := strconv.ParseUint(timeParts[i], 10, 64)
		if err != nil {
			return fmt.Errorf("Error: %s is not a valid integer\n", timeParts[i])
		}
		pos := float64(len(timeParts) - 1 - i)
		totalMillis += uint64(math.Pow(60, pos)) * intVal * 1000
	}
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Seek(ctx, &platune.SeekRequest{
			Time: durationpb.New(time.Duration(totalMillis * uint64(time.Millisecond))),
		})
	})
}

func (p *PlayerClient) getPathsFromLookup(entries []*platune.LookupEntry) []string {
	paths := []string{}
	for _, entry := range entries {
		paths = append(paths, entry.Path)
	}

	return paths
}

func (p *PlayerClient) initPlayerEventClient() error {
	ctx := context.Background()
	events, err := p.playerClient.SubscribeEvents(ctx, &emptypb.Empty{})

	p.playerEventClient = &events
	return err
}

func (p *PlayerClient) retryConnection() {
	if !p.attemptReconnect {
		return
	}
	state := p.conn.GetState()
	if state == connectivity.TransientFailure || state == connectivity.Shutdown {
		p.conn.ResetConnectBackoff()
	}
}

func (p *PlayerClient) ResetStreams() {
	if p.playerEventClient != nil {
		_ = p.initPlayerEventClient()
	}
}

func (p *PlayerClient) runCommand(
	cmdFunc func(context.Context) (*emptypb.Empty, error),
) error {
	p.retryConnection()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	_, err := cmdFunc(ctx)
	return err
}
