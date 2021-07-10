package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const PauseDescription = "Pauses the queue"

var pauseCmd = &cobra.Command{
	Use:   "pause",
	Short: PauseDescription,
	Long:  PauseDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		internal.Client.Pause()
	},
}

func init() {
	usageFunc := pauseCmd.UsageFunc()
	pauseCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	pauseCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(pauseCmd)
}
