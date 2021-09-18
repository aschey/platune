module github.com/aschey/platune/cli/v2

go 1.16

require (
	github.com/aschey/go-prompt v0.2.7-0.20210914072712-fe453763407f
	github.com/aschey/platune/client v0.0.0
	github.com/charmbracelet/lipgloss v0.2.1
	github.com/golang/mock v1.5.0
	github.com/mitchellh/go-homedir v1.1.0
	github.com/nathan-fiscaletti/consolesize-go v0.0.0-20210105204122-a87d9f614b9d
	github.com/spf13/cobra v1.1.3
	github.com/spf13/pflag v1.0.5
	github.com/spf13/viper v1.7.1
	github.com/superhawk610/bar v0.0.2
	github.com/superhawk610/terminal v0.1.0 // indirect
	google.golang.org/grpc v1.38.0
	google.golang.org/protobuf v1.26.0
)

replace github.com/aschey/platune/client => ../platuned/client/go
