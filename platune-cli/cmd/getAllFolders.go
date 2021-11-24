package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const getAllFoldersDescription = "Lists all configured folders"
const getAllFoldersCmdText = "get-all-folders"

var getAllFoldersCmd = &cobra.Command{
	Use:   getAllFoldersCmdText,
	Short: getAllFoldersDescription,
	Long:  getAllFoldersDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		client := GetClient(cmd)
		client.GetAllFolders()
	},
}

func init() {
	usageFunc := getAllFoldersCmd.UsageFunc()
	getAllFoldersCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	getAllFoldersCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(getAllFoldersCmd)
}
