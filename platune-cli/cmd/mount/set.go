package mount

import (
	"fmt"
	"strings"

	cprompt "github.com/aschey/bubbleprompt-cobra"
	"github.com/aschey/bubbleprompt/completer"
	"github.com/aschey/bubbleprompt/input/commandinput"
	"github.com/aschey/bubbleprompt/suggestion"
	"github.com/aschey/platune/cli/internal"
	"github.com/spf13/cobra"
)

type setMountCmd *cobra.Command

func newSetMountCmd(client *internal.ManagementClient) setMountCmd {
	setMountCmd := &cobra.Command{
		Use:   "set <mount point>",
		Short: "Set the mount point for the library",
		Args:  cobra.ExactArgs(1),

		RunE: func(cmd *cobra.Command, args []string) error {
			if err := client.SetMount(args[0]); err != nil {
				return err
			}
			return cprompt.ExecModel(
				cmd,
				internal.NewInfoModel(fmt.Sprintf("Mount point set to %s", args[0])),
			)
		},
	}

	cprompt.Completer(setMountCmd, func(cmd *cobra.Command, args []string, toComplete string) (
		[]suggestion.Suggestion[commandinput.CommandMetadata[internal.SearchMetadata]], error) {
		pathCompleter := completer.PathCompleter[commandinput.CommandMetadata[internal.SearchMetadata]]{
			IgnoreCase: true,
		}
		return pathCompleter.Complete(strings.Join(append(args, toComplete), " ")), nil

	})

	return setMountCmd
}
