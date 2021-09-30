package internal

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

func (p *PlatuneClient) SetQueueFromSearchResults(entries []*platune.LookupEntry) {
	paths := p.getPathsFromLookup(entries)
	p.SetQueue(paths)
}

func (p *PlatuneClient) AddSearchResultsToQueue(entries []*platune.LookupEntry, printMsg bool) {
	paths := p.getPathsFromLookup(entries)
	p.AddToQueue(paths, printMsg)
}

func (p *PlatuneClient) AddToQueue(songs []string, printMsg bool) {
	msg := ""
	if printMsg {
		msg = "Added"
	}
	p.runCommand(msg, func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.AddToQueue(ctx, &platune.AddToQueueRequest{Songs: songs})
	})
}

func (p *PlatuneClient) Pause() {
	p.runCommand("Paused", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Pause(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Stop() {
	p.runCommand("Stopped", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Stop(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Next() {
	p.runCommand("Next", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Next(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Previous() {
	p.runCommand("Previous", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Previous(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Resume() {
	p.runCommand("Resumed", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Resume(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) SetQueue(queue []string) {
	p.runCommand("Queue Set", func(ctx context.Context) (*emptypb.Empty, error) {
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
	p.runCommand("Seeked to "+time, func(ctx context.Context) (*emptypb.Empty, error) {
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

func (p *PlatuneClient) Search() platune.Management_SearchClient {
	ctx := context.Background()
	search, err := p.managementClient.Search(ctx)
	if err != nil {
		fmt.Println(err)
		return nil
	}
	return search
}

func (p *PlatuneClient) Lookup(entryType platune.EntryType, correlationIds []int32) *platune.LookupResponse {
	ctx := context.Background()
	response, err := p.managementClient.Lookup(ctx, &platune.LookupRequest{EntryType: entryType, CorrelationIds: correlationIds})
	if err != nil {
		fmt.Println(err)
		return nil
	}
	return response
}

func (p *PlatuneClient) GetAllFolders() {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	allFolders, err := p.managementClient.GetAllFolders(ctx, &emptypb.Empty{})
	if err != nil {
		fmt.Println(err)
	}
	cancel()
	fmt.Println(PrettyPrintList(allFolders.Folders))
}

func (p *PlatuneClient) AddFolder(folder string) {
	p.runCommand("Added", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.AddFolders(ctx, &platune.FoldersMessage{Folders: []string{folder}})
	})
}

func (p *PlatuneClient) SetMount(mount string) {
	p.runCommand("Set", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.RegisterMount(ctx, &platune.RegisteredMountMessage{Mount: mount})
	})
}

func (p *PlatuneClient) runCommand(successMsg string, cmdFunc func(context.Context) (*emptypb.Empty, error)) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	_, err := cmdFunc(ctx)
	cancel()
	if err != nil {
		fmt.Println(err)
		return
	}
	if successMsg != "" {
		fmt.Println(successMsg)
	}
}

func (p *PlatuneClient) getPathsFromLookup(entries []*platune.LookupEntry) []string {
	paths := []string{}
	for _, entry := range entries {
		paths = append(paths, entry.Path)
	}

	return paths
}

var Client = NewPlatuneClient()
