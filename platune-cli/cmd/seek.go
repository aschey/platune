package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const seekExampleText = "[hh:][mm:]ss"
const SeekDescription = "Seek to a specific time. Input should be formatted like " + seekExampleText

var seekCmd = &cobra.Command{
	Use:   "seek " + seekExampleText,
	Short: SeekDescription,
	Long:  SeekDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.Seek(args[0])
	},
}

func init() {
	usageFunc := seekCmd.UsageFunc()
	seekCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, seekExampleText)
		return nil
	})
	seekCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(seekCmd)
}
