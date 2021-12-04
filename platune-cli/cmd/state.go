package cmd

import (
	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	platune "github.com/aschey/platune/client"
)

type cmdState struct {
	mode         *Mode
	currentQueue []*platune.LookupEntry
	lookupResult []*platune.LookupEntry
	curPrompt    *prompt.Prompt
	suggestions  []prompt.Suggest
	searchClient *platune.Management_SearchClient
	client       *internal.PlatuneClient
	deleted      *deleted.Deleted
}

type ModeDef string

const (
	NormalMode   ModeDef = ">>> "
	SetQueueMode ModeDef = setQueueCmdText + "> "
	AlbumMode    ModeDef = "album> "
	SongMode     ModeDef = "song> "
)

type Mode struct {
	modeList []ModeDef
}

func NewMode(first ModeDef) *Mode {
	return &Mode{modeList: []ModeDef{first}}
}

func NewDefaultMode() *Mode {
	return NewMode(NormalMode)
}

func (mode *Mode) Current() ModeDef {
	return mode.modeList[len(mode.modeList)-1]
}

func (mode *Mode) First() ModeDef {
	return mode.modeList[0]
}

func (mode *Mode) Set(nextMode ModeDef) {
	mode.modeList = append(mode.modeList, nextMode)
}

func (mode *Mode) Reset() {
	mode.modeList = []ModeDef{mode.First()}
}

func (state *cmdState) changeLivePrefix() (string, bool) {
	return string(state.mode.Current()), true
}

func NewState(client *internal.PlatuneClient,
	deleted *deleted.Deleted) *cmdState {
	searchClient := client.Search()
	state := cmdState{
		mode:         NewDefaultMode(),
		currentQueue: []*platune.LookupEntry{},
		suggestions:  []prompt.Suggest{},
		client:       client,
		searchClient: &searchClient,
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
