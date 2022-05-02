use symphonia::core::units::TimeStamp;

use super::current_position::CurrentPosition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DecoderResponse {
    InitializationSucceeded,
    InitializationFailed,
    Received,
    SeekResponse(Result<TimeStamp, String>),
    CurrentPositionResponse(CurrentPosition),
}
