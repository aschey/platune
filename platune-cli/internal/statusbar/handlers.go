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

func (s *StatusBar) handlePlayerEvent(
	timer *timer,
	msg *platune.EventResponse,
	currentSong *platune.LookupEntry,
) playerEvent {
	switch msg.Event {
	case platune.Event_START_QUEUE,
		platune.Event_QUEUE_UPDATED,
		platune.Event_ENDED,
		platune.Event_NEXT,
		platune.Event_PREVIOUS:
		state := msg.GetState()
		res := s.platuneClient.GetSongByPath(state.Queue[state.QueuePosition])
		timer.setTime(0)
		if res != nil {
			return playerEvent{
				icon:    "",
				color:   "14",
				newSong: res.Song,
			}
		} else {
			return playerEvent{
				icon:    "",
				color:   "14",
				newSong: nil,
			}
		}

	case platune.Event_SEEK:
		timer.setTime(int64(msg.GetSeekData().SeekMillis))
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
	case platune.Event_POSITION:
		timer.setTime(msg.GetProgress().Position.AsDuration().Milliseconds())
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
		progress := status.Progress.Position.AsDuration()

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
		progress := status.Progress.Position.AsDuration()
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

func (s *StatusBar) handleStateChange(playerState connectivity.State, managementState connectivity.State) (string, string, string) {
	if playerState == connectivity.Ready || managementState == connectivity.Ready {
		s.platuneClient.ResetStreams()
	}

	switch {
	case playerState == connectivity.Connecting || managementState == connectivity.Connecting:
		return "", "0", "Connecting..."
	case playerState == connectivity.Idle || managementState == connectivity.Idle:
		return "", "0", "Idle"

	case playerState == connectivity.Shutdown || playerState == connectivity.TransientFailure || managementState == connectivity.Shutdown || managementState == connectivity.TransientFailure:
		s.statusNotifier.NotifyStatusChanged()
		return "", "9", "Disconnected"
	case playerState == connectivity.Ready && managementState == connectivity.Ready:
		s.statusNotifier.NotifyStatusChanged()
		return "", "10", "Connected"
	default:
		return "", "0", ""
	}
}
