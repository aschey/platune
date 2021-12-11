package internal

import (
	"context"
	"fmt"
	"io"
	"math"
	"os"
	"strconv"
	"strings"
	"time"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/lipgloss"
	"github.com/nathan-fiscaletti/consolesize-go"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/protobuf/types/known/emptypb"
)

type StatusChan chan string

func NewStatusChan() StatusChan {
	return make(StatusChan, 128)
}

type PlatuneClient struct {
	playerClient     platune.PlayerClient
	managementClient platune.ManagementClient
	searchClient     *platune.Management_SearchClient
	syncClient       *platune.Management_SyncClient
	eventClient      *platune.Player_SubscribeEventsClient
	statusChan       StatusChan
}

func (p *PlatuneClient) monitorConnectionState(conn *grpc.ClientConn, connCh chan connectivity.State, ctx context.Context) {
	for {
		state := conn.GetState()
		conn.Connect()
		conn.WaitForStateChange(ctx, state)
		newState := conn.GetState()
		connCh <- newState
	}
}

func (p *PlatuneClient) subscribeEvents(eventCh chan *platune.EventResponse) {
	p.initEventClient()
	for {
		msg, _ := (*p.eventClient).Recv()
		eventCh <- msg
	}
}

func (p *PlatuneClient) handlePlayerEvent(msg *platune.EventResponse, currentSong *platune.LookupEntry) (string, string, *platune.LookupEntry) {
	switch msg.Event {
	case platune.Event_START_QUEUE, platune.Event_QUEUE_UPDATED, platune.Event_ENDED, platune.Event_NEXT, platune.Event_PREVIOUS:
		res, _ := p.managementClient.GetSongByPath(context.Background(), &platune.PathMessage{Path: msg.Queue[msg.QueuePosition]})
		//playingStatus = " ﱘ " + res.Song.Song + " ﴁ " + res.Song.Artist //"▶ Playing" //queue[queuePos]
		// currentSong = res.Song
		// playingStatus = "  1:00/5:23 "
		return " ", " 1:00/5:23 ", res.Song
	// case platune.Event_ENDED, platune.Event_NEXT:
	// 	playingStatus = "▶ Playing" //queue[queuePos]
	// case platune.Event_PREVIOUS:
	// 	playingStatus = "▶ Playing" //queue[queuePos]
	case platune.Event_QUEUE_ENDED, platune.Event_STOP:
		return " ", " Stopped ", currentSong
	case platune.Event_PAUSE:
		return " ", " Paused ", currentSong
	case platune.Event_RESUME:
		return " ", " 1:00/5:23 ", currentSong //queue[queuePos]
	default:
		return "", "", currentSong
	}

}

func (p *PlatuneClient) handleStateChange(newState connectivity.State) (string, string) {
	if newState == connectivity.Ready {
		if p.searchClient != nil {
			p.initSearchClient() //nolint:errcheck
		}
		if p.syncClient != nil {
			p.initSyncClient() //nolint:errcheck
		}
		if p.eventClient != nil {
			p.initEventClient() //nolint:errcheck
		}
	}

	switch newState {
	case connectivity.Connecting:
		return "", " Connecting... "
	case connectivity.Idle:
		return "", " Idle "
	case connectivity.Ready:
		return " ", " Connected "
	case connectivity.Shutdown, connectivity.TransientFailure:
		return " ", " Disconnected "
	default:
		return "", ""
	}
}

func (p *PlatuneClient) eventLoop(eventCh chan *platune.EventResponse, stateCh chan connectivity.State) {
	// queue := []string{}
	// queuePos := 0
	connectionStatus := ""
	connectionIcon := ""
	var currentSong *platune.LookupEntry = nil
	playingStatus := " Stopped "
	playingIcon := " "

	sigCh := getSignalChannel()

	for {
		select {
		case msg := <-eventCh:
			if msg != nil {
				playingIcon, playingStatus, currentSong = p.handlePlayerEvent(msg, currentSong)
			}
		case newState := <-stateCh:
			connectionIcon, connectionStatus = p.handleStateChange(newState)
		case <-sigCh:
		}
		size, _ := consolesize.GetConsoleSize()

		song := ""
		songIcon := ""

		album := ""
		albumIcon := ""

		artist := ""
		artistIcon := ""

		extraSongChars := 0
		separator := ""
		connectionIconFormatted := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Foreground(lipgloss.Color("14")).
			Render(connectionIcon)
		connectionStatusFormatted := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Foreground(lipgloss.Color("15")).
			Render(connectionStatus + " ")

		if currentSong != nil {
			extraSongChars = -12
			separator = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("15")).
				Render(" ")

			songIcon = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("14")).
				Render("ﱘ ")
			song = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("15")).
				Render(currentSong.Song + " ")

			albumIcon = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("14")).
				Render(" ")
			album = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("15")).
				Render(currentSong.Album + " ")

			artistIcon = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("14")).
				Render("ﴁ ")
			artist = lipgloss.NewStyle().
				Background(lipgloss.Color("8")).
				Foreground(lipgloss.Color("15")).
				Render(currentSong.Artist + " ")
		}

		playingIconFormatted := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Foreground(lipgloss.Color("14")).
			Render(playingIcon)
		playingStatusFormatted := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Foreground(lipgloss.Color("15")).
			// Width(size - len(connectionStatus) + extraChars).
			// Align(lipgloss.Right).
			Render(playingStatus)

		middleBar := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Width(size -
				lipgloss.Width(connectionStatusFormatted) -
				lipgloss.Width(connectionIconFormatted) -
				lipgloss.Width(playingStatusFormatted) -
				lipgloss.Width(playingIconFormatted) -
				lipgloss.Width(song) -
				lipgloss.Width(album) -
				lipgloss.Width(artist) +
				extraSongChars).
			Align(lipgloss.Right).
			Render("")

		formattedStatus := lipgloss.JoinHorizontal(lipgloss.Bottom,
			connectionIconFormatted,
			connectionStatusFormatted,
			middleBar,
			playingIconFormatted,
			playingStatusFormatted,
			separator,
			songIcon,
			song,
			separator,
			albumIcon,
			album,
			separator,
			artistIcon,
			artist)

		p.statusChan <- formattedStatus

	}
}

