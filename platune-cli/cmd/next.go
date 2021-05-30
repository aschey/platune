package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const NextDescription = "Skips to the next track"

var nextCmd = &cobra.Command{
	Use:   "next",
	Short: NextDescription,
	Long:  NextDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.Next()
	},
}

func init() {
	usageFunc := nextCmd.UsageFunc()
	nextCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, "")
		return nil
	})
	nextCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(nextCmd)
}
