package cmd

import (
	"context"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/aschey/platune/cli/v2/utils"
	platune "github.com/aschey/platune/client"
	prompt "github.com/c-bata/go-prompt"
	"github.com/c-bata/go-prompt/completer"
	homedir "github.com/mitchellh/go-homedir"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"google.golang.org/grpc"
)

var cfgFile string

type rootState struct {
	livePrefix     string
	isEnabled      bool
	isSetQueueMode bool
	currentQueue   []string
	platuneClient  platune.PlayerClient
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

func (state *rootState) executor(in string) {
	if in == "set-queue" {
		fmt.Println("Enter file paths or http urls to add to the queue.")
		fmt.Println("Enter a blank line when done.")
		state.isSetQueueMode = true
		state.livePrefix = in + "> "
		state.isEnabled = true
	} else if state.isSetQueueMode {
		if strings.Trim(in, " ") == "" {
			fmt.Println("Queue set")
			ctx, _ := context.WithTimeout(context.Background(), 1000*time.Second)
			_, err := state.platuneClient.SetQueue(ctx, &platune.QueueRequest{Queue: state.currentQueue})
			if err != nil {
				fmt.Println(err)
			}
			state.isSetQueueMode = false
			state.currentQueue = []string{}
		} else {
			state.currentQueue = append(state.currentQueue, in)
			var formattedQueue = []string{}
			for i := 0; i < len(state.currentQueue); i++ {
				formattedQueue = append(formattedQueue, fmt.Sprintf("%d. %s", i+1, state.currentQueue[i]))
			}
			fmt.Println(strings.Join(formattedQueue, "\n"))
		}

	} else {
		fmt.Println("Your input: " + in)

	}

}
func (state *rootState) completer(in prompt.Document) []prompt.Suggest {
	if state.isSetQueueMode {
		return filePathCompleter.Complete(in)
	}
	before := strings.Split(in.TextBeforeCursor(), " ")
	first := ""
	if len(before) > 0 {
		first = before[0]
	}
	switch first {
	case "set-queue":
		return []prompt.Suggest{}
	}

	s := []prompt.Suggest{
		{Text: "set-queue", Description: "Sets the queue"},
		{Text: "articles", Description: "Store the article text posted by user"},
		{Text: "comments", Description: "Store the text commented to articles"},
		{Text: "groups", Description: "Combine users with specific rules"},
	}
	return prompt.FilterHasPrefix(s, in.GetWordBeforeCursor(), true)
}

var filePathCompleter = utils.FilePathCompleter{
	IgnoreCase: true,
}

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "platune",
	Short: "A brief description of your application",
	Long: `A longer description that spans multiple lines and likely contains
examples and usage of using your application. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	// Uncomment the following line if your bare application
	// has an action associated with it:
	Run: func(cmd *cobra.Command, args []string) {
		p := prompt.New(
			state.executor,
			state.completer,
			prompt.OptionPrefix(">>> "),
			prompt.OptionLivePrefix(state.changeLivePrefix),
			prompt.OptionTitle("live-prefix-example"),
			prompt.OptionCompletionWordSeparator(completer.FilePathCompletionSeparator),
		)
		p.Run()
	},
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	cobra.CheckErr(rootCmd.Execute())
}

func init() {
	cobra.OnInitialize(initConfig)

	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.platune.yaml)")

	// Cobra also supports local flags, which will only run
	// when this action is called directly.
	rootCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
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
