package cmd

import (
	"fmt"
	"runtime"
	"sort"
	"strings"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/nathan-fiscaletti/consolesize-go"
)

const selectAll = "(Select All)"
const back = "(Back)"

var filePathCompleter = internal.FilePathCompleter{
	IgnoreCase: true,
}

func (state *cmdState) completer(in prompt.Document) []prompt.Suggest {
	// Windows terminal doesn't handle overflow as well as Unix
	if isWindows() {
		state.updateMaxWidths(in, 1./3)
	} else {
		state.unsetMaxWidths()
	}
	before := strings.Split(in.TextBeforeCursor(), " ")
	if state.mode[len(state.mode)-1] != NormalMode {
		return state.completerMode(in)
	}

	if len(before) > 1 {
		return state.completerCmd(in, before)
	}

	return completerDefault(in)
}

func (state *cmdState) completerMode(in prompt.Document) []prompt.Suggest {
	suggestions := []prompt.Suggest{}
	if isWindows() {
		state.updateMaxWidths(in, 1.)
	}

	switch state.mode[len(state.mode)-1] {
	case SetQueueMode:
		return dbCompleter(in, in.TextBeforeCursor(), false)
	case AlbumMode:
		suggestionMap := map[string]prompt.Suggest{}
		for _, r := range state.lookupResult {
			album := r.Album
			if strings.Trim(r.Album, " ") == "" {
				album = "[Untitled]"
			}
			suggestionMap[album] = prompt.Suggest{Text: album, Metadata: r}
		}

		for r := range suggestionMap {
			suggestions = append(suggestions, suggestionMap[r])
		}
		sort.Slice(suggestions, func(i, j int) bool {
			return suggestions[i].Text < suggestions[j].Text
		})
		suggestions = append([]prompt.Suggest{
			{Text: selectAll, Metadata: state.lookupResult},
			{Text: back},
		}, suggestions...)

	case SongMode:
		suggestions = []prompt.Suggest{
			{Text: selectAll, Metadata: state.lookupResult},
			{Text: back},
		}
		for _, r := range state.lookupResult {
			completionText := r.Song
			if r.Track > 0 {
				completionText = fmt.Sprintf("%d. %s", r.Track, r.Song)
			}
			suggestions = append(suggestions, prompt.Suggest{
				Text:           r.Song,
				CompletionText: completionText,
				Metadata:       r})
		}
	}
	return prompt.FilterHasPrefix(suggestions, in.CurrentLineBeforeCursor(), true)
}

func (state *cmdState) completerCmd(in prompt.Document, before []string) []prompt.Suggest {
	first := before[0]
	switch first {
	case addFolderCmdText, setMountCmdText:
		return filePathCompleter.Complete(in, true)
	case addQueueCmdText:
		rest := strings.Join(before[1:], " ")
		return dbCompleter(in, rest, true)
	default:
		return []prompt.Suggest{}
	}
}

func (state *cmdState) updateMaxWidths(in prompt.Document, titleRatio float32) {
	col := in.CursorPositionCol()
	base := float32(getAvailableWidth(col))

	titleMaxLength := int(base * titleRatio)
	descriptionMaxLength := int(base * (1 - titleRatio))
	prompt.OptionMaxTextWidth(uint16(titleMaxLength))(state.curPrompt)
	prompt.OptionMaxDescriptionWidth(uint16(descriptionMaxLength))(state.curPrompt)
}

func (state *cmdState) updateMaxDescriptionWidth(in prompt.Document, maxWidth int) {
	prompt.OptionMaxDescriptionWidth(uint16(maxWidth))(state.curPrompt)
}

func (state *cmdState) unsetMaxWidths() {
	prompt.OptionMaxTextWidth(0)(state.curPrompt)
	prompt.OptionMaxDescriptionWidth(0)(state.curPrompt)
}

func completerDefault(in prompt.Document) []prompt.Suggest {
	cmds := []prompt.Suggest{
		{Text: setQueueCmdText, Description: setQueueDescription},
		{Text: addQueueCmdText, Description: addQueueDescription},
		{Text: pauseCmdText, Description: pauseDescription},
		{Text: resumeCmdText, Description: resumeDescription},
		{Text: seekCmdText, Description: seekDescription},
		{Text: nextCmdText, Description: nextDescription},
		{Text: previousCmdText, Description: previousDescription},
		{Text: stopCmdText, Description: stopDescription},
		{Text: syncCmdText, Description: syncDescription},
		{Text: getAllFoldersCmdText, Description: getAllFoldersDescription},
		{Text: addFolderCmdText, Description: addFolderDescription},
		{Text: setMountCmdText, Description: setMountDescription},
		{Text: setVolumeCmdText, Description: setVolumeDescription},
		{Text: "q", Description: "Quit interactive prompt"},
	}

	if runtime.GOOS == "windows" {
		state.unsetMaxWidths()
		maxCmdLength := 15
		maxWidth := getAvailableWidth(maxCmdLength)
		state.updateMaxDescriptionWidth(in, maxWidth)
	}

	return prompt.FilterHasPrefix(cmds, in.GetWordBeforeCursor(), true)
}

func getAvailableWidth(currentCol int) int {
	cols, _ := consolesize.GetConsoleSize()
	base := cols - currentCol - 10
	return base
}

func isWindows() bool {
	return runtime.GOOS == "windows"
}

func dbCompleter(in prompt.Document, rest string, filePathSkipFirst bool) []prompt.Suggest {
	state.updateMaxWidths(in, 1./3)
	if searchClient == nil {
		searchClient = internal.Client.Search()
	}

	if strings.HasPrefix(rest, "http://") || strings.HasPrefix(rest, "https://") {
		return []prompt.Suggest{}
	}

	suggestions := filePathCompleter.Complete(in, filePathSkipFirst)
	if len(suggestions) > 0 && strings.ContainsAny(rest, "/\\") {

		prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"})(state.curPrompt)
		return suggestions
	}

	sendErr := searchClient.Send(&platune.SearchRequest{
		Query: rest,
	})
	if sendErr != nil {
		fmt.Println("send error", sendErr)
		return []prompt.Suggest{}
	}
	res, recvErr := searchClient.Recv()
	if recvErr != nil {
		fmt.Println("recv error", recvErr)
		return []prompt.Suggest{}
	}

	for _, r := range res.Results {
		suggestions = append(suggestions, prompt.Suggest{
			Text:        r.Entry,
			Description: r.Description,
			Metadata:    r,
		})
	}
	prompt.OptionCompletionWordSeparator([]string{addQueueCmdText + " "})(state.curPrompt)
	return suggestions
}
