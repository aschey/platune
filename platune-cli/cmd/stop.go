package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const stopDescription = "Stops the queue. No effect if already stopped."
const stopCmdText = "stop"

var stopCmd = &cobra.Command{
	Use:   stopCmdText,
	Short: stopDescription,
	Long:  stopDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		ctx := cmd.Context()
		client := ctx.Value(Client).(*internal.PlatuneClient)
		client.Stop()
	},
}

func init() {
	usageFunc := stopCmd.UsageFunc()
	stopCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	stopCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(stopCmd)
}
