package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const PreviousDescription = "Skips to the previous track"

var previousCmd = &cobra.Command{
	Use:   "previous",
	Short: PreviousDescription,
	Long:  PreviousDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		internal.Client.Previous()
	},
}

func init() {
	usageFunc := previousCmd.UsageFunc()
	previousCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	previousCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(previousCmd)
}
