package mount

import (
	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type getMountCmd *cobra.Command

func newGetMountCmd(client *internal.ManagementClient) getMountCmd {
	getMountCmd := &cobra.Command{
		Use:   "get",
		Short: "Gets the mount point for the library",
		Args:  cobra.NoArgs,

		RunE: func(cmd *cobra.Command, args []string) error {
			mount, err := client.GetRegisteredMount()
			if err != nil {
				return err
			}
			return cprompt.ExecModel(
				cmd,
				internal.NewInfoModel(mount.Mount),
			)
		},
	}

	return getMountCmd
}
