//go:build wireinject
// +build wireinject

package folder

import (
	"github.com/aschey/platune/cli/internal"
	"github.com/google/wire"
	"github.com/spf13/cobra"
)

type commands struct {
	add  addFolderCmd
	list listFoldersCmd
}

func InitializeFolderCommand(playerClient *internal.PlayerClient, managementClient *internal.ManagementClient) FolderCmd {
	wire.Build(newAddFolderCmd, newFolderCmd, newListFoldersCmd, wire.Struct(new(commands), "*"))
	return &cobra.Command{}
}
