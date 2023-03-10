package queue

import (
	"strings"

	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/bubbleprompt/input/commandinput"
	"github.com/aschey/bubbleprompt/suggestion"
	"github.com/aschey/platune/cli/internal"
	platune "github.com/aschey/platune/client"
	"github.com/spf13/cobra"
)

type addQueueCmd *cobra.Command

func newAddQueueCmd(playerclient *internal.PlayerClient, managementClient *internal.ManagementClient, search *internal.Search) addQueueCmd {
	addQueueCmd := &cobra.Command{
		Use:   "add <song, artist, album, file path, or url>",
		Short: "Adds a song to the end of the queue",
		RunE: func(cmd *cobra.Command, args []string) error {
			selected := cprompt.GetSelectedSuggestion[internal.SearchMetadata](cmd)
			var searchResult *platune.SearchResult = nil
			if selected != nil {
				searchResult = selected.Metadata.Extra.Result
			}
			// println(selected.Metadata.Extra.Result.Entry)
			results, err := search.ProcessSearchResults(args, searchResult,
				func(file string) { playerclient.AddToQueue([]string{file}) },
				func(entries []*platune.LookupEntry) { playerclient.AddSearchResultsToQueue(entries) })
			if err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, results)
		},
	}
	cprompt.Completer(addQueueCmd, func(cmd *cobra.Command, args []string, toComplete string) ([]suggestion.Suggestion[commandinput.CommandMetadata[internal.SearchMetadata]], error) {
		searchResults, err := managementClient.Search(&platune.SearchRequest{Query: strings.Join(append(args, toComplete), " ")})
		if err != nil {
			return nil, err
		}
		suggestions := []suggestion.Suggestion[commandinput.CommandMetadata[internal.SearchMetadata]]{}
		for _, result := range searchResults.Results {
			text := result.Entry
			if len(strings.SplitN(result.Entry, " ", 2)) > 1 {
				text = `"` + text + `"`
			}
			suggestions = append(suggestions, suggestion.Suggestion[commandinput.CommandMetadata[internal.SearchMetadata]]{
				SuggestionText: result.Entry,
				Text:           text,
				Description:    result.Description,
				Metadata: commandinput.CommandMetadata[internal.SearchMetadata]{
					Extra: internal.SearchMetadata{
						Result: result,
					},
				},
			})
		}
		return suggestions, nil
	})
	return addQueueCmd
}
