//go:build windows

package statusbar

import "os"

// No sigwinch on Windows :(
func getSignalChannel() chan os.Signal {
	sigCh := make(chan os.Signal, 1)

	return sigCh
}
