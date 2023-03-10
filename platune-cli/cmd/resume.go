package cmd

import (
	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type resumeCmd *cobra.Command

func newResumeCmd(client *internal.PlayerClient) resumeCmd {
	pauseCmd := &cobra.Command{
		Use:   "resume",
		Short: "Resumes the player",
		Args:  cobra.NoArgs,

		RunE: func(cmd *cobra.Command, args []string) error {
			if err := client.Resume(); err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, internal.NewInfoModel("Resumed"))
		},
	}

	return pauseCmd
}
