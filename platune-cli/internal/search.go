package internal

import (
	"os"
	"path/filepath"
	"strings"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

var (
	itemStyle         = lipgloss.NewStyle().PaddingLeft(4)
	selectedItemStyle = lipgloss.NewStyle().PaddingLeft(2).Foreground(lipgloss.Color("170"))
	paginationStyle   = list.DefaultStyles().PaginationStyle.PaddingLeft(4)
	helpStyle         = list.DefaultStyles().HelpStyle.PaddingLeft(4).PaddingBottom(1)
	quitTextStyle     = lipgloss.NewStyle().Margin(1, 0, 2, 4)
)

type model struct {
	list     list.Model
	choice   item
	client   *ManagementClient
	callback func(entries []*platune.LookupEntry)
}

var noResultsStr string = "No results"

type Search struct {
	client *ManagementClient
}

func NewSearch(client *ManagementClient) *Search {
	return &Search{client: client}
}

func (s *Search) ProcessSearchResults(
	args []string,
	selected *platune.SearchResult,
	filesystemCallback func(file string),
	dbCallback func(entries []*platune.LookupEntry),
) (tea.Model, error) {
	allArgs := strings.Join(args, " ")
	_, err := os.Stat(allArgs)
	if err == nil {
		full, err := filepath.Abs(allArgs)
		if err != nil {
			return nil, err
		}
		filesystemCallback(full)
	} else if strings.HasPrefix(allArgs, "http://") || strings.HasPrefix(allArgs, "https://") {
		filesystemCallback(allArgs)
		return NewInfoModel("Added " + allArgs + " to the queue"), nil
	} else {
		if selected == nil {
			results, err := s.client.Search(&platune.SearchRequest{Query: allArgs})
			if err != nil {
				return nil, err
			}
			if len(results.Results) == 0 {
				return NewInfoModel(noResultsStr), nil
			} else if len(results.Results) == 1 {
				selected = results.Results[0]
			} else {
				return s.renderSearchResults(results, dbCallback), nil
			}
		}

		if selected != nil {
			if selected.EntryType == platune.EntryType_SONG {
				lookupResults, _ := s.client.Lookup(selected.EntryType, selected.CorrelationIds)

				dbCallback(lookupResults.Entries)
				return NewInfoModel("Added " + selected.Entry + " " + selected.Description + " to the queue"), nil
			} else {
				return s.renderSearchResults(&platune.SearchResponse{Results: []*platune.SearchResult{selected}}, dbCallback), nil
			}
		}

	}
	return nil, nil
}
