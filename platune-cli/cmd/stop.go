package cmd

import (
	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type stopCmd *cobra.Command

func newStopCmd(client *internal.PlayerClient) stopCmd {
	pauseCmd := &cobra.Command{
		Use:   "stop",
		Short: "Stops the player",
		Args:  cobra.NoArgs,

		RunE: func(cmd *cobra.Command, args []string) error {
			if err := client.Stop(); err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, internal.NewInfoModel("Stopped"))
		},
	}

	return pauseCmd
}
