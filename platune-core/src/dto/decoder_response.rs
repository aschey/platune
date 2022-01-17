use symphonia::core::units::TimeStamp;

use super::current_time::CurrentTime;

#[derive(Clone, Debug)]
pub(crate) enum DecoderResponse {
    SeekResponse(Option<TimeStamp>),
    CurrentTimeResponse(CurrentTime),
}
