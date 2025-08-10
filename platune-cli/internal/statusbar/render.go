package statusbar

import (
	"fmt"
	"time"

	"github.com/aschey/platune/client/player_v1"
	"github.com/charmbracelet/lipgloss"
	"github.com/nathan-fiscaletti/consolesize-go"
)

type songInfo struct {
	currentMeta *player_v1.Metadata
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

func formatDuration(dur time.Duration) string {
	durTime := time.Unix(0, 0).UTC().Add(dur)
	return fmt.Sprintf("%02d:%02d:%02d", int(durTime.Hour()), int(durTime.Minute()), int(durTime.Second()))
}

func (s *StatusBar) renderStatusBar(params renderParams) {
	size, _ := consolesize.GetConsoleSize()

	paddingWidth := 2
	formattedStatus := ""
	if params.songInfo != nil {
		renderStatus := params.renderStatus
		if lipgloss.Width(params.renderStatus) == 0 {
			newTime := params.timer.elapsed()
			songTime := params.songInfo.currentMeta.Duration.AsDuration()
			// If the current time > the song time, we're probably just waiting for the server to tell us
			// that the song completed. Cap the display time here so we don't show that it's past the end of the song.
			if newTime > songTime {
				newTime = songTime
			}
			newText := fmt.Sprintf(
				"%s/%s",
				formatDuration(newTime),
				formatDuration(params.songInfo.currentMeta.Duration.AsDuration()),
			)
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
	s.statusChan <- lipgloss.JoinHorizontal(lipgloss.Bottom, spacer, formattedStatus, spacer)
}
