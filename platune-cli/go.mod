module github.com/aschey/platune/cli/v2

go 1.16

require (
	github.com/aschey/go-prompt v0.2.7-0.20211004044345-9f70ca2dd8a6
	github.com/aschey/platune/client v0.0.0
	github.com/atotto/clipboard v0.1.4 // indirect
	github.com/charmbracelet/bubbles v0.9.0
	github.com/charmbracelet/bubbletea v0.19.1
	github.com/charmbracelet/lipgloss v0.4.0
	github.com/containerd/console v1.0.3 // indirect
	github.com/golang/mock v1.5.0
	github.com/mattn/go-colorable v0.1.11 // indirect
	github.com/muesli/ansi v0.0.0-20211031195517-c9f0611b6c70 // indirect
	github.com/nathan-fiscaletti/consolesize-go v0.0.0-20210105204122-a87d9f614b9d
	github.com/spf13/cobra v1.2.1
	github.com/spf13/pflag v1.0.5
	github.com/superhawk610/bar v0.0.2
	github.com/superhawk610/terminal v0.1.0 // indirect
	golang.org/x/net v0.0.0-20211105192438-b53810dc28af // indirect
	golang.org/x/sys v0.0.0-20211106132015-ebca88c72f68 // indirect
	golang.org/x/term v0.0.0-20210927222741-03fcf44c2211 // indirect
	golang.org/x/text v0.3.7 // indirect
	google.golang.org/genproto v0.0.0-20211104193956-4c6863e31247 // indirect
	google.golang.org/grpc v1.42.0
	google.golang.org/protobuf v1.27.1
)

replace github.com/aschey/platune/client => ../platuned/client/go
