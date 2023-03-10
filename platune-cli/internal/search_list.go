package internal

import (
	"fmt"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
)

type searchItem struct {
	searchResult *platune.SearchResult
}

type searchModel struct {
	list     list.Model
	choice   searchItem
	client   *ManagementClient
	callback func(entries []*platune.LookupEntry)
}

func (i searchItem) FilterValue() string { return i.searchResult.Entry }

func (i searchItem) Title() string { return i.searchResult.Entry }

func (i searchItem) Description() string { return i.searchResult.Description }

func newSearchDelegate() list.DefaultDelegate {
	delegate := list.NewDefaultDelegate()

	return delegate
}

func (m searchModel) Init() tea.Cmd {
	return nil
}

func (m searchModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.list.SetWidth(msg.Width)
		return m, nil

	case tea.KeyMsg:
		switch keypress := msg.String(); keypress {

		case "enter":
			i := m.list.SelectedItem().(searchItem)
			client := *m.client
			lookupResults, _ := client.Lookup(i.searchResult.EntryType, i.searchResult.CorrelationIds)
			m.callback(lookupResults.Entries)
			m.choice = i

			return m, tea.Quit

		default:
			var cmd tea.Cmd
			m.list, cmd = m.list.Update(msg)
			return m, cmd
		}

	default:
		return m, nil
	}
}

func (m searchModel) View() string {
	if m.choice.searchResult.Entry != "" {
		result := m.choice.searchResult
		if result.Artist != nil {
			return quitTextStyle.Render(
				fmt.Sprintf("%s - %s added to queue", result.Entry, *result.Artist),
			)
		}
		return quitTextStyle.Render(fmt.Sprintf("%s added to queue", result.Entry))
	}

	return "\n" + m.list.View()
}

func getSearchItems(results []*platune.SearchResult) []list.Item {
	items := []list.Item{}
	for _, result := range results {
		items = append(items, searchItem{searchResult: result})
	}

	return items
}

func (search *Search) renderSearchResults(
	results *platune.SearchResponse,
	callback func(entries []*platune.LookupEntry),
) tea.Model {
	const defaultWidth = 20
	const defaultHeight = 14

	l := list.New(getSearchItems(results.Results), newSearchDelegate(), defaultWidth, defaultHeight)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowTitle(false)

	l.Styles.PaginationStyle = paginationStyle
	l.Styles.HelpStyle = helpStyle
	m := searchModel{
		list:     l,
		client:   search.client,
		callback: callback,
		choice:   searchItem{searchResult: &platune.SearchResult{}},
	}
	return m
}
