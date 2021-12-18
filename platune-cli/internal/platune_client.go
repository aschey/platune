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

var (
	defaultStyle  = lipgloss.NewStyle().Background(lipgloss.Color("8"))
	infoIconStyle = defaultStyle.Copy().Foreground(lipgloss.Color("14"))
	textStyle     = defaultStyle.Copy().Foreground(lipgloss.Color("15"))
	separator     = defaultStyle.Copy().Foreground(lipgloss.Color("7")).Render("  ")
	songIcon      = "ﱘ"
	albumIcon     = ""
	artistIcon    = "ﴁ"
	spacer        = textStyle.Render(" ")
)

type StatusChan chan string

func NewStatusChan() StatusChan {
	return make(StatusChan, 128)
}

type PlatuneClient struct {
	conn                *grpc.ClientConn
	playerClient        platune.PlayerClient
	managementClient    platune.ManagementClient
	searchClient        *platune.Management_SearchClient
	syncClient          *platune.Management_SyncClient
	eventClient         *platune.Player_SubscribeEventsClient
	statusChan          StatusChan
	waitForStatusChange chan struct{}
	statusChanged       chan struct{}
	interactive         bool
}

func (p *PlatuneClient) monitorConnectionState(connCh chan connectivity.State, ctx context.Context) {
	for {
		state := p.conn.GetState()
		p.conn.Connect()
		p.conn.WaitForStateChange(ctx, state)
		newState := p.conn.GetState()

		connCh <- newState
	}
}

func (p *PlatuneClient) subscribeEvents(eventCh chan *platune.EventResponse) {
	p.initEventClient()
	for {
		if (*p.eventClient) == nil {
			time.Sleep(10 * time.Millisecond)
			continue
		}

		msg, err := (*p.eventClient).Recv()
		if err == nil {
			eventCh <- msg
		}
	}
}

func (p *PlatuneClient) retryConnection() {
	if !p.interactive {
		return
	}
	state := p.conn.GetState()
	if state == connectivity.TransientFailure || state == connectivity.Shutdown {
		p.waitForStatusChange <- struct{}{}
		p.conn.ResetConnectBackoff()
		<-p.statusChanged
	}
}

type playerEvent struct {
	icon    string
	color   string
	status  string
	newSong *platune.LookupEntry
}

type eventInput struct {
	currentSong *platune.LookupEntry
}

func (p *PlatuneClient) handlePlayerEvent(timer *timer, msg *platune.EventResponse, eventInput eventInput) playerEvent {
	switch msg.Event {
	case platune.Event_START_QUEUE, platune.Event_QUEUE_UPDATED, platune.Event_ENDED, platune.Event_NEXT, platune.Event_PREVIOUS:
		res, _ := p.managementClient.GetSongByPath(context.Background(), &platune.PathMessage{Path: msg.Queue[msg.QueuePosition]})
		timer.setTime(0)
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: res.Song,
		}
	case platune.Event_SEEK:
		timer.setTime(int64(*msg.SeekMillis))
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: eventInput.currentSong,
		}
	case platune.Event_QUEUE_ENDED, platune.Event_STOP:
		timer.stop()
		return playerEvent{
			icon:    "",
			color:   "9",
			status:  "Stopped",
			newSong: nil,
		}
	case platune.Event_PAUSE:
		timer.pause()
		return playerEvent{
			icon:    "",
			color:   "11",
			status:  "Paused",
			newSong: eventInput.currentSong,
		}
	case platune.Event_RESUME:
		timer.resume()
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: eventInput.currentSong,
		}
	default:
		return playerEvent{}
	}
}

