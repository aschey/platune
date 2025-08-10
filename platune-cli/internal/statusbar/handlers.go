package statusbar

import (
	player_v1 "github.com/aschey/platune/client/player_v1"
	"google.golang.org/grpc/connectivity"
)

type playerEvent struct {
	icon    string
	color   string
	status  string
	newMeta *player_v1.Metadata
}

func (s *StatusBar) handlePlayerEvent(
	timer *timer,
	msg *player_v1.EventResponse,
	currentMeta *player_v1.Metadata,
) playerEvent {
	switch msg.Event {
	case player_v1.Event_START_QUEUE,
		player_v1.Event_QUEUE_UPDATED,
		player_v1.Event_TRACK_CHANGED:
		state := msg.GetState()
		timer.setTime(0)
		return playerEvent{
			icon:    "",
			color:   "14",
			newMeta: state.Metadata,
		}

	case player_v1.Event_SEEK:
		timer.setTime(int64(msg.GetSeekData().SeekMillis))
		return playerEvent{
			icon:    "",
			color:   "14",
			newMeta: currentMeta,
		}
	case player_v1.Event_QUEUE_ENDED, player_v1.Event_STOP:
		timer.stop()
		return playerEvent{
			icon:    "",
			color:   "9",
			status:  "Stopped",
			newMeta: nil,
		}
	case player_v1.Event_PAUSE:
		timer.pause()
		if currentMeta == nil {
			state := msg.GetState()
			currentMeta = state.Metadata
		}
		return playerEvent{
			icon:    "",
			color:   "11",
			status:  "Paused",
			newMeta: currentMeta,
		}
	case player_v1.Event_RESUME:
		timer.resume()
		if currentMeta == nil {
			state := msg.GetState()
			currentMeta = state.Metadata
		}
		return playerEvent{
			icon:    "",
			color:   "14",
			newMeta: currentMeta,
		}
	case player_v1.Event_POSITION:
		timer.setTime(msg.GetProgress().Position.AsDuration().Milliseconds())
		return playerEvent{
			icon:    "",
			color:   "14",
			newMeta: currentMeta,
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
