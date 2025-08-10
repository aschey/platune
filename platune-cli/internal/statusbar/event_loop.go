package statusbar

import (
	"context"
	"fmt"
	"time"

	player_v1 "github.com/aschey/platune/client/player_v1"
	"github.com/charmbracelet/lipgloss"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
)

var (
	defaultStyle  = lipgloss.NewStyle().Background(lipgloss.Color("8"))
	infoIconStyle = defaultStyle.Foreground(lipgloss.Color("14"))
	textStyle     = defaultStyle.Foreground(lipgloss.Color("15"))
	separator     = defaultStyle.Foreground(lipgloss.Color("7")).Render(" | ")
	songIcon      = ""
	albumIcon     = "󰀥"
	artistIcon    = ""
	spacer        = textStyle.Render(" ")
)

type label struct {
	icon string
	text *string
}

func (s *StatusBar) StartEventLoop() {
	s.platuneClient.EnableReconnect()

	eventCh := make(chan *player_v1.EventResponse, 1)
	go s.platuneClient.SubscribePlayerEvents(eventCh)

	playerConnCh := make(chan connectivity.State, 1)
	managementConnCh := make(chan connectivity.State, 1)
	ctx := context.Background()
	go s.monitorConnectionState(s.platuneClient.GetPlayerConnection(), playerConnCh, ctx)
	go s.monitorConnectionState(s.platuneClient.GetManagementConnection(), managementConnCh, ctx)

	go s.eventLoop(eventCh, playerConnCh, managementConnCh)
}

func (l label) render(iconStyle lipgloss.Style) string {
	text := ""
	if l.text != nil {
		text = *l.text
	}
	return fmt.Sprintf("%s%s", iconStyle.Render(l.icon), textStyle.Render(" "+text))
}

func (s *StatusBar) eventLoop(
	eventCh chan *player_v1.EventResponse,
	playerStateCh chan connectivity.State,
	managementStateCh chan connectivity.State,
) {
	s.renderStatusBar(renderParams{connection: textStyle.Render(" Connecting...")})
	sigCh := getSignalChannel()
	ticker := time.NewTicker(500 * time.Millisecond)

	timer := timer{}

	playingIconColor := ""
	playingIconStyle := defaultStyle
	var currentMeta *player_v1.Metadata
	renderParams := renderParams{
		timer:        &timer,
		connection:   "",
		playingIcon:  "",
		renderStatus: "",
	}
	s.renderStatusBar(renderParams)
	var playerState connectivity.State
	var managementState connectivity.State
	for {
		select {
		case msg := <-eventCh:
			if msg != nil {
				event := s.handlePlayerEvent(&timer, msg, currentMeta)
				currentMeta = event.newMeta
				playingIconColor = event.color
				playingIconStyle = defaultStyle.Foreground(lipgloss.Color(playingIconColor))
				renderParams.renderStatus = textStyle.Render(event.status)
				renderParams.playingIcon = playingIconStyle.Render(event.icon + " ")
			}
		case newState := <-playerStateCh:
			playerState = newState
			connectionIcon, connectionIconColor, connectionStatus := s.handleStateChange(playerState, managementState)
			connectionIconStyle := defaultStyle.
				Foreground(lipgloss.Color(connectionIconColor))
			renderParams.connection = label{
				icon: connectionIcon,
				text: &connectionStatus,
			}.render(
				connectionIconStyle,
			)
		case newState := <-managementStateCh:
			managementState = newState
			connectionIcon, connectionIconColor, connectionStatus := s.handleStateChange(playerState, managementState)
			connectionIconStyle := defaultStyle.
				Foreground(lipgloss.Color(connectionIconColor))
			renderParams.connection = label{
				icon: connectionIcon,
				text: &connectionStatus,
			}.render(
				connectionIconStyle,
			)
		case <-ticker.C:
			// Timer tick, don't need to do anything except re-render
		case <-sigCh:
			// Resize event, don't need to do anything except re-render
		}

		if currentMeta != nil {
			renderParams.songInfo = &songInfo{
				currentMeta: currentMeta,
				artist: label{
					icon: artistIcon,
					text: currentMeta.Artist,
				}.render(
					infoIconStyle,
				),
				album: label{icon: albumIcon, text: currentMeta.Album}.render(infoIconStyle),
				song:  label{icon: songIcon, text: currentMeta.Song}.render(infoIconStyle),
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
