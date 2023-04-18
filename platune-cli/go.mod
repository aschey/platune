module github.com/aschey/platune/cli/v2

go 1.16

require (
	github.com/MarvinJWendt/testza v0.2.12
	github.com/Microsoft/go-winio v0.6.1 // indirect
	github.com/aschey/go-prompt v0.2.7-0.20211219014443-47e06fafa70b
	github.com/aschey/platune/client v0.0.0
	github.com/aymanbagabas/go-osc52 v1.2.2 // indirect
	github.com/charmbracelet/bubbles v0.15.0
	github.com/charmbracelet/bubbletea v0.23.2
	github.com/charmbracelet/lipgloss v0.7.1
	github.com/golang/mock v1.6.0
	github.com/mattn/go-colorable v0.1.13 // indirect
	github.com/mattn/go-isatty v0.0.18 // indirect
	github.com/mattn/go-tty v0.0.4 // indirect
	github.com/muesli/ansi v0.0.0-20230316100256-276c6243b2f6 // indirect
	github.com/nathan-fiscaletti/consolesize-go v0.0.0-20220204101620-317176b6684d
	github.com/rivo/uniseg v0.4.4 // indirect
	github.com/spf13/cobra v1.7.0
	github.com/spf13/pflag v1.0.5
	github.com/superhawk610/bar v0.0.2
	github.com/superhawk610/terminal v0.1.0 // indirect
	go.uber.org/atomic v1.10.0 // indirect
	go.uber.org/fx v1.19.2
	go.uber.org/multierr v1.11.0 // indirect
	go.uber.org/zap v1.24.0
	golang.org/x/tools v0.8.0 // indirect
	google.golang.org/genproto v0.0.0-20230410155749-daa745c078e1 // indirect
	google.golang.org/grpc v1.54.0
	google.golang.org/protobuf v1.30.0
)

replace github.com/aschey/platune/client => ../platuned/client/go