func NewPlatuneClient(statusChan StatusChan) *PlatuneClient {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())
	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	ctx := context.Background()

	playerClient := platune.NewPlayerClient(conn)
	managementClient := platune.NewManagementClient(conn)
	client := &PlatuneClient{playerClient: playerClient, managementClient: managementClient, statusChan: statusChan}
	eventCh := make(chan *platune.EventResponse)
	go client.subscribeEvents(eventCh)
	connCh := make(chan connectivity.State)
	go client.monitorConnectionState(conn, connCh, ctx)
	go client.eventLoop(eventCh, connCh)
	return client
}

func NewTestClient(playerClient platune.PlayerClient, managementClient platune.ManagementClient) PlatuneClient {
	return PlatuneClient{playerClient: playerClient, managementClient: managementClient}
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

func (p *PlatuneClient) AddToQueue(songs []string, printMsg bool) {
	msg := ""
	if printMsg {
		msg = "Added"
	}
	p.runCommand(msg, func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.AddToQueue(ctx, &platune.AddToQueueRequest{Songs: songs})
	})
}

func (p *PlatuneClient) SetVolume(volume float32) {
	p.runCommand("Set", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.playerClient.SetVolume(ctx, &platune.SetVolumeRequest{Volume: volume})
	})
}

func (p *PlatuneClient) SetQueue(queue []string, printMsg bool) {
	msg := ""
	if printMsg {
		msg = "Queue Set"
	}
	p.runCommand(msg, func(ctx context.Context) (*emptypb.Empty, error) {
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

func (p *PlatuneClient) initEventClient() error {
	ctx := context.Background()
	sync, err := p.playerClient.SubscribeEvents(ctx, &emptypb.Empty{})

	p.eventClient = &sync
	return err
}

func (p *PlatuneClient) initSyncClient() error {
	ctx := context.Background()
	sync, err := p.managementClient.Sync(ctx, &emptypb.Empty{})

	p.syncClient = &sync
	return err
}

func (p *PlatuneClient) Sync() <-chan *platune.Progress {
	if err := p.initSyncClient(); err != nil {
		fmt.Println(err)
	}

	out := make(chan *platune.Progress)
	sync := *p.syncClient
	go func() {
		defer close(out)
		for {
			progress, err := sync.Recv()
			if err == nil {
				out <- progress
			} else if err == io.EOF {
				p.syncClient = nil
				return
			} else {
				fmt.Println(err)
			}
		}
	}()

	return out
}

func (p *PlatuneClient) initSearchClient() error {
	ctx := context.Background()
	search, err := p.managementClient.Search(ctx)
	p.searchClient = &search
	return err
}

func (p *PlatuneClient) Search(req *platune.SearchRequest) (*platune.SearchResponse, error) {
	if p.searchClient == nil {
		if err := p.initSearchClient(); err != nil {
			fmt.Println(err)
		}
	}

	searchClient := *p.searchClient
	if err := searchClient.Send(req); err != nil {
		return nil, err
	}

	return searchClient.Recv()
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

func (p *PlatuneClient) GetDeleted() *platune.GetDeletedResponse {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	deleted, err := p.managementClient.GetDeleted(ctx, &emptypb.Empty{})
	if err != nil {
		fmt.Println(err)
	}
	cancel()

	return deleted
}

func (p *PlatuneClient) DeleteTracks(ids []int64) {
	p.runCommand("", func(ctx context.Context) (*emptypb.Empty, error) {
		return p.managementClient.DeleteTracks(ctx, &platune.IdMessage{Ids: ids})
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
