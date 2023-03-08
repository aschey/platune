package cmd

import (
	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/bubbleprompt/executor"
	"github.com/aschey/platune/cli/internal"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

type pauseCmd *cobra.Command

func newPauseCmd(client *internal.PlatuneClient) pauseCmd {
	pauseCmd := &cobra.Command{
		Use:   "pause",
		Short: "Pauses the player",
		Args:  cobra.NoArgs,

		RunE: func(cmd *cobra.Command, args []string) error {
			if err := client.Pause(); err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, executor.NewStringModel(lipgloss.NewStyle().Foreground(lipgloss.Color("245")).Render("Paused")))
		},
	}

	return pauseCmd
}
