//go:build wireinject
// +build wireinject

package cmd

import (
	"github.com/aschey/platune/cli/internal"
	"github.com/google/wire"
)

type commands struct {
	pause  pauseCmd
	resume resumeCmd
}

func InitializeCommands() commands {
	wire.Build(internal.NewPlatuneClient, newPauseCmd, newResumeCmd, wire.Struct(new(commands), "*"))
	return commands{}
}
