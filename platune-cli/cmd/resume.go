package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const resumeDescription = "Resumes the queue. No effect if already playing."
const resumeCmdText = "resume"

var resumeCmd = &cobra.Command{
	Use:   resumeCmdText,
	Short: resumeDescription,
	Long:  resumeDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		ctx := cmd.Context()
		client := ctx.Value(Client).(*internal.PlatuneClient)
		client.Resume()
	},
}

func init() {
	usageFunc := resumeCmd.UsageFunc()
	resumeCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	resumeCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})
	rootCmd.AddCommand(resumeCmd)
}
