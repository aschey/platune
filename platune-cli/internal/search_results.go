package internal

import (
	"fmt"
	"io"
	"os"

	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type item struct {
	searchResult platune.SearchResult
	queuePos     int
}

var (
	itemStyle         = lipgloss.NewStyle().PaddingLeft(2)
	selectedItemStyle = lipgloss.NewStyle().PaddingLeft(2).Foreground(lipgloss.Color("170"))
	paginationStyle   = list.DefaultStyles().PaginationStyle.PaddingLeft(4)
	helpStyle         = list.DefaultStyles().HelpStyle.PaddingLeft(4).PaddingBottom(1)
	quitTextStyle     = lipgloss.NewStyle().Margin(1, 0, 2, 4)
)

type model struct {
	list   list.Model
	choice item
	next   int
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
	if index == 0 {
		str = i.searchResult.Entry
	}

	position := i.queuePos
	var prefix string
	if position > 0 {
		prefix = fmt.Sprintf("[%d] ", position)
	} else if index == 0 {
		prefix = ""
	} else {
		prefix = "[ ] "
	}
	var fn func(string) string
	if index == m.Index() {
		fn = func(s string) string {
			return selectedItemStyle.Render(prefix + s)
		}
	} else {
		fn = func(s string) string {
			return itemStyle.Render(prefix + s)
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
			i, ok := m.list.SelectedItem().(item)
			if ok {
				if i.queuePos == 0 {
					i.queuePos = m.next
					m.list.Items()[m.list.Index()] = i
					m.next++
				} else {
					oldPos := i.queuePos
					i.queuePos = 0
					m.list.Items()[m.list.Index()] = i
					m.next--
					for index, l := range m.list.Items() {
						listItem := l.(item)
						if listItem.queuePos > oldPos {
							listItem.queuePos--
							m.list.Items()[index] = listItem
						}
					}
				}
				m.choice = i

				// lookupRequest := platune.LookupRequest{
				// 	EntryType:      i.EntryType,
				// 	CorrelationIds: i.CorrelationIds,
				// }
				// Client.AddSearchResultsToQueue(&lookupRequest)
			}
			var cmd tea.Cmd
			m.list, cmd = m.list.Update(msg)
			return m, cmd

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
	// if m.choice.Entry != "" {
	// 	return quitTextStyle.Render(fmt.Sprintf("%s added to queue", m.choice.Entry))
	// }

	return m.list.View()
}

func RenderSearchResults(results *platune.SearchResponse) {
	items := []list.Item{item{searchResult: platune.SearchResult{Entry: "[Add All]"}, queuePos: 0}}
	for _, result := range results.Results {
		items = append(items, item{searchResult: *result, queuePos: 0})
	}

	l := list.NewModel(items, itemDelegate{}, 20, 14)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowTitle(false)
	l.AdditionalShortHelpKeys = func() []key.Binding {
		return []key.Binding{
			key.NewBinding(key.WithKeys("s"), key.WithHelp("s", "save and close")),
		}
	}

	l.Styles.PaginationStyle = paginationStyle
	l.Styles.HelpStyle = helpStyle
	m := model{list: l, next: 1}

	if err := tea.NewProgram(m).Start(); err != nil {
		fmt.Println("Error running program:", err)
		os.Exit(1)
	}
}
