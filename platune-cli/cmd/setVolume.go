package cmd

import (
	"fmt"
	"strconv"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const setVolumeDescription = "Set the volume"
const setVolumeCmdText = "set-volume"
const setVolumeExampleText = "<value between 0 and 1>"

var setVolumeUsage = fmt.Sprintf("%s %s", setVolumeCmdText, setVolumeExampleText)

var setVolumeCmd = &cobra.Command{
	Use:   setVolumeUsage,
	Short: setVolumeDescription,
	Long:  setVolumeDescription,

	Args: cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		runSetVolume(args)
	},
}

func runSetVolume(args []string) {
	vol, err := strconv.ParseFloat(args[0], 32)
	errMsg := "Volume must be a number between 0 and 1"
	if err != nil {
		fmt.Println(errMsg)
		return
	}
	if vol < 0 || vol > 1 {
		fmt.Println(errMsg)
		return
	}
	internal.Client.SetVolume(float32(vol))
}

func init() {
	usageFunc := setVolumeCmd.UsageFunc()
	setVolumeCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, setVolumeExampleText)
		return nil
	})
	setVolumeCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(setVolumeCmd)
}
