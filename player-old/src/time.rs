use gstreamer::{Clock, ClockExt, ClockTime, SystemClock};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SYSTEM_CLOCK: Clock = SystemClock::obtain();
    pub static ref BASE_TIME: ClockTime = SYSTEM_CLOCK.get_time();
}
