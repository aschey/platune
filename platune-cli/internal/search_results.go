package internal

import (
	"fmt"
	"io"
	"os"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type item struct {
	searchResult platune.SearchResult
}

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
	callback func(entries []*platune.LookupEntry)
}

func (i item) FilterValue() string { return i.searchResult.Entry }

type itemDelegate struct{}

func (d itemDelegate) Height() int                               { return 1 }
func (d itemDelegate) Spacing() int                              { return 0 }
func (d itemDelegate) Update(msg tea.Msg, m *list.Model) tea.Cmd { return nil }
func (d itemDelegate) Render(w io.Writer, m list.Model, index int, listItem list.Item) {
	i, ok := listItem.(item)
	if !ok {
		return
	}

	str := fmt.Sprintf("%s - %s", i.searchResult.Entry, i.searchResult.Description)

	fn := itemStyle.Render
	if index == m.Index() {
		fn = func(s string) string {
			return selectedItemStyle.Render("> " + s)
		}
	}

	fmt.Fprint(w, fn(str))
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
			lookupResults := Client.Lookup(i.searchResult.EntryType, i.searchResult.CorrelationIds)
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
			return quitTextStyle.Render(fmt.Sprintf("%s - %s added to queue", result.Entry, *result.Artist))
		}
		return quitTextStyle.Render(fmt.Sprintf("%s added to queue", result.Entry))
	}

	return m.list.View()
}

func RenderSearchResults(results *platune.SearchResponse, callback func(entries []*platune.LookupEntry)) {
	items := []list.Item{}
	for _, result := range results.Results {
		items = append(items, item{searchResult: *result})
	}
	const defaultWidth = 20
	const defaultHeight = 14

	l := list.NewModel(items, itemDelegate{}, defaultWidth, defaultHeight)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowTitle(false)

	l.Styles.PaginationStyle = paginationStyle
	l.Styles.HelpStyle = helpStyle
	m := model{list: l, callback: callback}

	if err := tea.NewProgram(m).Start(); err != nil {
		fmt.Println("Error running program:", err)
		os.Exit(1)
	}
}
