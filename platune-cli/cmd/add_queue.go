package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	management_v1 "github.com/aschey/platune/client/management_v1"
	"github.com/spf13/cobra"
)

const addQueueDescription = "Adds a song to the end of the queue"
const addQueueCmdText = "add-queue"
const addQueueExampleText = "<song, artist, album, file path, or url>"

func newAddQueueCmd() *cobra.Command {
	addQueueCmd := &cobra.Command{
		Use:   fmt.Sprintf("%s %s", addQueueCmdText, addQueueExampleText),
		Short: addQueueDescription,
		Long:  addQueueDescription,

		Args: cobra.MinimumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			client := GetClient(cmd)
			search := GetSearch(cmd)

			search.ProcessSearchResults(args,
				func(file string) { client.AddToQueue([]string{file}, true) },
				func(entries []*management_v1.LookupEntry) { client.AddSearchResultsToQueue(entries, false) })
		},
	}

	usageFunc := addQueueCmd.UsageFunc()
	addQueueCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, addQueueExampleText)
		return nil
	})
	addQueueCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	return addQueueCmd
}