func (p *PlatuneClient) handlePlayerStatus(timer *timer, status *platune.StatusResponse) playerEvent {
	if status == nil {
		return playerEvent{
			icon:    "",
			color:   "9",
			status:  "Stopped",
			newSong: nil,
		}
	}
	switch status.Status {
	case platune.PlayerStatus_PLAYING:
		progress := status.Progress.AsTime()

		timer.start()
		timer.setTime(progress.UnixMilli())

		res, _ := p.managementClient.GetSongByPath(context.Background(), &platune.PathMessage{Path: *status.CurrentSong})

		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: res.Song,
		}
	case platune.PlayerStatus_STOPPED:
		timer.stop()
		return playerEvent{
			icon:    "",
			color:   "9",
			status:  "Stopped",
			newSong: nil,
		}
	case platune.PlayerStatus_PAUSED:
		timer.pause()
		progress := status.Progress.AsTime()
		timer.setTime(progress.UnixMilli())
		res, _ := p.managementClient.GetSongByPath(context.Background(), &platune.PathMessage{Path: *status.CurrentSong})

		return playerEvent{
			icon:    "",
			color:   "11",
			status:  "Paused",
			newSong: res.Song,
		}
	default:
		return playerEvent{}
	}
}

func (p *PlatuneClient) handleStateChange(newState connectivity.State) (string, string, string) {
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

	select {
	case <-p.waitForStatusChange:
		p.statusChanged <- struct{}{}
	default:
	}

	switch newState {
	case connectivity.Connecting:
		return "", "0", "Connecting..."
	case connectivity.Idle:
		return "", "0", "Idle"
	case connectivity.Ready:
		return "", "10", "Connected"
	case connectivity.Shutdown, connectivity.TransientFailure:
		return "", "9", "Disconnected"
	default:
		return "", "0", ""
	}
}

func formatTime(time time.Time) string {
	return fmt.Sprintf("%02d:%02d:%02d", int(time.Hour()), int(time.Minute()), int(time.Second()))
}

func (p *PlatuneClient) eventLoop(eventCh chan *platune.EventResponse, stateCh chan connectivity.State) {
	p.renderStatusBar(renderParams{connection: textStyle.Render(" Connecting...")})
	sigCh := getSignalChannel()
	ticker := time.NewTicker(500 * time.Millisecond)

	currentStatus, _ := p.playerClient.GetCurrentStatus(context.Background(), &emptypb.Empty{})
	timer := timer{}
	event := p.handlePlayerStatus(&timer, currentStatus)

	playingIconColor := event.color
	playingIconStyle := defaultStyle.Copy().Foreground(lipgloss.Color(playingIconColor))
	currentSong := event.newSong
	renderParams := renderParams{
		timer:        &timer,
		connection:   "",
		playingIcon:  playingIconStyle.Render(event.icon + " "),
		renderStatus: textStyle.Render(event.status),
	}
	p.renderStatusBar(renderParams)

	for {
		select {
		case msg := <-eventCh:
			if msg != nil {
				event := p.handlePlayerEvent(&timer, msg, eventInput{currentSong: event.newSong})
				currentSong = event.newSong
				playingIconColor = event.color
				playingIconStyle = defaultStyle.Copy().Foreground(lipgloss.Color(playingIconColor))
				renderParams.renderStatus = textStyle.Render(event.status)
				renderParams.playingIcon = playingIconStyle.Render(event.icon + " ")
			}
		case newState := <-stateCh:
			connectionIcon, connectionIconColor, connectionStatus := p.handleStateChange(newState)
			//connectionStatus = textStyle.Render(connectionStatus)
			connectionIconStyle := defaultStyle.Copy().Foreground(lipgloss.Color(connectionIconColor))
			//connectionIcon = connectionIconStyle.Render(connectionIcon + " ")
			renderParams.connection = label{icon: connectionIcon, text: connectionStatus}.render(connectionIconStyle)
		case <-ticker.C:
			// Timer tick, don't need to do anything except re-render
		case <-sigCh:
			// Resize event, don't need to do anything except re-render
		}

		if currentSong != nil {
			renderParams.songInfo = &songInfo{
				currentSong: *currentSong,
				artist:      label{icon: artistIcon, text: currentSong.Artist}.render(infoIconStyle),
				album:       label{icon: albumIcon, text: currentSong.Album}.render(infoIconStyle),
				song:        label{icon: songIcon, text: currentSong.Song}.render(infoIconStyle),
			}
		} else {
			renderParams.songInfo = nil
		}
		p.renderStatusBar(renderParams)
	}
}

