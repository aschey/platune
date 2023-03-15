//go:build wireinject
// +build wireinject

package mount

import (
	"github.com/aschey/platune/cli/internal"
	"github.com/google/wire"
	"github.com/spf13/cobra"
)

func InitializeMountCommand(
	managementClient *internal.ManagementClient,
) MountCmd {
	wire.Build(newMountCmd, newSetMountCmd, newGetMountCmd, wire.Struct(new(commands), "*"))
	return &cobra.Command{}
}
