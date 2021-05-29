module github.com/aschey/platune/cli/v2

go 1.16

require (
	github.com/aschey/platune/client v0.0.0
	github.com/c-bata/go-prompt v0.2.6
	github.com/charmbracelet/lipgloss v0.2.1
	github.com/mitchellh/go-homedir v1.1.0
	github.com/spf13/cobra v1.1.3
	github.com/spf13/pflag v1.0.5
	github.com/spf13/viper v1.7.1
	google.golang.org/grpc v1.38.0
	google.golang.org/protobuf v1.26.0
)

replace github.com/aschey/platune/client => ../platuned/client/go
