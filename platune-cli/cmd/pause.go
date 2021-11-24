package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const pauseDescription = "Pauses the queue"
const pauseCmdText = "pause"

var pauseCmd = &cobra.Command{
	Use:   pauseCmdText,
	Short: pauseDescription,
	Long:  pauseDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		client := GetClient(cmd)
		client.Pause()
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
