package cmd

import (
	"fmt"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const setMountDescription = "Set the mount point for the library"
const setMountCmdText = "set-mount"
const setMountExampleText = "<path>"

var setMountCmd = &cobra.Command{
	Use:   fmt.Sprintf("%s %s", setMountCmdText, setMountExampleText),
	Short: setMountDescription,
	Long:  setMountDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		client := GetClient(cmd)
		client.SetMount(args[0])
	},
}

func init() {
	usageFunc := setMountCmd.UsageFunc()
	setMountCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, setMountExampleText)
		return nil
	})
	setMountCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(setMountCmd)
}
