package internal

import (
	"fmt"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
)

type item struct {
	searchResult *platune.SearchResult
}

func (i item) FilterValue() string { return i.searchResult.Entry }

func (i item) Title() string { return i.searchResult.Entry }

func (i item) Description() string { return i.searchResult.Description }

func newDelegate() list.DefaultDelegate {
	delegate := list.NewDefaultDelegate()

	return delegate
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.list.SetWidth(msg.Width)
		return m, nil

	case tea.KeyMsg:
		switch keypress := msg.String(); keypress {

		case "enter":
			i := m.list.SelectedItem().(item)
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

func (m model) View() string {
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

func getItems(results []*platune.SearchResult) []list.Item {
	items := []list.Item{}
	for _, result := range results {
		items = append(items, item{searchResult: result})
	}

	return items
}

func (search *Search) renderSearchResults(
	results *platune.SearchResponse,
	callback func(entries []*platune.LookupEntry),
) tea.Model {
	const defaultWidth = 20
	const defaultHeight = 14

	l := list.New(getItems(results.Results), newDelegate(), defaultWidth, defaultHeight)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowTitle(false)

	l.Styles.PaginationStyle = paginationStyle
	l.Styles.HelpStyle = helpStyle
	m := model{
		list:     l,
		client:   search.client,
		callback: callback,
		choice:   item{searchResult: &platune.SearchResult{}},
	}
	return m
}
