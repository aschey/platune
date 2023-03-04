module github.com/aschey/platune/cli/v2

go 1.16

require (
	github.com/MarvinJWendt/testza v0.2.12
	github.com/aschey/go-prompt v0.2.7-0.20211219014443-47e06fafa70b
	github.com/aschey/platune/client v0.0.0
	github.com/aymanbagabas/go-osc52 v1.2.2 // indirect
	github.com/charmbracelet/bubbles v0.15.0
	github.com/charmbracelet/bubbletea v0.23.2
	github.com/charmbracelet/lipgloss v0.6.0
	github.com/golang/mock v1.6.0
	github.com/inconshreveable/mousetrap v1.1.0 // indirect
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-tty v0.0.4 // indirect
	github.com/muesli/ansi v0.0.0-20221106050444-61f0cd9a192a // indirect
	github.com/nathan-fiscaletti/consolesize-go v0.0.0-20220204101620-317176b6684d
	github.com/rivo/uniseg v0.4.4 // indirect
	github.com/spf13/cobra v1.6.1
	github.com/spf13/pflag v1.0.5
	github.com/superhawk610/bar v0.0.2
	github.com/superhawk610/terminal v0.1.0 // indirect
	go.uber.org/atomic v1.10.0 // indirect
	go.uber.org/fx v1.19.2
	go.uber.org/multierr v1.9.0 // indirect
	go.uber.org/zap v1.24.0
	golang.org/x/tools v0.6.0 // indirect
	google.golang.org/grpc v1.53.0
	google.golang.org/protobuf v1.28.1
)

replace github.com/aschey/platune/client => ../platuned/client/go
