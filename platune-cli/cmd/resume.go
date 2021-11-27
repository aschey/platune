package cmd

import (
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/spf13/cobra"
)

const resumeDescription = "Resumes the queue. No effect if already playing."
const resumeCmdText = "resume"

func newResumeCmd() *cobra.Command {
	resumeCmd := &cobra.Command{
		Use:   resumeCmdText,
		Short: resumeDescription,
		Long:  resumeDescription,

		Args: cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			client := GetClient(cmd)
			client.Resume()
		},
	}

	usageFunc := resumeCmd.UsageFunc()
	resumeCmd.SetUsageFunc(func(c *cobra.Command) error {
		internal.FormatUsage(c, usageFunc, "")
		return nil
	})
	resumeCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		internal.FormatHelp(c)
	})

	return resumeCmd
}
