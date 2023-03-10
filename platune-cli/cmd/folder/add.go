package folder

import (
	"fmt"

	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type addFolderCmd *cobra.Command

func newAddFolderCmd(client *internal.ManagementClient) addFolderCmd {
	addFolderCmd := &cobra.Command{
		Use:   "add <folder>",
		Short: "Adds a music folder to be synced to the database",
		Args:  cobra.ExactArgs(1),

		RunE: func(cmd *cobra.Command, args []string) error {
			if err := client.AddFolder(args[0]); err != nil {
				return err
			}
			return cprompt.ExecModel(cmd, internal.NewInfoModel(fmt.Sprintf("Folder %s added", args[0])))
		},
	}

	return addFolderCmd
}
