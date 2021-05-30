package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const StopDescription = "Stops the queue"

var stopCmd = &cobra.Command{
	Use:   "stop",
	Short: StopDescription,
	Long:  StopDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.AddToQueue(args[0])
	},
}

func init() {
	usageFunc := stopCmd.UsageFunc()
	stopCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, "")
		return nil
	})
	stopCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(stopCmd)
}
