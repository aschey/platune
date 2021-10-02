package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/spf13/cobra"
)

const setQueueDescription = "Sets the queue and starts playback. Resets the queue if playback has already started."
const setQueueCmdText = "set-queue"
const setQueueExampleText = "<file, url, or db entry>"

var setQueueCmd = &cobra.Command{
	Use:   fmt.Sprintf("%s %s", setQueueCmdText, setQueueExampleText),
	Short: setQueueDescription,
	Long:  setQueueDescription,

	Args: cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		internal.ProcessSearchResults(args,
			func(file string) { internal.Client.SetQueue([]string{file}, true) },
			func(entries []*platune.LookupEntry) { internal.Client.SetQueueFromSearchResults(entries, false) })
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
