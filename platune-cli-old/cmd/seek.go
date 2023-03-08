package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const seekExampleText = "[hh:][mm:]ss"
const seekDescription = "Seek to a specific time"
const seekCmdText = "seek"

func newSeekCmd() *cobra.Command {
	seekCmd := &cobra.Command{
		Use:   fmt.Sprintf("%s %s", seekCmdText, seekExampleText),
		Short: seekDescription,
		Long:  seekDescription,

		Args: cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			client := GetClient(cmd)
			client.Seek(args[0])
		},
	}
	usageFunc := seekCmd.UsageFunc()
	seekCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, seekExampleText)
		return nil
	})
	seekCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	return seekCmd
}
