package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	management_v1 "github.com/aschey/platune/client/management_v1"
	"github.com/spf13/cobra"
)

const setQueueDescription = "Sets the queue and starts playback. Resets the queue if playback has already started."
const setQueueCmdText = "set-queue"
const setQueueExampleText = "<song, artist, album, file path, or url>"

func newSetQueueCmd() *cobra.Command {
	setQueueCmd := &cobra.Command{
		Use:   fmt.Sprintf("%s %s", setQueueCmdText, setQueueExampleText),
		Short: setQueueDescription,
		Long:  setQueueDescription,

		Args: cobra.MinimumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			client := GetClient(cmd)
			search := GetSearch(cmd)
			search.ProcessSearchResults(args,
				func(file string) { client.SetQueue([]string{file}, true) },
				func(entries []*management_v1.LookupEntry) { client.SetQueueFromSearchResults(entries, false) })
		},
	}

	usageFunc := setQueueCmd.UsageFunc()
	setQueueCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, setQueueExampleText)
		return nil
	})
	setQueueCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	return setQueueCmd
}
