package cmd

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"strings"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/lipgloss"
	"github.com/mitchellh/go-homedir"
	"github.com/nathan-fiscaletti/consolesize-go"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var cfgFile string
var searchClient platune.Management_SearchClient

type cmdState struct {
	livePrefix     string
	isSetQueueMode bool
	currentQueue   []string
	dbResults      []*platune.SearchResult
	curPrompt      *prompt.Prompt
}

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

func newCmdState() cmdState {
	state := cmdState{livePrefix: "", isSetQueueMode: false, currentQueue: []string{}}
	state.curPrompt = prompt.New(
		state.executor,
		state.completer,
		prompt.OptionPrefix(">>> "),
		prompt.OptionLivePrefix(state.changeLivePrefix),
		prompt.OptionTitle("Platune CLI"),
		prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"}),
	)
	return state
}

var state = newCmdState()

func (state *cmdState) changeLivePrefix() (string, bool) {
	return state.livePrefix, len(state.livePrefix) > 0
}

func (state *cmdState) executor(in string, selected *prompt.Suggest) {
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
		return
	case "add-queue":
		if len(cmds) < 2 {
			fmt.Println("Usage: add-queue <path or url>")
			return
		}

		if selected != nil {
			for _, dbResult := range state.dbResults {
				if dbResult.Entry == selected.Text && dbResult.Description == selected.Description {
					lookupResults := internal.Client.Lookup(&platune.LookupRequest{
						EntryType:      dbResult.EntryType,
						CorrelationIds: dbResult.CorrelationIds,
					})
					paths := []string{}
					for _, entry := range lookupResults.Entries {
						paths = append(paths, entry.Path)
					}
					internal.Client.AddToQueue(paths)
					return
				}
			}
		}

		full, err := expandFile(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		internal.Client.AddToQueue([]string{full})
	case "seek":
		if len(cmds) < 2 {
			fmt.Println("Usage: seek [hh]:[mm]:ss")
			return
		}
		internal.Client.Seek(cmds[1])
	case "pause":
		internal.Client.Pause()
	case "resume":
		internal.Client.Resume()
	case "stop":
		internal.Client.Stop()
	case "next":
		internal.Client.Next()
	case "previous":
		internal.Client.Previous()
	case "sync":
		SyncProgress()
		fmt.Println()
	case "get-all-folders":
		internal.Client.GetAllFolders()
	case "add-folder":
		if len(cmds) < 2 {
			fmt.Println("Usage: add-folder <path>")
			return
		}
		full, err := expandFolder(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		internal.Client.AddFolder(full)
	case "q":
		fmt.Println("Exiting...")
		os.Exit(0)
	}
	if state.isSetQueueMode {
		if strings.Trim(in, " ") == "" {
			internal.Client.SetQueue(state.currentQueue)
			state.isSetQueueMode = false
			state.currentQueue = []string{}
			state.livePrefix = ""
		} else {
			in, err := expandFile(in)
			if err != nil {
				fmt.Println(err)
				return
			}

			state.currentQueue = append(state.currentQueue, in)
			fmt.Println(internal.PrettyPrintList(state.currentQueue))
		}
	}
}

func (state *cmdState) completer(in prompt.Document) []prompt.Suggest {
	before := strings.Split(in.TextBeforeCursor(), " ")
	if state.isSetQueueMode {
		return filePathCompleter.Complete(in, false)
	}
	if len(before) > 1 {
		state.dbResults = []*platune.SearchResult{}
		first := before[0]
		if first == "add-folder" {
			return filePathCompleter.Complete(in, true)
		} else if first == "add-queue" {
			if searchClient == nil {
				searchClient = internal.Client.Search()
			}
			rest := strings.Join(before[1:], " ")

			if strings.HasPrefix(rest, "http://") || strings.HasPrefix(rest, "https://") {
				return []prompt.Suggest{}
			}

			suggestions := filePathCompleter.Complete(in, true)
			if len(suggestions) > 0 && strings.ContainsAny(rest, "/\\") {
				prompt.OptionCompletionWordSeparator([]string{" ", "/", "\\"})(state.curPrompt)
				return suggestions
			}
			cols, _ := consolesize.GetConsoleSize()
			col := in.CursorPositionCol()
			maxLength := int32((cols - col) / 2)
			sendErr := searchClient.Send(&platune.SearchRequest{Query: rest, MaxLength: &maxLength})
			if sendErr != nil {
				fmt.Println("send error", sendErr)
				return []prompt.Suggest{}
			}
			res, recvErr := searchClient.Recv()
			if recvErr != nil {
				fmt.Println("recv error", recvErr)
				return []prompt.Suggest{}
			}

			state.dbResults = res.Results
			for _, r := range res.Results {
				suggestions = append(suggestions, prompt.Suggest{Text: r.Entry, Description: r.Description})
			}
			prompt.OptionCompletionWordSeparator([]string{"add-queue "})(state.curPrompt)
			return suggestions
		}
		return []prompt.Suggest{}
	}

	s := []prompt.Suggest{
		{Text: "set-queue", Description: SetQueueDescription},
		{Text: "add-queue", Description: AddQueueDescription},
		{Text: "pause", Description: PauseDescription},
		{Text: "resume", Description: ResumeDescription},
		{Text: "seek", Description: SeekDescription},
		{Text: "next", Description: NextDescription},
		{Text: "previous", Description: PreviousDescription},
		{Text: "stop", Description: StopDescription},
		{Text: "sync", Description: SyncDescription},
		{Text: "get-all-folders", Description: GetAllFoldersDescription},
		{Text: "add-folder", Description: AddFolderDescription},
		{Text: "q", Description: "Quit interactive prompt"},
	}
	return prompt.FilterHasPrefix(s, in.GetWordBeforeCursor(), true)
}

var filePathCompleter = internal.FilePathCompleter{
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
	cobra.OnInitialize(initConfig)
	usageFunc := rootCmd.UsageFunc()
	rootCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})

	rootCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
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
