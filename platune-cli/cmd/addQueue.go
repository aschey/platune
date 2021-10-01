package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/spf13/cobra"
)

const addQueueDescription = "Adds a song to the end of the queue"
const addQueueCmdText = "add-queue"
const addQueueExampleText = "<file, url or db entry>"

var addQueueCmd = &cobra.Command{
	Use:   fmt.Sprintf("%s %s", addQueueCmdText, addQueueExampleText),
	Short: addQueueDescription,
	Long:  addQueueDescription,

	Args: cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		allArgs := strings.Join(args, " ")
		_, err := os.Stat(allArgs)
		if err == nil {
			full, err := filepath.Abs(allArgs)
			if err != nil {
				fmt.Println(err)
			}
			internal.Client.AddToQueue([]string{full}, true)
		} else {
			searchClient = internal.Client.Search()
			err := searchClient.Send(&platune.SearchRequest{Query: allArgs})
			if err != nil {
				fmt.Println(err)
			}
			results, err := searchClient.Recv()
			if err != nil {
				fmt.Println(err)
			}
			internal.RenderSearchResults(results, func(entries []*platune.LookupEntry) { internal.Client.AddSearchResultsToQueue(entries, false) })
		}

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
