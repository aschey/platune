package cmd

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
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
