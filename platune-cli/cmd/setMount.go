package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const SetMountDescription = "Set the mount point for the library"

const setMountExampleText = "path"

var setMountCmd = &cobra.Command{
	Use:   "set-mount " + setMountExampleText,
	Short: SetMountDescription,
	Long:  SetMountDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		internal.Client.SetMount(args[0])
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
