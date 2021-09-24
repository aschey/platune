package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/spf13/cobra"
)

const AddQueueDescription = "Adds a song to the end of the queue"

const addQueueExampleText = "fileOrUrl"

var addQueueCmd = &cobra.Command{
	Use:   "add-queue " + addQueueExampleText,
	Short: AddQueueDescription,
	Long:  AddQueueDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		arg := args[0]
		_, err := os.Stat(arg)
		if err == nil {
			full, err := filepath.Abs(arg)
			if err != nil {
				fmt.Println(err)
			}
			internal.Client.AddToQueue([]string{full})
		} else {
			searchClient = internal.Client.Search()
			err := searchClient.Send(&platune.SearchRequest{Query: arg})
			if err != nil {
				fmt.Println(err)
			}
			results, err := searchClient.Recv()
			if err != nil {
				fmt.Println(err)
			}
			internal.RenderSearchResults(results)
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
