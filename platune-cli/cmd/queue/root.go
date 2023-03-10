package queue

import "github.com/spf13/cobra"

type QueueCmd *cobra.Command

func newQueueCmd(subcommands commands) QueueCmd {
	rootCmd := &cobra.Command{
		Use:   "queue <command>",
		Short: "View and modify the queue",
		Args:  cobra.MinimumNArgs(1),
	}

	rootCmd.AddCommand(subcommands.add)

	return rootCmd
}
