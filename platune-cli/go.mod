module github.com/aschey/platune/cli/v2

go 1.16

require (
	github.com/aschey/go-prompt v0.2.7-0.20211107192052-857b38ecf016
	github.com/aschey/platune/client v0.0.0
	github.com/atotto/clipboard v0.1.4 // indirect
	github.com/charmbracelet/bubbles v0.9.0
	github.com/charmbracelet/bubbletea v0.19.1
	github.com/charmbracelet/lipgloss v0.4.0
	github.com/containerd/console v1.0.3 // indirect
	github.com/golang/mock v1.5.0
	github.com/mattn/go-colorable v0.1.12 // indirect
	github.com/muesli/ansi v0.0.0-20211031195517-c9f0611b6c70 // indirect
	github.com/nathan-fiscaletti/consolesize-go v0.0.0-20210105204122-a87d9f614b9d
	github.com/spf13/cobra v1.2.1
	github.com/spf13/pflag v1.0.5
	github.com/superhawk610/bar v0.0.2
	github.com/superhawk610/terminal v0.1.0 // indirect
	go.uber.org/atomic v1.9.0 // indirect
	go.uber.org/dig v1.13.0 // indirect
	go.uber.org/fx v1.15.0
	go.uber.org/multierr v1.7.0 // indirect
	go.uber.org/zap v1.19.1
	golang.org/x/net v0.0.0-20211123203042-d83791d6bcd9 // indirect
	golang.org/x/sys v0.0.0-20211124211545-fe61309f8881 // indirect
	golang.org/x/term v0.0.0-20210927222741-03fcf44c2211 // indirect
	golang.org/x/text v0.3.7 // indirect
	google.golang.org/genproto v0.0.0-20211118181313-81c1377c94b1 // indirect
	google.golang.org/grpc v1.42.0
	google.golang.org/protobuf v1.27.1
)

replace github.com/aschey/platune/client => ../platuned/client/go
