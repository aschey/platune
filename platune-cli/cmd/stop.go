package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const StopDescription = "Stops the queue. No effect if already stopped."

var stopCmd = &cobra.Command{
	Use:   "stop",
	Short: StopDescription,
	Long:  StopDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		internal.Client.Stop()
	},
}

func init() {
	usageFunc := stopCmd.UsageFunc()
	stopCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	stopCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(stopCmd)
}
