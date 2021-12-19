package statusbar

import "github.com/aschey/platune/cli/v2/internal"

type StatusBar struct {
	statusChan     StatusChan
	platuneClient  *internal.PlatuneClient
	statusNotifier *internal.StatusNotifier
}

func NewStatusBar(statusChan StatusChan, platuneClient *internal.PlatuneClient, statusNotifier *internal.StatusNotifier) *StatusBar {
	return &StatusBar{
		statusChan:     statusChan,
		platuneClient:  platuneClient,
		statusNotifier: statusNotifier,
	}
}

type StatusChan chan string

func NewStatusChan() StatusChan {
	return make(StatusChan, 128)
}
