use decal::decoder::{CurrentPosition, TimeStamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DecoderResponse {
    InitializationSucceeded,
    InitializationFailed,
    Received,
    SeekResponse(Result<TimeStamp, String>),
    CurrentPositionResponse(CurrentPosition),
}
