package cmd

import (
	"fmt"
	"runtime"
	"sort"
	"strings"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/mode"
	platune "github.com/aschey/platune/client"
	"github.com/nathan-fiscaletti/consolesize-go"
)

const selectAll = "(Select All)"
const back = "(Back)"

var filePathCompleter = internal.FilePathCompleter{
	IgnoreCase: true,
}

func (state *cmdState) completer(in prompt.Document) []prompt.Suggest {
	before := strings.Split(in.TextBeforeCursor(), " ")
	if state.mode.Current() != mode.NormalMode {
		return state.completerMode(in)
	} else if len(before) > 1 {
		return state.completerCmd(in, before)
	} else {
		return state.completerDefault(in)
	}
}

func (state *cmdState) completerMode(in prompt.Document) []prompt.Suggest {
	suggestions := []prompt.Suggest{}
	// Windows terminal doesn't handle overflow as well as Unix
	if isWindows() {
		state.updateMaxWidths(in, 1.)
	} else {
		state.unsetMaxWidths()
	}

	switch state.mode.Current() {
	case mode.SetQueueMode:
		return state.dbCompleter(in, in.TextBeforeCursor(), false)
	case mode.AlbumMode:
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
		state.suggestions = suggestions

	case mode.SongMode:
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
		state.suggestions = suggestions
	}
	return prompt.FilterHasPrefix(suggestions, in.CurrentLineBeforeCursor(), true)
}

func (state *cmdState) completerCmd(in prompt.Document, before []string) []prompt.Suggest {
	if isWindows() {
		state.updateMaxWidths(in, 1.)
	} else {
		state.unsetMaxWidths()
	}

	first := before[0]
	switch first {
	case addFolderCmdText, setMountCmdText:
		return filePathCompleter.Complete(in, true)
	case addQueueCmdText:
		rest := strings.Join(before[1:], " ")
		return state.dbCompleter(in, rest, true)
	default:
		return []prompt.Suggest{}
	}
}

func (state *cmdState) updateMaxWidths(in prompt.Document, titleRatio float32) {
	col := in.CursorPositionCol()
	base := float32(getAvailableWidth(col))

	titleMaxLength := int(base * titleRatio)
	descriptionMaxLength := int(base * (1 - titleRatio))
	prompt.OptionMaxTextWidth(uint16(titleMaxLength))(state.curPrompt)              //nolint:errcheck
	prompt.OptionMaxDescriptionWidth(uint16(descriptionMaxLength))(state.curPrompt) //nolint:errcheck
}

func (state *cmdState) updateMaxTextWidth(in prompt.Document, maxWidth int) {
	prompt.OptionMaxTextWidth(uint16(maxWidth))(state.curPrompt) //nolint:errcheck
}

func (state *cmdState) updateMaxDescriptionWidth(in prompt.Document, maxWidth int) {
	prompt.OptionMaxDescriptionWidth(uint16(maxWidth))(state.curPrompt) //nolint:errcheck
}

func (state *cmdState) unsetMaxWidths() {
	prompt.OptionMaxTextWidth(0)(state.curPrompt)        //nolint:errcheck
	prompt.OptionMaxDescriptionWidth(0)(state.curPrompt) //nolint:errcheck
}

func (state *cmdState) completerDefault(in prompt.Document) []prompt.Suggest {
	cmds := []prompt.Suggest{
		{Text: setQueueCmdText, Description: setQueueDescription},
		{Text: addQueueCmdText, Description: addQueueDescription, Placeholder: addQueueExampleText},
		{Text: pauseCmdText, Description: pauseDescription},
		{Text: resumeCmdText, Description: resumeDescription},
		{Text: seekCmdText, Description: seekDescription, Placeholder: seekExampleText},
		{Text: nextCmdText, Description: nextDescription},
		{Text: previousCmdText, Description: previousDescription},
		{Text: stopCmdText, Description: stopDescription},
		{Text: syncCmdText, Description: syncDescription},
		{Text: getAllFoldersCmdText, Description: getAllFoldersDescription},
		{Text: addFolderCmdText, Description: addFolderDescription, Placeholder: addFolderExampleText},
		{Text: setMountCmdText, Description: setMountDescription, Placeholder: setMountExampleText},
		{Text: setVolumeCmdText, Description: setVolumeDescription, Placeholder: setVolumeExampleText},
		{Text: "q", Description: "Quit interactive prompt"},
	}

	if isWindows() {
		maxCmdLength := 15
		maxWidth := getAvailableWidth(maxCmdLength)
		state.updateMaxDescriptionWidth(in, maxWidth)
		state.updateMaxTextWidth(in, maxCmdLength)
	} else {
		state.unsetMaxWidths()
	}

	return prompt.FilterHasPrefix(cmds, in.GetWordBeforeCursor(), true)
}

func getAvailableWidth(currentCol int) int {
	consoleSize, _ := consolesize.GetConsoleSize()

	if !isWindows() {
		return consoleSize
	}
	available := float32(consoleSize)*.85 - float32(currentCol)
	return int(available)
}

func isWindows() bool {
	return runtime.GOOS == "windows"
}

func (state *cmdState) dbCompleter(in prompt.Document, rest string, filePathSkipFirst bool) []prompt.Suggest {
	state.updateMaxWidths(in, 1./3)

	suggestions := filePathCompleter.Complete(in, filePathSkipFirst)
	if len(suggestions) > 0 && strings.ContainsAny(rest, "/\\") {
		prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"})(state.curPrompt) //nolint:errcheck
		return suggestions
	}

	searchClient := *state.searchClient
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

	if len(res.Results) > 0 {
		for _, r := range res.Results {
			suggestions = append(suggestions, prompt.Suggest{
				Text:        r.Entry,
				Description: r.Description,
				Metadata:    r,
			})
		}
		state.suggestions = suggestions
	}
	prompt.OptionCompletionWordSeparator([]string{addQueueCmdText + " "})(state.curPrompt) //nolint:errcheck

	return suggestions
}
