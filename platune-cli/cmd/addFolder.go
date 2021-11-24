package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const addFolderDescription = "Adds a folder to the library"
const addFolderCmdText = "add-folder"
const addFolderExampleText = "<path>"

var addFolderCmd = &cobra.Command{
	Use:   fmt.Sprintf("%s %s", addFolderCmdText, addFolderExampleText),
	Short: addFolderDescription,
	Long:  addFolderDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		ctx := cmd.Context()
		client := ctx.Value(Client).(*internal.PlatuneClient)
		client.AddFolder(args[0])
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
