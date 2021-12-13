package internal

import "time"

type timer struct {
	startTime      int64
	pauseStartTime int64
	pauseTime      int64
}

func (t *timer) start() {
	t.pauseTime = 0
	t.pauseStartTime = 0
	t.startTime = time.Now().UnixMilli()
}

func (t *timer) resume() {
	if t.startTime == 0 {
		t.start()
	} else if t.pauseStartTime > 0 {
		t.pauseTime += time.Now().UnixMilli() - t.pauseStartTime
		t.pauseStartTime = 0
	}
}

func (t *timer) pause() {
	t.pauseStartTime = time.Now().UnixMilli()
}

func (t *timer) stop() {
	t.startTime = 0
	t.pauseStartTime = 0
	t.pauseTime = 0
}

func (t *timer) setTime(unixMillis int64) {
	t.startTime = time.Now().UnixMilli() - unixMillis
	t.pauseTime = 0
	if t.pauseStartTime > 0 {
		t.pause()
	}
}

func (t *timer) elapsed() time.Duration {
	now := time.Now().UnixMilli()
	var currentPauseTime int64 = 0
	if t.pauseStartTime > 0 {
		currentPauseTime = now - t.pauseStartTime
	}
	if t.startTime > 0 {
		return time.Duration((now - t.startTime - t.pauseTime - currentPauseTime) * 1000000)
	}
	return 0
}
