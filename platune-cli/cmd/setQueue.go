package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const SetQueueDescription = "Sets the queue and starts playback. Resets the queue if playback has already started."
const setQueueExampleText = "fileOrUrl1 fileOrUrl2 fileOrUrl3..."

var setQueueCmd = &cobra.Command{
	Use:   "setQueue " + setQueueExampleText,
	Short: SetQueueDescription,
	Long:  SetQueueDescription,

	Args: cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.SetQueue(args)
	},
}

func init() {
	usageFunc := addQueueCmd.UsageFunc()
	setQueueCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, setQueueExampleText)
		return nil
	})
	setQueueCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(setQueueCmd)
}
