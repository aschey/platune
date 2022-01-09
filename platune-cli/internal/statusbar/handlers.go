package statusbar

import (
	platune "github.com/aschey/platune/client"
	"google.golang.org/grpc/connectivity"
)

type playerEvent struct {
	icon    string
	color   string
	status  string
	newSong *platune.LookupEntry
}

func (s *StatusBar) handlePlayerEvent(timer *timer, msg *platune.EventResponse, currentSong *platune.LookupEntry) playerEvent {
	switch msg.Event {
	case platune.Event_START_QUEUE, platune.Event_QUEUE_UPDATED, platune.Event_ENDED, platune.Event_NEXT, platune.Event_PREVIOUS:
		res := s.platuneClient.GetSongByPath(msg.Queue[msg.QueuePosition])
		timer.setTime(0)
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: res.Song,
		}
	case platune.Event_SEEK:
		timer.setTime(int64(*msg.SeekMillis))
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: currentSong,
		}
	case platune.Event_QUEUE_ENDED, platune.Event_STOP:
		timer.stop()
		return playerEvent{
			icon:    "",
			color:   "9",
			status:  "Stopped",
			newSong: nil,
		}
	case platune.Event_PAUSE:
		timer.pause()
		return playerEvent{
			icon:    "",
			color:   "11",
			status:  "Paused",
			newSong: currentSong,
		}
	case platune.Event_RESUME:
		timer.resume()
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: currentSong,
		}
	default:
		return playerEvent{}
	}
}

func (s *StatusBar) handlePlayerStatus(timer *timer, status *platune.StatusResponse) playerEvent {
	stoppedEvent := playerEvent{
		icon:    "",
		color:   "9",
		status:  "Stopped",
		newSong: nil,
	}
	if status == nil {
		return stoppedEvent
	}
	switch status.Status {
	case platune.PlayerStatus_PLAYING:
		progress := status.Progress.AsDuration()

		timer.start()
		timer.setTime(progress.Milliseconds())

		res := s.platuneClient.GetSongByPath(*status.CurrentSong)

		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: res.Song,
		}
	case platune.PlayerStatus_STOPPED:
		timer.stop()
		return stoppedEvent
	case platune.PlayerStatus_PAUSED:
		timer.pause()
		progress := status.Progress.AsDuration()
		timer.setTime(progress.Milliseconds())
		res := s.platuneClient.GetSongByPath(*status.CurrentSong)

		return playerEvent{
			icon:    "",
			color:   "11",
			status:  "Paused",
			newSong: res.Song,
		}
	default:
		return playerEvent{}
	}
}

func (s *StatusBar) handleStateChange(newState connectivity.State) (string, string, string) {
	if newState == connectivity.Ready {
		s.platuneClient.ResetStreams()
	}

	switch newState {
	case connectivity.Connecting:
		return "", "0", "Connecting..."
	case connectivity.Idle:
		return "", "0", "Idle"
	case connectivity.Ready:
		s.statusNotifier.NotifyStatusChanged()
		return "", "10", "Connected"
	case connectivity.Shutdown, connectivity.TransientFailure:
		s.statusNotifier.NotifyStatusChanged()
		return "", "9", "Disconnected"
	default:
		return "", "0", ""
	}
}
