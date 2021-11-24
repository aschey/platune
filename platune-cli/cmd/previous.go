package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const previousDescription = "Skips to the previous track"
const previousCmdText = "previous"

var previousCmd = &cobra.Command{
	Use:   previousCmdText,
	Short: previousDescription,
	Long:  previousDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		client := GetClient(cmd)
		client.Previous()
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
