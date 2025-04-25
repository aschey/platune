package cmd

import (
	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	"github.com/aschey/platune/cli/v2/internal/mode"
	"github.com/aschey/platune/cli/v2/internal/statusbar"
	management_v1 "github.com/aschey/platune/client/management_v1"
)

type cmdState struct {
	mode         *mode.Mode
	currentQueue []*management_v1.LookupEntry
	lookupResult []*management_v1.LookupEntry
	curPrompt    *prompt.Prompt
	client       *internal.PlatuneClient
	statusBar    *statusbar.StatusBar
	deleted      *deleted.Deleted
}

func (state *cmdState) changeLivePrefix() (string, bool) {
	return string(state.mode.Current()), true
}

func (state *cmdState) RunInteractive() int {
	state.statusBar.StartEventLoop()
	return state.curPrompt.Run()
}

func NewState(client *internal.PlatuneClient, deleted *deleted.Deleted, statusChan statusbar.StatusChan, statusBar *statusbar.StatusBar) *cmdState {
	state := cmdState{
		mode:         mode.NewDefaultMode(),
		currentQueue: []*management_v1.LookupEntry{},
		client:       client,
		statusBar:    statusBar,
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
