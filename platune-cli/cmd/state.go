package cmd

import (
	"github.com/aschey/go-prompt"
	platune "github.com/aschey/platune/client"
)

type cmdState struct {
	mode         Mode
	currentQueue []string
	lookupResult []*platune.LookupEntry
	curPrompt    *prompt.Prompt
}

type Mode string

const (
	NormalMode   Mode = ">>> "
	SetQueueMode Mode = "set-queue> "
	AlbumMode    Mode = "album> "
	SongMode     Mode = "song> "
)

func (state *cmdState) changeLivePrefix() (string, bool) {
	return string(state.mode), true
}

func initState() {
	state = cmdState{mode: NormalMode, currentQueue: []string{}}
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