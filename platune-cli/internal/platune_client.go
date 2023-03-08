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
	"google.golang.org/grpc/connectivity"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/emptypb"
)

type PlatuneClient struct {
	conn                  *grpc.ClientConn
	playerClient          platune.PlayerClient
	managementClient      platune.ManagementClient
	searchClient          *platune.Management_SearchClient
	playerEventClient     *platune.Player_SubscribeEventsClient
	managementEventClient *platune.Management_SubscribeEventsClient
	attemptReconnect      bool
}

func NewPlatuneClient() *PlatuneClient {
	conn, err := platune.GetIpcClient()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	playerClient := platune.NewPlayerClient(conn)
	managementClient := platune.NewManagementClient(conn)
	client := &PlatuneClient{
		conn:             conn,
		playerClient:     playerClient,
		managementClient: managementClient,
	}

	return client
}

func (p *PlatuneClient) EnableReconnect() {
	p.attemptReconnect = true
}

func (p *PlatuneClient) GetConnection() *grpc.ClientConn {
	return p.conn
}

func NewTestClient(
	playerClient platune.PlayerClient,
	managementClient platune.ManagementClient,
) PlatuneClient {
	return PlatuneClient{playerClient: playerClient, managementClient: managementClient}
}

func (p *PlatuneClient) SubscribePlayerEvents(eventCh chan *platune.EventResponse) {
	if err := p.initPlayerEventClient(); err != nil {
		fmt.Println(err)
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

func (p *PlatuneClient) SubscribeManagementEvents(progressCh chan *platune.Progress) {
	if err := p.initManagementEventClient(); err != nil {
		fmt.Println(err)
	}
	for {
		if *p.managementEventClient == nil {
			time.Sleep(10 * time.Millisecond)
			continue
		}

		msg, err := (*p.managementEventClient).Recv()
		if err == nil {
			progressCh <- msg
		}
	}
}

func (p *PlatuneClient) retryConnection() {
	if !p.attemptReconnect {
		return
	}
	state := p.conn.GetState()
	if state == connectivity.TransientFailure || state == connectivity.Shutdown {
		p.conn.ResetConnectBackoff()
	}
}

func (p *PlatuneClient) GetCurrentStatus() (*platune.StatusResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return p.playerClient.GetCurrentStatus(ctx, &emptypb.Empty{})
}

func (p *PlatuneClient) ResetStreams() {
	if p.searchClient != nil {
		_ = p.initSearchClient()
	}
	if p.playerEventClient != nil {
		_ = p.initPlayerEventClient()
	}
}

func (p *PlatuneClient) SetQueueFromSearchResults(entries []*platune.LookupEntry) error {
	paths := p.getPathsFromLookup(entries)
	return p.SetQueue(paths)
}

func (p *PlatuneClient) AddSearchResultsToQueue(entries []*platune.LookupEntry) error {
	paths := p.getPathsFromLookup(entries)
	return p.AddToQueue(paths)
}

func (p *PlatuneClient) Pause() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Pause(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Stop() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Stop(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Next() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Next(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Previous() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Previous(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Resume() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Resume(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) AddToQueue(songs []string) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.AddToQueue(ctx, &platune.AddToQueueRequest{Songs: songs})
	})
}

func (p *PlatuneClient) SetVolume(volume float64) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetVolume(ctx, &platune.SetVolumeRequest{Volume: volume})
	})
}

func (p *PlatuneClient) SetQueue(queue []string) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetQueue(ctx, &platune.QueueRequest{Queue: queue})
	})
}

func (p *PlatuneClient) Seek(seekTime string) error {
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

func (p *PlatuneClient) initPlayerEventClient() error {
	ctx := context.Background()
	events, err := p.playerClient.SubscribeEvents(ctx, &emptypb.Empty{})

	p.playerEventClient = &events
	return err
}

func (p *PlatuneClient) initManagementEventClient() error {
	ctx := context.Background()
	events, err := p.managementClient.SubscribeEvents(ctx, &emptypb.Empty{})

	p.managementEventClient = &events
	return err
}

func (p *PlatuneClient) StartSync() error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.StartSync(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) initSearchClient() error {
	ctx := context.Background()
	search, err := p.managementClient.Search(ctx)
	p.searchClient = &search

	return err
}

func (p *PlatuneClient) Search(req *platune.SearchRequest) (*platune.SearchResponse, error) {
	p.retryConnection()
	if p.searchClient == nil {
		if err := p.initSearchClient(); err != nil {
			return nil, err
		}
	}

	searchClient := *p.searchClient
	if searchClient == nil {
		return nil, fmt.Errorf("Not connected")
	}
	if err := searchClient.Send(req); err != nil {
		return nil, err
	}

	return searchClient.Recv()
}

func (p *PlatuneClient) Lookup(
	entryType platune.EntryType,
	correlationIds []int32,
) (*platune.LookupResponse, error) {
	p.retryConnection()
	ctx := context.Background()
	return p.managementClient.Lookup(
		ctx,
		&platune.LookupRequest{EntryType: entryType, CorrelationIds: correlationIds},
	)
}

func (p *PlatuneClient) GetAllFolders() {
	// p.retryConnection()
	// ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	// defer cancel()
	// allFolders, err := p.managementClient.GetAllFolders(ctx, &emptypb.Empty{})
	// if err != nil {
	// 	fmt.Println(err)
	// }

	// fmt.Println(PrettyPrintList(allFolders.Folders))
}

func (p *PlatuneClient) AddFolder(folder string) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.AddFolders(
			ctx,
			&platune.FoldersMessage{Folders: []string{folder}},
		)
	})
}

func (p *PlatuneClient) SetMount(mount string) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.RegisterMount(ctx, &platune.RegisteredMountMessage{Mount: mount})
	})
}

func (p *PlatuneClient) GetSongByPath(path string) (*platune.SongResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return p.managementClient.GetSongByPath(ctx, &platune.PathMessage{Path: path})
}

func (p *PlatuneClient) GetDeleted() (*platune.GetDeletedResponse, error) {
	p.retryConnection()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	return p.managementClient.GetDeleted(ctx, &emptypb.Empty{})
}

func (p *PlatuneClient) DeleteTracks(ids []int64) error {
	return p.runCommand(func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.DeleteTracks(ctx, &platune.IdMessage{Ids: ids})
	})
}

func (p *PlatuneClient) runCommand(
	cmdFunc func(context.Context) (*emptypb.Empty, error),
) error {
	p.retryConnection()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	_, err := cmdFunc(ctx)
	return err
}

func (p *PlatuneClient) getPathsFromLookup(entries []*platune.LookupEntry) []string {
	paths := []string{}
	for _, entry := range entries {
		paths = append(paths, entry.Path)
	}

	return paths
}
