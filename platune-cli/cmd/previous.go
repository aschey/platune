package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const PreviousDescription = "Skips to the previous track"

var previousCmd = &cobra.Command{
	Use:   "previous",
	Short: PreviousDescription,
	Long:  PreviousDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.Previous()
	},
}

func init() {
	usageFunc := previousCmd.UsageFunc()
	previousCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, "")
		return nil
	})
	previousCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(previousCmd)
}
