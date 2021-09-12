package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const AddQueueDescription = "Adds a song to the end of the queue"

const addQueueExampleText = "fileOrUrl"

// setQueueCmd represents the setQueue command
var addQueueCmd = &cobra.Command{
	Use:   "add-queue " + addQueueExampleText,
	Short: AddQueueDescription,
	Long:  AddQueueDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		internal.Client.AddToQueue(args)
	},
}

func init() {
	usageFunc := addQueueCmd.UsageFunc()
	addQueueCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, addQueueExampleText)
		return nil
	})
	addQueueCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(addQueueCmd)
}
