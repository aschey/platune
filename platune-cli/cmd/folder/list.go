package folder

import (
	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/bubbleprompt/executor"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type listFoldersCmd *cobra.Command

func newListFoldersCmd(client *internal.ManagementClient) listFoldersCmd {
	addFolderCmd := &cobra.Command{
		Use:   "list",
		Short: "Lists all configured folders",
		Args:  cobra.NoArgs,

		RunE: func(cmd *cobra.Command, args []string) error {
			folders, err := client.GetAllFolders()
			if err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, executor.NewStringModel(internal.PrettyPrintList(folders)))
		},
	}

	return addFolderCmd
}
