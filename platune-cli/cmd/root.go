package cmd

import (
	"fmt"
	"os"
	"path"
	"strings"

	"github.com/aschey/platune/cli/v2/utils"
	"github.com/c-bata/go-prompt"
	"github.com/c-bata/go-prompt/completer"
	"github.com/charmbracelet/lipgloss"
	"github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var cfgFile string

type cmdState struct {
	livePrefix     string
	isEnabled      bool
	isSetQueueMode bool
	currentQueue   []string
}

func expandPath(song string) (string, error) {
	if strings.HasPrefix(song, "http") {
		return song, nil
	}

	dir, base, err := utils.CleanFilePath(song)

	if err != nil {
		return "", err
	}
	full := path.Join(dir, base)
	stat, err := os.Stat(full)
	if err != nil {

		return "", err
	}
	if stat.Mode().IsDir() {
		return "", fmt.Errorf("cannot add a directory")
	}
	return full, nil

}

func newCmdState() cmdState {
	return cmdState{livePrefix: "", isEnabled: false, isSetQueueMode: false, currentQueue: []string{}}
}

var state = newCmdState()

func (state *cmdState) changeLivePrefix() (string, bool) {
	return state.livePrefix, state.isEnabled
}

func (state *cmdState) executor(in string) {
	cmds := strings.SplitN(in, " ", 2)
	if len(cmds) == 0 {
		return
	}

	switch cmds[0] {
	case "set-queue":
		fmt.Println("Enter file paths or urls to add to the queue.")
		fmt.Println("Enter a blank line when done.")
		state.isSetQueueMode = true
		state.livePrefix = in + "> "
		state.isEnabled = true
		return
	case "add-queue":
		if len(cmds) < 2 {
			fmt.Println("Usage: add-queue <path or url>")
			return
		}
		full, err := expandPath(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		utils.Client.AddToQueue(full)
	case "seek":
		if len(cmds) < 2 {
			fmt.Println("Usage: seek [hh]:[mm]:ss")
			return
		}
		utils.Client.Seek(cmds[1])
	case "pause":
		utils.Client.Pause()
	case "resume":
		utils.Client.Resume()
	case "stop":
		utils.Client.Stop()
	case "next":
		utils.Client.Next()
	case "previous":
		utils.Client.Previous()
	case "q":
		fmt.Println("Exiting...")
		os.Exit(0)
	}
	if state.isSetQueueMode {
		if strings.Trim(in, " ") == "" {
			utils.Client.SetQueue(state.currentQueue)
			state.isSetQueueMode = false
			state.currentQueue = []string{}
			state.isEnabled = false
		} else {
			in, err := expandPath(in)
			if err != nil {
				fmt.Println(err)
				return
			}

			state.currentQueue = append(state.currentQueue, in)
			var formattedQueue = []string{}
			for i := 0; i < len(state.currentQueue); i++ {
				formattedQueue = append(formattedQueue, fmt.Sprintf("%d. %s", i+1, state.currentQueue[i]))
			}
			fmt.Println(strings.Join(formattedQueue, "\n"))
		}
	}
}

func (state *cmdState) completer(in prompt.Document) []prompt.Suggest {
	before := strings.Split(in.TextBeforeCursor(), " ")
	if state.isSetQueueMode {
		return filePathCompleter.Complete(in, false)
	}
	if len(before) > 1 {
		if before[0] == "add-queue" {
			return filePathCompleter.Complete(in, true)
		}
		return []prompt.Suggest{}
	}

	s := []prompt.Suggest{
		{Text: "set-queue", Description: SetQueueDescription},
		{Text: "add-queue", Description: AddQueueDescription},
		{Text: "pause", Description: PauseDescription},
		{Text: "resume", Description: ResumeDescription},
		{Text: "seek", Description: "Seek to a specific time. Input should be formatted like [hh]:[mm]:ss"},
		{Text: "next", Description: "Skips to the next track"},
		{Text: "previous", Description: "Skips to the previous track"},
		{Text: "stop", Description: StopDescription},
		{Text: "q", Description: "Quit interactive prompt"},
	}
	return prompt.FilterHasPrefix(s, in.GetWordBeforeCursor(), true)
}

var filePathCompleter = utils.FilePathCompleter{
	IgnoreCase: true,
}

var title = lipgloss.NewStyle().
	Foreground(lipgloss.Color("9")).
	BorderStyle(lipgloss.RoundedBorder()).
	BorderForeground(lipgloss.Color("6")).
	PaddingLeft(1).
	PaddingRight(1).
	// This says "Platune CLI" but I can't find a way to make gofmt make it legible
	Render(`█▀█ █░░ ▄▀█ ▀█▀ █░█ █▄░█ █▀▀   █▀▀ █░░ █
█▀▀ █▄▄ █▀█ ░█░ █▄█ █░▀█ ██▄   █▄▄ █▄▄ █`)

var subtitle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("9")).
	Render(" A simple CLI to manage your Platune server")

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "platune-cli",
	Short: subtitle,
	Long:  lipgloss.JoinVertical(lipgloss.Left, title, subtitle),

	// Uncomment the following line if your bare application
	// has an action associated with it:
	Run: func(cmd *cobra.Command, args []string) {
		interactive, err := cmd.Flags().GetBool("interactive")
		if err != nil {
			fmt.Println(err)
			return
		}
		if interactive {
			p := prompt.New(
				state.executor,
				state.completer,
				prompt.OptionPrefix(">>> "),
				prompt.OptionLivePrefix(state.changeLivePrefix),
				prompt.OptionTitle("Platune CLI"),
				prompt.OptionCompletionWordSeparator(completer.FilePathCompletionSeparator),
			)
			p.Run()
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
	cobra.OnInitialize(initConfig)
	usageFunc := rootCmd.UsageFunc()
	rootCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, "")
		return nil
	})

	rootCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.platune.yaml)")

	// Cobra also supports local flags, which will only run
	// when this action is called directly.
	rootCmd.Flags().BoolP("interactive", "i", false, "Run in interactive mode")
}

// initConfig reads in config file and ENV variables if set.
func initConfig() {
	if cfgFile != "" {
		// Use config file from the flag.
		viper.SetConfigFile(cfgFile)
	} else {
		// Find home directory.
		home, err := homedir.Dir()
		cobra.CheckErr(err)

		// Search config in home directory with name ".platune" (without extension).
		viper.AddConfigPath(home)
		viper.SetConfigName(".platune")
	}

	viper.AutomaticEnv() // read in environment variables that match

	// If a config file is found, read it in.
	if err := viper.ReadInConfig(); err == nil {
		fmt.Fprintln(os.Stderr, "Using config file:", viper.ConfigFileUsed())
	}
}
