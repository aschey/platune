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
		ctx := cmd.Context()
		state := ctx.Value(State).(*cmdState)
		interactive, err := cmd.Flags().GetBool("interactive")
		if err != nil {
			fmt.Println(err)
			return
		}
		if interactive {
			state.curPrompt.Run()
			handleExit()
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

type Dependency int

const (
	Logger Dependency = iota
	Client
	State
	Deleted
	Search
)

func register(lifecycle fx.Lifecycle, logger *zap.Logger, client *internal.PlatuneClient,
	state *cmdState, deleted *deleted.Deleted, search *search.Search) {
	lifecycle.Append(
		fx.Hook{
			OnStart: func(ctx context.Context) error {
				cmdCtx := context.WithValue(ctx, Logger, logger)
				cmdCtx = context.WithValue(cmdCtx, Client, client)
				cmdCtx = context.WithValue(cmdCtx, State, state)
				cmdCtx = context.WithValue(cmdCtx, Deleted, deleted)
				cmdCtx = context.WithValue(cmdCtx, Search, search)
				cobra.CheckErr(rootCmd.ExecuteContext(cmdCtx))
				return nil
			},
			OnStop: func(context.Context) error {
				return nil
			},
		},
	)
}

func NewLogger() *zap.Logger {
	dir, err := os.Executable()
	if err != nil {
		panic(err)
	}
	fullpath := filepath.Join(filepath.Dir(dir), "platune-cli.log")
	cfg := zap.NewProductionConfig()
	cfg.OutputPaths = []string{
		fullpath,
	}
	logger, _ := cfg.Build()
	return logger
}

func Execute() {
	app := fx.New(fx.Invoke(register),
		fx.Provide(NewLogger),
		fx.Provide(NewState),
		fx.Provide(internal.NewPlatuneClient),
		fx.Provide(internal.NewSearchClient),
		fx.Provide(search.NewSearch),
		fx.Provide(deleted.NewDeleted),
		// fx.WithLogger(func(logger *zap.Logger) fxevent.Logger {
		// 	return &fxevent.ZapLogger{Logger: logger}
		// }),
	)

	stopCtx := context.Background()
	app.Start(stopCtx)
	//cobra.CheckErr(rootCmd.Execute())
}

func init() {
	//initState()

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
