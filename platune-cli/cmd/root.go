package cmd

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"strings"

	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/lipgloss"
	"github.com/mattn/go-runewidth"
	"github.com/nathan-fiscaletti/consolesize-go"
	"github.com/spf13/cobra"
)

var searchClient platune.Management_SearchClient

func expandPath(song string) (string, fs.FileInfo, error) {
	if strings.HasPrefix(song, "http") {
		return song, nil, nil
	}

	dir, base, err := internal.CleanFilePath(song)

	if err != nil {
		return "", nil, err
	}
	full := path.Join(dir, base)
	stat, err := os.Stat(full)

	return full, stat, err
}

func expandFile(song string) (string, error) {
	full, stat, err := expandPath(song)

	if stat != nil && stat.Mode().IsDir() {
		return "", fmt.Errorf("cannot add a directory")
	}
	return full, err
}

func expandFolder(song string) (string, error) {
	full, stat, err := expandPath(song)

	if stat != nil && !stat.Mode().IsDir() {
		return "", fmt.Errorf("cannot add a file")
	}
	return full, err
}

func getAvailableWidth(currentCol int) float32 {
	cols, _ := consolesize.GetConsoleSize()
	base := float32(cols-currentCol) - 10
	return base
}

func ellipsize(text string, max int) string {
	if max > 0 && runewidth.StringWidth(text) > max {
		return text[:max-3] + "..."
	}
	return text
}

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

	// Uncomment the following line if your bare application
	// has an action associated with it:
	Run: func(cmd *cobra.Command, args []string) {
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

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	cobra.CheckErr(rootCmd.Execute())
}

func init() {
	initState()

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
