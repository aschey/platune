use symphonia::core::units::TimeStamp;

use super::current_position::CurrentPosition;

#[derive(Clone, Debug)]
pub(crate) enum DecoderResponse {
    SeekResponse(Result<TimeStamp, String>),
    CurrentPositionResponse(CurrentPosition),
}