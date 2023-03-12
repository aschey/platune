package mount

import "github.com/spf13/cobra"

type MountCmd *cobra.Command

type commands struct {
	get getMountCmd
	set setMountCmd
}

func newMountCmd(subcommands commands) MountCmd {
	rootCmd := &cobra.Command{
		Use:   "mount <command>",
		Short: "View and modify mount configuration",
		Args:  cobra.MinimumNArgs(1),
	}

	rootCmd.AddCommand(subcommands.get)
	rootCmd.AddCommand(subcommands.set)

	return rootCmd
}
