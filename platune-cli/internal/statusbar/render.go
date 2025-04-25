package statusbar

import (
	"fmt"
	"time"

	management_v1 "github.com/aschey/platune/client/management_v1"
	"github.com/charmbracelet/lipgloss"
	"github.com/nathan-fiscaletti/consolesize-go"
)

type songInfo struct {
	currentSong *management_v1.LookupEntry
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

func formatTime(time time.Time) string {
	return fmt.Sprintf("%02d:%02d:%02d", int(time.Hour()), int(time.Minute()), int(time.Second()))
}

func (s *StatusBar) renderStatusBar(params renderParams) {
	size, _ := consolesize.GetConsoleSize()

	paddingWidth := 2
	formattedStatus := ""
	if params.songInfo != nil {
		renderStatus := params.renderStatus
		if lipgloss.Width(params.renderStatus) == 0 {
			z := time.Unix(0, 0).UTC()
			newTime := z.Add(params.timer.elapsed())
			songTime := params.songInfo.currentSong.Duration.AsTime()
			// If the current time > the song time, we're probably just waiting for the server to tell us
			// that the song completed. Cap the display time here so we don't show that it's past the end of the song.
			if newTime.Compare(songTime) == 1 {
				newTime = songTime
			}
			newText := fmt.Sprintf(
				"%s/%s",
				formatTime(newTime),
				formatTime(params.songInfo.currentSong.Duration.AsTime()),
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
