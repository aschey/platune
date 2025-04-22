package statusbar

import (
	management_v1 "github.com/aschey/platune/client/management_v1"
	player_v1 "github.com/aschey/platune/client/player_v1"
	"google.golang.org/grpc/connectivity"
)

type playerEvent struct {
	icon    string
	color   string
	status  string
	newSong *management_v1.LookupEntry
}

func (s *StatusBar) handlePlayerEvent(
	timer *timer,
	msg *player_v1.EventResponse,
	currentSong *management_v1.LookupEntry,
) playerEvent {
	switch msg.Event {
	case player_v1.Event_START_QUEUE,
		player_v1.Event_QUEUE_UPDATED,
		player_v1.Event_ENDED,
		player_v1.Event_NEXT,
		player_v1.Event_PREVIOUS:
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

	case player_v1.Event_SEEK:
		timer.setTime(int64(msg.GetSeekData().SeekMillis))
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: currentSong,
		}
	case player_v1.Event_QUEUE_ENDED, player_v1.Event_STOP:
		timer.stop()
		return playerEvent{
			icon:    "",
			color:   "9",
			status:  "Stopped",
			newSong: nil,
		}
	case player_v1.Event_PAUSE:
		timer.pause()
		if currentSong == nil {
			state := msg.GetState()
			res := s.platuneClient.GetSongByPath(state.Queue[state.QueuePosition])
			currentSong = res.Song
		}
		return playerEvent{
			icon:    "",
			color:   "11",
			status:  "Paused",
			newSong: currentSong,
		}
	case player_v1.Event_RESUME:
		timer.resume()
		if currentSong == nil {
			state := msg.GetState()
			res := s.platuneClient.GetSongByPath(state.Queue[state.QueuePosition])
			currentSong = res.Song
		}
		return playerEvent{
			icon:    "",
			color:   "14",
			newSong: currentSong,
		}
	case player_v1.Event_POSITION:
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
