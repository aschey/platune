package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/spf13/cobra"
)

const addQueueDescription = "Adds a song to the end of the queue"
const addQueueCmdText = "add-queue"
const addQueueExampleText = "<file, url, or db entry>"

var addQueueCmd = &cobra.Command{
	Use:   fmt.Sprintf("%s %s", addQueueCmdText, addQueueExampleText),
	Short: addQueueDescription,
	Long:  addQueueDescription,

	Args: cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		client := GetClient(cmd)
		search := GetSearch(cmd)
		search.ProcessSearchResults(args,
			func(file string) { client.AddToQueue([]string{file}, true) },
			func(entries []*platune.LookupEntry) { client.AddSearchResultsToQueue(entries, false) })
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
