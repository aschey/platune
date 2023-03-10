package cmd

import (
	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type pauseCmd *cobra.Command

func newPauseCmd(client *internal.PlayerClient) pauseCmd {
	pauseCmd := &cobra.Command{
		Use:   "pause",
		Short: "Pauses the player",
		Args:  cobra.NoArgs,

		RunE: func(cmd *cobra.Command, args []string) error {
			if err := client.Pause(); err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, internal.NewInfoModel("Paused"))
		},
	}

	return pauseCmd
}