type label struct {
	icon string
	text string
}

func (l label) render(iconStyle lipgloss.Style) string {
	return fmt.Sprintf("%s%s", iconStyle.Render(l.icon), textStyle.Render(" "+l.text))
}

type songInfo struct {
	currentSong platune.LookupEntry
	song        string
	album       string
	artist      string
}

type renderParams struct {
	songInfo *songInfo
	timer    *timer

	connection string

	playingIcon  string
	renderStatus string
}

func (p *PlatuneClient) renderStatusBar(params renderParams) {
	size, _ := consolesize.GetConsoleSize()

	paddingWidth := 2
	formattedStatus := ""
	if params.songInfo != nil {
		renderStatus := params.renderStatus
		if lipgloss.Width(params.renderStatus) == 0 {
			z := time.Unix(0, 0).UTC()
			newTime := z.Add(params.timer.elapsed())
			newText := fmt.Sprintf("%s/%s", formatTime(newTime), formatTime(params.songInfo.currentSong.Duration.AsTime()))
			renderStatus = textStyle.Render(newText)
		}

		middleBar := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Width(size -
				lipgloss.Width(params.connection) -
				lipgloss.Width(renderStatus) -
				lipgloss.Width(params.playingIcon) -
				lipgloss.Width(params.songInfo.song) -
				lipgloss.Width(params.songInfo.album) -
				lipgloss.Width(params.songInfo.artist) -
				(lipgloss.Width(separator) * 3) -
				paddingWidth).
			Align(lipgloss.Right).
			Render("")

		formattedStatus = lipgloss.JoinHorizontal(lipgloss.Bottom,
			params.connection,
			middleBar,
			params.playingIcon,
			renderStatus,
			separator,
			params.songInfo.song,
			separator,
			params.songInfo.album,
			separator,
			params.songInfo.artist)
	} else {
		middleBar := lipgloss.NewStyle().
			Background(lipgloss.Color("8")).
			Width(size -
				lipgloss.Width(params.connection) -
				lipgloss.Width(params.playingIcon) -
				lipgloss.Width(params.renderStatus) -
				paddingWidth).
			Align(lipgloss.Right).
			Render("")

		formattedStatus = lipgloss.JoinHorizontal(lipgloss.Bottom,
			params.connection,
			middleBar,
			params.playingIcon,
			params.renderStatus)
	}
	p.statusChan <- lipgloss.JoinHorizontal(lipgloss.Bottom, spacer, formattedStatus, spacer)
}

func NewPlatuneClient(statusChan StatusChan) *PlatuneClient {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())
	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	playerClient := platune.NewPlayerClient(conn)
	managementClient := platune.NewManagementClient(conn)
	client := &PlatuneClient{
		conn:                conn,
		playerClient:        playerClient,
		managementClient:    managementClient,
		statusChan:          statusChan,
		statusChanged:       make(chan struct{}, 1),
		waitForStatusChange: make(chan struct{}, 1),
	}

	return client
}

func (p *PlatuneClient) StartEventLoop() {
	p.interactive = true

	eventCh := make(chan *platune.EventResponse, 1)
	go p.subscribeEvents(eventCh)

	connCh := make(chan connectivity.State, 1)
	ctx := context.Background()
	go p.monitorConnectionState(connCh, ctx)

	go p.eventLoop(eventCh, connCh)
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
	p.retryConnection()
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
	p.retryConnection()
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
	p.retryConnection()
	ctx := context.Background()
	response, err := p.managementClient.Lookup(ctx, &platune.LookupRequest{EntryType: entryType, CorrelationIds: correlationIds})
	if err != nil {
		fmt.Println(err)
		return nil
	}
	return response
}

func (p *PlatuneClient) GetAllFolders() {
	p.retryConnection()
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
	p.retryConnection()
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
	p.retryConnection()
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
