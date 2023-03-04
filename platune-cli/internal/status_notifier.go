package internal

type StatusNotifier struct {
	waitForStatusChangeCh chan struct{}
	statusChangedCh       chan struct{}
}

func NewStatusNotifier() *StatusNotifier {
	return &StatusNotifier{
		waitForStatusChangeCh: make(chan struct{}, 1),
		statusChangedCh:       make(chan struct{}, 1),
	}
}

func (s *StatusNotifier) WaitForStatusChange() {
	s.waitForStatusChangeCh <- struct{}{}
	<-s.statusChangedCh
}

func (s *StatusNotifier) NotifyStatusChanged() {
	select {
	case <-s.waitForStatusChangeCh:
		s.statusChangedCh <- struct{}{}
	default:
	}
}
