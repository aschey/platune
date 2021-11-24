package cmd

import (
	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	platune "github.com/aschey/platune/client"
)

type cmdState struct {
	mode         []Mode
	currentQueue []*platune.LookupEntry
	lookupResult []*platune.LookupEntry
	curPrompt    *prompt.Prompt
	suggestions  []prompt.Suggest
	searchClient *platune.Management_SearchClient
	client       *internal.PlatuneClient
	deleted      *deleted.Deleted
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

func NewState(client *internal.PlatuneClient, searchClient *platune.Management_SearchClient,
	deleted *deleted.Deleted) *cmdState {
	state := cmdState{
		mode:         []Mode{NormalMode},
		currentQueue: []*platune.LookupEntry{},
		suggestions:  []prompt.Suggest{},
		client:       client,
		searchClient: searchClient,
		deleted:      deleted,
	}
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

	return &state
}
