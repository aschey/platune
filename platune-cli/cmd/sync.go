package cmd

import (
	"fmt"
	"time"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"github.com/superhawk610/bar"
)

const syncDescription = "Syncs the database with the configured folders to import"
const syncCmdText = "sync"

func syncProgress(client *internal.PlatuneClient, deleted *deleted.Deleted) {
	b := bar.NewWithOpts(
		bar.WithDimensions(1000, 30),
		bar.WithFormat(
			fmt.Sprintf("Syncing... %s %s | %s",
				lipgloss.NewStyle().Foreground(lipgloss.Color("9")).Render(":bar"),
				lipgloss.NewStyle().Foreground(lipgloss.Color("6")).Render(":percent"),
				lipgloss.NewStyle().Foreground(lipgloss.Color("15")).Render(":elapsed"))))

	progressCh := make(chan *platune.Progress, 1)

	go client.SubscribeManagementEvents(progressCh)
	client.StartSync()

	start := time.Now()
	for progress := range progressCh {
		if progress.Job != "sync" {
			continue
		}

		b.Update(int(progress.Percentage*1000),
			bar.Context{bar.Ctx("elapsed", time.Since(start).Round(time.Millisecond*10).String())})
		if progress.Finished {
			break
		}
	}

	fmt.Println()
	deleted.RenderDeletedFiles()

}

func newSyncCmd() *cobra.Command {
	syncCmd := &cobra.Command{
		Use:   syncCmdText,
		Short: syncDescription,
		Long:  syncDescription,

		Args: cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			client := GetClient(cmd)
			deleted := GetDeleted(cmd)
			syncProgress(client, deleted)
		},
	}
	usageFunc := syncCmd.UsageFunc()
	syncCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	syncCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	return syncCmd
}
