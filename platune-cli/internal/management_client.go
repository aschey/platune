package internal

import (
	"context"
	"fmt"
	"time"

	platune "github.com/aschey/platune/client"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/protobuf/types/known/emptypb"
)

type ManagementClient struct {
	conn                  *grpc.ClientConn
	managementClient      platune.ManagementClient
	searchClient          *platune.Management_SearchClient
	managementEventClient *platune.Management_SubscribeEventsClient
	attemptReconnect      bool
}

func NewManagementClient() (*ManagementClient, error) {
	conn, err := platune.GetIpcClient()
	if err != nil {
		return nil, err
	}

	managementClient := platune.NewManagementClient(conn)
	client := &ManagementClient{
		conn:             conn,
		managementClient: managementClient,
	}

	return client, nil
}

func NewTestManagementClient(
	managementClient platune.ManagementClient,
) ManagementClient {
	return ManagementClient{managementClient: managementClient}
}

func (p *ManagementClient) EnableReconnect() {
	p.attemptReconnect = true
}

func (p *ManagementClient) GetConnection() *grpc.ClientConn {
	return p.conn
}

func (p *ManagementClient) SubscribeManagementEvents(progressCh chan *platune.Progress) error {
	if err := p.initManagementEventClient(); err != nil {
		return err
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

func (p *ManagementClient) initManagementEventClient() error {
	ctx := context.Background()
	events, err := p.managementClient.SubscribeEvents(ctx, &emptypb.Empty{})

	p.managementEventClient = &events
	return err
}

func (p *ManagementClient) retryConnection() {
	if !p.attemptReconnect {
		return
	}
	state := p.conn.GetState()
	if state == connectivity.TransientFailure || state == connectivity.Shutdown {
		p.conn.ResetConnectBackoff()
	}
}

func (p *ManagementClient) ResetStreams() {
	if p.searchClient != nil {
		_ = p.initSearchClient()
	}
}

func (p *ManagementClient) StartSync() error {
	ctx, cancel := p.initRequest()
	defer cancel()
	_, err := p.managementClient.StartSync(ctx, &emptypb.Empty{})
	return err
}

func (p *ManagementClient) Search(req *platune.SearchRequest) (*platune.SearchResponse, error) {
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

func (p *ManagementClient) Lookup(
	entryType platune.EntryType,
	correlationIds []int64,
) (*platune.LookupResponse, error) {
	p.retryConnection()
	ctx := context.Background()
	return p.managementClient.Lookup(
		ctx,
		&platune.LookupRequest{EntryType: entryType, CorrelationIds: correlationIds},
	)
}

func (p *ManagementClient) GetAllFolders() ([]string, error) {
	ctx, cancel := p.initRequest()
	defer cancel()
	allFolders, err := p.managementClient.GetAllFolders(ctx, &emptypb.Empty{})
	if err != nil {
		fmt.Println(err)
	}

	return allFolders.Folders, nil
}

func (p *ManagementClient) AddFolder(folder string) error {
	ctx, cancel := p.initRequest()
	defer cancel()
	_, err := p.managementClient.AddFolders(
		ctx,
		&platune.FoldersMessage{Folders: []string{folder}},
	)
	return err
}

func (p *ManagementClient) SetMount(mount string) error {
	ctx, cancel := p.initRequest()
	defer cancel()
	_, err := p.managementClient.RegisterMount(ctx, &platune.RegisteredMountMessage{Mount: mount})
	return err
}

func (p *ManagementClient) GetRegisteredMount() (*platune.RegisteredMountMessage, error) {
	ctx, cancel := p.initRequest()
	defer cancel()
	return p.managementClient.GetRegisteredMount(ctx, &emptypb.Empty{})
}

func (p *ManagementClient) GetSongByPath(path string) (*platune.SongResponse, error) {
	ctx, cancel := p.initRequest()
	defer cancel()
	return p.managementClient.GetSongByPath(ctx, &platune.PathMessage{Path: path})
}

func (p *ManagementClient) GetAlbumsByAlbumArtists(ids []int64) (*platune.AlbumResponse, error) {
	ctx, cancel := p.initRequest()
	defer cancel()
	return p.managementClient.GetAlbumsByAlbumArtists(ctx, &platune.IdMessage{Ids: ids})
}

func (p *ManagementClient) GetDeleted() (*platune.GetDeletedResponse, error) {
	ctx, cancel := p.initRequest()
	defer cancel()
	return p.managementClient.GetDeleted(ctx, &emptypb.Empty{})
}

func (p *ManagementClient) DeleteTracks(ids []int64) error {
	ctx, cancel := p.initRequest()
	defer cancel()
	_, err := p.managementClient.DeleteTracks(ctx, &platune.IdMessage{Ids: ids})
	return err
}

func (p *ManagementClient) initSearchClient() error {
	ctx := context.Background()
	search, err := p.managementClient.Search(ctx)
	p.searchClient = &search

	return err
}

func (p *ManagementClient) initRequest() (context.Context, context.CancelFunc) {
	p.retryConnection()
	return context.WithTimeout(context.Background(), 10*time.Second)
}
