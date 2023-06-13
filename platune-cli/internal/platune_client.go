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
	playerConn            *grpc.ClientConn
	managementConn        *grpc.ClientConn
	playerClient          platune.PlayerClient
	managementClient      platune.ManagementClient
	searchClient          *platune.Management_SearchClient
	playerEventClient     *platune.Player_SubscribeEventsClient
	managementEventClient *platune.Management_SubscribeEventsClient
	statusNotifier        *StatusNotifier
	attemptReconnect      bool
}

func NewPlatuneClient(statusNotifier *StatusNotifier) *PlatuneClient {
	playerConn, err := platune.GetIpcClient()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	var managementConn *grpc.ClientConn
	managementUrls := os.Getenv("PLATUNE_MANAGEMENT_URL")
	if managementUrls != "" {
		numUrls := len(managementUrls)
		for i, managementUrl := range strings.Split(managementUrls, ",") {
			managementConn, err = platune.GetHttpClient(managementUrl)
			if err != nil && i == numUrls-1 {
				fmt.Println(err)
				os.Exit(1)
			}
		}

	} else {
		managementConn = playerConn
	}

	playerClient := platune.NewPlayerClient(playerConn)
	managementClient := platune.NewManagementClient(managementConn)
	client := &PlatuneClient{
		playerConn:       playerConn,
		managementConn:   managementConn,
		playerClient:     playerClient,
		managementClient: managementClient,
		statusNotifier:   statusNotifier,
	}

	return client
}

func (p *PlatuneClient) EnableReconnect() {
	p.attemptReconnect = true
}

func (p *PlatuneClient) GetPlayerConnection() *grpc.ClientConn {
	return p.playerConn
}

func (p *PlatuneClient) GetManagementConnection() *grpc.ClientConn {
	return p.managementConn
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

func (p *PlatuneClient) retryPlayerConnection() {
	if !p.attemptReconnect {
		return
	}
	state := p.playerConn.GetState()
	if state == connectivity.TransientFailure || state == connectivity.Shutdown {
		p.playerConn.ResetConnectBackoff()
		p.statusNotifier.WaitForStatusChange()
	}
}

func (p *PlatuneClient) retryManagementConnection() {
	if !p.attemptReconnect {
		return
	}
	state := p.managementConn.GetState()
	if state == connectivity.TransientFailure || state == connectivity.Shutdown {
		p.managementConn.ResetConnectBackoff()
		p.statusNotifier.WaitForStatusChange()
	}
}

func (p *PlatuneClient) GetCurrentStatus() *platune.StatusResponse {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	status, err := p.playerClient.GetCurrentStatus(ctx, &emptypb.Empty{})
	if err != nil {
		fmt.Println(err)
	}

	return status
}

func (p *PlatuneClient) ResetStreams() {
	if p.searchClient != nil {
		p.initSearchClient() //nolint:errcheck
	}
	if p.playerEventClient != nil {
		p.initPlayerEventClient() //nolint:errcheck
	}
}

func (p *PlatuneClient) SetQueueFromSearchResults(entries []*platune.LookupEntry, printMsg bool) {
	paths := p.getPathsFromLookup(entries)
	p.SetQueue(paths, printMsg)
}

func (p *PlatuneClient) AddSearchResultsToQueue(entries []*platune.LookupEntry, printMsg bool) {
	paths := p.getPathsFromLookup(entries)
	p.AddToQueue(paths, printMsg)
}

func (p *PlatuneClient) Pause() {
	p.retryPlayerConnection()
	p.runCommand("Paused", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Pause(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Stop() {
	p.retryPlayerConnection()
	p.runCommand("Stopped", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Stop(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Next() {
	p.retryPlayerConnection()
	p.runCommand("Next", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Next(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Previous() {
	p.retryPlayerConnection()
	p.runCommand("Previous", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Previous(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) Resume() {
	p.retryPlayerConnection()
	p.runCommand("Resumed", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.Resume(ctx, &emptypb.Empty{})
	})
}

func (p *PlatuneClient) AddToQueue(songs []string, printMsg bool) {
	p.retryPlayerConnection()
	msg := ""
	if printMsg {
		msg = "Added"
	}
	p.runCommand(msg, func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.AddToQueue(ctx, &platune.AddToQueueRequest{Songs: songs})
	})
}

func (p *PlatuneClient) SetVolume(volume float64) {
	p.retryPlayerConnection()
	p.runCommand("Set", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetVolume(ctx, &platune.SetVolumeRequest{Volume: volume})
	})
}

func (p *PlatuneClient) SetQueue(queue []string, printMsg bool) {
	msg := ""
	if printMsg {
		msg = "Queue Set"
	}
	p.retryPlayerConnection()
	p.runCommand(msg, func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetQueue(ctx, &platune.QueueRequest{Queue: queue})
	})
}

func (p *PlatuneClient) Seek(seekTime string) {
	timeParts := strings.Split(seekTime, ":")
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
	p.retryPlayerConnection()
	p.runCommand("Seeked to "+seekTime, func(ctx context.Context) (*emptypb.Empty, error) {
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

func (p *PlatuneClient) StartSync() {
	p.retryManagementConnection()
	p.runCommand("Sync started", func(ctx context.Context) (*emptypb.Empty, error) {
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
	p.retryManagementConnection()
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
	correlationIds []int64,
) *platune.LookupResponse {
	p.retryManagementConnection()
	ctx := context.Background()
	response, err := p.managementClient.Lookup(
		ctx,
		&platune.LookupRequest{EntryType: entryType, CorrelationIds: correlationIds},
	)
	if err != nil {
		fmt.Println(err)
		return nil
	}
	return response
}

func (p *PlatuneClient) GetAllFolders() {
	p.retryManagementConnection()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	allFolders, err := p.managementClient.GetAllFolders(ctx, &emptypb.Empty{})
	if err != nil {
		fmt.Println(err)
	}

	fmt.Println(PrettyPrintList(allFolders.Folders))
}

func (p *PlatuneClient) AddFolder(folder string) {
	p.retryManagementConnection()
	p.runCommand("Added", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.AddFolders(
			ctx,
			&platune.FoldersMessage{Folders: []string{folder}},
		)
	})
}

func (p *PlatuneClient) SetMount(mount string) {
	p.retryManagementConnection()
	p.runCommand("Set", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.RegisterMount(ctx, &platune.RegisteredMountMessage{Mount: mount})
	})
}

func (p *PlatuneClient) GetSongByPath(path string) *platune.SongResponse {
	p.retryManagementConnection()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	song, err := p.managementClient.GetSongByPath(ctx, &platune.PathMessage{Path: path})
	if err != nil {
		fmt.Println(err)
	}

	return song
}

func (p *PlatuneClient) GetDeleted() *platune.GetDeletedResponse {
	p.retryManagementConnection()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	deleted, err := p.managementClient.GetDeleted(ctx, &emptypb.Empty{})
	if err != nil {
		fmt.Println(err)
	}

	return deleted
}

func (p *PlatuneClient) DeleteTracks(ids []int64) {
	p.retryManagementConnection()
	p.runCommand("", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.DeleteTracks(ctx, &platune.IdMessage{Ids: ids})
	})
}

func (p *PlatuneClient) runCommand(
	successMsg string,
	cmdFunc func(context.Context) (*emptypb.Empty, error),
) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	_, err := cmdFunc(ctx)
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
