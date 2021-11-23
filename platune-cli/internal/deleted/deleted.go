package deleted

import (
	"fmt"
	"io"
	"os"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type item struct {
	path     string
	index    int
	selected bool
}

var (
	itemStyle         = lipgloss.NewStyle().PaddingLeft(4)
	selectedItemStyle = lipgloss.NewStyle().PaddingLeft(2).Foreground(lipgloss.Color("170"))
	paginationStyle   = list.DefaultStyles().PaginationStyle.PaddingLeft(4)
	helpStyle         = list.DefaultStyles().HelpStyle.PaddingLeft(4).PaddingBottom(1)
	quitTextStyle     = lipgloss.NewStyle().Margin(1, 0, 2, 4)
	subtle            = lipgloss.AdaptiveColor{Light: "#D9DCCF", Dark: "#383838"}
	buttonStyle       = lipgloss.NewStyle().
				Foreground(lipgloss.Color("#FFF7DB")).
				Background(lipgloss.Color("#888B7E")).
				Padding(0, 3).
				MarginTop(1)
	activeButtonStyle = buttonStyle.Copy().
				Foreground(lipgloss.Color("#FFF7DB")).
				Background(lipgloss.Color("#F25D94")).
				Underline(true)
	dialogBoxStyle = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("#874BFD")).
			Padding(1, 0).
			BorderTop(true).
			BorderLeft(true).
			BorderRight(true).
			BorderBottom(true)
)

type model struct {
	list              list.Model
	showConfirmDialog bool
	cancelChosen      bool
	quitText          string
}

type itemDelegate struct{}

func (i item) FilterValue() string { return i.path }

func (d itemDelegate) Height() int                               { return 1 }
func (d itemDelegate) Spacing() int                              { return 0 }
func (d itemDelegate) Update(msg tea.Msg, m *list.Model) tea.Cmd { return nil }
func (d itemDelegate) Render(w io.Writer, m list.Model, index int, listItem list.Item) {
	i := listItem.(item)

	var str string
	fn := itemStyle.Render

	if i.selected {
		str = fmt.Sprintf("◉ %s", i.path)
	} else {
		str = fmt.Sprintf("◯ %s", i.path)
	}

	if index == m.Index() {
		fn = func(s string) string {
			return selectedItemStyle.Render("▶ " + s)
		}
	}

	fmt.Fprint(w, fn(str))
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) updateList(msg tea.KeyMsg, keypress string) (tea.Model, tea.Cmd) {
	i := m.list.SelectedItem().(item)

	switch keypress {
	case "a":
		allSelected := true
		items := m.list.Items()
		for _, it := range items {
			i := it.(item)
			if !i.selected {
				allSelected = false
				break
			}
		}
		shouldSelectAll := !allSelected
		for index, it := range items {
			i := it.(item)
			i.selected = shouldSelectAll
			m.list.SetItem(index, i)
		}
		var cmd tea.Cmd
		m.list, cmd = m.list.Update(msg)
		return m, cmd

	case " ":
		i.selected = !i.selected
		m.list.SetItem(m.list.Index(), i)

		var cmd tea.Cmd
		m.list, cmd = m.list.Update(msg)
		return m, cmd

	case "enter":
		if m.getNumSelected() > 0 {
			m.showConfirmDialog = true
			return m, nil
		}
		m.quitText = "No Songs Deleted"
		return m, tea.Quit

	default:
		var cmd tea.Cmd
		m.list, cmd = m.list.Update(msg)
		return m, cmd
	}
}

func (m model) updateConfirmDialog(keypress string) (tea.Model, tea.Cmd) {
	switch keypress {
	case "tab", "left", "right", " ":
		m.cancelChosen = !m.cancelChosen
	case "enter":
		if m.cancelChosen {
			m.showConfirmDialog = false
		} else {
			m.quitText = fmt.Sprintf("%d song(s) deleted", m.getNumSelected())
			return m, tea.Quit
		}
	case "ctrl+c":
		return m, tea.Quit
	}

	return m, nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.list.SetWidth(msg.Width)
		return m, nil

	case tea.KeyMsg:
		keypress := msg.String()
		if m.showConfirmDialog {
			return m.updateConfirmDialog(keypress)
		} else {
			return m.updateList(msg, keypress)
		}
	default:
		return m, nil
	}
}

var helpKeys = []key.Binding{
	key.NewBinding(key.WithKeys("a"),
		key.WithHelp("a", "select all")),
	key.NewBinding(key.WithKeys("space"),
		key.WithHelp("space", "choose")),
	key.NewBinding(key.WithKeys("enter"),
		key.WithHelp("enter", "submit")),
}

func (d itemDelegate) ShortHelp() []key.Binding {
	return helpKeys
}

func (d itemDelegate) FullHelp() [][]key.Binding {
	return [][]key.Binding{helpKeys}
}

func (m model) getNumSelected() int {
	numSelectedSongs := 0
	for _, listItem := range m.list.Items() {
		i := listItem.(item)
		if i.selected {
			numSelectedSongs++
		}
	}
	return numSelectedSongs
}

func (m model) viewConfirmDialog() string {
	text := fmt.Sprintf("Are you sure you want to permanently delete %d song(s)?", m.getNumSelected())

	question := lipgloss.NewStyle().Width(50).Align(lipgloss.Center).Render(text)
	okButtonStyle := buttonStyle
	cancelButtonStyle := buttonStyle
	if m.cancelChosen {
		cancelButtonStyle = activeButtonStyle
	} else {
		okButtonStyle = activeButtonStyle
	}
	okButtonStyle = okButtonStyle.MarginRight(2)
	cancelButtonStyle = cancelButtonStyle.MarginLeft(2)

	okButton := okButtonStyle.Render("Ok")
	cancelButton := cancelButtonStyle.Render("Cancel")
	buttons := lipgloss.JoinHorizontal(lipgloss.Top, okButton, cancelButton)
	ui := lipgloss.JoinVertical(lipgloss.Center, question, buttons)

	dialog := dialogBoxStyle.Render(ui)
	return dialog
}

func (m model) View() string {
	if m.quitText != "" {
		return quitTextStyle.Render(m.quitText)
	}
	if m.showConfirmDialog {
		m.viewConfirmDialog()
	}
	return m.list.View()
}

func getItems(results []string) []list.Item {
	items := []list.Item{}
	for i, result := range results {
		items = append(items, item{path: result, index: i, selected: false})
	}

	return items
}

func RenderDeletedFiles() {
	const defaultWidth = 20
	const defaultHeight = 14
	deleted := internal.Client.GetDeleted()
	for i := 0; i < 20; i++ {
		deleted.Paths = append(deleted.Paths, fmt.Sprintf("%d", i))
	}

	l := list.NewModel(getItems(deleted.Paths), itemDelegate{}, defaultWidth, defaultHeight)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)
	l.SetShowPagination(true)
	l.SetShowTitle(true)
	l.Title = fmt.Sprintf("Found %d missing songs", len(deleted.Paths))
	l.NewStatusMessage("Choose which songs to remove")

	l.Styles.PaginationStyle = paginationStyle
	l.Styles.HelpStyle = helpStyle
	m := model{list: l, showConfirmDialog: false, quitText: ""}

	if err := tea.NewProgram(m).Start(); err != nil {
		fmt.Println("Error running program:", err)
		os.Exit(1)
	}
}
