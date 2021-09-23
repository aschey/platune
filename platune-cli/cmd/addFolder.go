package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const AddFolderDescription = "Adds a folder to the library"

const addFolderExampleText = "path"

var addFolderCmd = &cobra.Command{
	Use:   "add-folder " + addFolderExampleText,
	Short: AddFolderDescription,
	Long:  AddFolderDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		internal.Client.AddFolder(args[0])
	},
}

func init() {
	usageFunc := addFolderCmd.UsageFunc()
	addFolderCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, addFolderExampleText)
		return nil
	})
	addFolderCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(addFolderCmd)
}
