package cmd

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	"github.com/aschey/platune/cli/v2/internal/search"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"go.uber.org/fx"
	"go.uber.org/fx/fxevent"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
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

func newRootCmd() *cobra.Command {
	rootCmd := &cobra.Command{
		Use:  "platune-cli",
		Long: title,

		RunE: func(cmd *cobra.Command, args []string) error {
			state := GetState(cmd)
			interactive, err := cmd.Flags().GetBool("interactive")
			if err != nil {
				return err
			}

			if interactive {
				exitCode := state.RunInteractive()
				if exitCode != 0 {
					return fmt.Errorf("Prompt exited with code %d", exitCode)
				} else {
					return nil
				}
			}

			return cmd.Help()
		},
	}

	usageFunc := rootCmd.UsageFunc()
	rootCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})

	rootCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	rootCmd.Flags().BoolP("interactive", "i", false, "Run in interactive mode")

	rootCmd.AddCommand(newAddFolderCmd())
	rootCmd.AddCommand(newAddQueueCmd())
	rootCmd.AddCommand(newGetAllFoldersCmd())
	rootCmd.AddCommand(newNextCmd())
	rootCmd.AddCommand(newPauseCmd())
	rootCmd.AddCommand(newPreviousCmd())
	rootCmd.AddCommand(newResumeCmd())
	rootCmd.AddCommand(newSeekCmd())
	rootCmd.AddCommand(newSetMountCmd())
	rootCmd.AddCommand(newSetQueueCmd())
	rootCmd.AddCommand(newSetVolumeCmd())
	rootCmd.AddCommand(newStopCmd())
	rootCmd.AddCommand(newSyncCmd())

	return rootCmd
}

func handleExit() error {
	if runtime.GOOS == "windows" {
		return nil
	}
	rawModeOff := exec.Command("/bin/stty", "-raw", "echo")
	rawModeOff.Stdin = os.Stdin
	err := rawModeOff.Run()
	return err
}

func start(rootCmd *cobra.Command, ctx context.Context, client *internal.PlatuneClient,
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
				rootCmd := newRootCmd()
				return start(rootCmd, ctx, client, state, deleted, search)
			},
			OnStop: func(context.Context) error {
				return handleExit()
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

	// Workaround for Windows support
	// see https://github.com/uber-go/zap/issues/994
	// TODO: update this when Windows support is fixed
	enc := zapcore.NewJSONEncoder(zap.NewProductionEncoderConfig())
	ws, err := os.OpenFile(fullpath, os.O_WRONLY|os.O_APPEND|os.O_CREATE, 0666)
	if err != nil {
		return nil, err
	}

	core := zapcore.NewCore(enc, ws, zapcore.InfoLevel)
	logger := zap.New(core)

	return logger, nil
}

func Execute() {
	app := fx.New(fx.Invoke(register),
		fx.Provide(NewLogger),
		fx.Provide(NewState),
		fx.Provide(internal.NewPlatuneClient),
		fx.Provide(search.NewSearch),
		fx.Provide(deleted.NewDeleted),
		fx.Provide(internal.NewStatusChan),
		fx.WithLogger(func(logger *zap.Logger) fxevent.Logger {
			return &fxevent.ZapLogger{Logger: logger}
		}),
	)

	ctx := context.Background()

	cobra.CheckErr(app.Start(ctx))
	cobra.CheckErr(app.Stop(ctx))

}
