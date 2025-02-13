module github.com/aschey/platune/cli/v2

go 1.23

require (
	github.com/MarvinJWendt/testza v0.2.12
	github.com/aschey/go-prompt v0.2.7-0.20211219014443-47e06fafa70b
	github.com/aschey/platune/client v0.0.0
	github.com/charmbracelet/bubbles v0.20.0
	github.com/charmbracelet/bubbletea v1.3.3
	github.com/charmbracelet/lipgloss v1.0.0
	github.com/golang/mock v1.6.0
	github.com/nathan-fiscaletti/consolesize-go v0.0.0-20220204101620-317176b6684d
	github.com/spf13/cobra v1.8.1
	github.com/spf13/pflag v1.0.6
	github.com/superhawk610/bar v0.0.2
	go.uber.org/fx v1.23.0
	go.uber.org/zap v1.27.0
	google.golang.org/grpc v1.70.0
	google.golang.org/protobuf v1.36.5
)

require (
	github.com/Microsoft/go-winio v0.6.2 // indirect
	github.com/atomicgo/cursor v0.0.1 // indirect
	github.com/atotto/clipboard v0.1.4 // indirect
	github.com/aymanbagabas/go-osc52/v2 v2.0.1 // indirect
	github.com/charmbracelet/x/ansi v0.8.0 // indirect
	github.com/charmbracelet/x/term v0.2.1 // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/erikgeiser/coninput v0.0.0-20211004153227-1c3628e74d0f // indirect
	github.com/gookit/color v1.4.2 // indirect
	github.com/inconshreveable/mousetrap v1.1.0 // indirect
	github.com/klauspost/cpuid/v2 v2.0.9 // indirect
	github.com/lucasb-eyer/go-colorful v1.2.0 // indirect
	github.com/mattn/go-colorable v0.1.14 // indirect
	github.com/mattn/go-isatty v0.0.20 // indirect
	github.com/mattn/go-localereader v0.0.1 // indirect
	github.com/mattn/go-runewidth v0.0.16 // indirect
	github.com/mattn/go-tty v0.0.7 // indirect
	github.com/muesli/ansi v0.0.0-20230316100256-276c6243b2f6 // indirect
	github.com/muesli/cancelreader v0.2.2 // indirect
	github.com/muesli/termenv v0.15.2 // indirect
	github.com/pkg/term v1.2.0-beta.2 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/pterm/pterm v0.12.33 // indirect
	github.com/rivo/uniseg v0.4.7 // indirect
	github.com/sahilm/fuzzy v0.1.1 // indirect
	github.com/superhawk610/terminal v0.1.0 // indirect
	github.com/xo/terminfo v0.0.0-20210125001918-ca9a967f8778 // indirect
	go.uber.org/dig v1.18.0 // indirect
	go.uber.org/multierr v1.11.0 // indirect
	golang.org/x/net v0.35.0 // indirect
	golang.org/x/sync v0.11.0 // indirect
	golang.org/x/sys v0.30.0 // indirect
	golang.org/x/term v0.29.0 // indirect
	golang.org/x/text v0.22.0 // indirect
	google.golang.org/genproto/googleapis/rpc v0.0.0-20250212204824-5a70512c5d8b // indirect
)

replace github.com/aschey/platune/client => ../platuned/client/go
