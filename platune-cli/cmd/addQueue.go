package cmd

import (
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

const AddQueueDescription = "Adds a song to the end of the queue"

const addQueueExampleText = "fileOrUrl"

type item platune.SearchResult

var (
	itemStyle         = lipgloss.NewStyle().PaddingLeft(4)
	selectedItemStyle = lipgloss.NewStyle().PaddingLeft(2).Foreground(lipgloss.Color("170"))
	paginationStyle   = list.DefaultStyles().PaginationStyle.PaddingLeft(4)
	helpStyle         = list.DefaultStyles().HelpStyle.PaddingLeft(4).PaddingBottom(1)
	quitTextStyle     = lipgloss.NewStyle().Margin(1, 0, 2, 4)
)

type model struct {
	list   list.Model
	choice item
}

func (i item) FilterValue() string { return i.Entry }

type itemDelegate struct{}

func (d itemDelegate) Height() int                               { return 1 }
func (d itemDelegate) Spacing() int                              { return 0 }
func (d itemDelegate) Update(msg tea.Msg, m *list.Model) tea.Cmd { return nil }
func (d itemDelegate) Render(w io.Writer, m list.Model, index int, listItem list.Item) {
	i, ok := listItem.(item)
	if !ok {
		return
	}

	str := fmt.Sprintf("%d. %s - %s", index+1, i.Entry, i.Description)

	fn := itemStyle.Render
	if index == m.Index() {
		fn = func(s string) string {
			return selectedItemStyle.Render("> " + s)
		}
	}

	fmt.Fprintln(w, fn(str))
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
				m.choice = i
				lookupRequest := platune.LookupRequest{
					EntryType:      i.EntryType,
					CorrelationIds: i.CorrelationIds,
				}
				lookupResults := internal.Client.Lookup(&lookupRequest)
				paths := []string{}
				for _, entry := range lookupResults.Entries {
					paths = append(paths, entry.Path)
				}
				internal.Client.AddToQueue(paths)
			}
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
	if m.choice.Entry != "" {
		return quitTextStyle.Render(fmt.Sprintf("%s added to queue", m.choice.Entry))
	}

	return "\n" + m.list.View()
}

var addQueueCmd = &cobra.Command{
	Use:   "add-queue " + addQueueExampleText,
	Short: AddQueueDescription,
	Long:  AddQueueDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		arg := args[0]
		_, err := os.Stat(arg)
		if err == nil {
			full, err := filepath.Abs(arg)
			if err != nil {
				fmt.Println(err)
			}
			internal.Client.AddToQueue([]string{full})
		} else {
			searchClient = internal.Client.Search()
			err := searchClient.Send(&platune.SearchRequest{Query: arg})
			if err != nil {
				fmt.Println(err)
			}
			results, err := searchClient.Recv()
			if err != nil {
				fmt.Println(err)
			}
			items := []list.Item{}
			for _, result := range results.Results {
				items = append(items, item(*result))
			}

			l := list.NewModel(items, itemDelegate{}, 20, 14)
			l.SetShowStatusBar(false)
			l.SetFilteringEnabled(false)
			l.SetShowTitle(false)

			l.Styles.PaginationStyle = paginationStyle
			l.Styles.HelpStyle = helpStyle
			m := model{list: l}

			if err := tea.NewProgram(m).Start(); err != nil {
				fmt.Println("Error running program:", err)
				os.Exit(1)
			}
		}

	},
}

func init() {
	usageFunc := addQueueCmd.UsageFunc()
	addQueueCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, addQueueExampleText)
		return nil
	})
	addQueueCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(addQueueCmd)
}
