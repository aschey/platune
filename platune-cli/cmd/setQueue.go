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

const setQueueDescription = "Sets the queue and starts playback. Resets the queue if playback has already started."
const setQueueCmdText = "set-queue"
const setQueueExampleText = "<file> ..."

var setQueueCmd = &cobra.Command{
	Use:   fmt.Sprintf("%s %s", setQueueCmdText, setQueueExampleText),
	Short: setQueueDescription,
	Long:  setQueueDescription,

	Args: cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		allArgs := strings.Join(args, " ")
		_, err := os.Stat(allArgs)
		if err == nil {
			full, err := filepath.Abs(allArgs)
			if err != nil {
				fmt.Println(err)
			}
			internal.Client.SetQueue([]string{full})
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
			internal.RenderSearchResults(results, func(entries []*platune.LookupEntry) { internal.Client.SetQueueFromSearchResults(entries) })
		}

	},
}

func init() {
	usageFunc := setQueueCmd.UsageFunc()
	setQueueCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, setQueueExampleText)
		return nil
	})
	setQueueCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(setQueueCmd)
}
