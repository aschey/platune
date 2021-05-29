package cmd

import (
	"context"
	"fmt"
	"math"
	"os"
	"path"
	"strconv"
	"strings"
	"time"

	"github.com/aschey/platune/cli/v2/utils"
	platune "github.com/aschey/platune/client"
	"github.com/c-bata/go-prompt"
	"github.com/c-bata/go-prompt/completer"
	"github.com/charmbracelet/lipgloss"
	"github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

var cfgFile string

type rootState struct {
	livePrefix     string
	isEnabled      bool
	isSetQueueMode bool
	currentQueue   []string
	platuneClient  platune.PlayerClient
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

func newRootState() rootState {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())
	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	client := platune.NewPlayerClient(conn)

	return rootState{livePrefix: "", isEnabled: false, isSetQueueMode: false, currentQueue: []string{}, platuneClient: client}
}

var state = newRootState()

func (state *rootState) changeLivePrefix() (string, bool) {
	return state.livePrefix, state.isEnabled
}

func (state *rootState) runCommand(successMsg string, cmdFunc func(platune.PlayerClient, context.Context) (*emptypb.Empty, error)) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	_, err := cmdFunc(state.platuneClient, ctx)
	cancel()
	if err != nil {
		fmt.Println(err)
		return
	}

	fmt.Println(successMsg)

}

func (state *rootState) executor(in string) {
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
		state.runCommand("Added", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.AddToQueue(ctx, &platune.AddToQueueRequest{Song: full})
		})
	case "seek":
		if len(cmds) < 2 {
			fmt.Println("Usage: seek [hh]:[mm]:ss")
			return
		}
		timeParts := strings.Split(cmds[1], ":")
		totalMillis := uint64(0)
		for i := 0; i < len(timeParts); i++ {
			intVal, err := strconv.ParseUint(timeParts[i], 10, 64)
			if err != nil {
				fmt.Println(timeParts[i] + " is not a valid integer")
			}
			pos := float64(len(timeParts) - 1 - i)
			totalMillis += uint64(math.Pow(60, pos)) * intVal * 1000
		}
		state.runCommand("Seeked to "+cmds[1], func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.Seek(ctx, &platune.SeekRequest{Millis: totalMillis})
		})
	case "pause":
		state.runCommand("Paused", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.Pause(ctx, &emptypb.Empty{})
		})
	case "resume":
		state.runCommand("Resumed", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.Resume(ctx, &emptypb.Empty{})
		})
	case "stop":
		state.runCommand("Stopped", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.Stop(ctx, &emptypb.Empty{})
		})
	case "next":
		state.runCommand("Next", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.Next(ctx, &emptypb.Empty{})
		})
	case "previous":
		state.runCommand("Previous", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
			return state.platuneClient.Previous(ctx, &emptypb.Empty{})
		})
	}
	if state.isSetQueueMode {
		if strings.Trim(in, " ") == "" {
			state.runCommand("Queue set", func(client platune.PlayerClient, ctx context.Context) (*emptypb.Empty, error) {
				return state.platuneClient.SetQueue(ctx, &platune.QueueRequest{Queue: state.currentQueue})
			})
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

func (state *rootState) completer(in prompt.Document) []prompt.Suggest {
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
		{Text: "set-queue", Description: "Sets the queue and starts playback. Resets the queue if playback has already started."},
		{Text: "add-queue", Description: "Adds a song to the end of the queue"},
		{Text: "pause", Description: "Pauses the queue"},
		{Text: "resume", Description: "Resumes the queue. No effect if already playing."},
		{Text: "seek", Description: "Seek to a specific time. Input should be formatted like [hh]:[mm]:ss"},
		{Text: "next", Description: "Skips to the next track"},
		{Text: "previous", Description: "Skips to the previous track"},
		{Text: "stop", Description: "Stops playback"},
	}
	return prompt.FilterHasPrefix(s, in.GetWordBeforeCursor(), true)
}

var filePathCompleter = utils.FilePathCompleter{
	IgnoreCase: true,
}

var title = lipgloss.NewStyle().
	Bold(true).
	Foreground(lipgloss.Color("9")).
	BorderStyle(lipgloss.RoundedBorder()).
	BorderForeground(lipgloss.Color("6")).
	PaddingLeft(1).
	PaddingRight(1).
	Render("Platune CLI")

var subtitle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("9")).
	Render("Simple CLI for the Platune server")

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

func addColor(replaceStr string, searchStr string, style lipgloss.Style) string {
	return strings.Replace(replaceStr, searchStr, style.Render(searchStr), 1)
}

func init() {
	cobra.OnInitialize(initConfig)

	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.
	rootCmd.SetHelpFunc(func(c *cobra.Command, a []string) {

		//f := c.LocalFlags().Lookup("help")
		//fmt.Println(c.LocalFlags().FlagUsages())
		fmt.Printf("%s\n\n", c.Long)
		outStr := c.UsageString()

		subtext := lipgloss.NewStyle().Foreground(lipgloss.Color("245"))
		title := lipgloss.NewStyle().Foreground(lipgloss.Color("4"))

		outStr = addColor(outStr, "Usage:", title)
		outStr = addColor(outStr, "Available Commands:", title)
		outStr = addColor(outStr, "Flags:", title)
		outStr = addColor(outStr, "[flags]", subtext)
		outStr = addColor(outStr, "[command]", subtext)
		fmt.Println(outStr)
	})
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
