package folder

import "github.com/spf13/cobra"

type FolderCmd *cobra.Command

type commands struct {
	add  addFolderCmd
	list listFoldersCmd
}

func newFolderCmd(subcommands commands) FolderCmd {
	rootCmd := &cobra.Command{
		Use:   "folder <command>",
		Short: "View and modify folder configuration",
		Args:  cobra.MinimumNArgs(1),
	}

	rootCmd.AddCommand(subcommands.add)
	rootCmd.AddCommand(subcommands.list)

	return rootCmd
}
