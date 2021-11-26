package cmd

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	"github.com/aschey/platune/cli/v2/internal/search"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"go.uber.org/fx"
	"go.uber.org/fx/fxevent"
	"go.uber.org/zap"
)

var title1 = "█▀█ █░░ ▄▀█ ▀█▀ █░█ █▄░█ █▀▀   █▀▀ █░░ █"
var title2 = "█▀▀ █▄▄ █▀█ ░█░ █▄█ █░▀█ ██▄   █▄▄ █▄▄ █"

var title = lipgloss.NewStyle().
	Foreground(lipgloss.Color("9")).
	BorderStyle(lipgloss.RoundedBorder()).
	BorderForeground(lipgloss.Color("6")).
	PaddingLeft(1).
	PaddingRight(1).
	Render(title1 + "\n" + title2)

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:  "platune-cli",
	Long: title,

	Run: func(cmd *cobra.Command, args []string) {
		state := GetState(cmd)
		interactive, err := cmd.Flags().GetBool("interactive")
		if err != nil {
			fmt.Println(err)
			return
		}
		if interactive {
			state.curPrompt.Run()
		} else {
			err := cmd.Help()
			if err != nil {
				fmt.Println(err)
				return
			}
		}

	},
}

func handleExit() {
	rawModeOff := exec.Command("/bin/stty", "-raw", "echo")
	rawModeOff.Stdin = os.Stdin
	err := rawModeOff.Run()
	if err != nil {
		fmt.Println(err)
	}
}

func start(ctx context.Context, client *internal.PlatuneClient,
	state *cmdState, deleted *deleted.Deleted, search *search.Search) error {
	ctx = RegisterClient(ctx, client)
	ctx = RegisterState(ctx, state)
	ctx = RegisterDeleted(ctx, deleted)
	ctx = RegisterSearch(ctx, search)

	return rootCmd.ExecuteContext(ctx)
}

func register(lifecycle fx.Lifecycle, client *internal.PlatuneClient,
	state *cmdState, deleted *deleted.Deleted, search *search.Search) {
	lifecycle.Append(
		fx.Hook{
			OnStart: func(ctx context.Context) error {
				return start(ctx, client, state, deleted, search)
			},
			OnStop: func(context.Context) error {
				handleExit()
				return nil
			},
		},
	)
}

func NewLogger() (*zap.Logger, error) {
	dir, err := os.Executable()
	if err != nil {
		panic(err)
	}

	fullpath := filepath.Join(filepath.Dir(dir), "platune-cli.log")
	cfg := zap.NewProductionConfig()
	cfg.OutputPaths = []string{
		fullpath,
	}

	return cfg.Build()
}

func Execute() {
	app := fx.New(fx.Invoke(register),
		fx.Provide(NewLogger),
		fx.Provide(NewState),
		fx.Provide(internal.NewPlatuneClient),
		fx.Provide(search.NewSearch),
		fx.Provide(deleted.NewDeleted),
		fx.WithLogger(func(logger *zap.Logger) fxevent.Logger {
			return &fxevent.ZapLogger{Logger: logger}
		}),
	)

	ctx := context.Background()

	app.Start(ctx)
	app.Stop(ctx)

}

func init() {
	usageFunc := rootCmd.UsageFunc()
	rootCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})

	rootCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	rootCmd.Flags().BoolP("interactive", "i", false, "Run in interactive mode")
}
