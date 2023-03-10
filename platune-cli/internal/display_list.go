package internal

import (
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
)

type displayItem struct {
	title       string
	description string
}

type displayModel struct {
	list     list.Model
	choice   displayItem
	client   *ManagementClient
	callback func(entries []displayItem)
}

func (i displayItem) FilterValue() string { return i.title }

func (i displayItem) Title() string { return i.title }

func (i displayItem) Description() string { return i.description }

func newDisplayDelegate() list.DefaultDelegate {
	delegate := list.NewDefaultDelegate()
	delegate.ShowDescription = false
	return delegate
}

func (m displayModel) Init() tea.Cmd {
	return nil
}

func (m displayModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.list.SetWidth(msg.Width)
		return m, nil

	case tea.KeyMsg:
		switch keypress := msg.String(); keypress {

		case "enter":
			i := m.list.SelectedItem().(displayItem)

			m.callback([]displayItem{i})
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

func getDisplayItems(results []displayItem) []list.Item {
	items := []list.Item{}
	for _, result := range results {
		items = append(items, result)
	}

	return items
}

func (m displayModel) View() string {
	return "\n" + m.list.View()
}

func (search *Search) renderDisplay(
	title string,
	items []displayItem,
	callback func([]displayItem),
) tea.Model {
	const defaultWidth = 20
	const defaultHeight = 14

	l := list.New(getDisplayItems(items), newDisplayDelegate(), defaultWidth, defaultHeight)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(true)
	l.Title = title
	l.SetShowTitle(true)

	l.Styles.PaginationStyle = paginationStyle
	l.Styles.HelpStyle = helpStyle
	m := displayModel{
		list:     l,
		client:   search.client,
		callback: callback,
		choice:   displayItem{},
	}
	return m
}
