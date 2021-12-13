use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct Timer {
    // When the timer was started
    start: Option<Instant>,
    // When the timer was paused
    pause_start: Option<Instant>,
    // Total duration that the timer has been paused for
    // Since the clock is always increasing, we must add up all the times that the timer was paused
    // and subtract all of them to get the total non-paused duration
    pause_time: Duration,
}

#[derive(Clone, Debug)]
pub(crate) enum TimerStatus {
    Started,
    Paused,
    Stopped,
}

impl Timer {
    pub(crate) fn start(&mut self) {
        self.pause_start = None;
        self.pause_time = Duration::default();
        self.start = Some(Instant::now());
    }

    pub(crate) fn resume(&mut self) {
        if self.start.is_none() {
            // Resuming a non-started timer, treat this like a normal start
            self.start();
        } else if let Some(pause_start) = self.pause_start {
            // Add the additional time the timer was paused for
            self.pause_time += Instant::now() - pause_start;
            self.pause_start = None;
        }
    }

    pub(crate) fn pause(&mut self) {
        self.pause_start = Some(Instant::now());
    }

    pub(crate) fn stop(&mut self) {
        self.start = None;
        self.pause_start = None;
        self.pause_time = Duration::default();
    }

    pub(crate) fn set_time(&mut self, duration: Duration) {
        self.start = Some(Instant::now() - duration);
        // Resetting the start time, so we don't need any of the previous times that it was paused
        self.pause_time = Duration::default();
        if self.pause_start.is_some() {
            // If the timer is paused, keep it paused and reset the pause start
            self.pause();
        }
    }

    pub(crate) fn elapsed(&self) -> Option<Duration> {
        let now = Instant::now();
        let current_pause_time = match self.pause_start {
            Some(pause_start) => now - pause_start,
            None => Duration::default(),
        };
        self.start
            .map(|start| now - start - self.pause_time - current_pause_time)
    }

    pub(crate) fn status(&self) -> TimerStatus {
        if self.pause_start.is_some() {
            TimerStatus::Paused
        } else if self.start.is_some() {
            TimerStatus::Started
        } else {
            TimerStatus::Stopped
        }
    }
}
