package cmd

import (
	"fmt"
	"os"

	prompt "github.com/aschey/bubbleprompt"
	cprompt "github.com/aschey/bubbleprompt-cobra"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

var (
	title1      = "█▀█ █░░ ▄▀█ ▀█▀ █░█ █▄░█ █▀▀   █▀▀ █░░ █"
	title2      = "█▀▀ █▄▄ █▀█ ░█░ █▄█ █░▀█ ██▄   █▄▄ █▄▄ █"
	description = "CLI for the Platune audio server"
)

var title = lipgloss.NewStyle().
	Foreground(lipgloss.Color("9")).
	BorderStyle(lipgloss.RoundedBorder()).
	BorderForeground(lipgloss.Color("6")).
	PaddingLeft(1).
	PaddingRight(1).
	Render(fmt.Sprintf("%s\n%s", title1, title2)) + "\n" + description

func Execute() {
	rootCmd := &cobra.Command{
		Use:   "platune-cli",
		Short: description,
		Long:  title,
		RunE: func(cmd *cobra.Command, args []string) error {
			interactive, err := cmd.Flags().GetBool("interactive")
			if err != nil {
				return err
			}
			if interactive {
				promptModel := cprompt.NewPrompt(cmd)
				model := model{inner: promptModel}
				_, err := tea.NewProgram(&model, tea.WithFilter(prompt.MsgFilter)).Run()
				return err
			}

			return nil
		},
	}

	commands := InitializeCommands()

	rootCmd.AddCommand(commands.pause)
	rootCmd.AddCommand(commands.resume)

	rootCmd.Flags().BoolP("interactive", "i", false, "Run in interactive mode")

	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

type model struct {
	inner cprompt.Model
}

func (m model) Init() tea.Cmd {
	return m.inner.Init()
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	model, cmd := m.inner.Update(msg)
	m.inner = model.(cprompt.Model)
	return m, cmd
}

func (m model) View() string {
	return m.inner.View()
}
