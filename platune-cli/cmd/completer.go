package cmd

import (
	"fmt"
	"sort"
	"strings"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
)

var filePathCompleter = internal.FilePathCompleter{
	IgnoreCase: true,
}

func (state *cmdState) completer(in prompt.Document) []prompt.Suggest {

	if state.mode != NormalMode {
		return state.completerMode(in)
	}
	before := strings.Split(in.TextBeforeCursor(), " ")

	if len(before) > 1 {
		return state.completerCmd(in, before)
	}

	return completerDefault(in)
}

func (state *cmdState) completerMode(in prompt.Document) []prompt.Suggest {
	suggestions := []prompt.Suggest{}
	switch state.mode {
	case SetQueueMode:
		return filePathCompleter.Complete(in, false)
	case AlbumMode:
		suggestionMap := map[string]prompt.Suggest{}
		for _, r := range state.lookupResult {
			suggestionMap[r.Album] = prompt.Suggest{Text: r.Album, Metadata: r}
		}

		for r := range suggestionMap {
			suggestions = append(suggestions, suggestionMap[r])
		}
		sort.Slice(suggestions, func(i, j int) bool {
			return suggestions[i].Text < suggestions[j].Text
		})
		suggestions = append([]prompt.Suggest{{Text: "(Select All)", Metadata: state.lookupResult}}, suggestions...)
		return suggestions
	case SongMode:
		suggestions = []prompt.Suggest{{Text: "(Select All)", Metadata: state.lookupResult}}
		for _, r := range state.lookupResult {
			suggestions = append(suggestions, prompt.Suggest{Text: r.Song, Metadata: r})
		}
		return suggestions
	}
	return suggestions
}

func (state *cmdState) completerCmd(in prompt.Document, before []string) []prompt.Suggest {
	suggestions := []prompt.Suggest{}
	first := before[0]
	switch first {

	case "add-folder", "set-mount":
		return filePathCompleter.Complete(in, true)
	case "add-queue":
		if searchClient == nil {
			searchClient = internal.Client.Search()
		}
		rest := strings.Join(before[1:], " ")

		if strings.HasPrefix(rest, "http://") || strings.HasPrefix(rest, "https://") {
			return []prompt.Suggest{}
		}

		suggestions = filePathCompleter.Complete(in, true)
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

		col := in.CursorPositionCol()
		base := getAvailableWidth(col)

		titleMaxLength := int(base * (1.0 / 3.0))
		descriptionMaxLength := int(base * (2.0 / 3.0))
		prompt.OptionMaxTextWidth(uint16(titleMaxLength))(state.curPrompt)
		prompt.OptionMaxDescriptionWidth(uint16(descriptionMaxLength))(state.curPrompt)
		for _, r := range res.Results {
			suggestions = append(suggestions, prompt.Suggest{
				Text:        r.Entry,
				Description: r.Description,
				Metadata:    r,
			})
		}
		prompt.OptionCompletionWordSeparator([]string{"add-queue "})(state.curPrompt)
		return suggestions
	default:
		return suggestions
	}
}

func completerDefault(in prompt.Document) []prompt.Suggest {
	cmds := []prompt.Suggest{
		{Text: "set-queue", Description: SetQueueDescription},
		{Text: "add-queue", Description: AddQueueDescription},
		{Text: "pause", Description: PauseDescription},
		{Text: "resume", Description: ResumeDescription},
		{Text: "seek", Description: SeekDescription},
		{Text: "next", Description: NextDescription},
		{Text: "previous", Description: PreviousDescription},
		{Text: "stop", Description: StopDescription},
		{Text: "sync", Description: SyncDescription},
		{Text: "get-all-folders", Description: GetAllFoldersDescription},
		{Text: "add-folder", Description: AddFolderDescription},
		{Text: "set-mount", Description: SetMountDescription},
		{Text: "q", Description: "Quit interactive prompt"},
	}
	maxCmdLength := 15
	maxWidth := getAvailableWidth(maxCmdLength)
	for i, cmd := range cmds {
		cmds[i].Description = ellipsize(cmd.Description, int(maxWidth))
	}

	return prompt.FilterHasPrefix(cmds, in.GetWordBeforeCursor(), true)
}
