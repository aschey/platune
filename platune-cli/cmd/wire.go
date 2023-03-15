//go:build wireinject
// +build wireinject

package cmd

import (
	"github.com/aschey/platune/cli/cmd/folder"
	"github.com/aschey/platune/cli/cmd/mount"
	"github.com/aschey/platune/cli/cmd/queue"
	"github.com/aschey/platune/cli/internal"
	"github.com/google/wire"
)

func InitializeCommands() (commands, error) {
	wire.Build(
		internal.NewPlayerClient,
		internal.NewManagementClient,
		newPauseCmd,
		newResumeCmd,
		folder.InitializeFolderCommand,
		queue.InitializeQueueCommand,
		mount.InitializeMountCommand,
		wire.Struct(new(commands), "*"),
	)
	return commands{}, nil
}
