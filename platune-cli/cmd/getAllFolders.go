package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const GetAllFoldersDescription = "Lists all configured folders"

var getAllFoldersCmd = &cobra.Command{
	Use:   "get-all-folders",
	Short: GetAllFoldersDescription,
	Long:  GetAllFoldersDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.GetAllFolders()
	},
}

func init() {
	usageFunc := getAllFoldersCmd.UsageFunc()
	getAllFoldersCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, "")
		return nil
	})
	getAllFoldersCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(getAllFoldersCmd)
}
