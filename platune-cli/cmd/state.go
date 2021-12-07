package cmd

import (
	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	"github.com/aschey/platune/cli/v2/internal/mode"
	platune "github.com/aschey/platune/client"
)

type cmdState struct {
	mode         *mode.Mode
	currentQueue []*platune.LookupEntry
	lookupResult []*platune.LookupEntry
	curPrompt    *prompt.Prompt
	suggestions  []prompt.Suggest
	client       *internal.PlatuneClient
	deleted      *deleted.Deleted
}

func (state *cmdState) changeLivePrefix() (string, bool) {
	return string(state.mode.Current()), true
}

func NewState(client *internal.PlatuneClient, deleted *deleted.Deleted, statusChan internal.StatusChan) *cmdState {
	state := cmdState{
		mode:         mode.NewDefaultMode(),
		currentQueue: []*platune.LookupEntry{},
		suggestions:  []prompt.Suggest{},
		client:       client,
		deleted:      deleted,
	}
	state.curPrompt = prompt.New(
		state.executor,
		state.completer,
		prompt.OptionPrefix(string(mode.NormalMode)),
		prompt.OptionLivePrefix(state.changeLivePrefix),
		prompt.OptionTitle("Platune CLI"),
		prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"}),
		prompt.OptionShowCompletionAtStart(),
		prompt.OptionCompletionOnDown(),
		prompt.OptionStatusbarSignal(statusChan),
	)

	return &state
}
