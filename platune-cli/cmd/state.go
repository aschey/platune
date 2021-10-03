package cmd

import (
	"github.com/aschey/go-prompt"
	platune "github.com/aschey/platune/client"
)

type cmdState struct {
	mode         []Mode
	currentQueue []*platune.LookupEntry
	lookupResult []*platune.LookupEntry
	curPrompt    *prompt.Prompt
	suggestions  []prompt.Suggest
}

type Mode string

const (
	NormalMode   Mode = ">>> "
	SetQueueMode Mode = setQueueCmdText + "> "
	AlbumMode    Mode = "album> "
	SongMode     Mode = "song> "
)

func (state *cmdState) changeLivePrefix() (string, bool) {
	return string(state.mode[len(state.mode)-1]), true
}

func initState() {
	state = cmdState{mode: []Mode{NormalMode}, currentQueue: []*platune.LookupEntry{}, suggestions: []prompt.Suggest{}}
	state.curPrompt = prompt.New(
		state.executor,
		state.completer,
		prompt.OptionPrefix(string(NormalMode)),
		prompt.OptionLivePrefix(state.changeLivePrefix),
		prompt.OptionTitle("Platune CLI"),
		prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"}),
		prompt.OptionShowCompletionAtStart(),
		prompt.OptionCompletionOnDown(),
	)
}

var state cmdState
var searchClient platune.Management_SearchClient
