package cmd

import (
	"fmt"
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

func (state *cmdState) completer(in prompt.Document, returnChan chan []prompt.Suggest) {
	before := strings.Split(in.TextBeforeCursor(), " ")
	if state.mode.Current() != mode.NormalMode {
		state.completerMode(in, returnChan)
	} else if len(before) > 1 {
		state.completerCmd(in, before, returnChan)
	} else {
		state.completerDefault(in, returnChan)
	}
}

func (state *cmdState) completerMode(in prompt.Document, returnChan chan []prompt.Suggest) {
	suggestions := []prompt.Suggest{}

	switch state.mode.Current() {
	case mode.SetQueueMode:
		state.dbCompleter(in, in.TextBeforeCursor(), false, returnChan)
	case mode.AlbumMode:
		state.unsetMaxWidths()
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
		returnChan <- prompt.FilterHasPrefix(suggestions, in.CurrentLineBeforeCursor(), true)

	case mode.SongMode:
		state.unsetMaxWidths()
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
		returnChan <- prompt.FilterHasPrefix(suggestions, in.CurrentLineBeforeCursor(), true)
	}
}

func (state *cmdState) completerCmd(in prompt.Document, before []string, returnChan chan []prompt.Suggest) {
	first := before[0]
	switch first {
	case addFolderCmdText, setMountCmdText:
		state.unsetMaxWidths()
		returnChan <- filePathCompleter.Complete(in, true)
	case addQueueCmdText:
		rest := strings.Join(before[1:], " ")
		state.dbCompleter(in, rest, true, returnChan)
	default:
		returnChan <- []prompt.Suggest{}
	}
}

func (state *cmdState) updateMaxWidths(in prompt.Document, titleRatio float32) {
	size, _ := consolesize.GetConsoleSize()
	base := float32(size)

	titleMaxLength := int(base * titleRatio)
	descriptionMaxLength := int(base * (1 - titleRatio))
	prompt.OptionMaxTextWidth(uint16(titleMaxLength))(state.curPrompt)              //nolint:errcheck
	prompt.OptionMaxDescriptionWidth(uint16(descriptionMaxLength))(state.curPrompt) //nolint:errcheck
}

func (state *cmdState) unsetMaxWidths() {
	prompt.OptionMaxTextWidth(0)(state.curPrompt)        //nolint:errcheck
	prompt.OptionMaxDescriptionWidth(0)(state.curPrompt) //nolint:errcheck
}

func (state *cmdState) completerDefault(in prompt.Document, returnChan chan []prompt.Suggest) {
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

	state.unsetMaxWidths()

	returnChan <- prompt.FilterHasPrefix(cmds, in.GetWordBeforeCursor(), true)
}

func (state *cmdState) dbCompleter(in prompt.Document, rest string, filePathSkipFirst bool, returnChan chan []prompt.Suggest) []prompt.Suggest {
	state.updateMaxWidths(in, 1./3)

	suggestions := filePathCompleter.Complete(in, filePathSkipFirst)
	if len(suggestions) > 0 && strings.ContainsAny(rest, "/\\") {
		prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"})(state.curPrompt) //nolint:errcheck
		return suggestions
	}

	go func() {
		res, err := state.client.Search(&platune.SearchRequest{
			Query: rest,
		})
		if err != nil {
			fmt.Println(err)
			returnChan <- []prompt.Suggest{}
			return
		}

		if len(res.Results) > 0 {
			for _, r := range res.Results {
				suggestions = append(suggestions, prompt.Suggest{
					Text:        r.Entry,
					Description: r.Description,
					Metadata:    r,
				})
			}
		}
		returnChan <- suggestions
	}()

	prompt.OptionCompletionWordSeparator([]string{addQueueCmdText + " "})(state.curPrompt) //nolint:errcheck

	return suggestions
}
