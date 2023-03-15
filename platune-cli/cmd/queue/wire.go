//go:build wireinject
// +build wireinject

package queue

import (
	"github.com/aschey/platune/cli/internal"
	"github.com/google/wire"
	"github.com/spf13/cobra"
)

func InitializeQueueCommand(
	playerClient *internal.PlayerClient,
	managementClient *internal.ManagementClient,
) QueueCmd {
	wire.Build(newAddQueueCmd, newQueueCmd, internal.NewSearch, wire.Struct(new(commands), "*"))
	return &cobra.Command{}
}
