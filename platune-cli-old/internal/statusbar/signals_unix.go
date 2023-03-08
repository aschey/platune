//go:build !windows

package statusbar

import (
	"os"
	"os/signal"
	"syscall"
)

func getSignalChannel() chan os.Signal {
	sigCh := make(chan os.Signal, 1)
	signal.Notify(
		sigCh,
		syscall.SIGWINCH,
	)

	return sigCh
}
