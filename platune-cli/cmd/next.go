package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const nextDescription = "Skips to the next track"
const nextCmdText = "next"

var nextCmd = &cobra.Command{
	Use:   nextCmdText,
	Short: nextDescription,
	Long:  nextDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		client := GetClient(cmd)
		client.Next()
	},
}

func init() {
	usageFunc := nextCmd.UsageFunc()
	nextCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	nextCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(nextCmd)
}
