use decal::decoder::{CurrentPosition, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DecoderResponse {
    InitializationSucceeded,
    InitializationFailed,
    Received,
    SeekResponse(Result<Timestamp, String>),
    CurrentPositionResponse(CurrentPosition),
}
