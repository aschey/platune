package cmd

import (
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/spf13/cobra"
)

const ResumeDescription = "Resumes the queue. No effect if already playing."

var resumeCmd = &cobra.Command{
	Use:   "resume",
	Short: ResumeDescription,
	Long:  ResumeDescription,

	Args: cobra.NoArgs,
	Run: func(cmd *cobra.Command, args []string) {
		utils.Client.Resume()
	},
}

func init() {
	usageFunc := resumeCmd.UsageFunc()
	resumeCmd.SetUsageFunc(func(c *cobra.Command) error {
		utils.FormatUsage(c, usageFunc, "")
		return nil
	})
	resumeCmd.SetHelpFunc(func(c *cobra.Command, a []string) {
		utils.FormatHelp(c)
	})
	rootCmd.AddCommand(resumeCmd)
}
