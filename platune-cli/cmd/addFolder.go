package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const AddFolderDescription = "Adds a folder to the library"

const addFolderExampleText = "path"

// setQueueCmd represents the setQueue command
var addFolderCmd = &cobra.Command{
	Use:   "add-folder " + addFolderExampleText,
	Short: AddFolderDescription,
	Long:  AddFolderDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.AddFolder(args[0])
	},
}

func init() {
	usageFunc := addFolderCmd.UsageFunc()
	addFolderCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, addFolderExampleText)
		return nil
	})
	addFolderCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(addFolderCmd)
}
