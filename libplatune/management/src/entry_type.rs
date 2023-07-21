use strum::EnumString;

#[derive(Debug, EnumString, Clone)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
#[strum(ascii_case_insensitive)]
pub enum EntryType {
    Song,
    Artist,
    Album,
}
