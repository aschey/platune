package statusbar

import (
	"context"
	"fmt"
	"time"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/lipgloss"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
)

var (
	defaultStyle  = lipgloss.NewStyle().Background(lipgloss.Color("8"))
	infoIconStyle = defaultStyle.Copy().Foreground(lipgloss.Color("14"))
	textStyle     = defaultStyle.Copy().Foreground(lipgloss.Color("15"))
	separator     = defaultStyle.Copy().Foreground(lipgloss.Color("7")).Render(" ⸽ ")
	songIcon      = ""
	albumIcon     = "󰀥"
	artistIcon    = ""
	spacer        = textStyle.Render(" ")
)

type label struct {
	icon string
	text string
}

func (s *StatusBar) StartEventLoop() {
	s.platuneClient.EnableReconnect()

	eventCh := make(chan *platune.EventResponse, 1)
	go s.platuneClient.SubscribePlayerEvents(eventCh)

	playerConnCh := make(chan connectivity.State, 1)
	managementConnCh := make(chan connectivity.State, 1)
	ctx := context.Background()
	go s.monitorConnectionState(s.platuneClient.GetPlayerConnection(), playerConnCh, ctx)
	go s.monitorConnectionState(s.platuneClient.GetManagementConnection(), managementConnCh, ctx)

	go s.eventLoop(eventCh, playerConnCh, managementConnCh)
}

func (l label) render(iconStyle lipgloss.Style) string {
	return fmt.Sprintf("%s%s", iconStyle.Render(l.icon), textStyle.Render(" "+l.text))
}

func (s *StatusBar) eventLoop(
	eventCh chan *platune.EventResponse,
	playerStateCh chan connectivity.State,
	managementStateCh chan connectivity.State,
) {
	s.renderStatusBar(renderParams{connection: textStyle.Render(" Connecting...")})
	sigCh := getSignalChannel()
	ticker := time.NewTicker(500 * time.Millisecond)

	currentStatus := s.platuneClient.GetCurrentStatus()
	timer := timer{}
	event := s.handlePlayerStatus(&timer, currentStatus)

	playingIconColor := event.color
	playingIconStyle := defaultStyle.Copy().Foreground(lipgloss.Color(playingIconColor))
	currentSong := event.newSong
	renderParams := renderParams{
		timer:        &timer,
		connection:   "",
		playingIcon:  playingIconStyle.Render(event.icon + " "),
		renderStatus: textStyle.Render(event.status),
	}
	s.renderStatusBar(renderParams)
	var playerState connectivity.State
	var managementState connectivity.State
	for {
		select {
		case msg := <-eventCh:
			if msg != nil {
				event := s.handlePlayerEvent(&timer, msg, currentSong)
				currentSong = event.newSong
				playingIconColor = event.color
				playingIconStyle = defaultStyle.Copy().Foreground(lipgloss.Color(playingIconColor))
				renderParams.renderStatus = textStyle.Render(event.status)
				renderParams.playingIcon = playingIconStyle.Render(event.icon + " ")
			}
		case newState := <-playerStateCh:
			playerState = newState
			connectionIcon, connectionIconColor, connectionStatus := s.handleStateChange(playerState, managementState)
			connectionIconStyle := defaultStyle.Copy().
				Foreground(lipgloss.Color(connectionIconColor))
			renderParams.connection = label{
				icon: connectionIcon,
				text: connectionStatus,
			}.render(
				connectionIconStyle,
			)
		case newState := <-managementStateCh:
			managementState = newState
			connectionIcon, connectionIconColor, connectionStatus := s.handleStateChange(playerState, managementState)
			connectionIconStyle := defaultStyle.Copy().
				Foreground(lipgloss.Color(connectionIconColor))
			renderParams.connection = label{
				icon: connectionIcon,
				text: connectionStatus,
			}.render(
				connectionIconStyle,
			)
		case <-ticker.C:
			// Timer tick, don't need to do anything except re-render
		case <-sigCh:
			// Resize event, don't need to do anything except re-render
		}

		if currentSong != nil {
			renderParams.songInfo = &songInfo{
				currentSong: currentSong,
				artist: label{
					icon: artistIcon,
					text: currentSong.Artist,
				}.render(
					infoIconStyle,
				),
				album: label{icon: albumIcon, text: currentSong.Album}.render(infoIconStyle),
				song:  label{icon: songIcon, text: currentSong.Song}.render(infoIconStyle),
			}
		} else {
			renderParams.songInfo = nil
		}
		s.renderStatusBar(renderParams)
	}
}

func (s *StatusBar) monitorConnectionState(conn *grpc.ClientConn, connCh chan connectivity.State, ctx context.Context) {
	for {
		state := conn.GetState()
		conn.Connect()
		conn.WaitForStateChange(ctx, state)
		newState := conn.GetState()

		connCh <- newState
	}
}
